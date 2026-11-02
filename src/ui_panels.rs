use crate::cutter::{
    build_specification_i18n, compute_cutting_layout, CuttingReport, SheetLayout, SpecRow,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::export::{export_spec_to_csv_file_i18n, export_spec_to_pdf_file};
use crate::export::generate_cutting_clipboard_text_i18n;
use crate::i18n::AppLanguage;
use crate::model::{
    EdgeId, GroupId, Material, MaterialId, PartBlock, PartEdges, PartGroup, PartId,
    PartOrientation, Project,
};
use crate::theme::{
    FA_ARROW_DOWN, FA_ARROW_UP, FA_CLONE, FA_COPY, FA_CUBE, FA_EXPAND, FA_PASTE, FA_POINTER,
    FA_ROTATE, FA_RULER, FA_TABLE_CELLS_LARGE, FA_TRASH,
};
use crate::viewport::{EditorTool, QuadViewportController};
use egui::{Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, TextureHandle, Vec2 as EguiVec2};
use glam::Vec3;
use std::collections::HashMap;

/// Active application tab
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppTab {
    /// 4-viewport designer (quad viewports and precision transform bar)
    Designer4View,
    /// Parts specification table and export
    Specification,
    /// Guillotine sheet cutting layout utility
    CuttingLayout,
    /// Materials catalog (18 mm thickness, colors, textures) and edge bandings
    MaterialsManager,
}

/// Evaluates a math expression in a numeric input field (supports `+`, `-`, `*`, `/`, parentheses `(...)`,
/// dot and comma as decimal separator, spaces, and unit suffixes).
/// Examples: `"600+100"` -> `700.0`, `"600-4"` -> `596.0`, `"800-18-18"` -> `764.0`, `"(800-36)/2"` -> `382.0`.
pub fn eval_math_expression(input: &str) -> Option<f64> {
    let cleaned: String = input
        .chars()
        .map(|c| match c {
            ',' => '.',
            '−' | '–' | '—' => '-',
            '×' => '*',
            '÷' => '/',
            other => other,
        })
        .filter(|c| matches!(c, '0'..='9' | '.' | '+' | '-' | '*' | '/' | '(' | ')'))
        .collect();

    if cleaned.is_empty() {
        return None;
    }

    struct Parser<'a> {
        bytes: &'a [u8],
        pos: usize,
    }

    impl<'a> Parser<'a> {
        fn peek(&self) -> Option<u8> {
            self.bytes.get(self.pos).copied()
        }

        fn parse_expr(&mut self) -> Option<f64> {
            let mut left = self.parse_term()?;
            while let Some(op) = self.peek() {
                if op == b'+' || op == b'-' {
                    self.pos += 1;
                    let right = self.parse_term()?;
                    if op == b'+' {
                        left += right;
                    } else {
                        left -= right;
                    }
                } else {
                    break;
                }
            }
            Some(left)
        }

        fn parse_term(&mut self) -> Option<f64> {
            let mut left = self.parse_factor()?;
            while let Some(op) = self.peek() {
                if op == b'*' || op == b'/' {
                    self.pos += 1;
                    let right = self.parse_factor()?;
                    if op == b'*' {
                        left *= right;
                    } else {
                        if right.abs() < 1e-12 {
                            return None;
                        }
                        left /= right;
                    }
                } else {
                    break;
                }
            }
            Some(left)
        }

        fn parse_factor(&mut self) -> Option<f64> {
            match self.peek()? {
                b'+' => {
                    self.pos += 1;
                    self.parse_factor()
                }
                b'-' => {
                    self.pos += 1;
                    Some(-self.parse_factor()?)
                }
                b'(' => {
                    self.pos += 1;
                    let val = self.parse_expr()?;
                    if self.peek() == Some(b')') {
                        self.pos += 1;
                        Some(val)
                    } else {
                        None
                    }
                }
                _ => self.parse_number(),
            }
        }

        fn parse_number(&mut self) -> Option<f64> {
            let start = self.pos;
            let mut has_digit = false;
            let mut has_dot = false;
            while let Some(b) = self.peek() {
                if b.is_ascii_digit() {
                    has_digit = true;
                    self.pos += 1;
                } else if b == b'.' && !has_dot {
                    has_dot = true;
                    self.pos += 1;
                } else {
                    break;
                }
            }
            if !has_digit {
                return None;
            }
            let s = std::str::from_utf8(&self.bytes[start..self.pos]).ok()?;
            s.parse::<f64>().ok()
        }
    }

    let mut parser = Parser {
        bytes: cleaned.as_bytes(),
        pos: 0,
    };
    let val = parser.parse_expr()?;
    if parser.pos == parser.bytes.len() && val.is_finite() {
        Some(val)
    } else {
        None
    }
}

/// Creates an `egui::DragValue` supporting math expressions (`600+100`, `600-4`, `800-36`, etc.)
pub fn math_drag_value(value: &mut f32) -> egui::DragValue<'_> {
    egui::DragValue::new(value).custom_parser(eval_math_expression)
}

/// Creates an `egui::DragValue` for f64 supporting math expressions (`600+100`, `600-4`, etc.)
pub fn math_drag_value_f64(value: &mut f64) -> egui::DragValue<'_> {
    egui::DragValue::new(value).custom_parser(eval_math_expression)
}

/// Adds a quick standard panel (Shelf XY, Side YZ, or Front XZ) using the active material
pub fn add_quick_part(
    project: &mut Project,
    material_id: MaterialId,
    orientation: PartOrientation,
    lang: AppLanguage,
) -> PartId {
    let grain_fixed = project
        .get_material(material_id)
        .map(|m| m.default_grain_fixed)
        .unwrap_or(true);
    let id = project.alloc_id();
    let idx = project.parts.len() + 1;
    let (name, l, w, pos) = match orientation {
        PartOrientation::HorizontalXY => (
            format!("{} {}", lang.t("Shelf"), idx),
            764.0,
            500.0,
            Vec3::new(18.0, 0.0, 350.0),
        ),
        PartOrientation::VerticalYZ => (
            format!("{} {}", lang.t("Side Panel"), idx),
            720.0,
            500.0,
            Vec3::new(0.0, 0.0, 0.0),
        ),
        PartOrientation::FrontalXZ => (
            format!("{} {}", lang.t("Front Panel"), idx),
            720.0,
            400.0,
            Vec3::new(0.0, 500.0, 0.0),
        ),
    };
    project.parts.push(PartBlock {
        id,
        name,
        group_id: project.groups.first().map(|g| g.id),
        material_id,
        orientation,
        position_mm: pos,
        length_mm: l,
        width_mm: w,
        yaw_deg: 0.0,
        grain_fixed,
        grain_along_width: false,
        edges: PartEdges::default(),
        visible: true,
        locked: false,
        drill_holes: Vec::new(),
    });
    id
}

/// Shared helper to copy currently selected parts into clipboard and update status message
pub fn perform_copy_selection(
    ctx: &egui::Context,
    project: &Project,
    selected_part: Option<PartId>,
    selected_parts: &[PartId],
    clipboard: &mut Vec<ClipboardPart>,
    status_msg: &mut String,
    lang: AppLanguage,
) {
    let copied = copy_selected_parts_to_clipboard(project, selected_part, selected_parts);
    if !copied.is_empty() {
        if let Ok(json) = serde_json::to_string(&copied) {
            ctx.copy_text(format!("{}{}", CLIPBOARD_PREFIX, json));
        }
        let is_mac = crate::i18n::is_macos_platform();
        let mod_name = if is_mac { "Cmd" } else { "Ctrl" };
        if copied.len() == 1 {
            *status_msg = lang.tf("fmt.part_copied_one", &[&copied[0].part.name, &mod_name]);
        } else {
            *status_msg = lang.tf("fmt.parts_copied_many", &[&copied.len(), &mod_name]);
        }
        *clipboard = copied;
    }
}

/// Shared helper to paste clipboard parts into project and update selection & status message
pub fn perform_paste_clipboard(
    project: &mut Project,
    selected_part: &mut Option<PartId>,
    selected_parts: &mut Vec<PartId>,
    clipboard: &[ClipboardPart],
    status_msg: &mut String,
    lang: AppLanguage,
    mut on_before_mutate: impl FnMut(&Project),
) {
    if clipboard.is_empty() {
        return;
    }
    on_before_mutate(project);
    let new_ids = paste_parts_into_project(project, clipboard);
    *selected_part = new_ids.last().copied();
    *selected_parts = new_ids.clone();
    if new_ids.len() == 1 {
        *status_msg = lang.tf(
            "fmt.part_pasted_one",
            &[&clipboard[0].part.name, &project.name],
        );
    } else {
        *status_msg = lang.tf("fmt.parts_pasted_many", &[&new_ids.len(), &project.name]);
    }
}

/// Shared helper to duplicate selected parts within the project
pub fn perform_duplicate_selection(
    project: &mut Project,
    selected_part: &mut Option<PartId>,
    selected_parts: &mut Vec<PartId>,
    status_msg: &mut String,
    lang: AppLanguage,
    mut on_before_mutate: impl FnMut(&Project),
) {
    let ids_to_dup: Vec<PartId> = if !selected_parts.is_empty() {
        selected_parts.clone()
    } else {
        (*selected_part).into_iter().collect()
    };
    if !ids_to_dup.is_empty() {
        on_before_mutate(project);
        let mut new_ids = Vec::new();
        let copy_word = lang.t("copy");
        for sid in ids_to_dup {
            if let Some(orig) = project.parts.iter().find(|p| p.id == sid).cloned() {
                let new_id = project.alloc_id();
                let mut copy = orig.clone();
                copy.id = new_id;
                copy.name = project.generate_copy_name(&orig.name, copy_word);
                copy.position_mm += Vec3::new(36.0, 0.0, 36.0);
                project.parts.push(copy);
                new_ids.push(new_id);
            }
        }
        *selected_part = new_ids.last().copied();
        *selected_parts = new_ids;
        let mod_name = if crate::i18n::is_macos_platform() {
            "Cmd"
        } else {
            "Ctrl"
        };
        *status_msg = lang.tf("fmt.parts_duplicated", &[&mod_name]);
    }
}

/// Shared helper to delete selected parts from the project
pub fn perform_delete_selection(
    project: &mut Project,
    selected_part: &mut Option<PartId>,
    selected_parts: &mut Vec<PartId>,
    status_msg: &mut String,
    lang: AppLanguage,
    mut on_before_mutate: impl FnMut(&Project),
) {
    let ids_to_del: Vec<PartId> = if !selected_parts.is_empty() {
        selected_parts.clone()
    } else {
        (*selected_part).into_iter().collect()
    };
    if !ids_to_del.is_empty() {
        on_before_mutate(project);
        project.parts.retain(|p| !ids_to_del.contains(&p.id));
        *selected_part = None;
        selected_parts.clear();
        *status_msg = lang.t("Selected parts deleted").to_string();
    }
}

