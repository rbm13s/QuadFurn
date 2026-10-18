use crate::i18n::AppLanguage;
use crate::model::{PartBlock, PartId, Project};
use glam::{Vec2, Vec3};

/// 2D orthogonal projections (Top XY, Front XZ, Side YZ)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrthoAxis {
    /// Top view (Plan XY): Horizontal = X, Vertical = Y (+Y points up on screen)
    TopXY,
    /// Front view (Elevation XZ): Horizontal = X, Vertical = Z (+Z points up on screen)
    FrontXZ,
    /// Side view (Section YZ): Horizontal = Y, Vertical = Z (+Z points up on screen)
    SideYZ,
}

impl OrthoAxis {
    pub fn title(&self) -> &'static str {
        self.title_i18n(AppLanguage::Ru)
    }

    pub fn title_i18n(&self, lang: AppLanguage) -> &'static str {
        let d = lang.dict();
        match (self, lang) {
            (OrthoAxis::TopXY, AppLanguage::Ru) => "Вид сверху (Top XY — План)",
            (OrthoAxis::FrontXZ, AppLanguage::Ru) => "Вид спереди (Front XZ — Фасад)",
            (OrthoAxis::SideYZ, AppLanguage::Ru) => "Вид сбоку (Side YZ — Разрез)",
            (OrthoAxis::TopXY, AppLanguage::En) => "Top View (Top XY — Plan)",
            (OrthoAxis::FrontXZ, AppLanguage::En) => "Front View (Front XZ — Elevation)",
            (OrthoAxis::SideYZ, AppLanguage::En) => "Side View (Side YZ — Section)",
            (OrthoAxis::TopXY, _) => d.pdf_view_top,
            (OrthoAxis::FrontXZ, _) => d.pdf_view_front,
            (OrthoAxis::SideYZ, _) => d.pdf_view_side,
        }
    }

    pub fn axis_labels(&self) -> (&'static str, &'static str) {
        self.axis_labels_i18n(AppLanguage::Ru)
    }

    pub fn axis_labels_i18n(&self, lang: AppLanguage) -> (&'static str, &'static str) {
        let d = lang.dict();
        match self {
            OrthoAxis::TopXY => (d.pdf_axis_x, d.pdf_axis_y),
            OrthoAxis::FrontXZ => (d.pdf_axis_x, d.pdf_axis_z),
            OrthoAxis::SideYZ => (d.pdf_axis_y, d.pdf_axis_z),
        }
    }

    /// Projects a 3D point (X, Y, Z) into 2D view-plane coordinates (U, V)
    pub fn project(&self, p: Vec3) -> Vec2 {
        match self {
            OrthoAxis::TopXY => Vec2::new(p.x, p.y),
            OrthoAxis::FrontXZ => Vec2::new(p.x, p.z),
            OrthoAxis::SideYZ => Vec2::new(p.y, p.z),
        }
    }

    /// Applies 2D coordinates (U, V) back to an existing 3D vector (preserving the third coordinate)
    pub fn unproject_into(&self, uv: Vec2, base: Vec3) -> Vec3 {
        match self {
            OrthoAxis::TopXY => Vec3::new(uv.x, uv.y, base.z),
            OrthoAxis::FrontXZ => Vec3::new(uv.x, base.y, uv.y),
            OrthoAxis::SideYZ => Vec3::new(base.x, uv.x, uv.y),
        }
    }
}

/// Snapping result when dragging or resizing
#[derive(Clone, Debug, Default)]
pub struct SnapResult {
    pub snapped_pos: f32,
    pub guide_line_pos: Option<f32>,
}

/// Rounds value to grid step `grid_step_mm`
pub fn snap_to_grid(val: f32, grid_step_mm: f32) -> f32 {
    if grid_step_mm <= 0.001 {
        val
    } else {
        (val / grid_step_mm).round() * grid_step_mm
    }
}

