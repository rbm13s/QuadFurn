use crate::cutter::{build_specification, compute_cutting_layout, CuttingReport, SheetLayout, SpecRow};
use crate::export::{
    export_spec_to_csv_file, export_spec_to_pdf_file, generate_cutting3_clipboard_text,
};
use crate::i18n::AppLanguage;
use crate::model::{
    EdgeId, GroupId, Material, MaterialId, PartBlock, PartEdges, PartGroup, PartId,
    PartOrientation, Project,
};
use crate::viewport::{EditorTool, QuadViewportController};
use egui::{Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, TextureHandle, Vec2 as EguiVec2};
use glam::Vec3;
use std::collections::HashMap;

/// Активная вкладка программы
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppTab {
    /// 4-экранный конструктор (Hammer Editor + CorelDRAW)
    Designer4View,
    /// Таблица спецификации деталей и экспорт
    Specification,
    /// Утилита раскроя листа в стиле Cutting 3
    CuttingLayout,
    /// Справочник материалов (толщина 18 мм, цвета, текстуры) и кромок
    MaterialsManager,
}

/// Вычисляет математическое выражение в числовом поле ввода (поддерживает `+`, `-`, `*`, `/`, скобки `(...)`,
/// точку и запятую в качестве десятичного разделителя, а также пробелы и суффиксы единиц измерения).
/// Примеры: `"600+100"` -> `700.0`, `"600-4"` -> `596.0`, `"800-18-18"` -> `764.0`, `"(800-36)/2"` -> `382.0`.
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

/// Создает `egui::DragValue` с поддержкой ввода математических выражений (`600+100`, `600-4`, `800-36` и т.п.)
pub fn math_drag_value(value: &mut f32) -> egui::DragValue<'_> {
    egui::DragValue::new(value).custom_parser(eval_math_expression)
}

/// Отрисовка верхней панели CorelDRAW-style (точный ввод координат X/Y/Z, размеров Длина/Ширина/Толщина, поворота, сетки и инструментов)
pub fn ui_top_coreldraw_toolbar(
    ui: &mut egui::Ui,
    project: &mut Project,
    selected_part: &mut Option<PartId>,
    active_material_id: &mut MaterialId,
    viewport_ctrl: &mut QuadViewportController,
    mut on_before_mutate: impl FnMut(&Project),
) {
    let lang = viewport_ctrl.lang;
    let mm_suffix = lang.tr(" мм", " mm");

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;

        // 1. Инструменты редактора
        ui.label(RichText::new(lang.tr("Инструмент:", "Tool:")).strong());
        if ui
            .selectable_label(
                viewport_ctrl.tool == EditorTool::SelectAndMove,
                lang.tr("↖ Выбор / Двигать (V)", "↖ Select / Move (V)"),
            )
            .on_hover_text(lang.tr(
                "Выделение деталей, перемещение и растягивание за 8 маркеров",
                "Select parts, move and resize using 8 handles",
            ))
            .clicked()
        {
            viewport_ctrl.tool = EditorTool::SelectAndMove;
        }
        if ui
            .selectable_label(
                viewport_ctrl.tool == EditorTool::CreateBlock,
                lang.tr("➕ Создать деталь рамкой (B)", "➕ Create Part by Box (B)"),
            )
            .on_hover_text(lang.tr(
                "Нарисуйте прямоугольник в любом 2D-окне. Толщина берется из активного материала (напр. 18 мм). Клавиша Tab меняет Полка/Стойка/Фасад.",
                "Draw a rectangle in any 2D view. Thickness comes from active material (e.g. 18 mm). Press Tab to cycle Shelf/Side/Front.",
            ))
            .clicked()
        {
            viewport_ctrl.tool = EditorTool::CreateBlock;
        }
        if ui
            .selectable_label(
                viewport_ctrl.tool == EditorTool::MeasureTape,
                lang.tr("📏 Рулетка (M)", "📏 Tape Measure (M)"),
            )
            .on_hover_text(lang.tr(
                "Измерение расстояний между точками и краями в 2D проекциях",
                "Measure distances between points and edges in 2D projections",
            ))
            .clicked()
        {
            viewport_ctrl.tool = EditorTool::MeasureTape;
        }

        ui.separator();

        // 2. Быстрое добавление типовой детали (Стойка / Полка / Фасад)
        ui.label(lang.tr("Быстро:", "Quick:"));
        if ui
            .button(lang.tr("+ Полка (XY)", "+ Shelf (XY)"))
            .on_hover_text(lang.tr(
                "Добавить горизонтальную полку/дно из активного материала",
                "Add horizontal shelf/bottom panel from active material",
            ))
            .clicked()
        {
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
        if ui
            .button(lang.tr("+ Стойка (YZ)", "+ Side Panel (YZ)"))
            .on_hover_text(lang.tr(
                "Добавить боковую вертикальную стойку из активного материала",
                "Add vertical side panel from active material",
            ))
            .clicked()
        {
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
        if ui
            .button(lang.tr("+ Фасад/Стенка (XZ)", "+ Front/Back (XZ)"))
            .on_hover_text(lang.tr(
                "Добавить фронтальную деталь (фасад, царгу или заднюю стенку)",
                "Add frontal panel (door, rail, or back panel)",
            ))
            .clicked()
        {
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

        ui.separator();

        // 3. Выбор активного материала (для новых деталей)
        ui.label(lang.tr("Материал:", "Material:"));
        let active_mat_name = project
            .get_material(*active_material_id)
            .map(|m| format!("{} ({:.0}{})", m.name, m.thickness_mm, mm_suffix))
            .unwrap_or_else(|| lang.tr("Выбрать...", "Select...").to_string());
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

        // 4. Привязки и шаг сетки (мм)
        ui.checkbox(
            &mut viewport_ctrl.grid_snap_enabled,
            lang.tr("Сетка", "Grid"),
        );
        ui.add(
            math_drag_value(&mut viewport_ctrl.grid_step_mm)
                .speed(1.0)
                .range(0.5..=200.0)
                .suffix(mm_suffix),
        )
        .on_hover_text(lang.tr(
            "Шаг координатной сетки в миллиметрах (1, 5, 9, 18, 50 мм)",
            "Grid snap step in millimeters (1, 5, 9, 18, 50 mm)",
        ));

        for preset in [1.0_f32, 9.0, 18.0, 50.0] {
            if ui
                .small_button(format!("{:.0}", preset))
                .on_hover_text(match lang {
                    AppLanguage::Ru => format!("Установить шаг сетки {:.0} мм", preset),
                    _ => format!("Set grid step to {:.0} mm", preset),
                })
                .clicked()
            {
                viewport_ctrl.grid_step_mm = preset;
            }
        }

        ui.checkbox(
            &mut viewport_ctrl.edge_snap_enabled,
            lang.tr("Магнит к краям", "Snap to edges"),
        )
        .on_hover_text(lang.tr(
            "Примагничивать края деталей друг к другу вплотную",
            "Snap part edges flush against neighboring parts",
        ));
        ui.checkbox(
            &mut viewport_ctrl.show_distance_callouts,
            lang.tr("Проемы (мм)", "Gaps (mm)"),
        )
        .on_hover_text(lang.tr(
            "Показывать расстояния от выбранной детали до соседей",
            "Display distances from selected part to neighboring parts",
        ));

        if ui
            .button(lang.tr("🔍 Центрировать вид", "🔍 Center View"))
            .on_hover_text(lang.tr(
                "Сцентрировать все 4 экрана на изделии без сброса масштаба",
                "Center all 4 views on the product without resetting zoom",
            ))
            .clicked()
        {
            viewport_ctrl.focus_all_on_project(project);
        }

        if ui
            .button(lang.tr("📐 Вписать в окно", "📐 Fit to Window"))
            .on_hover_text(lang.tr(
                "Подобрать масштаб и положение камеры во всех 2D проекциях и в 3D так, чтобы вся модель помещалась целиком в каждое окно",
                "Fit the entire model into all 2D projections and the 3D view",
            ))
            .clicked()
        {
            viewport_ctrl.fit_all_to_window(project);
        }
    });

    ui.separator();

    // Вторая строка панели CorelDRAW: числовой ввод координат X, Y, Z и габаритов Длина / Ширина / Толщина / Поворот для выбранного блока или группы
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;

        if viewport_ctrl.selected_parts.len() > 1 {
            // Групповое выделение нескольких деталей: показываем общий габарит и позволяем синхронно сдвигать всю группу по X, Y, Z
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
                let grp_text = match lang {
                    AppLanguage::Ru => format!("Группа: {} дет.", valid_count),
                    _ => format!("Group: {} pcs", valid_count),
                };
                ui.label(
                    RichText::new(grp_text)
                        .strong()
                        .color(Color32::from_rgb(120, 210, 255)),
                );
                ui.separator();

                ui.label(
                    RichText::new(lang.tr("Позиция группы (мм):", "Group position (mm):"))
                        .strong(),
                );
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
                let span_text = match lang {
                    AppLanguage::Ru => format!(
                        "Общий габарит группы: X {:.0} × Y {:.0} × Z {:.0} мм",
                        span.x, span.y, span.z
                    ),
                    _ => format!(
                        "Group bounding box: X {:.0} × Y {:.0} × Z {:.0} mm",
                        span.x, span.y, span.z
                    ),
                };
                ui.label(RichText::new(span_text).color(Color32::from_rgb(160, 225, 175)));
                return;
            }
        }

        if let Some(sel_id) = *selected_part {
            if let Some(idx) = project.parts.iter().position(|p| p.id == sel_id) {
                let mat_id = project.parts[idx].material_id;
                let thick = project.get_material_thickness(mat_id);
                let (cut_l, cut_w) = project.parts[idx].cut_dimensions_mm(&project.edge_bandings);

                let block_lbl = match lang {
                    AppLanguage::Ru => format!("Блок: «{}»", project.parts[idx].name),
                    _ => format!("Part: \"{}\"", project.parts[idx].name),
                };
                ui.label(
                    RichText::new(block_lbl)
                        .strong()
                        .color(Color32::from_rgb(255, 215, 90)),
                );
                ui.separator();

                // Координаты X, Y, Z (мм от нуля сцены)
                ui.label(RichText::new(lang.tr("Позиция (мм):", "Position (mm):")).strong());
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

                // Габаритные размеры детали (Длина x Ширина x Толщина) в миллиметрах, как в CorelDRAW
                ui.label(RichText::new(lang.tr("Габарит с кромкой:", "Full size w/ edge:")).strong());
                let mut len = project.parts[idx].length_mm;
                let mut wid = project.parts[idx].width_mm;
                let mut size_changed = false;

                ui.label(lang.tr("Длина (L):", "Length (L):"));
                size_changed |= ui
                    .add(
                        math_drag_value(&mut len)
                            .speed(1.0)
                            .range(10.0..=5000.0)
                            .suffix(mm_suffix),
                    )
                    .changed();

                ui.label(lang.tr("Ширина (W):", "Width (W):"));
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

                // Поле толщины (редактирует толщину материала детали или показывает 18 мм)
                let mut cur_thick = thick;
                ui.label(lang.tr("Толщ (T):", "Thick (T):"));
                if ui
                    .add(
                        math_drag_value(&mut cur_thick)
                            .speed(0.5)
                            .range(1.0..=100.0)
                            .suffix(mm_suffix),
                    )
                    .on_hover_text(lang.tr(
                        "Толщина листового материала детали (мм)",
                        "Sheet material thickness of the part (mm)",
                    ))
                    .changed()
                {
                    if let Some(m) = project.materials.iter_mut().find(|m| m.id == mat_id) {
                        m.thickness_mm = cur_thick;
                    }
                }

                let cut_badge = match lang {
                    AppLanguage::Ru => format!("[Пил: {:.1}×{:.1} мм]", cut_l, cut_w),
                    _ => format!("[Cut: {:.1}×{:.1} mm]", cut_l, cut_w),
                };
                ui.label(RichText::new(cut_badge).color(Color32::from_rgb(120, 225, 150)))
                    .on_hover_text(lang.tr(
                        "Чистый пилильный размер заготовки за вычетом толщины кромок",
                        "Net saw cutting dimensions after subtracting edge banding thickness",
                    ));

                ui.separator();

                // Поворот вокруг оси Z
                ui.label("∠Z:");
                ui.add(
                    math_drag_value(&mut project.parts[idx].yaw_deg)
                        .speed(1.0)
                        .range(-360.0..=360.0)
                        .suffix("°"),
                )
                .on_hover_text(lang.tr(
                    "Поворот детали вокруг вертикальной оси Z (для угловых секций)",
                    "Part rotation around the vertical Z axis (for corner units)",
                ));

                if ui.small_button("↺ 90°").clicked() {
                    project.parts[idx].yaw_deg = (project.parts[idx].yaw_deg + 90.0) % 360.0;
                }
            }
        } else {
            ui.label(
                RichText::new(lang.tr(
                    "Ни одна деталь не выбрана — кликните по блоку, растяните рамку выделения в 2D окне или выберите «➕ Создать деталь рамкой»",
                    "No part selected — click a block, drag a selection box in 2D, or choose \"➕ Create Part by Box\"",
                ))
                .italics()
                .color(Color32::GRAY),
            );
        }
    });
}