/// Always-visible Left Tools Dock (modeled after OniAnvil: 1-column 58px or 2-column 106px)
#[allow(clippy::too_many_arguments)]
pub fn ui_left_tools_dock(
    ui: &mut egui::Ui,
    project: &mut Project,
    selected_part: &mut Option<PartId>,
    active_material_id: &mut MaterialId,
    viewport_ctrl: &mut QuadViewportController,
    active_tab: &mut AppTab,
    clipboard: &mut Vec<ClipboardPart>,
    status_msg: &mut String,
    two_columns: &mut bool,
    mut on_toggle_columns: impl FnMut(),
    mut on_before_mutate: impl FnMut(&Project),
) {
    let lang = viewport_ctrl.lang;
    let btn_w = 38.0;
    let btn_h = 34.0;
    let cols = if *two_columns { 2 } else { 1 };

    let is_dark = ui.visuals().dark_mode;

    // Dock header with 1-col / 2-col switcher button
    ui.horizontal(|ui| {
        if *two_columns {
            ui.label(RichText::new(lang.t("tools.header")).size(11.0).strong())
                .on_hover_text(lang.t("tools.header"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .small_button("«")
                    .on_hover_text(lang.t("tools.toggle_columns_tip"))
                    .clicked()
                {
                    *two_columns = !*two_columns;
                    on_toggle_columns();
                }
            });
        } else {
            let (rect, resp) = ui.allocate_exact_size(EguiVec2::new(btn_w, 20.0), Sense::click());
            let painter = ui.painter();
            if resp.hovered() {
                let hover_bg = if is_dark {
                    Color32::from_white_alpha(20)
                } else {
                    Color32::from_black_alpha(15)
                };
                painter.rect_filled(rect, egui::Rounding::same(4.0), hover_bg);
            }
            let toggle_col = if resp.hovered() {
                if is_dark {
                    Color32::WHITE
                } else {
                    Color32::from_gray(20)
                }
            } else if is_dark {
                Color32::from_gray(170)
            } else {
                Color32::from_gray(90)
            };
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "»",
                FontId::proportional(13.0),
                toggle_col,
            );
            let clicked = resp.clicked();
            resp.on_hover_text(lang.t("tools.toggle_columns_tip"));
            if clicked {
                *two_columns = !*two_columns;
                on_toggle_columns();
            }
        }
    });

    let draw_dock_divider = |ui: &mut egui::Ui| {
        ui.add_space(4.0);
        let avail_w = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(EguiVec2::new(avail_w, 1.0), Sense::hover());
        let div_col = if is_dark {
            Color32::from_white_alpha(22)
        } else {
            Color32::from_black_alpha(25)
        };
        ui.painter().rect_filled(rect, 0.0, div_col);
        ui.add_space(4.0);
    };

    draw_dock_divider(ui);

    egui::ScrollArea::vertical()
        .id_salt("left_tools_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Helper closure to render a sleek icon button
            let draw_dock_btn = |ui: &mut egui::Ui,
                                 icon: &str,
                                 is_text: bool,
                                 selected: bool,
                                 enabled: bool,
                                 tooltip: &str|
             -> bool {
                let (rect, resp) = ui.allocate_exact_size(
                    EguiVec2::new(btn_w, btn_h),
                    if enabled { Sense::click() } else { Sense::hover() },
                );
                let rounding = egui::Rounding::same(6.0);
                let painter = ui.painter();

                if selected {
                    painter.rect_filled(rect, rounding, Color32::from_rgb(0, 122, 204));
                    if resp.hovered() {
                        painter.rect_filled(rect, rounding, Color32::from_white_alpha(30));
                    }
                } else if enabled {
                    if resp.is_pointer_button_down_on() {
                        let press_bg = if is_dark {
                            Color32::from_white_alpha(42)
                        } else {
                            Color32::from_black_alpha(30)
                        };
                        painter.rect_filled(rect, rounding, press_bg);
                    } else if resp.hovered() {
                        let hover_bg = if is_dark {
                            Color32::from_white_alpha(24)
                        } else {
                            Color32::from_black_alpha(18)
                        };
                        let hover_stroke = if is_dark {
                            Stroke::new(1.0, Color32::from_white_alpha(45))
                        } else {
                            Stroke::new(1.0, Color32::from_black_alpha(40))
                        };
                        painter.rect_filled(rect, rounding, hover_bg);
                        painter.rect_stroke(rect, rounding, hover_stroke);
                    }
                }

                let text_color = if !enabled {
                    if is_dark {
                        Color32::from_white_alpha(55)
                    } else {
                        Color32::from_black_alpha(55)
                    }
                } else if selected {
                    Color32::WHITE
                } else if resp.hovered() {
                    if is_dark {
                        Color32::WHITE
                    } else {
                        Color32::from_gray(15)
                    }
                } else if is_dark {
                    Color32::from_gray(215)
                } else {
                    Color32::from_gray(50)
                };

                let font_id = if is_text {
                    FontId::proportional(11.5)
                } else {
                    FontId::proportional(15.0)
                };

                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    icon,
                    font_id,
                    text_color,
                );

                let clicked = resp.clicked();
                if !tooltip.is_empty() {
                    resp.on_hover_text(tooltip);
                }

                enabled && clicked
            };

            // 1. Primary Editor Tools (Select V, Create Box B, Tape Measure M)
            egui::Grid::new("left_dock_editor_tools")
                .num_columns(cols)
                .spacing([4.0, 4.0])
                .show(ui, |ui| {
                    let mut col_idx = 0;

                    if draw_dock_btn(
                        ui,
                        FA_POINTER,
                        false,
                        *active_tab == AppTab::Designer4View
                            && viewport_ctrl.tool == EditorTool::SelectAndMove,
                        true,
                        &format!(
                            "{}\n{}",
                            lang.t("↖ Select / Move (V)"),
                            lang.t("Select parts, move and resize using 8 handles")
                        ),
                    ) {
                        *active_tab = AppTab::Designer4View;
                        viewport_ctrl.tool = EditorTool::SelectAndMove;
                    }
                    col_idx += 1;
                    if col_idx % cols == 0 {
                        ui.end_row();
                    }

                    if draw_dock_btn(
                        ui,
                        FA_CUBE,
                        false,
                        *active_tab == AppTab::Designer4View
                            && viewport_ctrl.tool == EditorTool::CreateBlock,
                        true,
                        &format!(
                            "{}\n{}",
                            lang.t("➕ Create Part by Box (B)"),
                            lang.t("Draw a rectangle in any 2D view. Thickness comes from active material (e.g. 18 mm). Press Tab to cycle Shelf/Side/Front.")
                        ),
                    ) {
                        *active_tab = AppTab::Designer4View;
                        viewport_ctrl.tool = EditorTool::CreateBlock;
                    }
                    col_idx += 1;
                    if col_idx % cols == 0 {
                        ui.end_row();
                    }

                    if draw_dock_btn(
                        ui,
                        FA_RULER,
                        false,
                        *active_tab == AppTab::Designer4View
                            && viewport_ctrl.tool == EditorTool::MeasureTape,
                        true,
                        &format!(
                            "{}\n{}",
                            lang.t("📏 Tape Measure (M)"),
                            lang.t("Measure distances between points and edges in 2D projections")
                        ),
                    ) {
                        *active_tab = AppTab::Designer4View;
                        viewport_ctrl.tool = EditorTool::MeasureTape;
                    }
                    col_idx += 1;
                    if col_idx % cols != 0 {
                        ui.end_row();
                    } else {
                        ui.end_row();
                    }
                });

            draw_dock_divider(ui);

            // 2. Quick Add Panels (+XY Shelf, +YZ Side, +XZ Front)
            egui::Grid::new("left_dock_quick_parts")
                .num_columns(cols)
                .spacing([4.0, 4.0])
                .show(ui, |ui| {
                    let mut col_idx = 0;

                    if draw_dock_btn(
                        ui,
                        "+XY",
                        true,
                        false,
                        true,
                        &format!(
                            "{}\n{}",
                            lang.t("+ Shelf (XY)"),
                            lang.t("Add horizontal shelf/bottom panel from active material")
                        ),
                    ) {
                        *active_tab = AppTab::Designer4View;
                        on_before_mutate(project);
                        let id = add_quick_part(
                            project,
                            *active_material_id,
                            PartOrientation::HorizontalXY,
                            lang,
                        );
                        *selected_part = Some(id);
                        viewport_ctrl.selected_parts = vec![id];
                    }
                    col_idx += 1;
                    if col_idx % cols == 0 {
                        ui.end_row();
                    }

                    if draw_dock_btn(
                        ui,
                        "+YZ",
                        true,
                        false,
                        true,
                        &format!(
                            "{}\n{}",
                            lang.t("+ Side Panel (YZ)"),
                            lang.t("Add vertical side panel from active material")
                        ),
                    ) {
                        *active_tab = AppTab::Designer4View;
                        on_before_mutate(project);
                        let id = add_quick_part(
                            project,
                            *active_material_id,
                            PartOrientation::VerticalYZ,
                            lang,
                        );
                        *selected_part = Some(id);
                        viewport_ctrl.selected_parts = vec![id];
                    }
                    col_idx += 1;
                    if col_idx % cols == 0 {
                        ui.end_row();
                    }

                    if draw_dock_btn(
                        ui,
                        "+XZ",
                        true,
                        false,
                        true,
                        &format!(
                            "{}\n{}",
                            lang.t("+ Front/Back (XZ)"),
                            lang.t("Add frontal panel (door, rail, or back panel)")
                        ),
                    ) {
                        *active_tab = AppTab::Designer4View;
                        on_before_mutate(project);
                        let id = add_quick_part(
                            project,
                            *active_material_id,
                            PartOrientation::FrontalXZ,
                            lang,
                        );
                        *selected_part = Some(id);
                        viewport_ctrl.selected_parts = vec![id];
                    }
                    col_idx += 1;
                    if col_idx % cols != 0 {
                        ui.end_row();
                    } else {
                        ui.end_row();
                    }
                });

            draw_dock_divider(ui);

            // 3. Selection Actions (Rotate 90°, Dup, Copy, Paste, Delete)
            let has_sel = selected_part.is_some() || !viewport_ctrl.selected_parts.is_empty();
            egui::Grid::new("left_dock_actions")
                .num_columns(cols)
                .spacing([4.0, 4.0])
                .show(ui, |ui| {
                    let mut col_idx = 0;

                    if draw_dock_btn(
                        ui,
                        FA_ROTATE,
                        false,
                        false,
                        selected_part.is_some(),
                        lang.t("menu.edit.rotate_90"),
                    ) {
                        if let Some(sel_id) = *selected_part {
                            if let Some(idx) = project.parts.iter().position(|p| p.id == sel_id) {
                                on_before_mutate(project);
                                project.parts[idx].yaw_deg =
                                    (project.parts[idx].yaw_deg + 90.0) % 360.0;
                            }
                        }
                    }
                    col_idx += 1;
                    if col_idx % cols == 0 {
                        ui.end_row();
                    }

                    if draw_dock_btn(
                        ui,
                        FA_CLONE,
                        false,
                        false,
                        has_sel,
                        &crate::i18n::adapt_shortcuts(lang.t("📋 Duplicate (Ctrl+D)")),
                    ) {
                        perform_duplicate_selection(
                            project,
                            selected_part,
                            &mut viewport_ctrl.selected_parts,
                            status_msg,
                            lang,
                            &mut on_before_mutate,
                        );
                    }
                    col_idx += 1;
                    if col_idx % cols == 0 {
                        ui.end_row();
                    }

                    if draw_dock_btn(
                        ui,
                        FA_COPY,
                        false,
                        false,
                        has_sel,
                        &crate::i18n::adapt_shortcuts(lang.t("📄 Copy (Ctrl+C)")),
                    ) {
                        perform_copy_selection(
                            ui.ctx(),
                            project,
                            *selected_part,
                            &viewport_ctrl.selected_parts,
                            clipboard,
                            status_msg,
                            lang,
                        );
                    }
                    col_idx += 1;
                    if col_idx % cols == 0 {
                        ui.end_row();
                    }

                    if draw_dock_btn(
                        ui,
                        FA_PASTE,
                        false,
                        false,
                        !clipboard.is_empty(),
                        &crate::i18n::adapt_shortcuts(lang.t("📥 Paste (Ctrl+V)")),
                    ) {
                        perform_paste_clipboard(
                            project,
                            selected_part,
                            &mut viewport_ctrl.selected_parts,
                            clipboard,
                            status_msg,
                            lang,
                            &mut on_before_mutate,
                        );
                    }
                    col_idx += 1;
                    if col_idx % cols == 0 {
                        ui.end_row();
                    }

                    if draw_dock_btn(
                        ui,
                        FA_TRASH,
                        false,
                        false,
                        has_sel,
                        lang.t("🗑 Delete (Del)"),
                    ) {
                        perform_delete_selection(
                            project,
                            selected_part,
                            &mut viewport_ctrl.selected_parts,
                            status_msg,
                            lang,
                            &mut on_before_mutate,
                        );
                    }
                    col_idx += 1;
                    if col_idx % cols != 0 {
                        ui.end_row();
                    } else {
                        ui.end_row();
                    }
                });

            draw_dock_divider(ui);

            // 4. Viewport Management (4x Equal Views, Fit to Window)
            egui::Grid::new("left_dock_views")
                .num_columns(cols)
                .spacing([4.0, 4.0])
                .show(ui, |ui| {
                    let mut col_idx = 0;

                    if draw_dock_btn(
                        ui,
                        FA_TABLE_CELLS_LARGE,
                        false,
                        false,
                        true,
                        lang.t("⊞ Equal Views"),
                    ) {
                        *active_tab = AppTab::Designer4View;
                        viewport_ctrl.equalize_viewports();
                    }
                    col_idx += 1;
                    if col_idx % cols == 0 {
                        ui.end_row();
                    }

                    if draw_dock_btn(
                        ui,
                        FA_EXPAND,
                        false,
                        false,
                        true,
                        lang.t("📐 Fit to Window"),
                    ) {
                        *active_tab = AppTab::Designer4View;
                        viewport_ctrl.fit_all_to_window(project);
                    }
                    col_idx += 1;
                    if col_idx % cols != 0 {
                        ui.end_row();
                    } else {
                        ui.end_row();
                    }
                });
        });
}

/// Renders precision top transform bar (grid & snapping controls + fast numeric coordinate inputs X/Y/Z, dimensions L/W/T, and rotation)
pub fn ui_top_transform_toolbar(
    ui: &mut egui::Ui,
    project: &mut Project,
    selected_part: &mut Option<PartId>,
    active_material_id: &mut MaterialId,
    viewport_ctrl: &mut QuadViewportController,
    mut on_before_mutate: impl FnMut(&Project),
) {
    let lang = viewport_ctrl.lang;
    let mm_suffix = lang.t(" mm");

    ui.spacing_mut().interact_size.y = 22.0;
    ui.spacing_mut().item_spacing.x = 6.0;

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().interact_size.y = 22.0;
        ui.spacing_mut().item_spacing.x = 6.0;

        // 1. Active material selection (for new parts)
        ui.label(RichText::new(lang.t("Material:")).strong());
        let active_mat_name = project
            .get_material(*active_material_id)
            .map(|m| format!("{} ({:.0}{})", m.name, m.thickness_mm, mm_suffix))
            .unwrap_or_else(|| lang.t("Select...").to_string());
        egui::ComboBox::from_id_salt("active_mat_combo")
            .selected_text(active_mat_name)
            .width(195.0)
            .show_ui(ui, |ui| {
                for m in &project.materials {
                    let label = format!("{} ({:.0}{})", m.name, m.thickness_mm, mm_suffix);
                    ui.selectable_value(active_material_id, m.id, label);
                }
            });

        ui.separator();

        // 2. Snapping and grid step (mm)
        ui.checkbox(&mut viewport_ctrl.grid_snap_enabled, lang.t("Grid"));
        ui.add(
            math_drag_value(&mut viewport_ctrl.grid_step_mm)
                .speed(1.0)
                .range(0.5..=200.0)
                .suffix(mm_suffix),
        )
        .on_hover_text(lang.t("Grid snap step in millimeters (1, 5, 9, 18, 50 mm)"));

        for preset in [1.0_f32, 9.0, 18.0, 50.0] {
            let is_sel = (viewport_ctrl.grid_step_mm - preset).abs() < 0.1;
            let preset_str = format!("{:.0}", preset);
            if ui
                .selectable_label(is_sel, &preset_str)
                .on_hover_text(lang.tf("fmt.set_grid_step_tip", &[&preset_str]))
                .clicked()
            {
                viewport_ctrl.grid_step_mm = preset;
            }
        }

        ui.separator();

        ui.checkbox(
            &mut viewport_ctrl.edge_snap_enabled,
            lang.t("Snap to edges"),
        )
        .on_hover_text(lang.t("Snap part edges flush against neighboring parts"));

        ui.checkbox(
            &mut viewport_ctrl.show_distance_callouts,
            lang.t("Gaps (mm)"),
        )
        .on_hover_text(lang.t("Display distances from selected part to neighboring parts"));

        ui.separator();

        if ui
            .button(lang.t("⊞ Equal Views"))
            .on_hover_text(
                lang.t("Reset splitters and equalize all 4 viewport window sizes (50% / 50%)"),
            )
            .clicked()
        {
            viewport_ctrl.equalize_viewports();
        }

        if ui
            .button(lang.t("📐 Fit to Window"))
            .on_hover_text(lang.t("Fit the entire model into all 2D projections and the 3D view"))
            .clicked()
        {
            viewport_ctrl.fit_all_to_window(project);
        }
    });

    ui.add_space(2.0);
    ui.separator();
    ui.add_space(2.0);

    // Second row of transform bar: numeric input of coordinates X, Y, Z and dimensions Length / Width / Thickness / Rotation for selected part or group
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().interact_size.y = 22.0;
        ui.spacing_mut().item_spacing.x = 6.0;

        if viewport_ctrl.selected_parts.len() > 1 {
            // Multi-part selection: show combined bounding box and allow synchronized shifting along X, Y, Z
            let mut bmin = Vec3::splat(f32::INFINITY);
            let mut bmax = Vec3::splat(f32::NEG_INFINITY);
            let mut valid_count = 0usize;
            for pid in &viewport_ctrl.selected_parts {
                if let Some(p) = project.parts.iter().find(|p| p.id == *pid) {
                    let t = project.get_material_thickness(p.material_id);
                    let (p_min, p_max) = p.world_aabb(t);
                    bmin = bmin.min(p_min);
                    bmax = bmax.max(p_max);
                    valid_count += 1;
                }
            }

            if valid_count > 1 {
                let is_dark = ui.visuals().dark_mode;
                let grp_col = if is_dark {
                    Color32::from_rgb(120, 210, 255)
                } else {
                    Color32::from_rgb(15, 95, 175)
                };
                let grp_text = lang.tf("fmt.group_count_badge", &[&valid_count]);
                ui.label(RichText::new(grp_text).strong().color(grp_col));
                ui.separator();

                ui.label(RichText::new(lang.t("Group position (mm):")).strong());
                let mut gpos = bmin;
                let mut pos_changed = false;

                ui.label("X:");
                pos_changed |= ui
                    .add(
                        math_drag_value(&mut gpos.x)
                            .speed(1.0)
                            .range(-10000.0..=10000.0)
                            .suffix(mm_suffix),
                    )
                    .changed();
                ui.label("Y:");
                pos_changed |= ui
                    .add(
                        math_drag_value(&mut gpos.y)
                            .speed(1.0)
                            .range(-10000.0..=10000.0)
                            .suffix(mm_suffix),
                    )
                    .changed();
                ui.label("Z:");
                pos_changed |= ui
                    .add(
                        math_drag_value(&mut gpos.z)
                            .speed(1.0)
                            .range(-10000.0..=10000.0)
                            .suffix(mm_suffix),
                    )
                    .changed();

                if pos_changed {
                    on_before_mutate(project);
                    let delta = gpos - bmin;
                    let sel_set = viewport_ctrl.selected_parts.clone();
                    for p in &mut project.parts {
                        if sel_set.contains(&p.id) && !p.locked {
                            p.position_mm += delta;
                        }
                    }
                }

                ui.separator();
                let span = bmax - bmin;
                let span_text = lang.tf(
                    "fmt.group_bbox_span",
                    &[
                        &format!("{:.0}", span.x),
                        &format!("{:.0}", span.y),
                        &format!("{:.0}", span.z),
                    ],
                );
                let span_col = if is_dark {
                    Color32::from_rgb(160, 225, 175)
                } else {
                    Color32::from_rgb(20, 125, 55)
                };
                ui.label(RichText::new(span_text).color(span_col));
                return;
            }
        }

        if let Some(sel_id) = *selected_part {
            if let Some(idx) = project.parts.iter().position(|p| p.id == sel_id) {
                let is_dark = ui.visuals().dark_mode;
                let mat_id = project.parts[idx].material_id;
                let thick = project.get_material_thickness(mat_id);
                let (cut_l, cut_w) = project.parts[idx].cut_dimensions_mm(&project.edge_bandings);

                let block_lbl = lang.tf("fmt.part_title_badge", &[&project.parts[idx].name]);
                let badge_col = if is_dark {
                    Color32::from_rgb(255, 215, 90)
                } else {
                    Color32::from_rgb(175, 95, 0)
                };
                ui.label(RichText::new(block_lbl).strong().color(badge_col));
                ui.separator();

                // Coordinates X, Y, Z (mm from scene origin)
                ui.label(RichText::new(lang.t("Position (mm):")).strong());
                let mut pos = project.parts[idx].position_mm;
                let mut pos_changed = false;

                ui.label("X:");
                pos_changed |= ui
                    .add(
                        math_drag_value(&mut pos.x)
                            .speed(1.0)
                            .range(-10000.0..=10000.0)
                            .suffix(mm_suffix),
                    )
                    .changed();
                ui.label("Y:");
                pos_changed |= ui
                    .add(
                        math_drag_value(&mut pos.y)
                            .speed(1.0)
                            .range(-10000.0..=10000.0)
                            .suffix(mm_suffix),
                    )
                    .changed();
                ui.label("Z:");
                pos_changed |= ui
                    .add(
                        math_drag_value(&mut pos.z)
                            .speed(1.0)
                            .range(-10000.0..=10000.0)
                            .suffix(mm_suffix),
                    )
                    .changed();

                if pos_changed {
                    project.parts[idx].position_mm = pos;
                }

                ui.separator();

                // Part dimensions (Length x Width x Thickness) in millimeters
                ui.label(RichText::new(lang.t("Full size w/ edge:")).strong());
                let mut len = project.parts[idx].length_mm;
                let mut wid = project.parts[idx].width_mm;
                let mut size_changed = false;

                ui.label(lang.t("Length (L):"));
                size_changed |= ui
                    .add(
                        math_drag_value(&mut len)
                            .speed(1.0)
                            .range(10.0..=5000.0)
                            .suffix(mm_suffix),
                    )
                    .changed();

                ui.label(lang.t("Width (W):"));
                size_changed |= ui
                    .add(
                        math_drag_value(&mut wid)
                            .speed(1.0)
                            .range(10.0..=5000.0)
                            .suffix(mm_suffix),
                    )
                    .changed();

                if size_changed {
                    project.parts[idx].length_mm = len;
                    project.parts[idx].width_mm = wid;
                }

                // Thickness field (edits material thickness of part or shows 18 mm)
                let mut cur_thick = thick;
                ui.label(lang.t("Thick (T):"));
                if ui
                    .add(
                        math_drag_value(&mut cur_thick)
                            .speed(0.5)
                            .range(1.0..=100.0)
                            .suffix(mm_suffix),
                    )
                    .on_hover_text(lang.t("Sheet material thickness of the part (mm)"))
                    .changed()
                {
                    if let Some(m) = project.materials.iter_mut().find(|m| m.id == mat_id) {
                        m.thickness_mm = cur_thick;
                    }
                }

                let cut_badge = lang.tf(
                    "fmt.cut_size_badge",
                    &[&format!("{:.1}", cut_l), &format!("{:.1}", cut_w)],
                );
                let cut_col = if is_dark {
                    Color32::from_rgb(120, 225, 150)
                } else {
                    Color32::from_rgb(18, 130, 55)
                };
                ui.label(RichText::new(cut_badge).strong().color(cut_col))
                    .on_hover_text(
                        lang.t("Net saw cutting dimensions after subtracting edge banding thickness"),
                    );

                ui.separator();

                // Rotation around Z axis
                ui.label("∠Z:");
                ui.add(
                    math_drag_value(&mut project.parts[idx].yaw_deg)
                        .speed(1.0)
                        .range(-360.0..=360.0)
                        .suffix("°"),
                )
                .on_hover_text(
                    lang.t("Part rotation around the vertical Z axis (for corner units)"),
                );

                if ui.button("↺ 90°").clicked() {
                    project.parts[idx].yaw_deg = (project.parts[idx].yaw_deg + 90.0) % 360.0;
                }
            }
        } else {
            ui.label(
                RichText::new(lang.t(
                    "No part selected — click a block, drag a selection box in 2D, or choose \"➕ Create Part by Box\"",
                ))
                .italics()
                .color(Color32::GRAY),
            );
        }
    });
}

