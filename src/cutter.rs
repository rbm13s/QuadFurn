use crate::i18n::AppLanguage;
use crate::model::{EdgeBanding, MaterialId, PartEdges, PartId, Project};

/// Row in summary parts specification (accounting for optional grouping of identical parts).
#[derive(Clone, Debug)]
pub struct SpecRow {
    pub index: usize,
    pub part_ids: Vec<PartId>,
    pub group_name: String,
    pub name: String,
    pub material_id: MaterialId,
    pub material_name: String,
    pub thickness_mm: f32,
    /// Finished (overall) dimensions including edge banding.
    pub full_length_mm: f32,
    pub full_width_mm: f32,
    /// Saw cutting dimensions for panel saw / CNC (excluding edge banding thickness).
    pub cut_length_mm: f32,
    pub cut_width_mm: f32,
    pub quantity: usize,
    pub grain_fixed: bool,
    pub edges: PartEdges,
}

impl SpecRow {
    /// Formats edge banding description using the default language.
    #[allow(dead_code)]
    pub fn edge_label(edge_id: Option<u64>, edges_lib: &[EdgeBanding]) -> String {
        Self::edge_label_i18n(edge_id, edges_lib, AppLanguage::Ru)
    }

    /// Formats edge banding description localized to `lang`.
    pub fn edge_label_i18n(
        edge_id: Option<u64>,
        edges_lib: &[EdgeBanding],
        lang: AppLanguage,
    ) -> String {
        let mm = lang.dict().unit_mm;
        edge_id
            .and_then(|id| edges_lib.iter().find(|e| e.id == id))
            .map(|e| format!("{:.1} {} ({})", e.thickness_mm, mm, e.name))
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn edge_short_mm(edge_id: Option<u64>, edges_lib: &[EdgeBanding]) -> String {
        edge_id
            .and_then(|id| edges_lib.iter().find(|e| e.id == id))
            .map(|e| format!("{:.1}", e.thickness_mm))
            .unwrap_or_else(|| "-".to_string())
    }
}

/// Builds specification table using the default Russian language fallback.
pub fn build_specification(project: &Project) -> Vec<SpecRow> {
    build_specification_i18n(project, AppLanguage::Ru)
}

/// Builds specification table of project parts localized to `lang`.
pub fn build_specification_i18n(project: &Project, lang: AppLanguage) -> Vec<SpecRow> {
    let mut rows: Vec<SpecRow> = Vec::new();
    let group_identical = project.cutting_settings.group_identical_parts;

    for part in &project.parts {
        let mat = project.get_material(part.material_id);
        let mat_name = mat
            .map(|m| m.name.clone())
            .unwrap_or_else(|| lang.tr("Неизвестный", "Unknown").to_string());
        let thick = mat.map(|m| m.thickness_mm).unwrap_or(18.0);
        let grp_name = part
            .group_id
            .and_then(|gid| project.groups.iter().find(|g| g.id == gid))
            .map(|g| g.name.clone())
            .unwrap_or_else(|| lang.tr("Без группы", "Ungrouped").to_string());

        // If grain_along_width == true, swap effective length and width along texture
        let (full_l, full_w, edges) = if part.grain_along_width {
            (
                part.width_mm,
                part.length_mm,
                PartEdges {
                    l1: part.edges.w1,
                    l2: part.edges.w2,
                    w1: part.edges.l1,
                    w2: part.edges.l2,
                },
            )
        } else {
            (part.length_mm, part.width_mm, part.edges)
        };

        let (cut_l_raw, cut_w_raw) = part.cut_dimensions_mm(&project.edge_bandings);
        let (cut_l, cut_w) = if part.grain_along_width {
            (cut_w_raw, cut_l_raw)
        } else {
            (cut_l_raw, cut_w_raw)
        };

        if group_identical {
            if let Some(existing) = rows.iter_mut().find(|r| {
                r.material_id == part.material_id
                    && (r.full_length_mm - full_l).abs() < 0.05
                    && (r.full_width_mm - full_w).abs() < 0.05
                    && (r.cut_length_mm - cut_l).abs() < 0.05
                    && (r.cut_width_mm - cut_w).abs() < 0.05
                    && r.grain_fixed == part.grain_fixed
                    && r.edges == edges
            }) {
                existing.quantity += 1;
                existing.part_ids.push(part.id);
                if !existing.name.contains(&part.name) && existing.part_ids.len() <= 3 {
                    existing.name = format!("{} / {}", existing.name, part.name);
                }
                continue;
            }
        }

        let next_idx = rows.len() + 1;
        rows.push(SpecRow {
            index: next_idx,
            part_ids: vec![part.id],
            group_name: grp_name,
            name: part.name.clone(),
            material_id: part.material_id,
            material_name: mat_name,
            thickness_mm: thick,
            full_length_mm: full_l,
            full_width_mm: full_w,
            cut_length_mm: cut_l,
            cut_width_mm: cut_w,
            quantity: 1,
            grain_fixed: part.grain_fixed,
            edges,
        });
    }

    rows
}

/// A part placed on a sheet in the cutting layout.
#[derive(Clone, Debug)]
pub struct PlacedPart {
    pub spec_index: usize,
    #[allow(dead_code)]
    pub part_id: PartId,
    pub name: String,
    /// X coordinate on sheet (along sheet length, mm).
    pub x_mm: f32,
    /// Y coordinate on sheet (along sheet width, mm).
    pub y_mm: f32,
    /// Dimension along sheet length (mm).
    pub w_on_sheet_mm: f32,
    /// Dimension along sheet width (mm).
    pub h_on_sheet_mm: f32,
    /// Saw cutting dimensions (Length x Width).
    pub cut_length_mm: f32,
    pub cut_width_mm: f32,
    /// Overall dimensions (Length x Width).
    pub full_length_mm: f32,
    pub full_width_mm: f32,
    /// Whether the part is rotated 90 degrees relative to sheet grain direction.
    pub rotated_90: bool,
    pub grain_fixed: bool,
    pub edges: PartEdges,
}

/// Free offcut / remnant on the sheet.
#[derive(Clone, Debug)]
pub struct SheetRemnant {
    pub x_mm: f32,
    pub y_mm: f32,
    pub w_mm: f32,
    pub h_mm: f32,
}

/// Guillotine through-cut line of blade/router on the sheet.
#[derive(Clone, Debug)]
pub struct CutLine {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

/// One cut sheet of material.
#[derive(Clone, Debug)]
pub struct SheetLayout {
    pub sheet_number: usize,
    pub material_id: MaterialId,
    pub material_name: String,
    pub sheet_length_mm: f32,
    pub sheet_width_mm: f32,
    pub placed_parts: Vec<PlacedPart>,
    pub remnants: Vec<SheetRemnant>,
    pub cut_lines: Vec<CutLine>,
    pub utilization_percent: f32,
}

/// Information about a part that physically cannot fit onto the sheet usable area.
#[derive(Clone, Debug, PartialEq)]
pub struct UnplacedPart {
    pub name: String,
    pub cut_length_mm: f32,
    pub cut_width_mm: f32,
    pub work_width_mm: f32,
    pub work_height_mm: f32,
}

impl UnplacedPart {
    /// Formats localized diagnostic message explaining why the part could not be placed.
    pub fn format_message(&self, lang: AppLanguage) -> String {
        let mm = lang.dict().unit_mm;
        match lang {
            AppLanguage::Ru => format!(
                "{} ({:.1} x {:.1} {} > рабочего поля листа {:.0} x {:.0} {})",
                self.name,
                self.cut_length_mm,
                self.cut_width_mm,
                mm,
                self.work_width_mm,
                self.work_height_mm,
                mm
            ),
            AppLanguage::Uk => format!(
                "{} ({:.1} x {:.1} {} > робочого поля листа {:.0} x {:.0} {})",
                self.name,
                self.cut_length_mm,
                self.cut_width_mm,
                mm,
                self.work_width_mm,
                self.work_height_mm,
                mm
            ),
            _ => format!(
                "{} ({:.1} x {:.1} {} > sheet usable area {:.0} x {:.0} {})",
                self.name,
                self.cut_length_mm,
                self.cut_width_mm,
                mm,
                self.work_width_mm,
                self.work_height_mm,
                mm
            ),
        }
    }
}

impl std::fmt::Display for UnplacedPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({:.1} x {:.1} mm > sheet usable area {:.0} x {:.0} mm)",
            self.name,
            self.cut_length_mm,
            self.cut_width_mm,
            self.work_width_mm,
            self.work_height_mm
        )
    }
}