fn add_quick_part(
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
            format!("{} {}", lang.tr("Полка", "Shelf"), idx),
            764.0,
            500.0,
            Vec3::new(18.0, 0.0, 350.0),
        ),
        PartOrientation::VerticalYZ => (
            format!("{} {}", lang.tr("Стойка", "Side Panel"), idx),
            720.0,
            500.0,
            Vec3::new(0.0, 0.0, 0.0),
        ),
        PartOrientation::FrontalXZ => (
            format!("{} {}", lang.tr("Панель", "Front Panel"), idx),
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

/// Префикс системного буфера обмена для копирования деталей между вкладками и окнами QuadFurn
pub const CLIPBOARD_PREFIX: &str = "QUADFURN_CLIPBOARD_V1:";

/// Буфер обмена для копирования детали между разными вкладками (и окнами) объектов мебели
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ClipboardPart {
    pub part: PartBlock,
    pub source_material: Option<Material>,
    pub source_edges: [Option<crate::model::EdgeBanding>; 4],
    #[serde(default)]
    pub source_group_name: Option<String>,
}

/// Собирает все выделенные детали (одиночную или группу деталей) в список `ClipboardPart`
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

/// Вставляет список скопированных деталей из буфера обмена в целевой объект мебели
/// (автоматически сопоставляя или добавляя материалы, кромки и группы, а также сохраняя взаимное расположение деталей)
pub fn paste_parts_into_project(project: &mut Project, clips: &[ClipboardPart]) -> Vec<PartId> {
    if clips.is_empty() {
        return Vec::new();
    }

    // Вычисляем единый сдвиг для всей вставляемой группы деталей, если хотя бы одна деталь
    // совпадает по координатам с уже существующей деталью в целевом проекте
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
        // 1. Сопоставляем материал (по имени и толщине, либо создаем его в целевом проекте)
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

        // 2. Сопоставляем кромки по 4 сторонам
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

        // 3. Сопоставляем группу по имени (либо берем первую группу проекта)
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

/// Правый инспектор блоков, групп, материалов и интерактивной схемы кромления детали
#[allow(clippy::too_many_arguments)]
pub fn ui_right_inspector(
    ui: &mut egui::Ui,
    project: &mut Project,
    selected_part: &mut Option<PartId>,
    selected_parts: &mut Vec<PartId>,
    active_material_id: &mut MaterialId,
    new_group_name_buf: &mut String,
    clipboard: &mut Vec<ClipboardPart>,
    status_msg: &mut String,
    lang: AppLanguage,
    mut on_before_mutate: impl FnMut(&Project),
) {
    let mm_suffix = lang.tr(" мм", " mm");
    ui.heading(lang.tr("Инспектор блоков", "Part Inspector"));

    // Панель управления группами и блоками
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(new_group_name_buf)
                .hint_text(lang.tr("Имя новой группы...", "New group name..."))
                .desired_width(135.0),
        );
        if ui.button(lang.tr("📁 +Группа", "📁 +Group")).clicked() {
            on_before_mutate(project);
            let gid = project.alloc_id();
            let gname = if new_group_name_buf.trim().is_empty() {
                format!("{} {}", lang.tr("Группа", "Group"), project.groups.len() + 1)
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
            new_group_name_buf.clear();
        }
    });

    ui.horizontal_wrapped(|ui| {
        let has_sel = selected_part.is_some() || !selected_parts.is_empty();
        if ui
            .add_enabled(
                has_sel,
                egui::Button::new(lang.tr("📄 Копировать (Ctrl+C)", "📄 Copy (Ctrl+C)")),
            )
            .on_hover_text(lang.tr(
                "Скопировать выделенные детали в буфер (можно вставить в другую вкладку или окно мебели)",
                "Copy selected parts to clipboard (can be pasted into another tab or window)",
            ))
            .clicked()
        {
            let copied = copy_selected_parts_to_clipboard(project, *selected_part, selected_parts);
            if !copied.is_empty() {
                if let Ok(json) = serde_json::to_string(&copied) {
                    ui.ctx().copy_text(format!("{}{}", CLIPBOARD_PREFIX, json));
                }
                if copied.len() == 1 {
                    *status_msg = match lang {
                        AppLanguage::Ru => format!(
                            "Деталь «{}» скопирована в буфер обмена (перейдите в любую вкладку мебели и нажмите Ctrl+V)",
                            copied[0].part.name
                        ),
                        _ => format!(
                            "Part \"{}\" copied to clipboard (switch to any product tab and press Ctrl+V)",
                            copied[0].part.name
                        ),
                    };
                } else {
                    *status_msg = match lang {
                        AppLanguage::Ru => format!(
                            "Скопировано деталей: {} шт. (перейдите в любую вкладку мебели и нажмите Ctrl+V)",
                            copied.len()
                        ),
                        _ => format!(
                            "Copied {} parts (switch to any product tab and press Ctrl+V)",
                            copied.len()
                        ),
                    };
                }
                *clipboard = copied;
            }
        }

        if ui
            .add_enabled(
                !clipboard.is_empty(),
                egui::Button::new(lang.tr("📥 Вставить (Ctrl+V)", "📥 Paste (Ctrl+V)")),
            )
            .on_hover_text(lang.tr(
                "Вставить скопированные детали в текущий объект мебели",
                "Paste copied parts into the current furniture product",
            ))
            .clicked()
        {
            if !clipboard.is_empty() {
                on_before_mutate(project);
                let new_ids = paste_parts_into_project(project, clipboard);
                *selected_part = new_ids.last().copied();
                *selected_parts = new_ids.clone();
                if new_ids.len() == 1 {
                    *status_msg = match lang {
                        AppLanguage::Ru => format!(
                            "Деталь «{}» вставлена в «{}»",
                            clipboard[0].part.name, project.name
                        ),
                        _ => format!(
                            "Part \"{}\" pasted into \"{}\"",
                            clipboard[0].part.name, project.name
                        ),
                    };
                } else {
                    *status_msg = match lang {
                        AppLanguage::Ru => format!(
                            "Вставлено деталей: {} шт. в изделие «{}»",
                            new_ids.len(),
                            project.name
                        ),
                        _ => format!(
                            "Pasted {} parts into product \"{}\"",
                            new_ids.len(),
                            project.name
                        ),
                    };
                }
            }
        }
    });

    ui.horizontal_wrapped(|ui| {
        let has_sel = selected_part.is_some() || !selected_parts.is_empty();
        if ui
            .add_enabled(
                has_sel,
                egui::Button::new(lang.tr("📋 Дублировать (Ctrl+D)", "📋 Duplicate (Ctrl+D)")),
            )
            .clicked()
        {
            let ids_to_dup: Vec<PartId> = if !selected_parts.is_empty() {
                selected_parts.clone()
            } else {
                (*selected_part).into_iter().collect()
            };
            if !ids_to_dup.is_empty() {
                on_before_mutate(project);
                let mut new_ids = Vec::new();
                for sid in ids_to_dup {
                    if let Some(orig) = project.parts.iter().find(|p| p.id == sid).cloned() {
                        let new_id = project.alloc_id();
                        let mut copy = orig;
                        copy.id = new_id;
                        copy.name = format!(
                            "{} ({})",
                            copy.name,
                            lang.tr("копия", "copy")
                        );
                        copy.position_mm += Vec3::new(36.0, 0.0, 36.0);
                        project.parts.push(copy);
                        new_ids.push(new_id);
                    }
                }
                *selected_part = new_ids.last().copied();
                *selected_parts = new_ids;
            }
        }
        if ui
            .add_enabled(
                has_sel,
                egui::Button::new(lang.tr("🗑 Удалить (Del)", "🗑 Delete (Del)")),
            )
            .clicked()
        {
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
            }
        }
    });

    ui.separator();

    // Дерево групп и деталей (с прокруткой, прижатой к правой границе панели)
    egui::ScrollArea::vertical()
        .id_salt("hierarchy_scroll")
        .auto_shrink([false, false])
        .max_height(235.0)
        .show(ui, |ui| {
            let group_ids: Vec<GroupId> = project.groups.iter().map(|g| g.id).collect();
            for gid in group_ids {
                let Some(g_idx) = project.groups.iter().position(|g| g.id == gid) else {
                    continue;
                };
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
                        .on_hover_text(lang.tr(
                            "Скрыть/показать всю группу",
                            "Show/hide entire group",
                        ))
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
                        .on_hover_text(lang.tr(
                            "Заблокировать/разблокировать группу от сдвига мышью",
                            "Lock/unlock group against mouse dragging",
                        ))
                        .clicked()
                    {
                        project.groups[g_idx].locked = !project.groups[g_idx].locked;
                    }

                    if ui
                        .small_button("✔")
                        .on_hover_text(lang.tr(
                            "Выделить все детали этой группы для совместного перемещения",
                            "Select all parts in this group to move them together",
                        ))
                        .clicked()
                    {
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

                    let avail_w = (ui.available_width() - 8.0).max(110.0);
                    ui.add(
                        egui::TextEdit::singleline(&mut project.groups[g_idx].name)
                            .desired_width(avail_w),
                    );
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
                                    lang,
                                );
                            }
                        }
                    });
                }
            }

            // Детали без группы
            let ungrouped_indices: Vec<usize> = project
                .parts
                .iter()
                .enumerate()
                .filter(|(_, p)| {
                    p.group_id.is_none()
                        || !project.groups.iter().any(|g| Some(g.id) == p.group_id)
                })
                .map(|(i, _)| i)
                .collect();

            if !ungrouped_indices.is_empty() {
                ui.label(RichText::new(lang.tr("📁 Вне групп:", "📁 Ungrouped:")).strong());
                for p_idx in ungrouped_indices {
                    render_part_tree_row(
                        ui,
                        project,
                        p_idx,
                        selected_part,
                        selected_parts,
                        lang,
                    );
                }
            }
        });

    ui.separator();

    // Нижняя часть Правого Инспектора: детальные свойства выбранного блока, материал, текстура и интерактивная схема кромления
    egui::ScrollArea::vertical()
        .id_salt("part_props_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if let Some(sel_id) = *selected_part {
                if let Some(idx) = project.parts.iter().position(|p| p.id == sel_id) {
                    ui.heading(lang.tr("Свойства детали", "Part Properties"));

                    egui::Grid::new("part_inspector_grid")
                        .num_columns(2)
                        .spacing([8.0, 6.0])
                        .show(ui, |ui| {
                            ui.label(lang.tr("Наименование:", "Name:"));
                            ui.text_edit_singleline(&mut project.parts[idx].name);
                            ui.end_row();

                            ui.label(lang.tr("Группа:", "Group:"));
                            let no_grp_txt = lang.tr("Без группы", "No group");
                            let cur_grp_name = project.parts[idx]
                                .group_id
                                .and_then(|gid| project.groups.iter().find(|g| g.id == gid))
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

                            ui.label(lang.tr("Ориентация:", "Orientation:"));
                            egui::ComboBox::from_id_salt("part_orient_combo")
                                .selected_text(project.parts[idx].orientation.label_i18n(lang))
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

                            ui.label(lang.tr("Материал:", "Material:"));
                            let cur_mat_id = project.parts[idx].material_id;
                            let cur_mat_label = project
                                .get_material(cur_mat_id)
                                .map(|m| format!("{} ({:.0}{})", m.name, m.thickness_mm, mm_suffix))
                                .unwrap_or_else(|| lang.tr("Не выбран", "None").to_string());
                            egui::ComboBox::from_id_salt("part_mat_combo")
                                .selected_text(cur_mat_label)
                                .show_ui(ui, |ui| {
                                    for m in &project.materials {
                                        if ui
                                            .selectable_value(
                                                &mut project.parts[idx].material_id,
                                                m.id,
                                                format!("{} ({:.0}{})", m.name, m.thickness_mm, mm_suffix),
                                            )
                                            .clicked()
                                        {
                                            *active_material_id = m.id;
                                        }
                                    }
                                });
                            ui.end_row();

                            ui.label(lang.tr("Габарит (Д×Ш):", "Full Size (L×W):"));
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
                                    .on_hover_text(lang.tr(
                                        "Поменять Длину и Ширину местами",
                                        "Swap Length and Width",
                                    ))
                                    .clicked()
                                {
                                    let tmp = project.parts[idx].length_mm;
                                    project.parts[idx].length_mm = project.parts[idx].width_mm;
                                    project.parts[idx].width_mm = tmp;
                                }
                            });
                            ui.end_row();

                            let (cut_l, cut_w) =
                                project.parts[idx].cut_dimensions_mm(&project.edge_bandings);
                            ui.label(lang.tr("Размер пила:", "Cut Size:"));
                            ui.label(
                                RichText::new(format!("{:.1} × {:.1}{}", cut_l, cut_w, mm_suffix))
                                    .strong()
                                    .color(Color32::from_rgb(110, 235, 145)),
                            );
                            ui.end_row();
                        });

                    ui.add_space(4.0);
                    ui.checkbox(
                        &mut project.parts[idx].grain_fixed,
                        lang.tr(
                            "🌲 Зафиксировать направление волокон (не вращать при раскрое)",
                            "🌲 Lock grain direction (do not rotate on cutting sheet)",
                        ),
                    );
                    if project.parts[idx].grain_fixed {
                        ui.checkbox(
                            &mut project.parts[idx].grain_along_width,
                            lang.tr(
                                "Направить волокна поперек (вдоль Ширины W вместо Длины L)",
                                "Run grain across width (along Width W instead of Length L)",
                            ),
                        );
                    }

                    ui.separator();
                    ui.label(
                        RichText::new(lang.tr(
                            "Оклеивание кромкой по 4 граням:",
                            "Edge Banding on 4 Sides:",
                        ))
                        .strong(),
                    );

                    // Кнопки быстрого кромления в один клик
                    ui.horizontal_wrapped(|ui| {
                        ui.label(lang.tr("Быстро:", "Quick:"));
                        if let Some(first_edge) = project.edge_bandings.first().map(|e| e.id) {
                            if ui
                                .small_button(lang.tr("Вкруг ПВХ 2мм", "All 4 Edges"))
                                .clicked()
                            {
                                project.parts[idx].edges = PartEdges {
                                    l1: Some(first_edge),
                                    l2: Some(first_edge),
                                    w1: Some(first_edge),
                                    w2: Some(first_edge),
                                };
                            }
                        }
                        if ui
                            .small_button(lang.tr("Очистить кромки", "Clear Edges"))
                            .clicked()
                        {
                            project.parts[idx].edges = PartEdges::default();
                        }
                    });

                    // Интерактивная графическая схема детали с 4 гранями (Д1, Д2, Ш1, Ш2)
                    draw_interactive_edge_diagram(ui, project, idx, lang);

                    // Выпадающие списки по каждой из 4 граней
                    let edges_clone = project.edge_bandings.clone();
                    egui::Grid::new("edges_select_grid")
                        .num_columns(2)
                        .spacing([8.0, 4.0])
                        .show(ui, |ui| {
                            edge_combo_row(
                                ui,
                                lang.tr("Д1 (Длина 1 / Перед):", "L1 (Length 1 / Front):"),
                                "edge_l1",
                                &mut project.parts[idx].edges.l1,
                                &edges_clone,
                                lang,
                            );
                            edge_combo_row(
                                ui,
                                lang.tr("Д2 (Длина 2 / Зад):", "L2 (Length 2 / Back):"),
                                "edge_l2",
                                &mut project.parts[idx].edges.l2,
                                &edges_clone,
                                lang,
                            );
                            edge_combo_row(
                                ui,
                                lang.tr("Ш1 (Ширина 1 / Торец 1):", "W1 (Width 1 / End 1):"),
                                "edge_w1",
                                &mut project.parts[idx].edges.w1,
                                &edges_clone,
                                lang,
                            );
                            edge_combo_row(
                                ui,
                                lang.tr("Ш2 (Ширина 2 / Торец 2):", "W2 (Width 2 / End 2):"),
                                "edge_w2",
                                &mut project.parts[idx].edges.w2,
                                &edges_clone,
                                lang,
                            );
                        });
                }
            } else {
                ui.label(
                    RichText::new(lang.tr(
                        "Выберите деталь в списке выше или кликните по ней на чертеже / в 3D, чтобы настроить размеры, материал и кромки.",
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

        // Цветной квадратик материала детали
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
            .on_hover_text(lang.tr(
                "Клик — выбрать деталь; Shift/Ctrl + клик — добавить/убрать из группового выделения",
                "Click to select part; Shift/Ctrl + click to toggle in multi-selection",
            ));
        if resp.clicked() {
            let multi_mod = ui.input(|i| i.modifiers.shift || i.modifiers.command || i.modifiers.ctrl);
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
    let mm_suffix = lang.tr(" мм", " mm");
    let none_txt = lang.tr("Нет (без кромки)", "None (no edge)");
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

/// Рисует наглядную интерактивную схему прямоугольника детали в правом инспекторе:
/// клик по стороне Д1 / Д2 / Ш1 / Ш2 циклически переключает кромку (Нет -> 2.0 мм -> 0.8 мм -> 0.4 мм)
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

    // Стрелка направления волокон внутри прямоугольника
    if part.grain_fixed {
        let c = inner.center();
        if !part.grain_along_width {
            painter.line_segment(
                [Pos2::new(inner.left() + 18.0, c.y), Pos2::new(inner.right() - 18.0, c.y)],
                Stroke::new(1.2, Color32::from_rgb(170, 190, 215)),
            );
        } else {
            painter.line_segment(
                [Pos2::new(c.x, inner.top() + 10.0), Pos2::new(c.x, inner.bottom() - 10.0)],
                Stroke::new(1.2, Color32::from_rgb(170, 190, 215)),
            );
        }
    }

    let center_txt = match lang {
        AppLanguage::Ru => format!(
            "Габарит: {:.0}×{:.0}\nПил: {:.1}×{:.1}",
            part.length_mm, part.width_mm, cut_l, cut_w
        ),
        _ => format!(
            "Full: {:.0}×{:.0}\nCut: {:.1}×{:.1}",
            part.length_mm, part.width_mm, cut_l, cut_w
        ),
    };
    painter.text(
        inner.center(),
        egui::Align2::CENTER_CENTER,
        center_txt,
        FontId::monospace(10.0),
        Color32::WHITE,
    );

    let mm_unit = lang.tr("мм", "mm");
    let none_unit = lang.tr("нет", "none");
    let draw_edge_bar = |p1: Pos2, p2: Pos2, label_pos: Pos2, title: &str, eid: Option<EdgeId>| {
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

    // Верхняя сторона = Д1, Нижняя = Д2, Левая = Ш1, Правая = Ш2
    draw_edge_bar(
        inner.left_top(),
        inner.right_top(),
        Pos2::new(inner.center().x, rect.top() + 10.0),
        lang.tr("Д1", "L1"),
        part.edges.l1,
    );
    draw_edge_bar(
        inner.left_bottom(),
        inner.right_bottom(),
        Pos2::new(inner.center().x, rect.bottom() - 10.0),
        lang.tr("Д2", "L2"),
        part.edges.l2,
    );
    draw_edge_bar(
        inner.left_top(),
        inner.left_bottom(),
        Pos2::new(rect.left() + 19.0, inner.center().y),
        lang.tr("Ш1", "W1"),
        part.edges.w1,
    );
    draw_edge_bar(
        inner.right_top(),
        inner.right_bottom(),
        Pos2::new(rect.right() - 19.0, inner.center().y),
        lang.tr("Ш2", "W2"),
        part.edges.w2,
    );

    // Клик по области стороны на мини-схеме циклически меняет кромку
    if resp.clicked() {
        if let Some(mp) = resp.interact_pointer_pos() {
            let cycle_edge = |cur: Option<EdgeId>, lib: &[crate::model::EdgeBanding]| -> Option<EdgeId> {
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
                project.parts[part_idx].edges.l1 = cycle_edge(project.parts[part_idx].edges.l1, &lib);
            } else if min_d == d_bot {
                project.parts[part_idx].edges.l2 = cycle_edge(project.parts[part_idx].edges.l2, &lib);
            } else if min_d == d_left {
                project.parts[part_idx].edges.w1 = cycle_edge(project.parts[part_idx].edges.w1, &lib);
            } else {
                project.parts[part_idx].edges.w2 = cycle_edge(project.parts[part_idx].edges.w2, &lib);
            }
        }
    }
}

/// Вкладка «Справочник материалов и кромок» (настройка толщины 18 мм, RGB-цвета, размеров листа, загрузка текстуры PNG/JPG/WEBP и удаление материалов и кромок с подтверждением)
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
    let mm_suffix = lang.tr(" мм", " mm");
    ui.heading(lang.tr(
        "Справочник листовых материалов и кромок",
        "Sheet Materials & Edge Banding Catalog",
    ));
    ui.add_space(6.0);

    ui.horizontal(|ui| {
        if ui
            .button(lang.tr(
                "➕ Добавить листовой материал (ЛДСП 18 мм)",
                "➕ Add Sheet Material (18 mm Chipboard)",
            ))
            .clicked()
        {
            let id = project.alloc_id();
            project.materials.push(Material {
                id,
                name: match lang {
                    AppLanguage::Ru => {
                        format!("ЛДСП 18мм Новый декор {}", project.materials.len() + 1)
                    }
                    _ => {
                        format!("Chipboard 18mm New Decor {}", project.materials.len() + 1)
                    }
                },
                thickness_mm: 18.0,
                color_rgb: [200, 175, 145],
                texture_path: None,
                texture_relative_path: None,
                texture_base64: None,
                sheet_length_mm: 2800.0,
                sheet_width_mm: 2070.0,
                default_grain_fixed: true,
            });
        }

        if ui
            .button(lang.tr("➕ Добавить тип кромки", "➕ Add Edge Banding Type"))
            .clicked()
        {
            let id = project.alloc_id();
            project.edge_bandings.push(crate::model::EdgeBanding {
                id,
                name: match lang {
                    AppLanguage::Ru => format!("ПВХ {:.1} мм", 1.0),
                    _ => format!("PVC {:.1} mm", 1.0),
                },
                thickness_mm: 1.0,
                color_rgb: [180, 100, 240],
            });
        }

        ui.separator();
        ui.checkbox(
            &mut project.embed_textures,
            lang.tr(
                "📦 Упаковывать текстуры в файл проекта (.qfurn)",
                "📦 Embed textures into project file (.qfurn)",
            ),
        )
        .on_hover_text(lang.tr(
            "При сохранении все используемые текстуры будут сохранены прямо в файл проекта (Base64). Файл можно будет переносить на любой ПК без потери изображений.",
            "When saving, all active textures will be embedded directly into the project file (Base64). The file can be transferred to any PC without losing images.",
        ));
    });

    ui.separator();
    ui.label(
        RichText::new(lang.tr(
            "Листовые материалы проекта (все размеры в миллиметрах):",
            "Project sheet materials (all dimensions in millimeters):",
        ))
        .strong(),
    );

    let can_delete_mat = project.materials.len() > 1;

    egui::Grid::new("materials_table_grid")
        .striped(true)
        .num_columns(8)
        .spacing([12.0, 8.0])
        .show(ui, |ui| {
            ui.label(RichText::new(lang.tr("Наименование материала", "Material Name")).strong());
            ui.label(RichText::new(lang.tr("Толщина (мм)", "Thickness (mm)")).strong());
            ui.label(RichText::new(lang.tr("Цвет 3D", "3D Color")).strong());
            ui.label(
                RichText::new(lang.tr("Текстура (PNG/JPG/WEBP)", "Texture (PNG/JPG/WEBP)")).strong(),
            );
            ui.label(RichText::new(lang.tr("Длина листа (мм)", "Sheet Length (mm)")).strong());
            ui.label(RichText::new(lang.tr("Ширина листа (мм)", "Sheet Width (mm)")).strong());
            ui.label(RichText::new(lang.tr("Текстура по умолч.", "Default Grain")).strong());
            ui.label(RichText::new(lang.tr("Действия", "Actions")).strong());
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
                    if ui
                        .button(lang.tr("📂 Загрузить...", "📂 Load..."))
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter(
                                lang.tr(
                                    "Изображения текстур (PNG, JPG, WEBP)",
                                    "Texture Images (PNG, JPG, WEBP)",
                                ),
                                &["png", "jpg", "jpeg", "webp"],
                            )
                            .pick_file()
                        {
                            match load_texture_from_path(ui.ctx(), &path, &m.name, lang) {
                                Ok(tex_info) => {
                                    loaded_textures.insert(m.id, tex_info.handle);
                                    m.texture_path = Some(path.clone());
                                    if let Some(fname) = path.file_name() {
                                        m.texture_relative_path = Some(fname.to_string_lossy().to_string());
                                    }
                                    if project.embed_textures {
                                        if let Ok(bytes) = std::fs::read(&path) {
                                            if bytes.len() <= 60 * 1024 * 1024 {
                                                use base64::Engine;
                                                m.texture_base64 = Some(base64::engine::general_purpose::STANDARD.encode(&bytes));
                                            }
                                        }
                                    } else {
                                        m.texture_base64 = None;
                                    }

                                    if let Some(ref w) = tex_info.warning {
                                        texture_warnings.insert(m.id, w.clone());
                                        *status_msg = match lang {
                                            AppLanguage::Ru => format!(
                                                "⚠️ Текстура для «{}» загружена: {}. {}",
                                                m.name,
                                                path.display(),
                                                w
                                            ),
                                            _ => format!(
                                                "⚠️ Texture for \"{}\" loaded: {}. {}",
                                                m.name,
                                                path.display(),
                                                w
                                            ),
                                        };
                                    } else {
                                        texture_warnings.remove(&m.id);
                                        *status_msg = match lang {
                                            AppLanguage::Ru => format!(
                                                "Загружена текстура для «{}»: {}",
                                                m.name,
                                                path.display()
                                            ),
                                            _ => format!(
                                                "Loaded texture for \"{}\": {}",
                                                m.name,
                                                path.display()
                                            ),
                                        };
                                    }
                                }
                                Err(e) => {
                                    *status_msg = e;
                                }
                            }
                        }
                    }
                    if m.texture_path.is_some() || m.texture_base64.is_some() {
                        if ui
                            .small_button(lang.tr("✖ Сброс", "✖ Reset"))
                            .clicked()
                        {
                            m.texture_path = None;
                            m.texture_relative_path = None;
                            m.texture_base64 = None;
                            loaded_textures.remove(&m.id);
                            texture_warnings.remove(&m.id);
                        }
                        let active_label = if m.texture_base64.is_some() {
                            lang.tr("✔ Текстура (упакована)", "✔ Texture (embedded)")
                        } else {
                            lang.tr("✔ Текстура активна", "✔ Texture active")
                        };
                        ui.label(
                            RichText::new(active_label)
                                .color(Color32::GREEN),
                        );
                        if let Some(warn) = texture_warnings.get(&m.id) {
                            ui.label(
                                RichText::new(format!("⚠️ {}", warn))
                                    .color(Color32::from_rgb(240, 180, 50))
                                    .small(),
                            );
                        }
                    } else {
                        ui.label(
                            RichText::new(lang.tr("Цвет (без фото)", "Solid color (no photo)"))
                                .color(Color32::GRAY),
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
                ui.checkbox(
                    &mut m.default_grain_fixed,
                    lang.tr("Фикс. волокна", "Lock grain"),
                );

                if ui
                    .add_enabled(
                        can_delete_mat,
                        egui::Button::new(
                            RichText::new(lang.tr("🗑 Удалить", "🗑 Delete"))
                                .color(Color32::from_rgb(245, 95, 95)),
                        ),
                    )
                    .on_hover_text(if can_delete_mat {
                        lang.tr(
                            "Удалить этот материал (потребуется подтверждение)",
                            "Delete this material (confirmation required)",
                        )
                    } else {
                        lang.tr(
                            "Нельзя удалить единственный материал в проекте",
                            "Cannot delete the only material in the project",
                        )
                    })
                    .clicked()
                {
                    *pending_delete_material = Some(m.id);
                }
                ui.end_row();
            }
        });

    // Модальное окно подтверждения удаления материала: «Точно удалить?»
    if let Some(del_id) = *pending_delete_material {
        let mat_name = project
            .get_material(del_id)
            .map(|m| m.name.clone())
            .unwrap_or_else(|| lang.tr("Материал", "Material").to_string());
        let used_parts_count = project
            .parts
            .iter()
            .filter(|p| p.material_id == del_id)
            .count();

        let mut close_dialog = false;
        let mut confirm_delete = false;

        egui::Window::new(lang.tr(
            "Подтверждение удаления материала",
            "Confirm Material Deletion",
        ))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, EguiVec2::ZERO)
        .show(ui.ctx(), |ui| {
            ui.add_space(4.0);
            let q_txt = match lang {
                AppLanguage::Ru => format!("Точно удалить материал «{}»?", mat_name),
                _ => format!("Are you sure you want to delete material \"{}\"?", mat_name),
            };
            ui.label(RichText::new(q_txt).strong().size(15.0));
            if used_parts_count > 0 {
                ui.add_space(4.0);
                let warn_txt = match lang {
                    AppLanguage::Ru => format!(
                        "Внимание: этот материал используется у {} деталей (им будет назначен первый доступный материал проекта).",
                        used_parts_count
                    ),
                    _ => format!(
                        "Warning: this material is used by {} parts (they will be reassigned to the first available material).",
                        used_parts_count
                    ),
                };
                ui.colored_label(Color32::from_rgb(240, 175, 50), warn_txt);
            }
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui
                    .button(
                        RichText::new(lang.tr("✔ Да, точно удалить", "✔ Yes, delete"))
                            .strong()
                            .color(Color32::from_rgb(250, 90, 90)),
                    )
                    .clicked()
                {
                    confirm_delete = true;
                    close_dialog = true;
                }
                ui.add_space(8.0);
                if ui.button(lang.tr("Отмена", "Cancel")).clicked() {
                    close_dialog = true;
                }
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
            *status_msg = match lang {
                AppLanguage::Ru => format!("Материал «{}» удален.", mat_name),
                _ => format!("Material \"{}\" deleted.", mat_name),
            };
        }

        if close_dialog {
            *pending_delete_material = None;
        }
    }

    ui.add_space(16.0);
    ui.separator();
    ui.label(
        RichText::new(lang.tr(
            "Справочник кромочных материалов (толщина вычитается при расчете пилильного размера):",
            "Edge banding catalog (thickness is subtracted when calculating net saw cutting size):",
        ))
        .strong(),
    );

    egui::Grid::new("edges_table_grid")
        .striped(true)
        .num_columns(4)
        .spacing([16.0, 8.0])
        .show(ui, |ui| {
            ui.label(RichText::new(lang.tr("Наименование кромки", "Edge Banding Name")).strong());
            ui.label(RichText::new(lang.tr("Толщина (мм)", "Thickness (mm)")).strong());
            ui.label(
                RichText::new(lang.tr("Цвет маркера на чертеже", "Drawing Marker Color")).strong(),
            );
            ui.label(RichText::new(lang.tr("Действия", "Actions")).strong());
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
                if ui
                    .button(
                        RichText::new(lang.tr("🗑 Удалить", "🗑 Delete"))
                            .color(Color32::from_rgb(245, 95, 95)),
                    )
                    .on_hover_text(lang.tr(
                        "Удалить этот тип кромки (потребуется подтверждение)",
                        "Delete this edge banding type (confirmation required)",
                    ))
                    .clicked()
                {
                    *pending_delete_edge = Some(e.id);
                }
                ui.end_row();
            }
        });

    // Модальное окно подтверждения удаления кромки
    if let Some(del_edge_id) = *pending_delete_edge {
        let edge_name = project
            .get_edge(del_edge_id)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| lang.tr("Кромка", "Edge Banding").to_string());
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

        egui::Window::new(lang.tr(
            "Подтверждение удаления кромки",
            "Confirm Edge Banding Deletion",
        ))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, EguiVec2::ZERO)
        .show(ui.ctx(), |ui| {
            ui.add_space(4.0);
            let q_txt = match lang {
                AppLanguage::Ru => format!("Точно удалить кромку «{}»?", edge_name),
                _ => {
                    format!("Are you sure you want to delete edge banding \"{}\"?", edge_name)
                }
            };
            ui.label(RichText::new(q_txt).strong().size(15.0));
            if used_sides_count > 0 {
                ui.add_space(4.0);
                let warn_txt = match lang {
                    AppLanguage::Ru => format!(
                        "Внимание: эта кромка назначена на {} гранях деталей (при удалении эти грани станут без кромки).",
                        used_sides_count
                    ),
                    _ => format!(
                        "Warning: this edge banding is assigned to {} part edges (they will be cleared upon deletion).",
                        used_sides_count
                    ),
                };
                ui.colored_label(Color32::from_rgb(240, 175, 50), warn_txt);
            }
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui
                    .button(
                        RichText::new(lang.tr("✔ Да, точно удалить", "✔ Yes, delete"))
                            .strong()
                            .color(Color32::from_rgb(250, 90, 90)),
                    )
                    .clicked()
                {
                    confirm_edge_delete = true;
                    close_edge_dialog = true;
                }
                ui.add_space(8.0);
                if ui.button(lang.tr("Отмена", "Cancel")).clicked() {
                    close_edge_dialog = true;
                }
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
            *status_msg = match lang {
                AppLanguage::Ru => format!("Кромка «{}» удалена.", edge_name),
                _ => format!("Edge banding \"{}\" deleted.", edge_name),
            };
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
    let dyn_img = image::load_from_memory(img_bytes).map_err(|e| match lang {
        AppLanguage::Ru => format!("Ошибка декодирования изображения текстуры: {}", e),
        _ => format!("Failed to decode texture image: {}", e),
    })?;

    let (orig_w, orig_h) = (dyn_img.width(), dyn_img.height());
    let gpu_limit = ctx.input(|i| i.max_texture_side) as u32;
    let max_allowed = if gpu_limit > 0 {
        gpu_limit.min(MAX_RECOMMENDED_TEXTURE_DIM)
    } else {
        MAX_RECOMMENDED_TEXTURE_DIM
    };

    let (final_img, warning) = if orig_w > max_allowed || orig_h > max_allowed {
        let resized = dyn_img.resize(max_allowed, max_allowed, image::imageops::FilterType::Triangle);
        let (nw, nh) = (resized.width(), resized.height());
        let warn_msg = match lang {
            AppLanguage::Ru => format!(
                "Разрешение изображения {}×{} px превышает лимит ({} px). Текстура автоматически уменьшена до {}×{} px для предотвращения перегрузки видеокарты.",
                orig_w, orig_h, max_allowed, nw, nh
            ),
            _ => format!(
                "Image resolution {}x{} px exceeds limit ({} px). Texture was automatically downscaled to {}x{} px to prevent GPU issues.",
                orig_w, orig_h, max_allowed, nw, nh
            ),
        };
        (resized, Some(warn_msg))
    } else {
        (dyn_img, None)
    };

    let rgba = final_img.to_rgba8();
    let dims = [rgba.width() as usize, rgba.height() as usize];
    let color_img = egui::ColorImage::from_rgba_unmultiplied(dims, rgba.as_flat_samples().as_slice());
    let handle = ctx.load_texture(name, color_img, egui::TextureOptions::LINEAR);

    Ok(LoadedTextureInfo {
        handle,
        warning,
    })
}

pub fn load_texture_from_path(
    ctx: &egui::Context,
    path: &std::path::Path,
    name: &str,
    lang: AppLanguage,
) -> Result<LoadedTextureInfo, String> {
    if let Ok(meta) = std::fs::metadata(path) {
        if meta.len() > 60 * 1024 * 1024 {
            return Err(match lang {
                AppLanguage::Ru => format!(
                    "Файл текстуры слишком большой ({:.1} МБ). Максимальный размер файла: 60 МБ.",
                    meta.len() as f64 / (1024.0 * 1024.0)
                ),
                _ => format!(
                    "Texture file is too large ({:.1} MB). Maximum supported file size is 60 MB.",
                    meta.len() as f64 / (1024.0 * 1024.0)
                ),
            });
        }
    }

    let img_bytes = std::fs::read(path).map_err(|e| match lang {
        AppLanguage::Ru => format!("Не удалось прочитать файл текстуры: {}", e),
        _ => format!("Failed to read texture file: {}", e),
    })?;

    load_texture_from_memory(ctx, &img_bytes, name, lang)
}

/// Находит и загружает текстуру материала, проверяя встроенный Base64 (Вариант B),
/// затем относительные пути от файла проекта (Вариант A: рядом с .qfurn или в подпапке textures/),
/// и исходный абсолютный путь
pub fn resolve_and_load_material_texture(
    ctx: &egui::Context,
    m: &mut Material,
    project_path: Option<&std::path::Path>,
    lang: AppLanguage,
) -> Result<LoadedTextureInfo, String> {
    // 1. Приоритет №1 (Вариант B): Встроенная в файл проекта Base64-текстура
    if let Some(ref b64) = m.texture_base64 {
        if !b64.trim().is_empty() {
            use base64::Engine;
            if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(b64.trim()) {
                return load_texture_from_memory(ctx, &bytes, &m.name, lang);
            }
        }
    }

    // 2. Приоритет №2 (Вариант A): Поиск по относительному пути от каталога проекта
    let project_dir = project_path
        .and_then(|p| p.parent())
        .unwrap_or(std::path::Path::new("."));

    let mut candidate_paths: Vec<std::path::PathBuf> = Vec::new();

    if let Some(ref rel) = m.texture_relative_path {
        candidate_paths.push(project_dir.join(rel));
        candidate_paths.push(project_dir.join("textures").join(rel));
    }

    if let Some(ref p) = m.texture_path {
        if let Some(fname) = p.file_name() {
            candidate_paths.push(project_dir.join(fname));
            candidate_paths.push(project_dir.join("textures").join(fname));
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

    Err(match lang {
        AppLanguage::Ru => format!("Файл текстуры для «{}» не найден", m.name),
        _ => format!("Texture file for \"{}\" not found", m.name),
    })
}

/// Вкладка «Спецификация и Экспорт»
pub fn ui_specification_tab(
    ui: &mut egui::Ui,
    project: &mut Project,
    status_msg: &mut String,
    lang: AppLanguage,
) {
    let mm_unit = lang.tr("мм", "mm");
    ui.horizontal(|ui| {
        ui.heading(lang.tr(
            "Ведомость деталей (Спецификация)",
            "Parts List (Specification)",
        ));
        ui.separator();
        ui.checkbox(
            &mut project.cutting_settings.group_identical_parts,
            lang.tr(
                "Группировать одинаковые детали (объединять в Кол-во: N шт.)",
                "Group identical parts (merge into Qty: N pcs)",
            ),
        );

        ui.separator();
        if ui
            .button(lang.tr("💾 Экспорт в CSV (Excel)", "💾 Export to CSV (Excel)"))
            .on_hover_text(lang.tr(
                "Сохранить полную таблицу деталей с габаритными и пилильными размерами в файл .csv",
                "Save full specification table with full and net saw dimensions to .csv",
            ))
            .clicked()
        {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name("spec_quadfurn.csv")
                .add_filter(lang.tr("Таблица CSV (Excel)", "CSV Spreadsheet"), &["csv"])
                .save_file()
            {
                match export_spec_to_csv_file(project, &path) {
                    Ok(()) => {
                        *status_msg = match lang {
                            AppLanguage::Ru => format!(
                                "Спецификация успешно экспортирована в {}",
                                path.display()
                            ),
                            _ => format!(
                                "Specification exported to {}",
                                path.display()
                            ),
                        };
                    }
                    Err(e) => *status_msg = e,
                }
            }
        }

        if ui
            .button(lang.tr("📄 Экспорт в PDF", "📄 Export to PDF"))
            .on_hover_text(lang.tr(
                "Сохранить полный паспорт изделия в .pdf (Спецификация + 4 проекции конструктора + Карты раскроя + Материалы и кромки)",
                "Save complete product passport to .pdf (Specification + 4 Designer Views + Cutting Layouts + Materials & Edges)",
            ))
            .clicked()
        {
            let default_pdf_name = format!("spec_{}.pdf", project.name.replace(' ', "_"));
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name(&default_pdf_name)
                .add_filter(lang.tr("Документ PDF", "PDF Document"), &["pdf"])
                .save_file()
            {
                match export_spec_to_pdf_file(project, &path, lang) {
                    Ok(()) => {
                        *status_msg = match lang {
                            AppLanguage::Ru => format!(
                                "Паспорт изделия (спецификация, проекции, раскрой и материалы) экспортирован в PDF: {}",
                                path.display()
                            ),
                            _ => format!(
                                "Product passport (specification, views, cutting layouts & materials) exported to PDF: {}",
                                path.display()
                            ),
                        };
                    }
                    Err(e) => *status_msg = e,
                }
            }
        }

        if ui
            .button(lang.tr(
                "📋 Копировать список раскроя",
                "📋 Copy Cutting List",
            ))
            .on_hover_text(lang.tr(
                "Скопировать таблицу пилильных размеров в буфер обмена",
                "Copy net saw cutting list to clipboard",
            ))
            .clicked()
        {
            let text = generate_cutting3_clipboard_text(project);
            ui.ctx().copy_text(text);
            *status_msg = lang
                .tr(
                    "Список деталей (пилильные размеры и кромки) скопирован в буфер обмена!",
                    "Parts cutting list (saw dimensions & edges) copied to clipboard!",
                )
                .to_string();
        }
    });

    ui.separator();

    let rows = build_specification(project);

    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
        egui::Grid::new("spec_full_grid")
            .striped(true)
            .num_columns(12)
            .spacing([14.0, 6.0])
            .show(ui, |ui| {
                ui.label(RichText::new("№").strong());
                ui.label(RichText::new(lang.tr("Группа", "Group")).strong());
                ui.label(RichText::new(lang.tr("Наименование", "Part Name")).strong());
                ui.label(RichText::new(lang.tr("Материал", "Material")).strong());
                ui.label(RichText::new(lang.tr("Габарит Д×Ш (мм)", "Full L×W (mm)")).strong());
                ui.label(
                    RichText::new(lang.tr("ПИЛ Д×Ш (мм)", "CUT L×W (mm)"))
                        .strong()
                        .color(Color32::from_rgb(110, 235, 145)),
                );
                ui.label(RichText::new(lang.tr("Кол-во", "Qty")).strong());
                ui.label(RichText::new(lang.tr("Волокна", "Grain")).strong());
                ui.label(RichText::new(lang.tr("Кромка Д1", "Edge L1")).strong());
                ui.label(RichText::new(lang.tr("Кромка Д2", "Edge L2")).strong());
                ui.label(RichText::new(lang.tr("Кромка Ш1", "Edge W1")).strong());
                ui.label(RichText::new(lang.tr("Кромка Ш2", "Edge W2")).strong());
                ui.end_row();

                for r in &rows {
                    ui.label(format!("{}", r.index));
                    ui.label(&r.group_name);
                    ui.label(RichText::new(&r.name).strong());
                    ui.label(format!("{} ({:.0}{})", r.material_name, r.thickness_mm, mm_unit));
                    ui.label(format!("{:.1} × {:.1}", r.full_length_mm, r.full_width_mm));
                    ui.label(
                        RichText::new(format!("{:.1} × {:.1}", r.cut_length_mm, r.cut_width_mm))
                            .strong()
                            .color(Color32::from_rgb(110, 235, 145)),
                    );
                    let qty_lbl = match lang {
                        AppLanguage::Ru => format!("{} шт.", r.quantity),
                        _ => format!("{} pcs", r.quantity),
                    };
                    ui.label(RichText::new(qty_lbl).strong());
                    ui.label(if r.grain_fixed {
                        lang.tr("🌲 Фикс. (По текстуре)", "🌲 Locked (Along grain)")
                    } else {
                        lang.tr("🔄 Вращ. свободно", "🔄 Free rotation")
                    });
                    ui.label(SpecRow::edge_label(r.edges.l1, &project.edge_bandings));
                    ui.label(SpecRow::edge_label(r.edges.l2, &project.edge_bandings));
                    ui.label(SpecRow::edge_label(r.edges.w1, &project.edge_bandings));
                    ui.label(SpecRow::edge_label(r.edges.w2, &project.edge_bandings));
                    ui.end_row();
                }
            });
    });
}

/// Вкладка «Карта раскроя»
pub fn ui_cutting_layout_tab(
    ui: &mut egui::Ui,
    project: &mut Project,
    selected_sheet_idx: &mut usize,
    lang: AppLanguage,
) {
    let mm_suffix = lang.tr(" мм", " mm");
    // Верхняя панель настроек раскроя (размер листа, отступы обпила от краев, ширина реза фрезы/пилы)
    ui.horizontal_wrapped(|ui| {
        ui.heading(lang.tr("Карта раскроя", "Cutting Layout"));
        ui.separator();

        ui.label(lang.tr("Ширина реза фрезы/пилы:", "Saw kerf width:"));
        ui.add(
            math_drag_value(&mut project.cutting_settings.kerf_mm)
                .speed(0.2)
                .range(0.0..=25.0)
                .suffix(mm_suffix),
        );

        ui.separator();
        ui.label(lang.tr(
            "Отступы от края листа (торцовка):",
            "Sheet edge trim margins:",
        ));
        ui.label(lang.tr("Слева:", "Left:"));
        ui.add(
            math_drag_value(&mut project.cutting_settings.trim_left_mm)
                .speed(1.0)
                .range(0.0..=100.0)
                .suffix(mm_suffix),
        );
        ui.label(lang.tr("Справа:", "Right:"));
        ui.add(
            math_drag_value(&mut project.cutting_settings.trim_right_mm)
                .speed(1.0)
                .range(0.0..=100.0)
                .suffix(mm_suffix),
        );
        ui.label(lang.tr("Сверху:", "Top:"));
        ui.add(
            math_drag_value(&mut project.cutting_settings.trim_top_mm)
                .speed(1.0)
                .range(0.0..=100.0)
                .suffix(mm_suffix),
        );
        ui.label(lang.tr("Снизу:", "Bottom:"));
        ui.add(
            math_drag_value(&mut project.cutting_settings.trim_bottom_mm)
                .speed(1.0)
                .range(0.0..=100.0)
                .suffix(mm_suffix),
        );

        ui.separator();
        ui.checkbox(
            &mut project.cutting_settings.group_identical_parts,
            lang.tr("Группировать одинаковые номера", "Group identical part numbers"),
        );
    });

    // Быстрое редактирование размеров листа для каждого материала прямо из окна раскроя
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new(lang.tr("Формат листов материалов:", "Material sheet sizes:")).strong(),
        );
        for m in &mut project.materials {
            ui.separator();
            ui.label(&m.name);
            ui.add(
                math_drag_value(&mut m.sheet_length_mm)
                    .speed(10.0)
                    .range(500.0..=6000.0)
                    .prefix(lang.tr("Д: ", "L: "))
                    .suffix(mm_suffix),
            );
            ui.add(
                math_drag_value(&mut m.sheet_width_mm)
                    .speed(10.0)
                    .range(500.0..=4000.0)
                    .prefix(lang.tr("Ш: ", "W: "))
                    .suffix(mm_suffix),
            );
        }
    });

    ui.separator();

    let report: CuttingReport = compute_cutting_layout(project);

    if !report.unplaced_parts.is_empty() {
        for err in &report.unplaced_parts {
            let err_msg = match lang {
                AppLanguage::Ru => format!("⚠ Деталь не помещается на лист: {}", err),
                _ => format!("⚠ Part does not fit on sheet: {}", err),
            };
            ui.colored_label(Color32::from_rgb(255, 95, 95), err_msg);
        }
        ui.separator();
    }

    // Сводная статистика раскроя и погонажа кромок
    ui.horizontal_wrapped(|ui| {
        let sheets_txt = match lang {
            AppLanguage::Ru => format!("Всего листов: {} шт.", report.total_sheets_count),
            _ => format!("Total sheets: {} pcs", report.total_sheets_count),
        };
        ui.label(
            RichText::new(sheets_txt)
                .strong()
                .color(Color32::from_rgb(255, 215, 90)),
        );
        ui.separator();
        let area_txt = match lang {
            AppLanguage::Ru => format!(
                "Площадь деталей: {:.2} м² из {:.2} м² (КПИ: {:.1}%)",
                report.total_parts_area_m2,
                report.total_sheets_area_m2,
                report.overall_utilization_percent
            ),
            _ => format!(
                "Parts area: {:.2} m² of {:.2} m² (Yield: {:.1}%)",
                report.total_parts_area_m2,
                report.total_sheets_area_m2,
                report.overall_utilization_percent
            ),
        };
        ui.label(area_txt);
        ui.separator();
        let cut_len_txt = match lang {
            AppLanguage::Ru => format!("Длина реза: {:.1} м.п.", report.total_cut_length_m),
            _ => format!("Cut length: {:.1} m", report.total_cut_length_m),
        };
        ui.label(cut_len_txt);

        for (ename, meters, rgb) in &report.edge_banding_totals_m {
            ui.separator();
            let edge_tot_txt = match lang {
                AppLanguage::Ru => format!("Кромка {}: {:.2} м.п.", ename, meters),
                _ => format!("Edge {}: {:.2} m", ename, meters),
            };
            ui.colored_label(Color32::from_rgb(rgb[0], rgb[1], rgb[2]), edge_tot_txt);
        }
    });

    ui.separator();

    if report.sheets.is_empty() {
        ui.label(lang.tr(
            "В проекте пока нет деталей для раскроя.",
            "No parts in the project for cutting yet.",
        ));
        return;
    }

    if *selected_sheet_idx >= report.sheets.len() {
        *selected_sheet_idx = 0;
    }

    // Выбор листа для просмотра
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(lang.tr("Листы раскроя:", "Cutting sheets:")).strong());
        for (idx, sh) in report.sheets.iter().enumerate() {
            let title = match lang {
                AppLanguage::Ru => format!(
                    "Лист #{} (Мат. лист №{}) — {} ({:.0}×{:.0} мм, КПИ {:.1}%)",
                    idx + 1,
                    sh.sheet_number,
                    sh.material_name,
                    sh.sheet_length_mm,
                    sh.sheet_width_mm,
                    sh.utilization_percent
                ),
                _ => format!(
                    "Sheet #{} (Mat. sheet #{}) — {} ({:.0}×{:.0} mm, Yield {:.1}%)",
                    idx + 1,
                    sh.sheet_number,
                    sh.material_name,
                    sh.sheet_length_mm,
                    sh.sheet_width_mm,
                    sh.utilization_percent
                ),
            };
            if ui.selectable_label(*selected_sheet_idx == idx, title).clicked() {
                *selected_sheet_idx = idx;
            }
        }
    });

    ui.add_space(4.0);

    // Отрисовка выбранного листа раскроя и таблицы деталей на этом листе
    let active_sheet = &report.sheets[*selected_sheet_idx];
    let avail = ui.available_rect_before_wrap();
    let diagram_width = (avail.width() * 0.68).max(300.0);

    ui.horizontal(|ui| {
        let (canvas_rect, _) = ui.allocate_exact_size(
            EguiVec2::new(diagram_width, avail.height().max(260.0)),
            Sense::hover(),
        );
        draw_cutting_sheet_diagram(ui, canvas_rect, active_sheet, project, lang);

        ui.vertical(|ui| {
            let sheet_parts_hdr = match lang {
                AppLanguage::Ru => format!(
                    "Детали на листе #{} ({} шт.)",
                    *selected_sheet_idx + 1,
                    active_sheet.placed_parts.len()
                ),
                _ => format!(
                    "Parts on Sheet #{} ({} pcs)",
                    *selected_sheet_idx + 1,
                    active_sheet.placed_parts.len()
                ),
            };
            ui.heading(sheet_parts_hdr);
            ui.label(
                RichText::new(lang.tr(
                    "В раскрой передаются пилильные размеры (за вычетом кромок):",
                    "Net saw cutting sizes (after subtracting edge bandings):",
                ))
                .color(Color32::GRAY),
            );
            egui::ScrollArea::vertical()
                .id_salt("sheet_parts_list")
                .show(ui, |ui| {
                    egui::Grid::new("sheet_parts_grid")
                        .striped(true)
                        .num_columns(5)
                        .spacing([8.0, 4.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("№").strong());
                            ui.label(RichText::new(lang.tr("Деталь", "Part")).strong());
                            ui.label(RichText::new(lang.tr("Пил (мм)", "Cut (mm)")).strong());
                            ui.label(RichText::new(lang.tr("Габарит", "Full")).strong());
                            ui.label(RichText::new(lang.tr("Поворот", "Rotation")).strong());
                            ui.end_row();

                            for p in &active_sheet.placed_parts {
                                ui.label(format!("#{}", p.spec_index));
                                ui.label(&p.name);
                                ui.label(
                                    RichText::new(format!(
                                        "{:.1}×{:.1}",
                                        p.cut_length_mm, p.cut_width_mm
                                    ))
                                    .color(Color32::from_rgb(110, 235, 145)),
                                );
                                ui.label(format!("{:.0}×{:.0}", p.full_length_mm, p.full_width_mm));
                                ui.label(if p.rotated_90 {
                                    lang.tr("Повернута 90°", "Rotated 90°")
                                } else if p.grain_fixed {
                                    lang.tr("🌲 По волокнам", "🌲 Along grain")
                                } else {
                                    "0°"
                                });
                                ui.end_row();
                            }
                        });

                    if !active_sheet.remnants.is_empty() {
                        ui.add_space(10.0);
                        ui.separator();
                        ui.label(
                            RichText::new(lang.tr(
                                "Деловые остатки (обрезки) на листе:",
                                "Usable offcuts (remnants) on sheet:",
                            ))
                            .strong(),
                        );
                        for (ri, rem) in active_sheet.remnants.iter().enumerate() {
                            let rem_lbl = match lang {
                                AppLanguage::Ru => format!(
                                    "• Остаток {}: {:.0} × {:.0} мм",
                                    ri + 1,
                                    rem.w_mm,
                                    rem.h_mm
                                ),
                                _ => format!(
                                    "• Offcut {}: {:.0} × {:.0} mm",
                                    ri + 1,
                                    rem.w_mm,
                                    rem.h_mm
                                ),
                            };
                            ui.label(rem_lbl);
                        }
                    }
                });
        });
    });
}