/// System clipboard prefix for copying parts between QuadFurn tabs and windows
pub const CLIPBOARD_PREFIX: &str = "QUADFURN_CLIPBOARD_V1:";

/// Clipboard payload for copying parts between different tabs (and windows) of furniture products
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ClipboardPart {
    pub part: PartBlock,
    pub source_material: Option<Material>,
    pub source_edges: [Option<crate::model::EdgeBanding>; 4],
    #[serde(default)]
    pub source_group_name: Option<String>,
}

/// Collects all selected parts (single or group) into a list of `ClipboardPart`
pub fn copy_selected_parts_to_clipboard(
    project: &Project,
    selected_part: Option<PartId>,
    selected_parts: &[PartId],
) -> Vec<ClipboardPart> {
    let mut ids: Vec<PartId> = if !selected_parts.is_empty() {
        selected_parts.to_vec()
    } else {
        selected_part.into_iter().collect()
    };
    if let Some(sid) = selected_part {
        if !ids.contains(&sid) {
            ids.push(sid);
        }
    }

    let mut result = Vec::new();
    for orig in &project.parts {
        if ids.contains(&orig.id) {
            let src_mat = project.get_material(orig.material_id).cloned();
            let src_edges = [
                orig.edges.l1.and_then(|id| project.get_edge(id).cloned()),
                orig.edges.l2.and_then(|id| project.get_edge(id).cloned()),
                orig.edges.w1.and_then(|id| project.get_edge(id).cloned()),
                orig.edges.w2.and_then(|id| project.get_edge(id).cloned()),
            ];
            let src_group_name = orig
                .group_id
                .and_then(|gid| project.groups.iter().find(|g| g.id == gid))
                .map(|g| g.name.clone());
            result.push(ClipboardPart {
                part: orig.clone(),
                source_material: src_mat,
                source_edges: src_edges,
                source_group_name: src_group_name,
            });
        }
    }
    result
}

/// Pastes a list of copied parts from clipboard into the target furniture product
/// (automatically matching or adding materials, edge bandings, and groups, preserving relative layout)
pub fn paste_parts_into_project(project: &mut Project, clips: &[ClipboardPart]) -> Vec<PartId> {
    if clips.is_empty() {
        return Vec::new();
    }

    // Calculate uniform offset for the entire pasted group if at least one part
    // overlaps in coordinates with an existing part in the target project
    let mut group_offset = Vec3::ZERO;
    for step in 0..20 {
        let candidate_offset = Vec3::new(18.0 * step as f32, 0.0, 18.0 * step as f32);
        let has_collision = clips.iter().any(|clip| {
            let target_pos = clip.part.position_mm + candidate_offset;
            project
                .parts
                .iter()
                .any(|p| p.position_mm.distance(target_pos) < 1.0)
        });
        if !has_collision {
            group_offset = candidate_offset;
            break;
        }
    }

    let mut pasted_ids = Vec::with_capacity(clips.len());

    for clip in clips {
        // 1. Match material (by name and thickness, or create in target project)
        let target_mat_id = if let Some(src_mat) = &clip.source_material {
            if let Some(existing) = project.materials.iter().find(|m| {
                m.name == src_mat.name && (m.thickness_mm - src_mat.thickness_mm).abs() < 0.05
            }) {
                existing.id
            } else {
                let new_mid = project.alloc_id();
                let mut m_copy = src_mat.clone();
                m_copy.id = new_mid;
                project.materials.push(m_copy);
                new_mid
            }
        } else {
            project.materials.first().map(|m| m.id).unwrap_or(1)
        };

        // 2. Match edge bandings on 4 sides
        let mut map_edge = |opt_edge: &Option<crate::model::EdgeBanding>| -> Option<EdgeId> {
            let src_e = opt_edge.as_ref()?;
            if let Some(existing) = project.edge_bandings.iter().find(|e| {
                e.name == src_e.name && (e.thickness_mm - src_e.thickness_mm).abs() < 0.05
            }) {
                Some(existing.id)
            } else {
                let new_eid = project.alloc_id();
                let mut e_copy = src_e.clone();
                e_copy.id = new_eid;
                project.edge_bandings.push(e_copy);
                Some(new_eid)
            }
        };

        let new_edges = PartEdges {
            l1: map_edge(&clip.source_edges[0]),
            l2: map_edge(&clip.source_edges[1]),
            w1: map_edge(&clip.source_edges[2]),
            w2: map_edge(&clip.source_edges[3]),
        };

        // 3. Match group by name (or assign to first group of project)
        let target_group_id = if let Some(gname) = &clip.source_group_name {
            if let Some(existing_grp) = project.groups.iter().find(|g| &g.name == gname) {
                Some(existing_grp.id)
            } else {
                let new_gid = project.alloc_id();
                project.groups.push(PartGroup {
                    id: new_gid,
                    name: gname.clone(),
                    visible: true,
                    locked: false,
                    expanded: true,
                });
                Some(new_gid)
            }
        } else {
            project.groups.first().map(|g| g.id)
        };

        let new_id = project.alloc_id();
        let mut new_part = clip.part.clone();
        new_part.id = new_id;
        new_part.material_id = target_mat_id;
        new_part.edges = new_edges;
        new_part.group_id = target_group_id;
        new_part.position_mm += group_offset;
        project.parts.push(new_part);
        pasted_ids.push(new_id);
    }

    pasted_ids
}

/// Normalizes a part's `group_id`: if it points to a group that does not exist in `groups`, treats it as `None` (ungrouped)
fn effective_part_group(group_id: Option<GroupId>, groups: &[PartGroup]) -> Option<GroupId> {
    group_id.filter(|gid| groups.iter().any(|g| g.id == *gid))
}

/// Checks whether the currently selected group or part can be moved up (`dir = -1`) or down (`dir = 1`).
/// Parts can only move within their current group (never jumping across groups).
pub fn can_move_hierarchy_item(
    project: &Project,
    selected_group: Option<GroupId>,
    selected_part: Option<PartId>,
    dir: i32,
) -> bool {
    if let Some(gid) = selected_group {
        if let Some(g_idx) = project.groups.iter().position(|g| g.id == gid) {
            return if dir < 0 {
                g_idx > 0
            } else {
                g_idx + 1 < project.groups.len()
            };
        }
        return false;
    }

    if let Some(pid) = selected_part {
        if let Some(cur_pos) = project.parts.iter().position(|p| p.id == pid) {
            let cur_grp = effective_part_group(project.parts[cur_pos].group_id, &project.groups);
            if dir < 0 {
                return (0..cur_pos).rev().any(|i| {
                    effective_part_group(project.parts[i].group_id, &project.groups) == cur_grp
                });
            } else {
                return ((cur_pos + 1)..project.parts.len()).any(|i| {
                    effective_part_group(project.parts[i].group_id, &project.groups) == cur_grp
                });
            }
        }
    }

    false
}

/// Moves the currently selected group up/down among `project.groups`, or moves the currently
/// selected part up/down strictly within its own group in `project.parts` (never crosses group boundaries).
pub fn move_hierarchy_item(
    project: &mut Project,
    selected_group: Option<GroupId>,
    selected_part: Option<PartId>,
    dir: i32,
) -> bool {
    if let Some(gid) = selected_group {
        if let Some(g_idx) = project.groups.iter().position(|g| g.id == gid) {
            if dir < 0 && g_idx > 0 {
                project.groups.swap(g_idx, g_idx - 1);
                return true;
            } else if dir > 0 && g_idx + 1 < project.groups.len() {
                project.groups.swap(g_idx, g_idx + 1);
                return true;
            }
        }
        return false;
    }

    if let Some(pid) = selected_part {
        if let Some(cur_pos) = project.parts.iter().position(|p| p.id == pid) {
            let cur_grp = effective_part_group(project.parts[cur_pos].group_id, &project.groups);
            if dir < 0 {
                if let Some(prev_pos) = (0..cur_pos).rev().find(|&i| {
                    effective_part_group(project.parts[i].group_id, &project.groups) == cur_grp
                }) {
                    project.parts.swap(cur_pos, prev_pos);
                    return true;
                }
            } else if let Some(next_pos) = ((cur_pos + 1)..project.parts.len()).find(|&i| {
                effective_part_group(project.parts[i].group_id, &project.groups) == cur_grp
            }) {
                project.parts.swap(cur_pos, next_pos);
                return true;
            }
        }
    }

    false
}