/// Summary cutting report for the entire project.
#[derive(Clone, Debug, Default)]
pub struct CuttingReport {
    pub sheets: Vec<SheetLayout>,
    pub unplaced_parts: Vec<UnplacedPart>,
    pub total_sheets_count: usize,
    pub total_parts_area_m2: f32,
    pub total_sheets_area_m2: f32,
    pub overall_utilization_percent: f32,
    pub total_cut_length_m: f32,
    /// Total edge banding run per type: (Edge name, linear meters without waste).
    pub edge_banding_totals_m: Vec<(String, f32, [u8; 3])>,
}

/// Executes guillotine cutting optimization (Cutting 3 style) for all project materials.
pub fn compute_cutting_layout(project: &Project) -> CuttingReport {
    let spec_rows = build_specification(project);
    let settings = &project.cutting_settings;
    let kerf = settings.kerf_mm.max(0.0);

    let mut report = CuttingReport::default();

    // Calculate total edge banding linear meters across all parts
    for edge_def in &project.edge_bandings {
        let mut total_mm = 0.0_f32;
        for row in &spec_rows {
            let q = row.quantity as f32;
            if row.edges.l1 == Some(edge_def.id) {
                total_mm += row.full_length_mm * q;
            }
            if row.edges.l2 == Some(edge_def.id) {
                total_mm += row.full_length_mm * q;
            }
            if row.edges.w1 == Some(edge_def.id) {
                total_mm += row.full_width_mm * q;
            }
            if row.edges.w2 == Some(edge_def.id) {
                total_mm += row.full_width_mm * q;
            }
        }
        if total_mm > 0.0 {
            report.edge_banding_totals_m.push((
                edge_def.name.clone(),
                total_mm / 1000.0,
                edge_def.color_rgb,
            ));
        }
    }

    // Group cutting tasks by material
    for mat in &project.materials {
        let mut items_to_pack: Vec<PlacedPart> = Vec::new();
        for row in spec_rows.iter().filter(|r| r.material_id == mat.id) {
            for &pid in &row.part_ids {
                items_to_pack.push(PlacedPart {
                    spec_index: row.index,
                    part_id: pid,
                    name: row.name.clone(),
                    x_mm: 0.0,
                    y_mm: 0.0,
                    w_on_sheet_mm: row.cut_length_mm,
                    h_on_sheet_mm: row.cut_width_mm,
                    cut_length_mm: row.cut_length_mm,
                    cut_width_mm: row.cut_width_mm,
                    full_length_mm: row.full_length_mm,
                    full_width_mm: row.full_width_mm,
                    rotated_90: false,
                    grain_fixed: row.grain_fixed,
                    edges: row.edges,
                });
            }
        }

        if items_to_pack.is_empty() {
            continue;
        }

        // Sort parts descending by area and longer dimension (standard cutting heuristic)
        items_to_pack.sort_by(|a, b| {
            let max_a = a.cut_length_mm.max(a.cut_width_mm);
            let max_b = b.cut_length_mm.max(b.cut_width_mm);
            let area_a = a.cut_length_mm * a.cut_width_mm;
            let area_b = b.cut_length_mm * b.cut_width_mm;
            area_b
                .partial_cmp(&area_a)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| max_b.partial_cmp(&max_a).unwrap_or(std::cmp::Ordering::Equal))
        });

        let work_x0 = settings.trim_left_mm;
        let work_y0 = settings.trim_top_mm;
        let work_w =
            (mat.sheet_length_mm - settings.trim_left_mm - settings.trim_right_mm).max(10.0);
        let work_h =
            (mat.sheet_width_mm - settings.trim_top_mm - settings.trim_bottom_mm).max(10.0);

        let mut remaining = items_to_pack;
        let mut mat_sheet_idx = 1;

        while !remaining.is_empty() {
            let mut free_rects = vec![SheetRemnant {
                x_mm: work_x0,
                y_mm: work_y0,
                w_mm: work_w,
                h_mm: work_h,
            }];
            let mut placed_on_sheet = Vec::new();
            let mut cut_lines = Vec::new();

            // Perimeter trimming cuts
            if settings.trim_left_mm > 0.1 {
                cut_lines.push(CutLine {
                    x1: work_x0,
                    y1: 0.0,
                    x2: work_x0,
                    y2: mat.sheet_width_mm,
                });
            }
            if settings.trim_right_mm > 0.1 {
                cut_lines.push(CutLine {
                    x1: work_x0 + work_w,
                    y1: 0.0,
                    x2: work_x0 + work_w,
                    y2: mat.sheet_width_mm,
                });
            }
            if settings.trim_top_mm > 0.1 {
                cut_lines.push(CutLine {
                    x1: 0.0,
                    y1: work_y0,
                    x2: mat.sheet_length_mm,
                    y2: work_y0,
                });
            }
            if settings.trim_bottom_mm > 0.1 {
                cut_lines.push(CutLine {
                    x1: 0.0,
                    y1: work_y0 + work_h,
                    x2: mat.sheet_length_mm,
                    y2: work_y0 + work_h,
                });
            }

            let mut any_placed = false;
            let mut next_remaining = Vec::new();

            for mut item in remaining {
                // Find best free rectangle (Best Short Side Fit / Guillotine Split)
                let mut best_rect_idx: Option<usize> = None;
                let mut best_rot = false;
                let mut best_score = f32::MAX;

                for (r_idx, rect) in free_rects.iter().enumerate() {
                    // Option 1: without rotation (part length along sheet length X)
                    if item.cut_length_mm <= rect.w_mm + 0.01
                        && item.cut_width_mm <= rect.h_mm + 0.01
                    {
                        let leftover_w = (rect.w_mm - item.cut_length_mm).abs();
                        let leftover_h = (rect.h_mm - item.cut_width_mm).abs();
                        let score = leftover_w.min(leftover_h) + leftover_w.max(leftover_h) * 0.05;
                        if score < best_score {
                            best_score = score;
                            best_rect_idx = Some(r_idx);
                            best_rot = false;
                        }
                    }

                    // Option 2: 90 degree rotation (only if grain is NOT fixed!)
                    if !item.grain_fixed
                        && item.cut_width_mm <= rect.w_mm + 0.01
                        && item.cut_length_mm <= rect.h_mm + 0.01
                    {
                        let leftover_w = (rect.w_mm - item.cut_width_mm).abs();
                        let leftover_h = (rect.h_mm - item.cut_length_mm).abs();
                        let score = leftover_w.min(leftover_h) + leftover_w.max(leftover_h) * 0.05;
                        if score < best_score {
                            best_score = score;
                            best_rect_idx = Some(r_idx);
                            best_rot = true;
                        }
                    }
                }

                if let Some(r_idx) = best_rect_idx {
                    let rect = free_rects.remove(r_idx);
                    let (pw, ph) = if best_rot {
                        (item.cut_width_mm, item.cut_length_mm)
                    } else {
                        (item.cut_length_mm, item.cut_width_mm)
                    };

                    item.x_mm = rect.x_mm;
                    item.y_mm = rect.y_mm;
                    item.w_on_sheet_mm = pw;
                    item.h_on_sheet_mm = ph;
                    item.rotated_90 = best_rot;
                    placed_on_sheet.push(item);
                    any_placed = true;

                    // Guillotine split of remaining free space accounting for cutter kerf
                    let right_w = rect.w_mm - pw - kerf;
                    let bottom_h = rect.h_mm - ph - kerf;

                    // Choose guillotine cut direction (along longer offcut, typical for panel saws)
                    if right_w * rect.h_mm > bottom_h * rect.w_mm {
                        // First vertical cut across entire strip height rect.h_mm
                        if right_w > 1.0 {
                            cut_lines.push(CutLine {
                                x1: rect.x_mm + pw + kerf * 0.5,
                                y1: rect.y_mm,
                                x2: rect.x_mm + pw + kerf * 0.5,
                                y2: rect.y_mm + rect.h_mm,
                            });
                            free_rects.push(SheetRemnant {
                                x_mm: rect.x_mm + pw + kerf,
                                y_mm: rect.y_mm,
                                w_mm: right_w,
                                h_mm: rect.h_mm,
                            });
                        }
                        if bottom_h > 1.0 {
                            cut_lines.push(CutLine {
                                x1: rect.x_mm,
                                y1: rect.y_mm + ph + kerf * 0.5,
                                x2: rect.x_mm + pw,
                                y2: rect.y_mm + ph + kerf * 0.5,
                            });
                            free_rects.push(SheetRemnant {
                                x_mm: rect.x_mm,
                                y_mm: rect.y_mm + ph + kerf,
                                w_mm: pw,
                                h_mm: bottom_h,
                            });
                        }
                    } else {
                        // First horizontal cut along strip width rect.w_mm
                        if bottom_h > 1.0 {
                            cut_lines.push(CutLine {
                                x1: rect.x_mm,
                                y1: rect.y_mm + ph + kerf * 0.5,
                                x2: rect.x_mm + rect.w_mm,
                                y2: rect.y_mm + ph + kerf * 0.5,
                            });
                            free_rects.push(SheetRemnant {
                                x_mm: rect.x_mm,
                                y_mm: rect.y_mm + ph + kerf,
                                w_mm: rect.w_mm,
                                h_mm: bottom_h,
                            });
                        }
                        if right_w > 1.0 {
                            cut_lines.push(CutLine {
                                x1: rect.x_mm + pw + kerf * 0.5,
                                y1: rect.y_mm,
                                x2: rect.x_mm + pw + kerf * 0.5,
                                y2: rect.y_mm + ph,
                            });
                            free_rects.push(SheetRemnant {
                                x_mm: rect.x_mm + pw + kerf,
                                y_mm: rect.y_mm,
                                w_mm: right_w,
                                h_mm: ph,
                            });
                        }
                    }
                } else {
                    next_remaining.push(item);
                }
            }

            if !any_placed {
                // Parts that physically exceed the usable sheet dimensions
                for bad in next_remaining {
                    report.unplaced_parts.push(UnplacedPart {
                        name: bad.name,
                        cut_length_mm: bad.cut_length_mm,
                        cut_width_mm: bad.cut_width_mm,
                        work_width_mm: work_w,
                        work_height_mm: work_h,
                    });
                }
                break;
            }

            let sheet_area = mat.sheet_length_mm * mat.sheet_width_mm;
            let parts_area: f32 = placed_on_sheet
                .iter()
                .map(|p| p.w_on_sheet_mm * p.h_on_sheet_mm)
                .sum();
            let util = (parts_area / sheet_area.max(1.0)) * 100.0;

            for cl in &cut_lines {
                let dx = cl.x2 - cl.x1;
                let dy = cl.y2 - cl.y1;
                report.total_cut_length_m += (dx * dx + dy * dy).sqrt() / 1000.0;
            }

            report.total_parts_area_m2 += parts_area / 1_000_000.0;
            report.total_sheets_area_m2 += sheet_area / 1_000_000.0;

            // Retain only noticeable remnants in the list (>= 50 mm on each side)
            let useful_remnants = free_rects
                .into_iter()
                .filter(|r| r.w_mm >= 50.0 && r.h_mm >= 50.0)
                .collect();

            report.sheets.push(SheetLayout {
                sheet_number: mat_sheet_idx,
                material_id: mat.id,
                material_name: mat.name.clone(),
                sheet_length_mm: mat.sheet_length_mm,
                sheet_width_mm: mat.sheet_width_mm,
                placed_parts: placed_on_sheet,
                remnants: useful_remnants,
                cut_lines,
                utilization_percent: util,
            });

            mat_sheet_idx += 1;
            remaining = next_remaining;
        }
    }

    report.total_sheets_count = report.sheets.len();
    if report.total_sheets_area_m2 > 0.0001 {
        report.overall_utilization_percent =
            (report.total_parts_area_m2 / report.total_sheets_area_m2) * 100.0;
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Project;

    #[test]
    fn test_cut_dimensions_subtract_edge_banding() {
        let proj = Project::new_demo_cabinet();
        // Check left side panel (720 x 500, L1 = 2.0 mm, W1 = 0.4 mm, W2 = 0.4 mm)
        let side = proj.parts.iter().find(|p| p.id == 1001).unwrap();
        let (cut_l, cut_w) = side.cut_dimensions_mm(&proj.edge_bandings);
        // Length decreases by W1 + W2 (0.4 + 0.4 = 0.8 mm) -> 719.2 mm
        // Width decreases by L1 + L2 (2.0 + 0.0 = 2.0 mm) -> 498.0 mm
        assert!((cut_l - 719.2).abs() < 0.01);
        assert!((cut_w - 498.0).abs() < 0.01);
    }

    #[test]
    fn test_specification_grouping_and_cutting_layout() {
        let mut proj = Project::new_demo_cabinet();
        proj.cutting_settings.group_identical_parts = true;
        let spec = build_specification(&proj);
        // Two identical sides and two identical doors should be grouped by 2 pcs
        assert!(spec.iter().any(|r| r.quantity == 2 && r.full_length_mm == 720.0));
        assert!(spec.iter().any(|r| r.quantity == 2 && r.full_length_mm == 716.0));

        let report = compute_cutting_layout(&proj);
        assert!(report.unplaced_parts.is_empty());
        assert_eq!(report.total_sheets_count, 3); // 3 different materials -> 1 sheet for each
        // Parts with fixed grain direction must not be rotated 90 degrees
        for sh in &report.sheets {
            for p in &sh.placed_parts {
                if p.grain_fixed {
                    assert!(!p.rotated_90);
                }
            }
        }
    }
}