/// Отрисовка графической карты раскроя листа в стиле Cutting 3
fn draw_cutting_sheet_diagram(
    ui: &egui::Ui,
    canvas_rect: Rect,
    sheet: &SheetLayout,
    project: &Project,
    lang: AppLanguage,
) {
    let painter = ui.painter_at(canvas_rect);
    painter.rect_filled(canvas_rect, 4.0, Color32::from_rgb(20, 24, 29));

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

    // Фон сырого листа (зона обпила краев)
    painter.rect_filled(sheet_rect, 0.0, Color32::from_rgb(62, 48, 42));
    painter.rect_stroke(sheet_rect, 0.0, Stroke::new(2.0, Color32::WHITE));

    // Подпись габаритов листа и направления волокон листа
    let top_caption = match lang {
        AppLanguage::Ru => format!(
            "Длина листа: {:.0} мм (Направление текстуры / волокон ───►) | Ширина реза фрезы: {:.1} мм",
            sheet.sheet_length_mm, project.cutting_settings.kerf_mm
        ),
        _ => format!(
            "Sheet length: {:.0} mm (Grain direction ───►) | Saw kerf: {:.1} mm",
            sheet.sheet_length_mm, project.cutting_settings.kerf_mm
        ),
    };
    painter.text(
        Pos2::new(sheet_rect.center().x, sheet_rect.top() - 12.0),
        egui::Align2::CENTER_CENTER,
        top_caption,
        FontId::proportional(11.5),
        Color32::from_rgb(220, 225, 235),
    );

    // 1. Отрисовка свободных остатков (обрезков) со штриховкой и размерами, как в Cutting 3
    for rem in &sheet.remnants {
        let r_min = Pos2::new(origin.x + rem.x_mm * scale, origin.y + rem.y_mm * scale);
        let r_rect = Rect::from_min_size(
            r_min,
            EguiVec2::new(rem.w_mm * scale, rem.h_mm * scale),
        );
        painter.rect_filled(r_rect, 0.0, Color32::from_rgb(35, 42, 48));
        painter.rect_stroke(r_rect, 0.0, Stroke::new(1.0, Color32::from_gray(90)));
        if r_rect.width() > 45.0 && r_rect.height() > 22.0 {
            let rem_word = lang.tr("Остаток", "Offcut");
            painter.text(
                r_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("{}\n{:.0}×{:.0}", rem_word, rem.w_mm, rem.h_mm),
                FontId::monospace(9.5),
                Color32::from_gray(145),
            );
        }
    }

    // 2. Отрисовка размещенных деталей на листе
    let mat_rgb = project
        .get_material(sheet.material_id)
        .map(|m| m.color_rgb)
        .unwrap_or([215, 195, 165]);

    for p in &sheet.placed_parts {
        let p_min = Pos2::new(origin.x + p.x_mm * scale, origin.y + p.y_mm * scale);
        let p_rect = Rect::from_min_size(
            p_min,
            EguiVec2::new(p.w_on_sheet_mm * scale, p.h_on_sheet_mm * scale),
        );

        painter.rect_filled(
            p_rect,
            0.0,
            Color32::from_rgb(mat_rgb[0], mat_rgb[1], mat_rgb[2]),
        );
        painter.rect_stroke(p_rect, 0.0, Stroke::new(1.2, Color32::from_rgb(20, 20, 20)));

        // Отрисовка маркеров кромки внутри детали на карте раскроя (как в Cutting 3)
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

        // Подпись номера детали, имени и пилильного размера на карте раскроя
        if p_rect.width() > 36.0 && p_rect.height() > 16.0 {
            let label = if p_rect.width() > 85.0 && p_rect.height() > 34.0 {
                format!(
                    "#{} {}\n{:.1}×{:.1}",
                    p.spec_index, p.name, p.w_on_sheet_mm, p.h_on_sheet_mm
                )
            } else {
                format!("#{} {:.0}×{:.0}", p.spec_index, p.w_on_sheet_mm, p.h_on_sheet_mm)
            };
            painter.text(
                p_rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                FontId::monospace(10.0),
                Color32::from_rgb(15, 15, 15),
            );
        }
    }

    // 3. Отрисовка сквозных линий гильотинного реза фрезы/пилы
    for cl in &sheet.cut_lines {
        let p1 = Pos2::new(origin.x + cl.x1 * scale, origin.y + cl.y1 * scale);
        let p2 = Pos2::new(origin.x + cl.x2 * scale, origin.y + cl.y2 * scale);
        painter.line_segment([p1, p2], Stroke::new(1.2, Color32::from_rgb(250, 75, 75)));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        copy_selected_parts_to_clipboard, eval_math_expression, paste_parts_into_project,
    };
    use crate::model::Project;

    #[test]
    fn test_eval_math_expression() {
        assert_eq!(eval_math_expression("600+100"), Some(700.0));
        assert_eq!(eval_math_expression("600-4"), Some(596.0));
        assert_eq!(eval_math_expression("600 мм + 100"), Some(700.0));
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

        let copied =
            copy_selected_parts_to_clipboard(&src_proj, selected_ids.first().copied(), &selected_ids);
        assert_eq!(copied.len(), 4);

        let mut dst_proj = Project::new_empty("Новый проект".to_string());
        assert_eq!(dst_proj.parts.len(), 0);

        let new_ids = paste_parts_into_project(&mut dst_proj, &copied);
        assert_eq!(new_ids.len(), 4);
        assert_eq!(dst_proj.parts.len(), 4);

        // Проверяем, что взаимное расположение скопированных деталей полностью сохранилось
        let orig_delta = src_proj.parts[1].position_mm - src_proj.parts[0].position_mm;
        let pasted_delta = dst_proj.parts[1].position_mm - dst_proj.parts[0].position_mm;
        assert!((orig_delta - pasted_delta).length() < 1e-4);
    }
}