/// Right inspector for parts, groups, materials, and interactive edge banding diagram
/// Features collapsible sections, a vertically resizable parts tree with a draggable splitter,
/// and header-level Move Up (`↑`) / Move Down (`↓`) buttons for groups and parts.
#[allow(clippy::too_many_arguments)]
pub fn ui_right_inspector(
    ui: &mut egui::Ui,
    project: &mut Project,
    selected_part: &mut Option<PartId>,
    selected_parts: &mut Vec<PartId>,
    selected_group: &mut Option<GroupId>,
    hierarchy_tree_height: &mut f32,
    active_material_id: &mut MaterialId,
    new_group_name_buf: &mut String,
    clipboard: &mut Vec<ClipboardPart>,
    status_msg: &mut String,
    lang: AppLanguage,
    mut on_tree_height_changed: impl FnMut(),
    mut on_before_mutate: impl FnMut(&Project),
) {
    let mm_suffix = lang.t(" mm");
    ui.heading(lang.t("Part Inspector"));

    // 1. Collapsible Groups & Parts Tree Section
    egui::CollapsingHeader::new(RichText::new(lang.t("inspector.hierarchy_section")).strong())
        .id_salt("inspector_hierarchy_collapse")
        .default_open(true)
        .show(ui, |ui| {
            // New group creation row
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(new_group_name_buf)
                        .hint_text(lang.t("New group name..."))
                        .desired_width(140.0),
                );
                if ui.button(lang.t("📁 +Group")).clicked() {
                    on_before_mutate(project);
                    let gid = project.alloc_id();
                    let gname = if new_group_name_buf.trim().is_empty() {
                        format!("{} {}", lang.t("Group"), project.groups.len() + 1)
                    } else {
                        new_group_name_buf.trim().to_string()
                    };
                    project.groups.push(PartGroup {
                        id: gid,
                        name: gname,
                        visible: true,
                        locked: false,
                        expanded: true,
                    });
                    *selected_group = Some(gid);
                    new_group_name_buf.clear();
                }
            });

            // Header action strip: Move Up (↑), Move Down (↓), Copy, Paste, Duplicate, Delete
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                let can_up =
                    can_move_hierarchy_item(project, *selected_group, *selected_part, -1);
                let can_down =
                    can_move_hierarchy_item(project, *selected_group, *selected_part, 1);

                if ui
                    .add_enabled(
                        can_up,
                        egui::Button::new(FA_ARROW_UP),
                    )
                    .on_hover_text(lang.t("inspector.move_up_tip"))
                    .clicked()
                {
                    on_before_mutate(project);
                    move_hierarchy_item(project, *selected_group, *selected_part, -1);
                }

                if ui
                    .add_enabled(
                        can_down,
                        egui::Button::new(FA_ARROW_DOWN),
                    )
                    .on_hover_text(lang.t("inspector.move_down_tip"))
                    .clicked()
                {
                    on_before_mutate(project);
                    move_hierarchy_item(project, *selected_group, *selected_part, 1);
                }

                ui.separator();

                let has_sel = selected_part.is_some() || !selected_parts.is_empty();
                if ui
                    .add_enabled(has_sel, egui::Button::new(FA_COPY))
                    .on_hover_text(format!(
                        "{}\n{}",
                        crate::i18n::adapt_shortcuts(lang.t("📄 Copy (Ctrl+C)")),
                        lang.t("Copy selected parts to clipboard (can be pasted into another tab or window)")
                    ))
                    .clicked()
                {
                    perform_copy_selection(
                        ui.ctx(),
                        project,
                        *selected_part,
                        selected_parts,
                        clipboard,
                        status_msg,
                        lang,
                    );
                }

                if ui
                    .add_enabled(!clipboard.is_empty(), egui::Button::new(FA_PASTE))
                    .on_hover_text(format!(
                        "{}\n{}",
                        crate::i18n::adapt_shortcuts(lang.t("📥 Paste (Ctrl+V)")),
                        lang.t("Paste copied parts into the current furniture product")
                    ))
                    .clicked()
                {
                    perform_paste_clipboard(
                        project,
                        selected_part,
                        selected_parts,
                        clipboard,
                        status_msg,
                        lang,
                        &mut on_before_mutate,
                    );
                }

                if ui
                    .add_enabled(has_sel, egui::Button::new(FA_CLONE))
                    .on_hover_text(crate::i18n::adapt_shortcuts(
                        lang.t("📋 Duplicate (Ctrl+D)"),
                    ))
                    .clicked()
                {
                    perform_duplicate_selection(
                        project,
                        selected_part,
                        selected_parts,
                        status_msg,
                        lang,
                        &mut on_before_mutate,
                    );
                }

                if ui
                    .add_enabled(has_sel, egui::Button::new(FA_TRASH))
                    .on_hover_text(lang.t("🗑 Delete (Del)"))
                    .clicked()
                {
                    perform_delete_selection(
                        project,
                        selected_part,
                        selected_parts,
                        status_msg,
                        lang,
                        &mut on_before_mutate,
                    );
                }
            });

            ui.separator();

            // Vertically resizable tree of groups and parts
            let clamped_h = hierarchy_tree_height.clamp(100.0, 800.0);
            egui::ScrollArea::vertical()
                .id_salt("hierarchy_scroll")
                .auto_shrink([false, false])
                .max_height(clamped_h)
                .show(ui, |ui| {
                    let group_ids: Vec<GroupId> = project.groups.iter().map(|g| g.id).collect();
                    for gid in group_ids {
                        let Some(g_idx) = project.groups.iter().position(|g| g.id == gid) else {
                            continue;
                        };
                        let is_grp_selected = *selected_group == Some(gid);
                        ui.horizontal(|ui| {
                            let exp_icon = if project.groups[g_idx].expanded {
                                "▼"
                            } else {
                                "▶"
                            };
                            if ui.small_button(exp_icon).clicked() {
                                project.groups[g_idx].expanded = !project.groups[g_idx].expanded;
                            }
                            let vis_icon = if project.groups[g_idx].visible {
                                "👁"
                            } else {
                                "🚫"
                            };
                            if ui
                                .small_button(vis_icon)
                                .on_hover_text(lang.t("Show/hide entire group"))
                                .clicked()
                            {
                                project.groups[g_idx].visible = !project.groups[g_idx].visible;
                            }
                            let lock_icon = if project.groups[g_idx].locked {
                                "🔒"
                            } else {
                                "🔓"
                            };
                            if ui
                                .small_button(lock_icon)
                                .on_hover_text(lang.t("Lock/unlock group against mouse dragging"))
                                .clicked()
                            {
                                project.groups[g_idx].locked = !project.groups[g_idx].locked;
                            }

                            if ui
                                .selectable_label(is_grp_selected, "📁")
                                .on_hover_text(lang.t(
                                    "Select all parts in this group to move them together",
                                ))
                                .clicked()
                            {
                                *selected_group = Some(gid);
                                let grp_parts: Vec<PartId> = project
                                    .parts
                                    .iter()
                                    .filter(|p| p.group_id == Some(gid) && p.visible && !p.locked)
                                    .map(|p| p.id)
                                    .collect();
                                if !grp_parts.is_empty() {
                                    *selected_part = grp_parts.first().copied();
                                    *selected_parts = grp_parts;
                                }
                            }

                            let avail_w = (ui.available_width() - 8.0).max(95.0);
                            let name_resp = ui.add(
                                egui::TextEdit::singleline(&mut project.groups[g_idx].name)
                                    .desired_width(avail_w),
                            );
                            if name_resp.clicked() || name_resp.gained_focus() {
                                *selected_group = Some(gid);
                            }
                        });

                        if project.groups[g_idx].expanded {
                            ui.indent(("grp_indent", gid), |ui| {
                                for p_idx in 0..project.parts.len() {
                                    if project.parts[p_idx].group_id == Some(gid) {
                                        render_part_tree_row(
                                            ui,
                                            project,
                                            p_idx,
                                            selected_part,
                                            selected_parts,
                                            selected_group,
                                            lang,
                                        );
                                    }
                                }
                            });
                        }
                    }

                    // Ungrouped parts
                    let ungrouped_indices: Vec<usize> = project
                        .parts
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| {
                            effective_part_group(p.group_id, &project.groups).is_none()
                        })
                        .map(|(i, _)| i)
                        .collect();

                    if !ungrouped_indices.is_empty() {
                        ui.label(RichText::new(lang.t("📁 Ungrouped:")).strong());
                        for p_idx in ungrouped_indices {
                            render_part_tree_row(
                                ui,
                                project,
                                p_idx,
                                selected_part,
                                selected_parts,
                                selected_group,
                                lang,
                            );
                        }
                    }
                });

            // Draggable horizontal splitter handle right below the tree
            let splitter_w = ui.available_width().max(120.0);
            let (split_rect, split_resp) =
                ui.allocate_exact_size(EguiVec2::new(splitter_w, 10.0), Sense::drag());
            if split_resp.hovered() || split_resp.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
            }
            let grip_col = if split_resp.hovered() || split_resp.dragged() {
                ui.visuals().widgets.active.fg_stroke.color
            } else {
                ui.visuals().widgets.noninteractive.bg_stroke.color
            };
            ui.painter().line_segment(
                [
                    Pos2::new(split_rect.left() + 8.0, split_rect.center().y),
                    Pos2::new(split_rect.right() - 8.0, split_rect.center().y),
                ],
                Stroke::new(1.2, grip_col),
            );
            ui.painter().text(
                split_rect.center(),
                egui::Align2::CENTER_CENTER,
                "══ ↕ ══",
                FontId::proportional(9.5),
                grip_col,
            );
            if split_resp.dragged() {
                *hierarchy_tree_height =
                    (*hierarchy_tree_height + split_resp.drag_delta().y).clamp(100.0, 800.0);
            }
            if split_resp.drag_stopped() {
                on_tree_height_changed();
            }
            split_resp.on_hover_text(lang.t("inspector.resize_tree_tip"));
        });

    ui.separator();

    // Lower part of Right Inspector: collapsible detailed sections for selected part
    egui::ScrollArea::vertical()
        .id_salt("part_props_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if let Some(sel_id) = *selected_part {
                if let Some(idx) = project.parts.iter().position(|p| p.id == sel_id) {
                    // 2. Collapsible Part Properties Section
                    egui::CollapsingHeader::new(RichText::new(lang.t("Part Properties")).strong())
                        .id_salt("inspector_props_collapse")
                        .default_open(true)
                        .show(ui, |ui| {
                            egui::Grid::new("part_inspector_grid")
                                .num_columns(2)
                                .spacing([8.0, 6.0])
                                .show(ui, |ui| {
                                    ui.label(lang.t("Name:"));
                                    ui.text_edit_singleline(&mut project.parts[idx].name);
                                    ui.end_row();

                                    ui.label(lang.t("Group:"));
                                    let no_grp_txt = lang.t("No group");
                                    let cur_grp_name = project.parts[idx]
                                        .group_id
                                        .and_then(|gid| {
                                            project.groups.iter().find(|g| g.id == gid)
                                        })
                                        .map(|g| g.name.clone())
                                        .unwrap_or_else(|| no_grp_txt.to_string());
                                    egui::ComboBox::from_id_salt("part_group_combo")
                                        .selected_text(cur_grp_name)
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut project.parts[idx].group_id,
                                                None,
                                                no_grp_txt,
                                            );
                                            for g in &project.groups {
                                                ui.selectable_value(
                                                    &mut project.parts[idx].group_id,
                                                    Some(g.id),
                                                    &g.name,
                                                );
                                            }
                                        });
                                    ui.end_row();

                                    ui.label(lang.t("Orientation:"));
                                    egui::ComboBox::from_id_salt("part_orient_combo")
                                        .selected_text(
                                            project.parts[idx].orientation.label_i18n(lang),
                                        )
                                        .show_ui(ui, |ui| {
                                            for orient in [
                                                PartOrientation::HorizontalXY,
                                                PartOrientation::VerticalYZ,
                                                PartOrientation::FrontalXZ,
                                            ] {
                                                ui.selectable_value(
                                                    &mut project.parts[idx].orientation,
                                                    orient,
                                                    orient.label_i18n(lang),
                                                );
                                            }
                                        });
                                    ui.end_row();

                                    ui.label(lang.t("Material:"));
                                    let cur_mat_id = project.parts[idx].material_id;
                                    let cur_mat_label = project
                                        .get_material(cur_mat_id)
                                        .map(|m| {
                                            format!(
                                                "{} ({:.0}{})",
                                                m.name, m.thickness_mm, mm_suffix
                                            )
                                        })
                                        .unwrap_or_else(|| lang.t("None").to_string());
                                    egui::ComboBox::from_id_salt("part_mat_combo")
                                        .selected_text(cur_mat_label)
                                        .show_ui(ui, |ui| {
                                            for m in &project.materials {
                                                if ui
                                                    .selectable_value(
                                                        &mut project.parts[idx].material_id,
                                                        m.id,
                                                        format!(
                                                            "{} ({:.0}{})",
                                                            m.name, m.thickness_mm, mm_suffix
                                                        ),
                                                    )
                                                    .clicked()
                                                {
                                                    *active_material_id = m.id;
                                                }
                                            }
                                        });
                                    ui.end_row();

                                    ui.label(lang.t("Full Size (L×W):"));
                                    ui.horizontal(|ui| {
                                        ui.add(
                                            math_drag_value(&mut project.parts[idx].length_mm)
                                                .speed(1.0)
                                                .range(10.0..=5000.0)
                                                .suffix(mm_suffix),
                                        );
                                        ui.label("×");
                                        ui.add(
                                            math_drag_value(&mut project.parts[idx].width_mm)
                                                .speed(1.0)
                                                .range(10.0..=5000.0)
                                                .suffix(mm_suffix),
                                        );
                                        if ui
                                            .small_button("⇄")
                                            .on_hover_text(lang.t("Swap Length and Width"))
                                            .clicked()
                                        {
                                            on_before_mutate(project);
                                            let part = &mut project.parts[idx];
                                            std::mem::swap(&mut part.length_mm, &mut part.width_mm);
                                        }
                                    });
                                    ui.end_row();

                                    let (cut_l, cut_w) = project.parts[idx]
                                        .cut_dimensions_mm(&project.edge_bandings);
                                    let cut_col = if ui.visuals().dark_mode {
                                        Color32::from_rgb(110, 235, 145)
                                    } else {
                                        Color32::from_rgb(18, 130, 55)
                                    };
                                    ui.label(lang.t("Cut Size:"));
                                    ui.label(
                                        RichText::new(format!(
                                            "{:.1} × {:.1}{}",
                                            cut_l, cut_w, mm_suffix
                                        ))
                                        .strong()
                                        .color(cut_col),
                                    );
                                    ui.end_row();
                                });
                        });

                    // 3. Collapsible Grain & Orientation Section
                    egui::CollapsingHeader::new(
                        RichText::new(lang.t("inspector.grain_section")).strong(),
                    )
                    .id_salt("inspector_grain_collapse")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.checkbox(
                            &mut project.parts[idx].grain_fixed,
                            lang.t("🌲 Lock grain direction (do not rotate on cutting sheet)"),
                        );
                        if project.parts[idx].grain_fixed {
                            ui.checkbox(
                                &mut project.parts[idx].grain_along_width,
                                lang.t("Run grain across width (along Width W instead of Length L)"),
                            );
                        }
                    });

                    // 4. Collapsible Edge Banding Section
                    egui::CollapsingHeader::new(
                        RichText::new(lang.t("inspector.edges_section")).strong(),
                    )
                    .id_salt("inspector_edges_collapse")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(lang.t("Quick:"));
                            if let Some(first_edge) = project.edge_bandings.first().map(|e| e.id) {
                                if ui.small_button(lang.t("All 4 Edges")).clicked() {
                                    on_before_mutate(project);
                                    project.parts[idx].edges = PartEdges {
                                        l1: Some(first_edge),
                                        l2: Some(first_edge),
                                        w1: Some(first_edge),
                                        w2: Some(first_edge),
                                    };
                                }
                            }
                            if ui.small_button(lang.t("Clear Edges")).clicked() {
                                on_before_mutate(project);
                                project.parts[idx].edges = PartEdges::default();
                            }
                        });

                        // Interactive graphic diagram of part with 4 edges (L1, L2, W1, W2)
                        draw_interactive_edge_diagram(ui, project, idx, lang);

                        // Dropdown combos for each of the 4 edges
                        let edges_clone = project.edge_bandings.clone();
                        egui::Grid::new("edges_select_grid")
                            .num_columns(2)
                            .spacing([8.0, 4.0])
                            .show(ui, |ui| {
                                edge_combo_row(
                                    ui,
                                    lang.t("L1 (Length 1 / Front):"),
                                    "edge_l1",
                                    &mut project.parts[idx].edges.l1,
                                    &edges_clone,
                                    lang,
                                );
                                edge_combo_row(
                                    ui,
                                    lang.t("L2 (Length 2 / Back):"),
                                    "edge_l2",
                                    &mut project.parts[idx].edges.l2,
                                    &edges_clone,
                                    lang,
                                );
                                edge_combo_row(
                                    ui,
                                    lang.t("W1 (Width 1 / End 1):"),
                                    "edge_w1",
                                    &mut project.parts[idx].edges.w1,
                                    &edges_clone,
                                    lang,
                                );
                                edge_combo_row(
                                    ui,
                                    lang.t("W2 (Width 2 / End 2):"),
                                    "edge_w2",
                                    &mut project.parts[idx].edges.w2,
                                    &edges_clone,
                                    lang,
                                );
                            });
                    });
                }
            } else {
                ui.label(
                    RichText::new(lang.t(
                        "Select a part in the tree above or click it in the 2D/3D view to configure dimensions, material, and edge bandings.",
                    ))
                    .color(Color32::GRAY),
                );
            }
        });
}

fn render_part_tree_row(
    ui: &mut egui::Ui,
    project: &mut Project,
    p_idx: usize,
    selected_part: &mut Option<PartId>,
    selected_parts: &mut Vec<PartId>,
    selected_group: &mut Option<GroupId>,
    lang: AppLanguage,
) {
    let pid = project.parts[p_idx].id;
    let is_sel = *selected_part == Some(pid) || selected_parts.contains(&pid);
    let mat_rgb = project
        .get_material(project.parts[p_idx].material_id)
        .map(|m| m.color_rgb)
        .unwrap_or([180, 180, 180]);

    ui.horizontal(|ui| {
        let vis_icon = if project.parts[p_idx].visible {
            "👁"
        } else {
            "🚫"
        };
        if ui.small_button(vis_icon).clicked() {
            project.parts[p_idx].visible = !project.parts[p_idx].visible;
        }

        // Colored swatch of part material
        let (swatch_rect, _) = ui.allocate_exact_size(EguiVec2::splat(11.0), Sense::hover());
        ui.painter().rect_filled(
            swatch_rect,
            2.0,
            Color32::from_rgb(mat_rgb[0], mat_rgb[1], mat_rgb[2]),
        );

        let label_text = format!(
            "{} ({:.0}×{:.0})",
            project.parts[p_idx].name, project.parts[p_idx].length_mm, project.parts[p_idx].width_mm
        );
        let resp = ui
            .selectable_label(is_sel, label_text)
            .on_hover_text(crate::i18n::adapt_shortcuts(
                lang.t("Click to select part; Shift/Ctrl + click to toggle in multi-selection"),
            ));
        if resp.clicked() {
            *selected_group = None;
            let multi_mod =
                ui.input(|i| i.modifiers.shift || i.modifiers.command || i.modifiers.ctrl);
            if multi_mod {
                if let Some(prev) = *selected_part {
                    if !selected_parts.contains(&prev) {
                        selected_parts.push(prev);
                    }
                }
                if let Some(pos) = selected_parts.iter().position(|&id| id == pid) {
                    selected_parts.remove(pos);
                    *selected_part = selected_parts.last().copied();
                } else {
                    selected_parts.push(pid);
                    *selected_part = Some(pid);
                }
            } else {
                *selected_part = Some(pid);
                *selected_parts = vec![pid];
            }
        }
    });
}