/// Snaps coordinate `val` (or edge pair `[min_val, max_val]`) to edges of other parts and to the grid
pub fn snap_1d_edge(
    val: f32,
    candidates: &[f32],
    snap_threshold_mm: f32,
    use_edge_snap: bool,
    use_grid_snap: bool,
    grid_step_mm: f32,
) -> SnapResult {
    if use_edge_snap {
        let mut best_dist = snap_threshold_mm;
        let mut best_target = None;
        for &c in candidates {
            let d = (val - c).abs();
            if d <= best_dist {
                best_dist = d;
                best_target = Some(c);
            }
        }
        if let Some(target) = best_target {
            return SnapResult {
                snapped_pos: target,
                guide_line_pos: Some(target),
            };
        }
    }

    if use_grid_snap {
        return SnapResult {
            snapped_pos: snap_to_grid(val, grid_step_mm),
            guide_line_pos: None,
        };
    }

    SnapResult {
        snapped_pos: val,
        guide_line_pos: None,
    }
}

/// Snaps moving segment `[min_val, min_val + length]` to edge candidates of other blocks (magnetizes either left/bottom or right/top edge)
pub fn snap_interval_move(
    min_val: f32,
    length: f32,
    candidates: &[f32],
    snap_threshold_mm: f32,
    use_edge_snap: bool,
    use_grid_snap: bool,
    grid_step_mm: f32,
) -> SnapResult {
    if use_edge_snap {
        let mut best_dist = snap_threshold_mm;
        let mut best_min = None;
        let mut best_guide = None;
        let max_val = min_val + length;

        for &c in candidates {
            // Check snap for left/bottom edge (min_val -> c)
            let d_min = (min_val - c).abs();
            if d_min <= best_dist {
                best_dist = d_min;
                best_min = Some(c);
                best_guide = Some(c);
            }
            // Check snap for right/top edge (max_val -> c)
            let d_max = (max_val - c).abs();
            if d_max <= best_dist {
                best_dist = d_max;
                best_min = Some(c - length);
                best_guide = Some(c);
            }
        }

        if let Some(snapped_min) = best_min {
            return SnapResult {
                snapped_pos: snapped_min,
                guide_line_pos: best_guide,
            };
        }
    }

    if use_grid_snap {
        return SnapResult {
            snapped_pos: snap_to_grid(min_val, grid_step_mm),
            guide_line_pos: None,
        };
    }

    SnapResult {
        snapped_pos: min_val,
        guide_line_pos: None,
    }
}

/// Collects all characteristic edge coordinates (U and V) of all visible parts except `exclude_id` in the specified projection
pub fn collect_snap_candidates_2d(
    project: &Project,
    axis: OrthoAxis,
    exclude_id: Option<PartId>,
) -> (Vec<f32>, Vec<f32>) {
    let slice = exclude_id.as_slice();
    collect_snap_candidates_2d_excluding(project, axis, slice)
}

/// Collects all characteristic edge coordinates (U and V) of all visible parts except `exclude_ids` (for group movement)
pub fn collect_snap_candidates_2d_excluding(
    project: &Project,
    axis: OrthoAxis,
    exclude_ids: &[PartId],
) -> (Vec<f32>, Vec<f32>) {
    let mut u_cands = vec![0.0]; // Always include the scene zero axis
    let mut v_cands = vec![0.0];

    for part in &project.parts {
        if exclude_ids.contains(&part.id) || !project.is_part_visible(part) {
            continue;
        }
        let thick = project.get_material_thickness(part.material_id);
        let (min_3d, max_3d) = part.world_aabb(thick);
        let min_uv = axis.project(min_3d);
        let max_uv = axis.project(max_3d);

        u_cands.push(min_uv.x);
        u_cands.push(max_uv.x);
        v_cands.push(min_uv.y);
        v_cands.push(max_uv.y);
    }

    (u_cands, v_cands)
}

