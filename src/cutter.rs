use crate::model::{EdgeBanding, MaterialId, PartEdges, PartId, Project};

/// Позиция в сводной спецификации (с учетом опциональной группировки одинаковых деталей)
#[derive(Clone, Debug)]
pub struct SpecRow {
    pub index: usize,
    pub part_ids: Vec<PartId>,
    pub group_name: String,
    pub name: String,
    pub material_id: MaterialId,
    pub material_name: String,
    pub thickness_mm: f32,
    /// Чистовые (габаритные) размеры вместе с кромкой
    pub full_length_mm: f32,
    pub full_width_mm: f32,
    /// Пилильные размеры заготовки на форматно-раскроечный станок (без толщины кромок)
    pub cut_length_mm: f32,
    pub cut_width_mm: f32,
    pub quantity: usize,
    pub grain_fixed: bool,
    pub edges: PartEdges,
}

impl SpecRow {
    pub fn edge_label(edge_id: Option<u64>, edges_lib: &[EdgeBanding]) -> String {
        edge_id
            .and_then(|id| edges_lib.iter().find(|e| e.id == id))
            .map(|e| format!("{:.1} мм ({})", e.thickness_mm, e.name))
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn edge_short_mm(edge_id: Option<u64>, edges_lib: &[EdgeBanding]) -> String {
        edge_id
            .and_then(|id| edges_lib.iter().find(|e| e.id == id))
            .map(|e| format!("{:.1}", e.thickness_mm))
            .unwrap_or_else(|| "-".to_string())
    }
}

/// Строит сводную таблицу спецификации деталей проекта
pub fn build_specification(project: &Project) -> Vec<SpecRow> {
    let mut rows: Vec<SpecRow> = Vec::new();
    let group_identical = project.cutting_settings.group_identical_parts;

    for part in &project.parts {
        let mat = project.get_material(part.material_id);
        let mat_name = mat
            .map(|m| m.name.clone())
            .unwrap_or_else(|| "Неизвестный".to_string());
        let thick = mat.map(|m| m.thickness_mm).unwrap_or(18.0);
        let grp_name = part
            .group_id
            .and_then(|gid| project.groups.iter().find(|g| g.id == gid))
            .map(|g| g.name.clone())
            .unwrap_or_else(|| "Без группы".to_string());

        // Если grain_along_width == true, меняем местами эффективную длину и ширину вдоль текстуры
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

/// Размещенная на листе деталь в карте раскроя
#[derive(Clone, Debug)]
pub struct PlacedPart {
    pub spec_index: usize,
    #[allow(dead_code)]
    pub part_id: PartId,
    pub name: String,
    /// Координата X на листе (вдоль длины листа, мм)
    pub x_mm: f32,
    /// Координата Y на листе (вдоль ширины листа, мм)
    pub y_mm: f32,
    /// Размер вдоль длины листа (мм)
    pub w_on_sheet_mm: f32,
    /// Размер вдоль ширины листа (мм)
    pub h_on_sheet_mm: f32,
    /// Пилильные размеры (Длина x Ширина)
    pub cut_length_mm: f32,
    pub cut_width_mm: f32,
    /// Габаритные размеры (Длина x Ширина)
    pub full_length_mm: f32,
    pub full_width_mm: f32,
    /// Повернута ли деталь на 90 градусов относительно направления волокон листа
    pub rotated_90: bool,
    pub grain_fixed: bool,
    pub edges: PartEdges,
}

/// Свободный остаток (обрезок) на листе
#[derive(Clone, Debug)]
pub struct SheetRemnant {
    pub x_mm: f32,
    pub y_mm: f32,
    pub w_mm: f32,
    pub h_mm: f32,
}

/// Сквозная линия гильотинного реза фрезы/пилы на листе
#[derive(Clone, Debug)]
pub struct CutLine {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

/// Один раскроенный лист материала
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

/// Сводный результат раскроя по всему проекту
#[derive(Clone, Debug, Default)]
pub struct CuttingReport {
    pub sheets: Vec<SheetLayout>,
    pub unplaced_parts: Vec<String>,
    pub total_sheets_count: usize,
    pub total_parts_area_m2: f32,
    pub total_sheets_area_m2: f32,
    pub overall_utilization_percent: f32,
    pub total_cut_length_m: f32,
    /// Погонаж кромки по каждому виду кромки: (Название кромки, погонные метры без запаса)
    pub edge_banding_totals_m: Vec<(String, f32, [u8; 3])>,
}

/// Выполняет гильотинный раскрой (в стиле Cutting 3) для всех материалов проекта
pub fn compute_cutting_layout(project: &Project) -> CuttingReport {
    let spec_rows = build_specification(project);
    let settings = &project.cutting_settings;
    let kerf = settings.kerf_mm.max(0.0);

    let mut report = CuttingReport::default();

    // Подсчет погонажа кромки по всем деталям
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

    // Группируем задачи раскроя по материалам
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

        // Сортируем детали по убыванию площади и максимальной стороны (стандартная эвристика раскроя)
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
        let work_w = (mat.sheet_length_mm - settings.trim_left_mm - settings.trim_right_mm).max(10.0);
        let work_h = (mat.sheet_width_mm - settings.trim_top_mm - settings.trim_bottom_mm).max(10.0);

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

            // Линии обпила по периметру листа (торцовка)
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
                // Ищем лучший свободный прямоугольник (Best Short Side Fit / Guillotine Split)
                let mut best_rect_idx: Option<usize> = None;
                let mut best_rot = false;
                let mut best_score = f32::MAX;

                for (r_idx, rect) in free_rects.iter().enumerate() {
                    // Вариант 1: без поворота (Длина детали вдоль Длины листа X)
                    if item.cut_length_mm <= rect.w_mm + 0.01 && item.cut_width_mm <= rect.h_mm + 0.01
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

                    // Вариант 2: поворот на 90 градусов (только если текстура НЕ зафиксирована!)
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

                    // Гильотинное разбиение свободного прямоугольника с учетом реза фрезы `kerf`
                    let right_w = rect.w_mm - pw - kerf;
                    let bottom_h = rect.h_mm - ph - kerf;

                    // Выбираем направление сквозного гильотинного реза (по более длинному остатку, как на форматно-раскроечном станке)
                    if right_w * rect.h_mm > bottom_h * rect.w_mm {
                        // Сначала продольный/поперечный рез на всю высоту полосы `rect.h_mm`
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
                        // Сначала горизонтальный рез вдоль ширины полосы `rect.w_mm`
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
                // Остались детали, которые физически больше рабочего поля листа
                for bad in next_remaining {
                    report.unplaced_parts.push(format!(
                        "{} ({:.1} x {:.1} мм > рабочего поля листа {:.0} x {:.0} мм)",
                        bad.name, bad.cut_length_mm, bad.cut_width_mm, work_w, work_h
                    ));
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

            // Оставляем в списке остатков только заметные обрезки (>= 50 мм по каждой стороне)
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
        // Проверим левую боковину (720 x 500, Д1 = 2.0 мм, Ш1 = 0.4 мм, Ш2 = 0.4 мм)
        let side = proj.parts.iter().find(|p| p.id == 1001).unwrap();
        let (cut_l, cut_w) = side.cut_dimensions_mm(&proj.edge_bandings);
        // Длина уменьшается на Ш1 + Ш2 (0.4 + 0.4 = 0.8 мм) -> 719.2 мм
        // Ширина уменьшается на Д1 + Д2 (2.0 + 0.0 = 2.0 мм) -> 498.0 мм
        assert!((cut_l - 719.2).abs() < 0.01);
        assert!((cut_w - 498.0).abs() < 0.01);
    }

    #[test]
    fn test_specification_grouping_and_cutting_layout() {
        let mut proj = Project::new_demo_cabinet();
        proj.cutting_settings.group_identical_parts = true;
        let spec = build_specification(&proj);
        // Две одинаковые боковины и два одинаковых фасада должны объединиться по 2 шт.
        assert!(spec.iter().any(|r| r.quantity == 2 && r.full_length_mm == 720.0));
        assert!(spec.iter().any(|r| r.quantity == 2 && r.full_length_mm == 716.0));

        let report = compute_cutting_layout(&proj);
        assert!(report.unplaced_parts.is_empty());
        assert_eq!(report.total_sheets_count, 3); // 3 разных материала -> по 1 листу на каждый
        // Все детали с фиксированным направлением волокон не должны поворачиваться на 90 градусов
        for sh in &report.sheets {
            for p in &sh.placed_parts {
                if p.grain_fixed {
                    assert!(!p.rotated_90);
                }
            }
        }
    }
}