fn edge_combo_row(
    ui: &mut egui::Ui,
    label: &str,
    salt: &str,
    edge_slot: &mut Option<EdgeId>,
    edges_lib: &[crate::model::EdgeBanding],
    lang: AppLanguage,
) {
    let mm_suffix = lang.t(" mm");
    let none_txt = lang.t("None (no edge)");
    ui.label(label);
    let selected_text = edge_slot
        .and_then(|id| edges_lib.iter().find(|e| e.id == id))
        .map(|e| format!("{} ({:.1}{})", e.name, e.thickness_mm, mm_suffix))
        .unwrap_or_else(|| none_txt.to_string());

    egui::ComboBox::from_id_salt(salt)
        .selected_text(selected_text)
        .show_ui(ui, |ui| {
            ui.selectable_value(edge_slot, None, none_txt);
            for e in edges_lib {
                ui.selectable_value(
                    edge_slot,
                    Some(e.id),
                    format!("{} ({:.1}{})", e.name, e.thickness_mm, mm_suffix),
                );
            }
        });
    ui.end_row();
}

/// Draws an interactive diagram of a part rectangle in the right inspector:
/// clicking on edge L1 / L2 / W1 / W2 cycles through available edge bandings (None -> 2.0 mm -> 0.8 mm -> 0.4 mm)
fn draw_interactive_edge_diagram(
    ui: &mut egui::Ui,
    project: &mut Project,
    part_idx: usize,
    lang: AppLanguage,
) {
    let (rect, resp) = ui.allocate_exact_size(EguiVec2::new(250.0, 110.0), Sense::click());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 4.0, Color32::from_rgb(24, 28, 34));
    let inner = rect.shrink2(EguiVec2::new(38.0, 22.0));
    painter.rect_filled(inner, 0.0, Color32::from_rgb(45, 52, 62));
    painter.rect_stroke(inner, 0.0, Stroke::new(1.0, Color32::from_gray(110)));

    let part = &project.parts[part_idx];
    let (cut_l, cut_w) = part.cut_dimensions_mm(&project.edge_bandings);

    // Grain direction arrow inside rectangle
    if part.grain_fixed {
        let c = inner.center();
        if !part.grain_along_width {
            painter.line_segment(
                [
                    Pos2::new(inner.left() + 18.0, c.y),
                    Pos2::new(inner.right() - 18.0, c.y),
                ],
                Stroke::new(1.2, Color32::from_rgb(170, 190, 215)),
            );
        } else {
            painter.line_segment(
                [
                    Pos2::new(c.x, inner.top() + 10.0),
                    Pos2::new(c.x, inner.bottom() - 10.0),
                ],
                Stroke::new(1.2, Color32::from_rgb(170, 190, 215)),
            );
        }
    }

    let center_txt = lang.tf(
        "fmt.edge_diag_center",
        &[
            &format!("{:.0}", part.length_mm),
            &format!("{:.0}", part.width_mm),
            &format!("{:.1}", cut_l),
            &format!("{:.1}", cut_w),
        ],
    );
    painter.text(
        inner.center(),
        egui::Align2::CENTER_CENTER,
        center_txt,
        FontId::monospace(10.0),
        Color32::WHITE,
    );

    let mm_unit = lang.t("mm");
    let none_unit = lang.t("none");
    let draw_edge_bar =
        |p1: Pos2, p2: Pos2, label_pos: Pos2, title: &str, eid: Option<EdgeId>| {
            if let Some(id) = eid {
                if let Some(e) = project.get_edge(id) {
                    let col = Color32::from_rgb(e.color_rgb[0], e.color_rgb[1], e.color_rgb[2]);
                    painter.line_segment([p1, p2], Stroke::new(4.0, col));
                    painter.text(
                        label_pos,
                        egui::Align2::CENTER_CENTER,
                        format!("{}: {:.1}{}", title, e.thickness_mm, mm_unit),
                        FontId::proportional(10.5),
                        col,
                    );
                    return;
                }
            }
            painter.text(
                label_pos,
                egui::Align2::CENTER_CENTER,
                format!("{}: {}", title, none_unit),
                FontId::proportional(10.0),
                Color32::GRAY,
            );
        };

    // Top edge = L1, Bottom = L2, Left = W1, Right = W2
    draw_edge_bar(
        inner.left_top(),
        inner.right_top(),
        Pos2::new(inner.center().x, rect.top() + 10.0),
        lang.t("L1"),
        part.edges.l1,
    );
    draw_edge_bar(
        inner.left_bottom(),
        inner.right_bottom(),
        Pos2::new(inner.center().x, rect.bottom() - 10.0),
        lang.t("L2"),
        part.edges.l2,
    );
    draw_edge_bar(
        inner.left_top(),
        inner.left_bottom(),
        Pos2::new(rect.left() + 19.0, inner.center().y),
        lang.t("W1"),
        part.edges.w1,
    );
    draw_edge_bar(
        inner.right_top(),
        inner.right_bottom(),
        Pos2::new(rect.right() - 19.0, inner.center().y),
        lang.t("W2"),
        part.edges.w2,
    );

    // Clicking edge area on mini-diagram cycles edge banding
    if resp.clicked() {
        if let Some(mp) = resp.interact_pointer_pos() {
            let cycle_edge =
                |cur: Option<EdgeId>, lib: &[crate::model::EdgeBanding]| -> Option<EdgeId> {
                    if lib.is_empty() {
                        return None;
                    }
                    match cur {
                        None => Some(lib[0].id),
                        Some(id) => {
                            let pos = lib.iter().position(|e| e.id == id).unwrap_or(0);
                            if pos + 1 < lib.len() {
                                Some(lib[pos + 1].id)
                            } else {
                                None
                            }
                        }
                    }
                };
            let lib = project.edge_bandings.clone();
            let d_top = (mp.y - inner.top()).abs();
            let d_bot = (mp.y - inner.bottom()).abs();
            let d_left = (mp.x - inner.left()).abs();
            let d_right = (mp.x - inner.right()).abs();
            let min_d = d_top.min(d_bot).min(d_left).min(d_right);
            if min_d == d_top {
                project.parts[part_idx].edges.l1 =
                    cycle_edge(project.parts[part_idx].edges.l1, &lib);
            } else if min_d == d_bot {
                project.parts[part_idx].edges.l2 =
                    cycle_edge(project.parts[part_idx].edges.l2, &lib);
            } else if min_d == d_left {
                project.parts[part_idx].edges.w1 =
                    cycle_edge(project.parts[part_idx].edges.w1, &lib);
            } else {
                project.parts[part_idx].edges.w2 =
                    cycle_edge(project.parts[part_idx].edges.w2, &lib);
            }
        }
    }
}

/// "Sheet Materials & Edge Banding Catalog" tab (configure 18 mm thickness, RGB color, sheet dimensions, load PNG/JPG/WEBP texture, and delete with confirmation)
#[allow(clippy::too_many_arguments)]
pub fn ui_materials_manager_tab(
    ui: &mut egui::Ui,
    project: &mut Project,
    active_material_id: &mut MaterialId,
    loaded_textures: &mut HashMap<MaterialId, TextureHandle>,
    texture_warnings: &mut HashMap<MaterialId, String>,
    pending_delete_material: &mut Option<MaterialId>,
    pending_delete_edge: &mut Option<EdgeId>,
    status_msg: &mut String,
    lang: AppLanguage,
) {
    let mm_suffix = lang.t(" mm");
    ui.heading(lang.t("Sheet Materials & Edge Banding Catalog"));
    ui.separator();

    ui.label(RichText::new(lang.t("Project sheet materials (all dimensions in millimeters):")).strong());
    ui.add_space(2.0);

    ui.horizontal(|ui| {
        if ui.button(lang.t("➕ Add Sheet Material")).clicked() {
            let id = project.alloc_id();
            let color_rgb = crate::model::generate_next_material_color(&project.materials, id);
            project.materials.push(Material {
                id,
                name: crate::glossary::format_new_material_name(lang, project.materials.len() + 1),
                thickness_mm: 18.0,
                color_rgb,
                texture_path: None,
                texture_relative_path: None,
                texture_base64: None,
                sheet_length_mm: 2800.0,
                sheet_width_mm: 2070.0,
                default_grain_fixed: true,
                price_per_sheet: 0.0,
            });
        }

        ui.separator();
        ui.checkbox(
            &mut project.embed_textures,
            lang.t("📦 Embed textures into project file (.qfurn)"),
        )
        .on_hover_text(lang.t(
            "When saving, all active textures will be embedded directly into the project file (Base64). The file can be transferred to any PC without losing images.",
        ));
    });

    let can_delete_mat = project.materials.len() > 1;

    egui::Grid::new("materials_table_grid")
        .striped(true)
        .num_columns(9)
        .spacing([12.0, 8.0])
        .show(ui, |ui| {
            ui.label(RichText::new(lang.t("Material Name")).strong());
            ui.label(RichText::new(lang.t("Thickness (mm)")).strong());
            ui.label(RichText::new(lang.t("3D Color")).strong());
            ui.label(RichText::new(lang.t("Texture (PNG/JPG/WEBP)")).strong());
            ui.label(RichText::new(lang.t("Sheet Length (mm)")).strong());
            ui.label(RichText::new(lang.t("Sheet Width (mm)")).strong());
            ui.label(RichText::new(lang.t("Default Grain")).strong());
            ui.label(RichText::new(lang.t("Cost per sheet")).strong());
            ui.label(RichText::new(lang.t("Actions")).strong());
            ui.end_row();

            for m in &mut project.materials {
                ui.add(egui::TextEdit::singleline(&mut m.name).desired_width(220.0));
                ui.add(
                    math_drag_value(&mut m.thickness_mm)
                        .speed(0.5)
                        .range(1.0..=100.0)
                        .suffix(mm_suffix),
                );
                ui.color_edit_button_srgb(&mut m.color_rgb);

                ui.horizontal(|ui| {
                    if ui.button(lang.t("📂 Load...")).clicked() {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter(
                                    lang.t("Texture Images (PNG, JPG, WEBP)"),
                                    &["png", "jpg", "jpeg", "webp"],
                                )
                                .pick_file()
                            {
                                match load_texture_from_path(ui.ctx(), &path, &m.name, lang) {
                                    Ok(tex_info) => {
                                        loaded_textures.insert(m.id, tex_info.handle);
                                        m.texture_path = Some(path.clone());
                                        if let Some(fname) = path.file_name() {
                                            m.texture_relative_path =
                                                Some(fname.to_string_lossy().to_string());
                                        }
                                        if project.embed_textures {
                                            if let Ok(bytes) = std::fs::read(&path) {
                                                if bytes.len() <= 60 * 1024 * 1024 {
                                                    use base64::Engine;
                                                    m.texture_base64 = Some(
                                                        base64::engine::general_purpose::STANDARD
                                                            .encode(&bytes),
                                                    );
                                                }
                                            }
                                        } else {
                                            m.texture_base64 = None;
                                        }

                                        if let Some(ref w) = tex_info.warning {
                                            texture_warnings.insert(m.id, w.clone());
                                            *status_msg = lang.tf(
                                                "fmt.texture_loaded_warn",
                                                &[&m.name, &path.display(), &w],
                                            );
                                        } else {
                                            texture_warnings.remove(&m.id);
                                            *status_msg = lang.tf(
                                                "fmt.texture_loaded",
                                                &[&m.name, &path.display()],
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        *status_msg = e;
                                    }
                                }
                            }
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            crate::web::pick_texture_file_web(m.id);
                        }
                    }
                    if m.texture_path.is_some()
                        || m.texture_base64.is_some()
                        || m.texture_relative_path.is_some()
                    {
                        if ui.small_button(lang.t("✖ Reset")).clicked() {
                            m.texture_path = None;
                            m.texture_relative_path = None;
                            m.texture_base64 = None;
                            loaded_textures.remove(&m.id);
                            texture_warnings.remove(&m.id);
                        }
                        if let Some(tex) = loaded_textures.get(&m.id) {
                            ui.image((tex.id(), egui::vec2(18.0, 18.0)));
                        }
                        let active_label = if m.texture_base64.is_some() {
                            lang.t("✔ Texture (embedded)")
                        } else {
                            lang.t("✔ Texture active")
                        };
                        let active_col = if ui.visuals().dark_mode {
                            Color32::from_rgb(110, 235, 145)
                        } else {
                            Color32::from_rgb(25, 140, 60)
                        };
                        ui.label(RichText::new(active_label).color(active_col).strong());
                        if let Some(warn) = texture_warnings.get(&m.id) {
                            ui.label(
                                RichText::new(format!("⚠️ {}", warn))
                                    .color(Color32::from_rgb(240, 180, 50))
                                    .small(),
                            );
                        }
                    } else {
                        ui.label(
                            RichText::new(lang.t("Solid color (no photo)")).color(Color32::GRAY),
                        );
                    }
                });

                ui.add(
                    math_drag_value(&mut m.sheet_length_mm)
                        .speed(10.0)
                        .range(500.0..=6000.0)
                        .suffix(mm_suffix),
                );
                ui.add(
                    math_drag_value(&mut m.sheet_width_mm)
                        .speed(10.0)
                        .range(500.0..=4000.0)
                        .suffix(mm_suffix),
                );
                ui.checkbox(&mut m.default_grain_fixed, lang.t("Lock grain"));

                ui.add(
                    math_drag_value_f64(&mut m.price_per_sheet)
                        .speed(10.0)
                        .range(0.0..=10_000_000.0),
                );

                if ui
                    .add_enabled(
                        can_delete_mat,
                        egui::Button::new(
                            RichText::new(lang.t("🗑 Delete"))
                                .color(Color32::from_rgb(245, 95, 95)),
                        ),
                    )
                    .on_hover_text(if can_delete_mat {
                        lang.t("Delete this material (confirmation required)")
                    } else {
                        lang.t("Cannot delete the only material in the project")
                    })
                    .clicked()
                {
                    *pending_delete_material = Some(m.id);
                }
                ui.end_row();
            }
        });

    // Modal confirmation dialog for material deletion: "Are you sure?"
    if let Some(del_id) = *pending_delete_material {
        let mat_name = project
            .get_material(del_id)
            .map(|m| m.name.clone())
            .unwrap_or_else(|| lang.t("Material").to_string());
        let used_parts_count = project
            .parts
            .iter()
            .filter(|p| p.material_id == del_id)
            .count();

        let mut close_dialog = false;
        let mut confirm_delete = false;

        egui::Window::new(lang.t("Confirm Material Deletion"))
            .collapsible(false)
            .resizable(false)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ui.ctx().screen_rect().center())
            .show(ui.ctx(), |ui| {
                ui.set_width(380.0);
                ui.vertical_centered(|ui| {
                    ui.add_space(6.0);
                    let q_txt = crate::glossary::format_material_delete_confirm(lang, &mat_name);
                    ui.label(RichText::new(q_txt).strong().size(15.0));
                    if used_parts_count > 0 {
                        ui.add_space(6.0);
                        let warn_txt =
                            crate::glossary::format_material_delete_warn(lang, used_parts_count);
                        ui.colored_label(Color32::from_rgb(240, 175, 50), warn_txt);
                    }
                    ui.add_space(14.0);

                    let btn_del_w = 160.0;
                    let btn_cancel_w = 90.0;
                    let spacing = 12.0;
                    let total_w = btn_del_w + spacing + btn_cancel_w;
                    let pad = (ui.available_width() - total_w).max(0.0) / 2.0;

                    ui.horizontal(|ui| {
                        ui.add_space(pad);
                        if ui
                            .add_sized(
                                [btn_del_w, 28.0],
                                egui::Button::new(
                                    RichText::new(lang.t("✔ Yes, delete"))
                                        .strong()
                                        .color(Color32::from_rgb(250, 90, 90)),
                                ),
                            )
                            .clicked()
                        {
                            confirm_delete = true;
                            close_dialog = true;
                        }
                        ui.add_space(spacing);
                        if ui
                            .add_sized([btn_cancel_w, 28.0], egui::Button::new(lang.t("Cancel")))
                            .clicked()
                        {
                            close_dialog = true;
                        }
                    });
                    ui.add_space(6.0);
                });
            });

        if confirm_delete && project.materials.len() > 1 {
            project.materials.retain(|m| m.id != del_id);
            loaded_textures.remove(&del_id);
            let fallback_id = project.materials[0].id;
            if *active_material_id == del_id {
                *active_material_id = fallback_id;
            }
            for p in &mut project.parts {
                if p.material_id == del_id {
                    p.material_id = fallback_id;
                }
            }
            *status_msg = crate::glossary::format_material_deleted(lang, &mat_name);
        }

        if close_dialog {
            *pending_delete_material = None;
        }
    }

    ui.add_space(16.0);
    ui.separator();
    ui.label(RichText::new(lang.t("Edge banding catalog (thickness is subtracted when calculating net saw cutting size):")).strong());
    ui.add_space(2.0);

    ui.horizontal(|ui| {
        if ui.button(lang.t("➕ Add Edge Banding Type")).clicked() {
            let id = project.alloc_id();
            let color_rgb = crate::model::generate_next_edge_color(&project.edge_bandings, id);
            project.edge_bandings.push(crate::model::EdgeBanding {
                id,
                name: crate::glossary::format_new_edge_name(lang, project.edge_bandings.len() + 1),
                thickness_mm: 1.0,
                color_rgb,
                price_per_m: 0.0,
            });
        }
    });

    egui::Grid::new("edges_table_grid")
        .striped(true)
        .num_columns(5)
        .spacing([16.0, 8.0])
        .show(ui, |ui| {
            ui.label(RichText::new(lang.t("Edge Banding Name")).strong());
            ui.label(RichText::new(lang.t("Thickness (mm)")).strong());
            ui.label(RichText::new(lang.t("Drawing Marker Color")).strong());
            ui.label(RichText::new(lang.t("Cost per meter")).strong());
            ui.label(RichText::new(lang.t("Actions")).strong());
            ui.end_row();

            for e in &mut project.edge_bandings {
                ui.add(egui::TextEdit::singleline(&mut e.name).desired_width(180.0));
                ui.add(
                    math_drag_value(&mut e.thickness_mm)
                        .speed(0.1)
                        .range(0.0..=10.0)
                        .suffix(mm_suffix),
                );
                ui.color_edit_button_srgb(&mut e.color_rgb);
                ui.add(
                    math_drag_value_f64(&mut e.price_per_m)
                        .speed(1.0)
                        .range(0.0..=100_000.0),
                );
                if ui
                    .button(
                        RichText::new(lang.t("🗑 Delete")).color(Color32::from_rgb(245, 95, 95)),
                    )
                    .on_hover_text(lang.t("Delete this edge banding type (confirmation required)"))
                    .clicked()
                {
                    *pending_delete_edge = Some(e.id);
                }
                ui.end_row();
            }
        });

    // Modal confirmation dialog for edge banding deletion
    if let Some(del_edge_id) = *pending_delete_edge {
        let edge_name = project
            .get_edge(del_edge_id)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| lang.t("Edge Banding").to_string());
        let used_sides_count: usize = project
            .parts
            .iter()
            .map(|p| {
                usize::from(p.edges.l1 == Some(del_edge_id))
                    + usize::from(p.edges.l2 == Some(del_edge_id))
                    + usize::from(p.edges.w1 == Some(del_edge_id))
                    + usize::from(p.edges.w2 == Some(del_edge_id))
            })
            .sum();

        let mut close_edge_dialog = false;
        let mut confirm_edge_delete = false;

        egui::Window::new(lang.t("Confirm Edge Banding Deletion"))
            .collapsible(false)
            .resizable(false)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ui.ctx().screen_rect().center())
            .show(ui.ctx(), |ui| {
                ui.set_width(380.0);
                ui.vertical_centered(|ui| {
                    ui.add_space(6.0);
                    let q_txt = crate::glossary::format_edge_delete_confirm(lang, &edge_name);
                    ui.label(RichText::new(q_txt).strong().size(15.0));
                    if used_sides_count > 0 {
                        ui.add_space(6.0);
                        let warn_txt =
                            crate::glossary::format_edge_delete_warn(lang, used_sides_count);
                        ui.colored_label(Color32::from_rgb(240, 175, 50), warn_txt);
                    }
                    ui.add_space(14.0);

                    let btn_del_w = 160.0;
                    let btn_cancel_w = 90.0;
                    let spacing = 12.0;
                    let total_w = btn_del_w + spacing + btn_cancel_w;
                    let pad = (ui.available_width() - total_w).max(0.0) / 2.0;

                    ui.horizontal(|ui| {
                        ui.add_space(pad);
                        if ui
                            .add_sized(
                                [btn_del_w, 28.0],
                                egui::Button::new(
                                    RichText::new(lang.t("✔ Yes, delete"))
                                        .strong()
                                        .color(Color32::from_rgb(250, 90, 90)),
                                ),
                            )
                            .clicked()
                        {
                            confirm_edge_delete = true;
                            close_edge_dialog = true;
                        }
                        ui.add_space(spacing);
                        if ui
                            .add_sized([btn_cancel_w, 28.0], egui::Button::new(lang.t("Cancel")))
                            .clicked()
                        {
                            close_edge_dialog = true;
                        }
                    });
                    ui.add_space(6.0);
                });
            });

        if confirm_edge_delete {
            project.edge_bandings.retain(|e| e.id != del_edge_id);
            for p in &mut project.parts {
                if p.edges.l1 == Some(del_edge_id) {
                    p.edges.l1 = None;
                }
                if p.edges.l2 == Some(del_edge_id) {
                    p.edges.l2 = None;
                }
                if p.edges.w1 == Some(del_edge_id) {
                    p.edges.w1 = None;
                }
                if p.edges.w2 == Some(del_edge_id) {
                    p.edges.w2 = None;
                }
            }
            *status_msg = crate::glossary::format_edge_deleted(lang, &edge_name);
        }

        if close_edge_dialog {
            *pending_delete_edge = None;
        }
    }
}