/// Dimension callout (distance from selected part to nearest neighbor in 2D projection)
#[derive(Clone, Debug)]
pub struct DistanceCallout2D {
    pub start_uv: Vec2,
    pub end_uv: Vec2,
    pub distance_mm: f32,
    #[allow(dead_code)]
    pub is_horizontal: bool,
}

/// Calculates dimension callouts (distances in mm) from selected part to nearest neighbor parts
/// on left, right, bottom, and top in the specified `axis` projection
pub fn compute_neighbor_distances_2d(
    project: &Project,
    axis: OrthoAxis,
    selected_part: &PartBlock,
) -> Vec<DistanceCallout2D> {
    let mut callouts = Vec::new();
    let thick = project.get_material_thickness(selected_part.material_id);
    let (sel_min3, sel_max3) = selected_part.world_aabb(thick);
    let sel_min = axis.project(sel_min3);
    let sel_max = axis.project(sel_max3);
    let sel_center = (sel_min + sel_max) * 0.5;

    let mut best_left: Option<f32> = None;
    let mut best_right: Option<f32> = None;
    let mut best_bottom: Option<f32> = None;
    let mut best_top: Option<f32> = None;

    for other in &project.parts {
        if other.id == selected_part.id || !project.is_part_visible(other) {
            continue;
        }
        let o_thick = project.get_material_thickness(other.material_id);
        let (o_min3, o_max3) = other.world_aabb(o_thick);
        let o_min = axis.project(o_min3);
        let o_max = axis.project(o_max3);

        // Vertical overlap along V -> look for neighbors on left/right along U
        let v_overlap = sel_min.y.max(o_min.y) < sel_max.y.min(o_max.y) + 1.0;
        if v_overlap {
            if o_max.x <= sel_min.x + 0.05 {
                best_left = Some(best_left.map_or(o_max.x, |b| b.max(o_max.x)));
            }
            if o_min.x >= sel_max.x - 0.05 {
                best_right = Some(best_right.map_or(o_min.x, |b| b.min(o_min.x)));
            }
        }

        // Horizontal overlap along U -> look for neighbors below/above along V
        let u_overlap = sel_min.x.max(o_min.x) < sel_max.x.min(o_max.x) + 1.0;
        if u_overlap {
            if o_max.y <= sel_min.y + 0.05 {
                best_bottom = Some(best_bottom.map_or(o_max.y, |b| b.max(o_max.y)));
            }
            if o_min.y >= sel_max.y - 0.05 {
                best_top = Some(best_top.map_or(o_min.y, |b| b.min(o_min.y)));
            }
        }
    }

    if let Some(left_u) = best_left {
        let dist = sel_min.x - left_u;
        if dist > 0.5 {
            callouts.push(DistanceCallout2D {
                start_uv: Vec2::new(left_u, sel_center.y),
                end_uv: Vec2::new(sel_min.x, sel_center.y),
                distance_mm: dist,
                is_horizontal: true,
            });
        }
    }
    if let Some(right_u) = best_right {
        let dist = right_u - sel_max.x;
        if dist > 0.5 {
            callouts.push(DistanceCallout2D {
                start_uv: Vec2::new(sel_max.x, sel_center.y),
                end_uv: Vec2::new(right_u, sel_center.y),
                distance_mm: dist,
                is_horizontal: true,
            });
        }
    }
    if let Some(bottom_v) = best_bottom {
        let dist = sel_min.y - bottom_v;
        if dist > 0.5 {
            callouts.push(DistanceCallout2D {
                start_uv: Vec2::new(sel_center.x, bottom_v),
                end_uv: Vec2::new(sel_center.x, sel_min.y),
                distance_mm: dist,
                is_horizontal: false,
            });
        }
    }
    if let Some(top_v) = best_top {
        let dist = top_v - sel_max.y;
        if dist > 0.5 {
            callouts.push(DistanceCallout2D {
                start_uv: Vec2::new(sel_center.x, sel_max.y),
                end_uv: Vec2::new(sel_center.x, top_v),
                distance_mm: dist,
                is_horizontal: false,
            });
        }
    }

    callouts
}