pub struct LoadedTextureInfo {
    pub handle: TextureHandle,
    pub warning: Option<String>,
}

pub const MAX_RECOMMENDED_TEXTURE_DIM: u32 = 4096;

pub fn load_texture_from_memory(
    ctx: &egui::Context,
    img_bytes: &[u8],
    name: &str,
    lang: AppLanguage,
) -> Result<LoadedTextureInfo, String> {
    let dyn_img = image::load_from_memory(img_bytes)
        .map_err(|e| lang.tf("fmt.tex_decode_err", &[&e]))?;

    let (orig_w, orig_h) = (dyn_img.width(), dyn_img.height());
    let gpu_limit = ctx.input(|i| i.max_texture_side) as u32;
    let max_allowed = if gpu_limit > 0 {
        gpu_limit.min(MAX_RECOMMENDED_TEXTURE_DIM)
    } else {
        MAX_RECOMMENDED_TEXTURE_DIM
    };

    let (final_img, warning) = if orig_w > max_allowed || orig_h > max_allowed {
        let resized = dyn_img.resize(
            max_allowed,
            max_allowed,
            image::imageops::FilterType::Triangle,
        );
        let (nw, nh) = (resized.width(), resized.height());
        let warn_msg = lang.tf(
            "fmt.tex_downscaled_warn",
            &[&orig_w, &orig_h, &max_allowed, &nw, &nh],
        );
        (resized, Some(warn_msg))
    } else {
        (dyn_img, None)
    };

    let rgba = final_img.to_rgba8();
    let dims = [rgba.width() as usize, rgba.height() as usize];
    let color_img =
        egui::ColorImage::from_rgba_unmultiplied(dims, rgba.as_flat_samples().as_slice());
    let handle = ctx.load_texture(name, color_img, egui::TextureOptions::LINEAR);

    Ok(LoadedTextureInfo { handle, warning })
}

pub fn load_texture_from_path(
    ctx: &egui::Context,
    path: &std::path::Path,
    name: &str,
    lang: AppLanguage,
) -> Result<LoadedTextureInfo, String> {
    if let Ok(meta) = std::fs::metadata(path) {
        if meta.len() > 60 * 1024 * 1024 {
            let mb = format!("{:.1}", meta.len() as f64 / (1024.0 * 1024.0));
            return Err(lang.tf("fmt.tex_too_large_err", &[&mb]));
        }
    }

    let img_bytes =
        std::fs::read(path).map_err(|e| lang.tf("fmt.tex_read_err", &[&e]))?;

    load_texture_from_memory(ctx, &img_bytes, name, lang)
}

/// Finds and loads material texture, checking embedded Base64 (Option B),
/// then relative paths from project file (Option A: next to .qfurn or in textures/ subfolder),
/// and original absolute path
pub fn resolve_and_load_material_texture(
    ctx: &egui::Context,
    m: &mut Material,
    project_path: Option<&std::path::Path>,
    lang: AppLanguage,
) -> Result<LoadedTextureInfo, String> {
    // 1. Priority #1 (Option B): Base64 texture embedded in project file
    if let Some(ref b64) = m.texture_base64 {
        if !b64.trim().is_empty() {
            use base64::Engine;
            if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(b64.trim()) {
                return load_texture_from_memory(ctx, &bytes, &m.name, lang);
            }
        }
    }

    // 2. Priority #2 (Option A): Search relative to project directory
    let project_dir = project_path
        .and_then(|p| p.parent())
        .unwrap_or(std::path::Path::new("."));

    let mut candidate_paths: Vec<std::path::PathBuf> = Vec::new();

    if let Some(ref rel) = m.texture_relative_path {
        candidate_paths.push(project_dir.join(rel));
        candidate_paths.push(project_dir.join("textures").join(rel));
    }

    if let Some(ref p) = m.texture_path {
        let fname = crate::model::extract_cross_platform_filename(&p.to_string_lossy());
        if !fname.is_empty() {
            candidate_paths.push(project_dir.join(&fname));
            candidate_paths.push(project_dir.join("textures").join(&fname));
        }
        candidate_paths.push(p.clone());
    }

    for candidate in candidate_paths {
        if candidate.is_file() {
            let res = load_texture_from_path(ctx, &candidate, &m.name, lang)?;
            m.texture_path = Some(candidate);
            return Ok(res);
        }
    }

    Err(lang.tf("fmt.tex_not_found_err", &[&m.name]))
}

/// "Specification and Export" tab
pub fn ui_specification_tab(
    ui: &mut egui::Ui,
    project: &mut Project,
    status_msg: &mut String,
    lang: AppLanguage,
) {
    let mm_unit = lang.t("mm");
    ui.horizontal(|ui| {
        ui.heading(lang.t("Parts List (Specification)"));
        ui.separator();
        ui.checkbox(
            &mut project.cutting_settings.group_identical_parts,
            lang.t("Group identical parts (merge into Qty: N pcs)"),
        );

        ui.separator();
        if ui
            .button(lang.t("💾 Export to CSV (Excel)"))
            .on_hover_text(
                lang.t("Save full specification table with full and net saw dimensions to .csv"),
            )
            .clicked()
        {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name("spec_quadfurn.csv")
                    .add_filter(lang.t("CSV Spreadsheet"), &["csv"])
                    .save_file()
                {
                    match export_spec_to_csv_file_i18n(project, &path, lang) {
                        Ok(()) => {
                            *status_msg = lang.tf("fmt.spec_exported_csv", &[&path.display()]);
                        }
                        Err(e) => *status_msg = e,
                    }
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                let clean_name = project
                    .name
                    .replace(['/', '\\', '?', '%', '*', ':', '|', '"', '<', '>'], "_");
                let def_name = format!("{}_specification.csv", clean_name);
                let csv = crate::export::generate_csv_string_i18n(project, lang);
                let mut data = vec![0xEF, 0xBB, 0xBF];
                data.extend_from_slice(csv.as_bytes());
                crate::web::download_file(&def_name, "text/csv;charset=utf-8", &data);
                *status_msg = lang.tf("fmt.spec_downloaded_csv", &[&def_name]);
            }
        }

        if ui
            .button(lang.t("📄 Export to PDF"))
            .on_hover_text(lang.t(
                "Save complete product passport to .pdf (Specification + 4 Designer Views + Cutting Layouts + Materials & Edges)",
            ))
            .clicked()
        {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let default_pdf_name = format!("spec_{}.pdf", project.name.replace(' ', "_"));
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name(&default_pdf_name)
                    .add_filter(lang.t("PDF Document"), &["pdf"])
                    .save_file()
                {
                    match export_spec_to_pdf_file(project, &path, lang) {
                        Ok(()) => {
                            *status_msg = lang.tf("fmt.spec_exported_pdf", &[&path.display()]);
                        }
                        Err(e) => *status_msg = e,
                    }
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                let clean_name = project
                    .name
                    .replace(['/', '\\', '?', '%', '*', ':', '|', '"', '<', '>'], "_");
                let def_name = format!("{}_drawings.pdf", clean_name);
                match crate::export::generate_spec_pdf_bytes(project, lang) {
                    Ok(bytes) => {
                        crate::web::download_file(&def_name, "application/pdf", &bytes);
                        *status_msg = lang.tf("fmt.spec_downloaded_pdf", &[&def_name]);
                    }
                    Err(e) => *status_msg = e,
                }
            }
        }

        if ui
            .button(lang.t("📋 Copy Cutting List"))
            .on_hover_text(lang.t("Copy net saw cutting list to clipboard"))
            .clicked()
        {
            let text = generate_cutting_clipboard_text_i18n(project, lang);
            ui.ctx().copy_text(text);
            *status_msg = lang
                .t("Parts cutting list (saw dimensions & edges) copied to clipboard!")
                .to_string();
        }
    });

    ui.separator();

    let rows = build_specification_i18n(project, lang);

    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Grid::new("spec_full_grid")
                .striped(true)
                .num_columns(12)
                .spacing([14.0, 6.0])
                .show(ui, |ui| {
                    let is_dark = ui.visuals().dark_mode;
                    let full_dim_color = if is_dark {
                        Color32::from_rgb(110, 235, 145)
                    } else {
                        Color32::from_rgb(25, 140, 60)
                    };

                    ui.label(RichText::new("№").strong());
                    ui.label(RichText::new(lang.t("Group")).strong());
                    ui.label(RichText::new(lang.t("Part Name")).strong());
                    ui.label(RichText::new(lang.t("Material")).strong());
                    ui.label(
                        RichText::new(lang.t("Full L×W (mm)"))
                            .strong()
                            .color(full_dim_color),
                    );
                    ui.label(RichText::new(lang.t("CUT L×W (mm)")).strong());
                    ui.label(RichText::new(lang.t("Qty")).strong());
                    ui.label(RichText::new(lang.t("Grain")).strong());
                    ui.label(RichText::new(lang.t("Edge L1")).strong());
                    ui.label(RichText::new(lang.t("Edge L2")).strong());
                    ui.label(RichText::new(lang.t("Edge W1")).strong());
                    ui.label(RichText::new(lang.t("Edge W2")).strong());
                    ui.end_row();

                    for r in &rows {
                        ui.label(format!("{}", r.index));
                        ui.label(&r.group_name);
                        ui.label(RichText::new(&r.name).strong());
                        ui.label(format!(
                            "{} ({:.0}{})",
                            r.material_name, r.thickness_mm, mm_unit
                        ));
                        ui.label(
                            RichText::new(format!("{:.1} × {:.1}", r.full_length_mm, r.full_width_mm))
                                .strong()
                                .color(full_dim_color),
                        );
                        ui.label(format!(
                            "{:.1} × {:.1}",
                            r.cut_length_mm, r.cut_width_mm
                        ));
                        let qty_lbl = format!("{} {}", r.quantity, lang.dict().unit_pcs);
                        ui.label(RichText::new(qty_lbl).strong());
                        ui.label(if r.grain_fixed {
                            lang.t("🌲 Locked (Along grain)")
                        } else {
                            lang.t("🔄 Free rotation")
                        });
                        ui.label(SpecRow::edge_label_i18n(
                            r.edges.l1,
                            &project.edge_bandings,
                            lang,
                        ));
                        ui.label(SpecRow::edge_label_i18n(
                            r.edges.l2,
                            &project.edge_bandings,
                            lang,
                        ));
                        ui.label(SpecRow::edge_label_i18n(
                            r.edges.w1,
                            &project.edge_bandings,
                            lang,
                        ));
                        ui.label(SpecRow::edge_label_i18n(
                            r.edges.w2,
                            &project.edge_bandings,
                            lang,
                        ));
                        ui.end_row();
                    }
                });
        });
}

/// "Cutting Layout" tab (Variant A: dedicated Left Sidebar replacing Tools dock, Right Parts table, Central full-size diagram)
pub fn ui_cutting_layout_tab(
    ctx: &egui::Context,
    project: &mut Project,
    selected_sheet_idx: &mut usize,
    hovered_part: &mut Option<PartId>,
    loaded_textures: &HashMap<MaterialId, TextureHandle>,
    status_msg: &mut String,
    lang: AppLanguage,
) {
    let report: CuttingReport = compute_cutting_layout(project);
    let d = lang.dict();

    // 1. LEFT SIDEBAR: Cutting Sheets Navigator (grouped by material)
    let is_dark = ctx.style().visuals.dark_mode;

    egui::SidePanel::left("cutting_sheets_sidebar")
        .default_width(240.0)
        .width_range(180.0..=450.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading(lang.t("Cutting sheets list"));
            });
            let sheets_count_color = if is_dark {
                Color32::from_rgb(255, 215, 90)
            } else {
                Color32::from_rgb(160, 110, 15)
            };
            ui.label(
                RichText::new(format!(
                    "{}: {} {}",
                    lang.t("Sheets to cut"),
                    report.sheets.len(),
                    d.unit_pcs
                ))
                .small()
                .color(sheets_count_color),
            );
            ui.separator();

            if report.sheets.is_empty() {
                ui.label(lang.t("No parts in the project for cutting yet."));
                return;
            }

            if *selected_sheet_idx >= report.sheets.len() {
                *selected_sheet_idx = 0;
            }

            egui::ScrollArea::vertical()
                .id_salt("cutting_sheets_scroll")
                .show(ui, |ui| {
                    // Group sheets by material
                    for mat in &project.materials {
                        let matching_indices: Vec<usize> = report
                            .sheets
                            .iter()
                            .enumerate()
                            .filter(|(_, s)| s.material_id == mat.id)
                            .map(|(i, _)| i)
                            .collect();

                        if matching_indices.is_empty() {
                            continue;
                        }

                        let avg_kpi: f64 = matching_indices
                            .iter()
                            .map(|&idx| report.sheets[idx].utilization_percent as f64)
                            .sum::<f64>()
                            / matching_indices.len() as f64;

                        ui.add_space(6.0);
                        // Material group header
                        ui.horizontal(|ui| {
                            let (swatch, _) = ui.allocate_exact_size(EguiVec2::new(12.0, 12.0), Sense::hover());
                            ui.painter().rect_filled(
                                swatch,
                                2.0,
                                Color32::from_rgb(mat.color_rgb[0], mat.color_rgb[1], mat.color_rgb[2]),
                            );
                            ui.painter().rect_stroke(swatch, 2.0, Stroke::new(1.0, Color32::from_gray(100)));
                            ui.label(RichText::new(&mat.name).strong());
                        });

                        ui.label(
                            RichText::new(format!(
                                "{:.0} {} | {} {} | КПИ: {:.1}%",
                                mat.thickness_mm,
                                d.unit_mm,
                                matching_indices.len(),
                                d.unit_sheets,
                                avg_kpi
                            ))
                            .small()
                            .color(Color32::GRAY),
                        );
                        ui.add_space(2.0);

                        for &sheet_idx in &matching_indices {
                            let sh = &report.sheets[sheet_idx];
                            let is_selected = *selected_sheet_idx == sheet_idx;

                            let kpi_color = if is_dark {
                                if sh.utilization_percent >= 85.0 {
                                    Color32::from_rgb(110, 235, 145)
                                } else if sh.utilization_percent >= 70.0 {
                                    Color32::from_rgb(255, 215, 90)
                                } else {
                                    Color32::from_rgb(255, 140, 90)
                                }
                            } else {
                                if sh.utilization_percent >= 85.0 {
                                    Color32::from_rgb(30, 145, 60)
                                } else if sh.utilization_percent >= 70.0 {
                                    Color32::from_rgb(180, 115, 15)
                                } else {
                                    Color32::from_rgb(210, 65, 30)
                                }
                            };

                            let card_resp = ui.selectable_label(
                                is_selected,
                                RichText::new(format!(
                                    "#{:<2} ({:.0}×{:.0}) — {:.1}%",
                                    sheet_idx + 1,
                                    sh.sheet_length_mm,
                                    sh.sheet_width_mm,
                                    sh.utilization_percent
                                ))
                                .color(if is_selected {
                                    if is_dark { Color32::WHITE } else { Color32::from_rgb(15, 20, 30) }
                                } else {
                                    kpi_color
                                }),
                            );

                            if card_resp.clicked() {
                                *selected_sheet_idx = sheet_idx;
                            }

                            card_resp.on_hover_ui(|ui| {
                                ui.label(format!("Лист #{} (Материал: {})", sheet_idx + 1, sh.material_name));
                                ui.label(format!("Формат: {:.0} × {:.0} {}", sh.sheet_length_mm, sh.sheet_width_mm, d.unit_mm));
                                ui.label(format!("Деталей на листе: {} {}", sh.placed_parts.len(), d.unit_pcs));
                                ui.label(format!("Коэффициент использования: {:.1}%", sh.utilization_percent));
                                ui.label(format!("Остатков (обрезков): {}", sh.remnants.len()));
                            });
                        }
                        ui.separator();
                    }
                });
        });

    // 2. RIGHT PANEL: Parts table on active sheet & remnants
    egui::SidePanel::right("cutting_parts_panel")
        .default_width(290.0)
        .width_range(220.0..=550.0)
        .resizable(true)
        .show(ctx, |ui| {
            if report.sheets.is_empty() {
                return;
            }

            let cur_idx = (*selected_sheet_idx).min(report.sheets.len() - 1);
            let active_sheet = &report.sheets[cur_idx];

            let sheet_parts_hdr = lang.tf(
                "fmt.sheet_parts_heading",
                &[
                    &(cur_idx + 1),
                    &active_sheet.placed_parts.len(),
                    &d.unit_pcs,
                ],
            );
            ui.add_space(4.0);
            ui.heading(sheet_parts_hdr);
            ui.label(
                RichText::new(lang.t("Net saw cutting sizes (after subtracting edge bandings):"))
                    .small()
                    .color(Color32::GRAY),
            );
            ui.separator();

            egui::ScrollArea::vertical()
                .id_salt("sheet_parts_list")
                .show(ui, |ui| {
                    egui::Grid::new("sheet_parts_grid")
                        .striped(true)
                        .num_columns(5)
                        .spacing([8.0, 4.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("№").strong());
                            ui.label(RichText::new(lang.t("Part")).strong());
                            ui.label(RichText::new(lang.t("Cut (mm)")).strong());
                            ui.label(RichText::new(lang.t("Full")).strong());
                            ui.label(RichText::new(lang.t("Rotation")).strong());
                            ui.end_row();

                            let hover_highlight = if is_dark {
                                Color32::from_rgb(0, 240, 255)
                            } else {
                                Color32::from_rgb(0, 130, 210)
                            };

                            let cut_dim_color = if is_dark {
                                Color32::from_rgb(110, 235, 145)
                            } else {
                                Color32::from_rgb(25, 135, 60)
                            };

                            for p in &active_sheet.placed_parts {
                                let is_part_hovered = *hovered_part == Some(p.part_id);

                                let num_text = RichText::new(format!("#{}", p.spec_index));
                                let num_resp = ui.label(if is_part_hovered {
                                    num_text.color(hover_highlight).strong()
                                } else {
                                    num_text
                                });

                                let name_text = RichText::new(&p.name);
                                let name_resp = ui.label(if is_part_hovered {
                                    name_text.color(hover_highlight).strong()
                                } else {
                                    name_text
                                });

                                let cut_resp = ui.label(
                                    RichText::new(format!(
                                        "{:.1}×{:.1}",
                                        p.cut_length_mm, p.cut_width_mm
                                    ))
                                    .color(cut_dim_color),
                                );

                                let full_resp = ui.label(format!("{:.0}×{:.0}", p.full_length_mm, p.full_width_mm));

                                let rot_str = if p.rotated_90 {
                                    lang.t("Rotated 90°")
                                } else if p.grain_fixed {
                                    lang.t("🌲 Along grain")
                                } else {
                                    "0°"
                                };
                                let rot_resp = ui.label(rot_str);

                                if num_resp.hovered() || name_resp.hovered() || cut_resp.hovered() || full_resp.hovered() || rot_resp.hovered() {
                                    *hovered_part = Some(p.part_id);
                                }

                                ui.end_row();
                            }
                        });

                    if !active_sheet.remnants.is_empty() {
                        ui.add_space(10.0);
                        ui.separator();
                        ui.label(
                            RichText::new(lang.t("Usable offcuts (remnants) on sheet:")).strong(),
                        );
                        for (ri, rem) in active_sheet.remnants.iter().enumerate() {
                            let rem_lbl = lang.tf(
                                "fmt.remnant_bullet",
                                &[
                                    &(ri + 1),
                                    &format!("{:.0}", rem.w_mm),
                                    &format!("{:.0}", rem.h_mm),
                                    &d.unit_mm,
                                ],
                            );
                            ui.label(rem_lbl);
                        }
                    }
                });
        });

    // 3. CENTRAL PANEL: Toolbar, Summary Chips, Diagram
    egui::CentralPanel::default().show(ctx, |ui| {
        let mm_suffix = lang.t(" mm");

        // Top Toolbar: Row 1 (Parameters: Kerf & Trim Margins)
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;

            ui.label(RichText::new(lang.t("Saw kerf width:")).strong());
            ui.add(
                math_drag_value(&mut project.cutting_settings.kerf_mm)
                    .speed(0.2)
                    .range(0.0..=25.0)
                    .suffix(mm_suffix),
            );

            ui.separator();
            ui.label(RichText::new(lang.t("Sheet edge trim margins:")).strong());

            if project.cutting_settings.separate_trim_margins {
                ui.label(lang.t("Left:"));
                ui.add(
                    math_drag_value(&mut project.cutting_settings.trim_left_mm)
                        .speed(1.0)
                        .range(0.0..=100.0)
                        .suffix(mm_suffix),
                );
                ui.label(lang.t("Right:"));
                ui.add(
                    math_drag_value(&mut project.cutting_settings.trim_right_mm)
                        .speed(1.0)
                        .range(0.0..=100.0)
                        .suffix(mm_suffix),
                );
                ui.label(lang.t("Top:"));
                ui.add(
                    math_drag_value(&mut project.cutting_settings.trim_top_mm)
                        .speed(1.0)
                        .range(0.0..=100.0)
                        .suffix(mm_suffix),
                );
                ui.label(lang.t("Bottom:"));
                ui.add(
                    math_drag_value(&mut project.cutting_settings.trim_bottom_mm)
                        .speed(1.0)
                        .range(0.0..=100.0)
                        .suffix(mm_suffix),
                );
            } else {
                let mut unified_trim = project.cutting_settings.trim_left_mm;
                let trim_resp = ui.add(
                    math_drag_value(&mut unified_trim)
                        .speed(1.0)
                        .range(0.0..=100.0)
                        .suffix(mm_suffix),
                );
                if trim_resp.changed() {
                    project.cutting_settings.trim_left_mm = unified_trim;
                    project.cutting_settings.trim_right_mm = unified_trim;
                    project.cutting_settings.trim_top_mm = unified_trim;
                    project.cutting_settings.trim_bottom_mm = unified_trim;
                }
            }

            let prev_separate = project.cutting_settings.separate_trim_margins;
            if ui
                .checkbox(
                    &mut project.cutting_settings.separate_trim_margins,
                    lang.t("Custom margins"),
                )
                .changed()
            {
                if prev_separate && !project.cutting_settings.separate_trim_margins {
                    let unified = project.cutting_settings.trim_left_mm;
                    project.cutting_settings.trim_right_mm = unified;
                    project.cutting_settings.trim_top_mm = unified;
                    project.cutting_settings.trim_bottom_mm = unified;
                }
            }
        });

        ui.add_space(3.0);

        // Top Toolbar: Row 2 (Algorithm & Cut Cost)
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;

            ui.label(RichText::new(lang.t("cutting.algo_label")).strong());
            egui::ComboBox::from_id_salt("cutting_algorithm_choice")
                .selected_text(project.cutting_settings.algorithm.label_i18n(lang))
                .show_ui(ui, |ui| {
                    for algo in crate::model::CuttingAlgorithm::ALL {
                        ui.selectable_value(
                            &mut project.cutting_settings.algorithm,
                            algo,
                            algo.label_i18n(lang),
                        );
                    }
                });

            ui.separator();
            ui.label(RichText::new(lang.t("cutting.cut_price_per_m")).strong());
            ui.add(
                math_drag_value_f64(&mut project.cutting_settings.cut_price_per_m)
                    .speed(1.0)
                    .range(0.0..=100_000.0),
            );
        });

        ui.add_space(3.0);

        // Top Toolbar: Row 3 (Options & Actions)
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            ui.checkbox(
                &mut project.cutting_settings.group_identical_parts,
                lang.t("Group identical part numbers"),
            );

            ui.separator();
            ui.checkbox(
                &mut project.cutting_settings.show_textures_on_sheet,
                lang.t("Show material textures"),
            );

            ui.separator();
            if ui.button(lang.t("📋 Copy Cutting List")).clicked() {
                let txt = crate::export::generate_cutting_clipboard_text_i18n(project, lang);
                ui.output_mut(|o| o.copied_text = txt);
                *status_msg = lang
                    .t("Parts cutting list (saw dimensions & edges) copied to clipboard!")
                    .to_string();
            }
        });

        ui.separator();

        // Summary Statistics Badges
        ui.horizontal_wrapped(|ui| {
            let sheets_txt = crate::glossary::format_cutting_total_sheets(
                lang,
                report.total_sheets_count,
                d.unit_pcs,
            );
            let total_sheets_color = if is_dark {
                Color32::from_rgb(255, 215, 90)
            } else {
                Color32::from_rgb(160, 110, 15)
            };
            ui.label(
                RichText::new(sheets_txt)
                    .strong()
                    .color(total_sheets_color),
            );
            ui.separator();
            let area_txt = crate::glossary::format_cutting_parts_area(
                lang,
                report.total_parts_area_m2 as f64,
                report.total_sheets_area_m2 as f64,
                report.overall_utilization_percent as f64,
                d.unit_m2,
            );
            ui.label(area_txt);
            ui.separator();
            let cut_len_txt = crate::glossary::format_cutting_cut_length(
                lang,
                report.total_cut_length_m as f64,
                d.unit_mp,
            );
            ui.label(cut_len_txt);

            for (ename, meters, rgb) in &report.edge_banding_totals_m {
                ui.separator();
                let edge_tot_txt = format!("{}: {:.2} {}", ename, meters, d.unit_mp);
                ui.colored_label(Color32::from_rgb(rgb[0], rgb[1], rgb[2]), edge_tot_txt);
            }
        });

        if !report.unplaced_parts.is_empty() {
            ui.separator();
            for err in &report.unplaced_parts {
                let err_msg = crate::glossary::format_unplaced_warning(lang, &err.format_message(lang));
                ui.colored_label(Color32::from_rgb(255, 95, 95), err_msg);
            }
        }

        ui.separator();

        if report.sheets.is_empty() {
            ui.label(lang.t("No parts in the project for cutting yet."));
            return;
        }

        let cur_idx = (*selected_sheet_idx).min(report.sheets.len() - 1);
        let active_sheet = &report.sheets[cur_idx];

        let panel_clip = ui.clip_rect();
        let margin_x = 8.0;
        let margin_y = 6.0;
        let available_w = (panel_clip.width() - margin_x * 2.0).max(100.0);
        let available_h = (panel_clip.max.y - ui.cursor().top() - margin_y).max(100.0);

        let canvas_rect = Rect::from_min_size(
            Pos2::new(panel_clip.min.x + margin_x, ui.cursor().top() + 2.0),
            EguiVec2::new(available_w, available_h),
        );
        ui.allocate_rect(canvas_rect, Sense::hover());

        draw_cutting_sheet_diagram(
            ui,
            canvas_rect,
            active_sheet,
            project,
            hovered_part,
            loaded_textures,
            lang,
        );
    });
}

/// Renders graphical cutting sheet layout diagram with offcuts, textures, and edge markers
fn draw_cutting_sheet_diagram(
    ui: &egui::Ui,
    canvas_rect: Rect,
    sheet: &SheetLayout,
    project: &Project,
    hovered_part: &mut Option<PartId>,
    loaded_textures: &HashMap<MaterialId, TextureHandle>,
    lang: AppLanguage,
) {
    let is_dark = ui.visuals().dark_mode;
    let painter = ui.painter_at(canvas_rect);

    let canvas_bg = if is_dark {
        Color32::from_rgb(20, 24, 29)
    } else {
        Color32::from_rgb(238, 241, 246)
    };
    painter.rect_filled(canvas_rect, 4.0, canvas_bg);
    if !is_dark {
        painter.rect_stroke(
            canvas_rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(215, 220, 228)),
        );
    }

    let pad = 28.0;
    let usable = canvas_rect.shrink(pad);
    let scale_x = usable.width() / sheet.sheet_length_mm.max(1.0);
    let scale_y = usable.height() / sheet.sheet_width_mm.max(1.0);
    let scale = scale_x.min(scale_y);

    let sheet_px_w = sheet.sheet_length_mm * scale;
    let sheet_px_h = sheet.sheet_width_mm * scale;
    let origin = Pos2::new(
        canvas_rect.center().x - sheet_px_w * 0.5,
        canvas_rect.center().y - sheet_px_h * 0.5,
    );
    let sheet_rect = Rect::from_min_size(origin, EguiVec2::new(sheet_px_w, sheet_px_h));

    // Raw sheet background (trim margins zone)
    let margin_bg = if is_dark {
        Color32::from_rgb(62, 48, 42)
    } else {
        Color32::from_rgb(218, 206, 194)
    };
    let margin_stroke = if is_dark {
        Stroke::new(2.0, Color32::WHITE)
    } else {
        Stroke::new(2.0, Color32::from_rgb(85, 90, 100))
    };
    painter.rect_filled(sheet_rect, 0.0, margin_bg);
    painter.rect_stroke(sheet_rect, 0.0, margin_stroke);

    // Caption for sheet dimensions and grain direction
    let top_caption = lang.tf(
        "fmt.sheet_top_caption",
        &[
            &format!("{:.0}", sheet.sheet_length_mm),
            &lang.dict().unit_mm,
            &format!("{:.1}", project.cutting_settings.kerf_mm),
        ],
    );
    let caption_color = if is_dark {
        Color32::from_rgb(220, 225, 235)
    } else {
        Color32::from_rgb(35, 40, 50)
    };
    painter.text(
        Pos2::new(sheet_rect.center().x, sheet_rect.top() - 12.0),
        egui::Align2::CENTER_CENTER,
        top_caption,
        FontId::proportional(11.5),
        caption_color,
    );

    // 1. Render free offcuts (remnants) with dimensions
    let (rem_bg, rem_stroke_color, rem_text_color) = if is_dark {
        (
            Color32::from_rgb(35, 42, 48),
            Color32::from_gray(90),
            Color32::from_gray(145),
        )
    } else {
        (
            Color32::from_rgb(226, 230, 236),
            Color32::from_rgb(175, 180, 190),
            Color32::from_rgb(95, 105, 115),
        )
    };

    for rem in &sheet.remnants {
        let r_min = Pos2::new(origin.x + rem.x_mm * scale, origin.y + rem.y_mm * scale);
        let r_rect = Rect::from_min_size(
            r_min,
            EguiVec2::new(rem.w_mm * scale, rem.h_mm * scale),
        );
        painter.rect_filled(r_rect, 0.0, rem_bg);
        painter.rect_stroke(r_rect, 0.0, Stroke::new(1.0, rem_stroke_color));
        if r_rect.width() > 45.0 && r_rect.height() > 22.0 {
            let rem_word = lang.t("Offcut");
            painter.text(
                r_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("{}\n{:.0}×{:.0}", rem_word, rem.w_mm, rem.h_mm),
                FontId::monospace(9.5),
                rem_text_color,
            );
        }
    }

    // 2. Render placed parts on sheet
    let mat_rgb = project
        .get_material(sheet.material_id)
        .map(|m| m.color_rgb)
        .unwrap_or([215, 195, 165]);

    let pointer_pos = ui.input(|i| i.pointer.hover_pos());

    let (hover_stroke_color, normal_stroke_color) = if is_dark {
        (Color32::from_rgb(0, 240, 255), Color32::from_rgb(20, 20, 20))
    } else {
        (Color32::from_rgb(0, 145, 220), Color32::from_rgb(50, 55, 65))
    };

    for p in &sheet.placed_parts {
        let p_min = Pos2::new(origin.x + p.x_mm * scale, origin.y + p.y_mm * scale);
        let p_rect = Rect::from_min_size(
            p_min,
            EguiVec2::new(p.w_on_sheet_mm * scale, p.h_on_sheet_mm * scale),
        );

        let is_hovered = if let Some(ptr) = pointer_pos {
            if p_rect.contains(ptr) {
                *hovered_part = Some(p.part_id);
                true
            } else {
                *hovered_part == Some(p.part_id)
            }
        } else {
            *hovered_part == Some(p.part_id)
        };

        if project.cutting_settings.show_textures_on_sheet {
            if let Some(tex) = loaded_textures.get(&sheet.material_id) {
                painter.image(
                    tex.id(),
                    p_rect,
                    Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                painter.rect_filled(
                    p_rect,
                    0.0,
                    Color32::from_rgb(mat_rgb[0], mat_rgb[1], mat_rgb[2]),
                );
            }
        } else {
            painter.rect_filled(
                p_rect,
                0.0,
                Color32::from_rgb(mat_rgb[0], mat_rgb[1], mat_rgb[2]),
            );
        }

        if is_hovered {
            painter.rect_stroke(p_rect, 0.0, Stroke::new(2.5, hover_stroke_color));
        } else {
            painter.rect_stroke(p_rect, 0.0, Stroke::new(1.2, normal_stroke_color));
        }

        // Render edge banding markers inside part on cutting layout
        let inset = 3.0;
        let edge_stroke = |eid: Option<EdgeId>| -> Option<Stroke> {
            let e = project.get_edge(eid?)?;
            let c = Color32::from_rgb(e.color_rgb[0], e.color_rgb[1], e.color_rgb[2]);
            let w = if e.thickness_mm >= 1.5 { 3.0 } else { 1.8 };
            Some(Stroke::new(w, c))
        };

        let (top_e, bot_e, left_e, right_e) = if !p.rotated_90 {
            (p.edges.l1, p.edges.l2, p.edges.w1, p.edges.w2)
        } else {
            (p.edges.w1, p.edges.w2, p.edges.l1, p.edges.l2)
        };

        if let Some(s) = edge_stroke(top_e) {
            painter.line_segment(
                [
                    Pos2::new(p_rect.left() + inset, p_rect.top() + inset),
                    Pos2::new(p_rect.right() - inset, p_rect.top() + inset),
                ],
                s,
            );
        }
        if let Some(s) = edge_stroke(bot_e) {
            painter.line_segment(
                [
                    Pos2::new(p_rect.left() + inset, p_rect.bottom() - inset),
                    Pos2::new(p_rect.right() - inset, p_rect.bottom() - inset),
                ],
                s,
            );
        }
        if let Some(s) = edge_stroke(left_e) {
            painter.line_segment(
                [
                    Pos2::new(p_rect.left() + inset, p_rect.top() + inset),
                    Pos2::new(p_rect.left() + inset, p_rect.bottom() - inset),
                ],
                s,
            );
        }
        if let Some(s) = edge_stroke(right_e) {
            painter.line_segment(
                [
                    Pos2::new(p_rect.right() - inset, p_rect.top() + inset),
                    Pos2::new(p_rect.right() - inset, p_rect.bottom() - inset),
                ],
                s,
            );
        }

        // Part index, name, and saw cut dimensions label on cutting layout
        if p_rect.width() > 36.0 && p_rect.height() > 16.0 {
            let label = if p_rect.width() > 85.0 && p_rect.height() > 34.0 {
                format!(
                    "#{} {}\n{:.1}×{:.1}",
                    p.spec_index, p.name, p.w_on_sheet_mm, p.h_on_sheet_mm
                )
            } else {
                format!(
                    "#{} {:.0}×{:.0}",
                    p.spec_index, p.w_on_sheet_mm, p.h_on_sheet_mm
                )
            };

            // If textures are displayed, draw semi-transparent backing so text remains readable
            if project.cutting_settings.show_textures_on_sheet {
                let badge_h = if p_rect.width() > 85.0 && p_rect.height() > 34.0 { 28.0 } else { 16.0 };
                let badge_w = (p_rect.width() - 8.0).min(110.0).max(30.0);
                painter.rect_filled(
                    Rect::from_center_size(p_rect.center(), EguiVec2::new(badge_w, badge_h)),
                    3.0,
                    Color32::from_black_alpha(170),
                );
                painter.text(
                    p_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    label,
                    FontId::monospace(10.0),
                    Color32::WHITE,
                );
            } else {
                painter.text(
                    p_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    label,
                    FontId::monospace(10.0),
                    Color32::from_rgb(15, 15, 15),
                );
            }
        }
    }

    // 3. Render through-cut guillotine saw/blade lines
    let cut_line_color = if is_dark {
        Color32::from_rgb(250, 75, 75)
    } else {
        Color32::from_rgb(220, 45, 45)
    };
    for cl in &sheet.cut_lines {
        let p1 = Pos2::new(origin.x + cl.x1 * scale, origin.y + cl.y1 * scale);
        let p2 = Pos2::new(origin.x + cl.x2 * scale, origin.y + cl.y2 * scale);
        painter.line_segment([p1, p2], Stroke::new(1.2, cut_line_color));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        can_move_hierarchy_item, copy_selected_parts_to_clipboard, eval_math_expression,
        move_hierarchy_item, paste_parts_into_project,
    };
    use crate::model::{PartGroup, Project};

    #[test]
    fn test_eval_math_expression() {
        assert_eq!(eval_math_expression("600+100"), Some(700.0));
        assert_eq!(eval_math_expression("600-4"), Some(596.0));
        assert_eq!(eval_math_expression("600 mm + 100"), Some(700.0));
        assert_eq!(eval_math_expression("800-18-18"), Some(764.0));
        assert_eq!(eval_math_expression("(800 - 36) / 2"), Some(382.0));
        assert_eq!(eval_math_expression("600,5 + 0,5"), Some(601.0));
        assert_eq!(eval_math_expression("-45 + 90°"), Some(45.0));
    }

    #[test]
    fn test_multi_part_copy_paste_between_projects() {
        let src_proj = Project::new_demo_cabinet();
        let selected_ids: Vec<_> = src_proj.parts.iter().take(4).map(|p| p.id).collect();
        assert_eq!(selected_ids.len(), 4);

        let copied = copy_selected_parts_to_clipboard(
            &src_proj,
            selected_ids.first().copied(),
            &selected_ids,
        );
        assert_eq!(copied.len(), 4);

        let mut dst_proj = Project::new_empty("New Project".to_string());
        assert_eq!(dst_proj.parts.len(), 0);

        let new_ids = paste_parts_into_project(&mut dst_proj, &copied);
        assert_eq!(new_ids.len(), 4);
        assert_eq!(dst_proj.parts.len(), 4);

        // Verify that relative positions of copied parts are fully preserved
        let orig_delta = src_proj.parts[1].position_mm - src_proj.parts[0].position_mm;
        let pasted_delta = dst_proj.parts[1].position_mm - dst_proj.parts[0].position_mm;
        assert!((orig_delta - pasted_delta).length() < 1e-4);
    }

    #[test]
    fn test_move_part_up_down_within_group_and_group_reorder() {
        let mut proj = Project::new_demo_cabinet();
        // Keep only the first group and create a second group
        proj.groups.truncate(1);
        let grp1_id = proj.groups[0].id;
        let grp2_id = proj.alloc_id();
        proj.groups.push(PartGroup {
            id: grp2_id,
            name: "Group 2".to_string(),
            visible: true,
            locked: false,
            expanded: true,
        });

        let p0_id = proj.parts[0].id; // Group 1
        let p1_id = proj.parts[1].id; // Group 2
        let p2_id = proj.parts[2].id; // Group 1
        proj.parts[0].group_id = Some(grp1_id);
        proj.parts[1].group_id = Some(grp2_id);
        proj.parts[2].group_id = Some(grp1_id);

        // p0 is first in Group 1 -> cannot move up
        assert!(!can_move_hierarchy_item(&proj, None, Some(p0_id), -1));
        assert!(can_move_hierarchy_item(&proj, None, Some(p0_id), 1));

        // p1 is the only part in Group 2 -> cannot move up or down (must NOT jump into Group 1)
        assert!(!can_move_hierarchy_item(&proj, None, Some(p1_id), -1));
        assert!(!can_move_hierarchy_item(&proj, None, Some(p1_id), 1));
        assert!(!move_hierarchy_item(&mut proj, None, Some(p1_id), -1));

        // Move p2 up within Group 1 -> swaps with p0 (skipping p1 which is in Group 2!)
        assert!(move_hierarchy_item(&mut proj, None, Some(p2_id), -1));
        assert_eq!(proj.parts[0].id, p2_id);
        assert_eq!(proj.parts[1].id, p1_id);
        assert_eq!(proj.parts[2].id, p0_id);

        // Group reorder: move Group 2 up -> becomes first group
        assert!(can_move_hierarchy_item(&proj, Some(grp2_id), None, -1));
        assert!(move_hierarchy_item(&mut proj, Some(grp2_id), None, -1));
        assert_eq!(proj.groups[0].id, grp2_id);
        assert_eq!(proj.groups[1].id, grp1_id);
    }
}
