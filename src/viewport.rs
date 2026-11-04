use crate::model::{EdgeId, MaterialId, PartBlock, PartEdges, PartId, PartOrientation, Project};
use crate::snapping::{
    collect_snap_candidates_2d, collect_snap_candidates_2d_excluding,
    compute_neighbor_distances_2d, snap_1d_edge, snap_interval_move, snap_to_grid, OrthoAxis,
};
use egui::{Color32, FontId, Key, Painter, Pos2, Rect, Sense, Shape, Stroke, TextureHandle, Vec2 as EguiVec2};
use glam::{Mat4, Vec2, Vec3, Vec4};
use std::collections::HashMap;

/// Editor tool
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorTool {
    /// Select, move, and resize blocks using handles
    SelectAndMove,
    /// Create new part by drawing a box in any 2D view
    CreateBlock,
    /// Measuring tape between two points in 2D
    MeasureTape,
}

/// Which resize handle or part body is being dragged by the user in 2D view
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragHandle2D {
    MoveWholeBlock,
    ResizeLeft,
    ResizeRight,
    ResizeBottom,
    ResizeTop,
    ResizeBottomLeft,
    ResizeBottomRight,
    ResizeTopLeft,
    ResizeTopRight,
}

impl DragHandle2D {
    /// Returns double-sided arrow cursor icon when hovering/dragging a handle
    pub fn cursor_icon(&self) -> egui::CursorIcon {
        match self {
            DragHandle2D::MoveWholeBlock => egui::CursorIcon::Move,
            DragHandle2D::ResizeLeft | DragHandle2D::ResizeRight => {
                egui::CursorIcon::ResizeHorizontal
            }
            DragHandle2D::ResizeTop | DragHandle2D::ResizeBottom => {
                egui::CursorIcon::ResizeVertical
            }
            DragHandle2D::ResizeTopLeft | DragHandle2D::ResizeBottomRight => {
                egui::CursorIcon::ResizeNwSe
            }
            DragHandle2D::ResizeTopRight | DragHandle2D::ResizeBottomLeft => {
                egui::CursorIcon::ResizeNeSw
            }
        }
    }
}

/// Classification of how a part appears in a 2D orthogonal projection
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PartViewMode2D {
    /// Part is viewed face-on (broad panel face).
    /// All 8 resize handles (4 corners + 4 edges) are active and resizable.
    FaceOn,
    /// Part is viewed edge-on (butt-end) with thickness along the horizontal 2D axis (U).
    /// Thickness cannot be altered; only vertical handles (ResizeTop, ResizeBottom) are active.
    EdgeOnHorizontalThickness,
    /// Part is viewed edge-on (butt-end) with thickness along the vertical 2D axis (V).
    /// Thickness cannot be altered; only horizontal handles (ResizeLeft, ResizeRight) are active.
    EdgeOnVerticalThickness,
}

pub fn part_view_mode_2d(axis: OrthoAxis, part: &PartBlock, thick: f32) -> PartViewMode2D {
    let mode = match (axis, part.orientation) {
        (OrthoAxis::TopXY, PartOrientation::HorizontalXY)
        | (OrthoAxis::FrontXZ, PartOrientation::FrontalXZ)
        | (OrthoAxis::SideYZ, PartOrientation::VerticalYZ) => PartViewMode2D::FaceOn,

        (OrthoAxis::TopXY, PartOrientation::VerticalYZ)
        | (OrthoAxis::FrontXZ, PartOrientation::VerticalYZ)
        | (OrthoAxis::SideYZ, PartOrientation::FrontalXZ) => PartViewMode2D::EdgeOnHorizontalThickness,

        (OrthoAxis::TopXY, PartOrientation::FrontalXZ)
        | (OrthoAxis::FrontXZ, PartOrientation::HorizontalXY)
        | (OrthoAxis::SideYZ, PartOrientation::HorizontalXY) => PartViewMode2D::EdgeOnVerticalThickness,
    };

    if mode != PartViewMode2D::FaceOn {
        let (min_3d, max_3d) = part.world_aabb(thick);
        let min_uv = axis.project(min_3d);
        let max_uv = axis.project(max_3d);
        let size_uv = (max_uv - min_uv).abs();

        if size_uv.x <= thick * 1.5 && size_uv.y > thick * 1.5 {
            PartViewMode2D::EdgeOnHorizontalThickness
        } else if size_uv.y <= thick * 1.5 && size_uv.x > thick * 1.5 {
            PartViewMode2D::EdgeOnVerticalThickness
        } else {
            mode
        }
    } else {
        PartViewMode2D::FaceOn
    }
}

pub fn get_allowed_handles_2d(r: Rect, mode: PartViewMode2D) -> Vec<(Pos2, DragHandle2D)> {
    match mode {
        PartViewMode2D::FaceOn => vec![
            (r.left_top(), DragHandle2D::ResizeTopLeft),
            (Pos2::new(r.center().x, r.top()), DragHandle2D::ResizeTop),
            (r.right_top(), DragHandle2D::ResizeTopRight),
            (Pos2::new(r.right(), r.center().y), DragHandle2D::ResizeRight),
            (r.right_bottom(), DragHandle2D::ResizeBottomRight),
            (Pos2::new(r.center().x, r.bottom()), DragHandle2D::ResizeBottom),
            (r.left_bottom(), DragHandle2D::ResizeBottomLeft),
            (Pos2::new(r.left(), r.center().y), DragHandle2D::ResizeLeft),
        ],
        PartViewMode2D::EdgeOnHorizontalThickness => vec![
            (Pos2::new(r.center().x, r.top()), DragHandle2D::ResizeTop),
            (Pos2::new(r.center().x, r.bottom()), DragHandle2D::ResizeBottom),
        ],
        PartViewMode2D::EdgeOnVerticalThickness => vec![
            (Pos2::new(r.left(), r.center().y), DragHandle2D::ResizeLeft),
            (Pos2::new(r.right(), r.center().y), DragHandle2D::ResizeRight),
        ],
    }
}

/// State of active dragging (single or multi-part) in 2D view
#[derive(Clone, Debug)]
pub struct ActiveDrag2D {
    pub axis: OrthoAxis,
    pub part_id: PartId,
    pub handle: DragHandle2D,
    pub start_mouse_uv: Vec2,
    pub orig_min_3d: Vec3,
    pub orig_max_3d: Vec3,
    /// Original world coordinates `position_mm` of all parts dragged together as a group
    pub group_orig_positions: Vec<(PartId, Vec3)>,
    pub active_guides_u: Vec<f32>,
    pub active_guides_v: Vec<f32>,
}

/// State of a single 2D orthogonal viewport
#[derive(Clone, Debug)]
pub struct OrthoViewportState {
    pub axis: OrthoAxis,
    /// Camera center in world millimeters (U, V)
    pub pan_uv_mm: Vec2,
    /// Zoom scale: screen pixels per millimeter (e.g. 0.45 px/mm)
    pub zoom_px_per_mm: f32,
}

impl OrthoViewportState {
    pub fn new(axis: OrthoAxis) -> Self {
        let pan = match axis {
            OrthoAxis::TopXY => Vec2::new(400.0, 250.0),
            OrthoAxis::FrontXZ => Vec2::new(400.0, 360.0),
            OrthoAxis::SideYZ => Vec2::new(250.0, 360.0),
        };
        Self {
            axis,
            pan_uv_mm: pan,
            zoom_px_per_mm: 0.45,
        }
    }

    /// Converts world millimeters (U, V) to screen pixels `Pos2`
    /// U axis points right (+X), V axis points up (-Y on screen)
    pub fn world_to_screen(&self, rect: Rect, uv_mm: Vec2) -> Pos2 {
        let center = rect.center();
        let dx = (uv_mm.x - self.pan_uv_mm.x) * self.zoom_px_per_mm;
        let dy = -(uv_mm.y - self.pan_uv_mm.y) * self.zoom_px_per_mm;
        Pos2::new(center.x + dx, center.y + dy)
    }

    /// Converts screen pixels `Pos2` to world millimeters (U, V)
    pub fn screen_to_world(&self, rect: Rect, screen_pos: Pos2) -> Vec2 {
        let center = rect.center();
        let u = self.pan_uv_mm.x + (screen_pos.x - center.x) / self.zoom_px_per_mm;
        let v = self.pan_uv_mm.y - (screen_pos.y - center.y) / self.zoom_px_per_mm;
        Vec2::new(u, v)
    }
}

/// 3D camera state supporting two modes:
/// 1) CAD Orbit (orbiting around target)
/// 2) First-person Noclip Fly via `Z` key (WASD + free mouse look)
#[derive(Clone, Debug)]
pub struct Camera3D {
    /// Whether first-person Noclip fly mode is active (toggled by Z key)
    pub noclip_active: bool,
    /// Camera eye position in millimeters (X, Y horizontal, Z height up)
    pub eye_mm: Vec3,
    /// Target aim point for Orbit mode (mm)
    pub orbit_target_mm: Vec3,
    /// Distance to target in Orbit mode (mm)
    pub orbit_distance_mm: f32,
    /// Horizontal view angle (yaw around Z in radians)
    pub yaw_rad: f32,
    /// Vertical view angle (pitch in radians, from -1.5 to +1.5)
    pub pitch_rad: f32,
    /// Base fly speed in Z-Noclip mode (mm/sec)
    pub fly_speed_mm_per_sec: f32,
    /// Whether to show wireframe overlay (X-Ray) or solid fill
    pub wireframe_overlay: bool,
    /// Whether to show edge banding markers in 3D
    pub show_edge_banding_3d: bool,
    /// Whether precise camera Z-order depth sorting is enabled (subdivided per-tile depth sorting)
    pub z_order: bool,
    /// Last timestamp when flight mode was toggled (seconds), used for debouncing key events
    pub last_toggle_time: f64,
}

impl Default for Camera3D {
    fn default() -> Self {
        let mut cam = Self {
            noclip_active: false,
            eye_mm: Vec3::new(-650.0, -950.0, 950.0),
            orbit_target_mm: Vec3::new(400.0, 250.0, 360.0),
            orbit_distance_mm: 1550.0,
            yaw_rad: 0.95,
            pitch_rad: -0.32,
            fly_speed_mm_per_sec: 1200.0,
            wireframe_overlay: true,
            show_edge_banding_3d: true,
            z_order: true,
            last_toggle_time: 0.0,
        };
        cam.sync_eye_from_orbit();
        cam
    }
}

const NEAR_CLIP_MM: f32 = 12.0;

impl Camera3D {
    /// Unit forward direction vector of camera in world coordinates (X right, Y inward, Z up)
    pub fn forward_dir(&self) -> Vec3 {
        let cp = self.pitch_rad.cos();
        let sp = self.pitch_rad.sin();
        let cy = self.yaw_rad.cos();
        let sy = self.yaw_rad.sin();
        Vec3::new(cp * cy, cp * sy, sp).normalize_or_zero()
    }

    /// Unit "right" direction vector of camera (in horizontal XY plane)
    pub fn right_dir(&self) -> Vec3 {
        let f = self.forward_dir();
        f.cross(Vec3::Z).normalize_or_zero()
    }

    /// Unit "up" direction vector of camera
    pub fn up_dir(&self) -> Vec3 {
        let r = self.right_dir();
        let f = self.forward_dir();
        r.cross(f).normalize_or_zero()
    }

    /// Recalculates `eye_mm` from `orbit_target_mm`, `orbit_distance_mm`, `yaw_rad`, `pitch_rad`
    pub fn sync_eye_from_orbit(&mut self) {
        let fwd = self.forward_dir();
        self.eye_mm = self.orbit_target_mm - fwd * self.orbit_distance_mm;
    }

    /// When exiting Noclip, updates `orbit_target_mm` ahead of camera for seamless transition
    pub fn sync_orbit_from_eye(&mut self) {
        let fwd = self.forward_dir();
        self.orbit_target_mm = self.eye_mm + fwd * self.orbit_distance_mm;
    }

    /// View-Projection matrix for perspective 3D projection
    pub fn view_proj_matrix(&self, aspect: f32) -> Mat4 {
        let target = self.eye_mm + self.forward_dir() * 100.0;
        let view = Mat4::look_at_rh(self.eye_mm, target, Vec3::Z);
        let proj = Mat4::perspective_rh(45.0_f32.to_radians(), aspect.max(0.1), 10.0, 50_000.0);
        proj * view
    }

    /// Projects 3D point to screen coordinates `Pos2` and returns `(Pos2, depth_w)`
    /// Returns `None` if point is behind camera (`w <= 10.0`)
    pub fn project_point(&self, vp_mat: &Mat4, rect: Rect, pt_mm: Vec3) -> Option<(Pos2, f32)> {
        let clip: Vec4 = *vp_mat * pt_mm.extend(1.0);
        if clip.w <= 10.0 {
            return None;
        }
        let ndc_x = clip.x / clip.w;
        let ndc_y = clip.y / clip.w;
        let sx = rect.center().x + ndc_x * (rect.width() * 0.5);
        let sy = rect.center().y - ndc_y * (rect.height() * 0.5);
        Some((Pos2::new(sx, sy), clip.w))
    }

    /// Computes the exact unit ray direction from `eye_mm` through screen point `scr_pt` in `rect`.
    /// Uses the analytical camera basis directly without inverting the perspective matrix.
    pub fn ray_from_screen_point(&self, scr_pt: Pos2, rect: Rect) -> Vec3 {
        let aspect = (rect.width() / rect.height().max(1.0)).max(0.1);
        let ndc_x = (scr_pt.x - rect.center().x) / (rect.width() * 0.5).max(1.0);
        let ndc_y = -(scr_pt.y - rect.center().y) / (rect.height() * 0.5).max(1.0);
        let half_tan = (45.0_f32.to_radians() * 0.5).tan();
        (self.forward_dir()
            + self.right_dir() * (ndc_x * aspect * half_tan)
            + self.up_dir() * (ndc_y * half_tan))
            .normalize_or_zero()
    }
}

/// Which of the 4 viewports is maximized to full screen (via Space key or `[Maximize]` button)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaximizedPane {
    None,
    TopXY,
    Perspective3D,
    FrontXZ,
    SideYZ,
}

/// 4-viewport desktop state (Top XY, Front XZ, Side YZ, and 3D Perspective)
pub struct QuadViewportController {
    pub top_xy: OrthoViewportState,
    pub front_xz: OrthoViewportState,
    pub side_yz: OrthoViewportState,
    pub cam_3d: Camera3D,
    pub maximized: MaximizedPane,
    /// Position of vertical and horizontal splitters of the 4 viewports (0.2 to 0.8, default 0.5)
    pub split_x: f32,
    pub split_y: f32,
    /// Last known pane sizes in pixels (TopXY, FrontXZ, SideYZ, 3D) for fitting the model into windows
    pub last_pane_sizes: [EguiVec2; 4],

    /// Active tool
    pub tool: EditorTool,
    /// List of all selected parts (for marquee box multi-selection or via Shift/Ctrl+click)
    pub selected_parts: Vec<PartId>,
    /// Marquee selection box drag state: (OrthoAxis, start_uv_mm, current_uv_mm, append_mode)
    pub box_select_drag: Option<(OrthoAxis, Vec2, Vec2, bool, Vec<PartId>)>,
    /// Default orientation override when creating new part (can be cycled with Tab key)
    pub create_orientation_override: Option<PartOrientation>,
    /// Box drag state when creating new part in 2D: (OrthoAxis, start_uv_mm, current_uv_mm)
    pub creating_drag: Option<(OrthoAxis, Vec2, Vec2)>,
    /// Active dragging or resizing of existing part (or group of parts)
    pub active_drag: Option<ActiveDrag2D>,
    /// Measuring tape points: (OrthoAxis, start_uv_mm, end_uv_mm)
    pub measure_points: Option<(OrthoAxis, Vec2, Vec2)>,

    /// Grid and snapping settings
    pub grid_snap_enabled: bool,
    pub edge_snap_enabled: bool,
    pub show_distance_callouts: bool,
    pub grid_step_mm: f32,
    /// Last active or hovered 2D orthogonal viewport (TopXY, FrontXZ, SideYZ)
    pub last_active_ortho: OrthoAxis,
    /// Current UI language for viewport and 3D titles
    pub lang: crate::i18n::AppLanguage,
}

impl Default for QuadViewportController {
    fn default() -> Self {
        Self {
            top_xy: OrthoViewportState::new(OrthoAxis::TopXY),
            front_xz: OrthoViewportState::new(OrthoAxis::FrontXZ),
            side_yz: OrthoViewportState::new(OrthoAxis::SideYZ),
            cam_3d: Camera3D::default(),
            maximized: MaximizedPane::None,
            split_x: 0.5,
            split_y: 0.5,
            last_pane_sizes: [EguiVec2::new(560.0, 380.0); 4],
            tool: EditorTool::SelectAndMove,
            selected_parts: Vec::new(),
            box_select_drag: None,
            create_orientation_override: None,
            creating_drag: None,
            active_drag: None,
            measure_points: None,
            grid_snap_enabled: true,
            edge_snap_enabled: true,
            show_distance_callouts: true,
            grid_step_mm: 18.0, // Default grid step is 18 mm (for standard 18 mm chipboard)
            last_active_ortho: OrthoAxis::TopXY,
            lang: crate::i18n::AppLanguage::Ru,
        }
    }
}

impl QuadViewportController {
    /// Synchronizes multi-selection list `selected_parts` with primary `selected_part`
    pub fn sync_selection(&mut self, selected_part: &mut Option<PartId>, project: &Project) {
        self.selected_parts
            .retain(|&id| project.parts.iter().any(|p| p.id == id && project.is_part_visible(p)));
        match *selected_part {
            Some(sel_id) => {
                if !self.selected_parts.contains(&sel_id) {
                    self.selected_parts = vec![sel_id];
                }
            }
            None => {
                self.selected_parts.clear();
            }
        }
    }

    pub fn set_single_selection(&mut self, selected_part: &mut Option<PartId>, id: Option<PartId>) {
        *selected_part = id;
        self.selected_parts = id.into_iter().collect();
    }

    pub fn toggle_part_in_selection(&mut self, selected_part: &mut Option<PartId>, id: PartId) {
        if let Some(pos) = self.selected_parts.iter().position(|&x| x == id) {
            self.selected_parts.remove(pos);
        } else {
            self.selected_parts.push(id);
        }
        *selected_part = self.selected_parts.first().copied();
    }

    pub fn is_part_selected(&self, selected_part: Option<PartId>, id: PartId) -> bool {
        selected_part == Some(id) || self.selected_parts.contains(&id)
    }

    /// Computes combined 3D AABB of all visible project parts
    pub fn compute_project_bounds(project: &Project) -> Option<(Vec3, Vec3)> {
        let mut min_pt = Vec3::splat(f32::MAX);
        let mut max_pt = Vec3::splat(f32::MIN);
        for p in &project.parts {
            if !project.is_part_visible(p) {
                continue;
            }
            let t = project.get_material_thickness(p.material_id);
            let (pmin, pmax) = p.world_aabb(t);
            min_pt = min_pt.min(pmin);
            max_pt = max_pt.max(pmax);
        }
        if min_pt.x <= max_pt.x {
            Some((min_pt, max_pt))
        } else {
            None
        }
    }

    /// Computes combined 3D AABB for currently selected parts
    pub fn compute_selected_bounds(&self, project: &Project) -> Option<(Vec3, Vec3)> {
        let mut min_pt = Vec3::splat(f32::MAX);
        let mut max_pt = Vec3::splat(f32::MIN);
        for &id in &self.selected_parts {
            if let Some(p) = project.parts.iter().find(|p| p.id == id) {
                if project.is_part_visible(p) {
                    let t = project.get_material_thickness(p.material_id);
                    let (pmin, pmax) = p.world_aabb(t);
                    min_pt = min_pt.min(pmin);
                    max_pt = max_pt.max(pmax);
                }
            }
        }
        if min_pt.x <= max_pt.x {
            Some((min_pt, max_pt))
        } else {
            None
        }
    }

    /// Centers all 4 viewports on current project center (without resetting zoom)
    pub fn focus_all_on_project(&mut self, project: &Project) {
        let Some((min_pt, max_pt)) = Self::compute_project_bounds(project) else {
            return;
        };
        let center = (min_pt + max_pt) * 0.5;
        self.top_xy.pan_uv_mm = Vec2::new(center.x, center.y);
        self.front_xz.pan_uv_mm = Vec2::new(center.x, center.z);
        self.side_yz.pan_uv_mm = Vec2::new(center.y, center.z);
        self.cam_3d.orbit_target_mm = center;
        self.cam_3d.sync_eye_from_orbit();
    }

    /// Resets the 4-viewport splitters to exactly 50% / 50% (equal sizes for all 4 panes).
    pub fn equalize_viewports(&mut self) {
        self.split_x = 0.5;
        self.split_y = 0.5;
    }

    /// Fits entire model into each of the 4 viewports (3 ortho 2D projections + 3D view)
    pub fn fit_all_to_window(&mut self, project: &Project) {
        let (min_pt, max_pt) = Self::compute_project_bounds(project)
            .unwrap_or((Vec3::ZERO, Vec3::new(800.0, 500.0, 720.0)));
        let center = (min_pt + max_pt) * 0.5;
        let span = (max_pt - min_pt).max(Vec3::splat(60.0));

        // 1. Fit into all three 2D projections accounting for pane sizes, headers, and scrollbars
        let fit_ortho = |vp: &mut OrthoViewportState, span_u: f32, span_v: f32, pane_sz: EguiVec2| {
            let usable_w = (pane_sz.x - 28.0).max(120.0) * 0.80;
            let usable_h = (pane_sz.y - 52.0).max(100.0) * 0.80;
            let z_u = usable_w / span_u.max(40.0);
            let z_v = usable_h / span_v.max(40.0);
            vp.zoom_px_per_mm = z_u.min(z_v).clamp(0.03, 8.0);
        };

        self.top_xy.pan_uv_mm = Vec2::new(center.x, center.y);
        fit_ortho(&mut self.top_xy, span.x, span.y, self.last_pane_sizes[0]);

        self.front_xz.pan_uv_mm = Vec2::new(center.x, center.z);
        fit_ortho(&mut self.front_xz, span.x, span.z, self.last_pane_sizes[1]);

        self.side_yz.pan_uv_mm = Vec2::new(center.y, center.z);
        fit_ortho(&mut self.side_yz, span.y, span.z, self.last_pane_sizes[2]);

        // 2. Fit 3D model into viewing frustum of 3D camera
        self.cam_3d.noclip_active = false;
        self.cam_3d.orbit_target_mm = center;
        let pane_3d = self.last_pane_sizes[3];
        let aspect = (pane_3d.x / pane_3d.y.max(1.0)).max(0.2);
        let tan_half_fov_y = (22.5_f32).to_radians().tan();
        let tan_half_fov_x = tan_half_fov_y * aspect;

        let fwd = self.cam_3d.forward_dir();
        let right = self.cam_3d.right_dir();
        let up = self.cam_3d.up_dir();

        let corners = [
            Vec3::new(min_pt.x, min_pt.y, min_pt.z),
            Vec3::new(max_pt.x, min_pt.y, min_pt.z),
            Vec3::new(max_pt.x, max_pt.y, min_pt.z),
            Vec3::new(min_pt.x, max_pt.y, min_pt.z),
            Vec3::new(min_pt.x, min_pt.y, max_pt.z),
            Vec3::new(max_pt.x, min_pt.y, max_pt.z),
            Vec3::new(max_pt.x, max_pt.y, max_pt.z),
            Vec3::new(min_pt.x, max_pt.y, max_pt.z),
        ];

        let mut req_dist = 250.0_f32;
        for c in corners {
            let rel = c - center;
            let x_c = rel.dot(right).abs();
            let y_c = rel.dot(up).abs();
            let z_c = rel.dot(fwd);
            let d_v = (y_c / tan_half_fov_y) - z_c;
            let d_h = (x_c / tan_half_fov_x) - z_c;
            req_dist = req_dist.max(d_v).max(d_h);
        }

        self.cam_3d.orbit_distance_mm = (req_dist * 1.24).clamp(220.0, 25_000.0);
        self.cam_3d.sync_eye_from_orbit();
    }

    /// Computes 3D displacement vector for arrow key nudge in a 2D viewport.
    /// Returns None if no displacement or invalid inputs.
    pub fn compute_arrow_nudge_delta(
        axis: OrthoAxis,
        left: bool,
        right: bool,
        up: bool,
        down: bool,
        grid_step_mm: f32,
        shift: bool,
    ) -> Option<Vec3> {
        let base_step = if grid_step_mm > 0.001 {
            grid_step_mm
        } else {
            1.0
        };
        let step = if shift { base_step * 2.0 } else { base_step };

        let mut delta_uv = Vec2::ZERO;
        if left {
            delta_uv.x -= step;
        }
        if right {
            delta_uv.x += step;
        }
        if up {
            delta_uv.y += step;
        }
        if down {
            delta_uv.y -= step;
        }

        if delta_uv == Vec2::ZERO {
            None
        } else {
            Some(axis.unproject_into(delta_uv, Vec3::ZERO))
        }
    }

    /// Handles arrow keys nudging of selected parts in the active 2D orthogonal viewport.
    fn handle_arrow_keys_nudge(
        &self,
        ui: &mut egui::Ui,
        axis: OrthoAxis,
        project: &mut Project,
        selected_part: Option<PartId>,
        on_before_mutate: &mut impl FnMut(&Project),
    ) {
        if ui.ctx().egui_wants_keyboard_input() {
            return;
        }
        if self.cam_3d.noclip_active {
            return;
        }
        if self.active_drag.is_some()
            || self.box_select_drag.is_some()
            || self.creating_drag.is_some()
        {
            return;
        }

        let delta_3d = ui.input(|i| {
            if i.modifiers.ctrl || i.modifiers.command || i.modifiers.alt {
                return None;
            }
            let left = i.key_pressed(Key::ArrowLeft);
            let right = i.key_pressed(Key::ArrowRight);
            let up = i.key_pressed(Key::ArrowUp);
            let down = i.key_pressed(Key::ArrowDown);

            if !left && !right && !up && !down {
                return None;
            }

            Self::compute_arrow_nudge_delta(
                axis,
                left,
                right,
                up,
                down,
                self.grid_step_mm,
                i.modifiers.shift,
            )
        });

        if let Some(delta) = delta_3d {
            let sel_ids: Vec<PartId> = if !self.selected_parts.is_empty() {
                self.selected_parts.clone()
            } else {
                selected_part.into_iter().collect()
            };

            let visible_sel_ids: Vec<PartId> = sel_ids
                .into_iter()
                .filter(|pid| project.parts.iter().any(|p| p.id == *pid && project.is_part_visible(p)))
                .collect();

            if visible_sel_ids.is_empty() {
                return;
            }

            on_before_mutate(project);

            for pid in visible_sel_ids {
                if let Some(part) = project.parts.iter_mut().find(|p| p.id == pid) {
                    part.position_mm += delta;
                }
            }

            ui.ctx().request_repaint();
        }
    }

    /// Renders all 4 viewports (or one maximized) in central workspace
    pub fn ui_quad_viewports(
        &mut self,
        ui: &mut egui::Ui,
        project: &mut Project,
        selected_part: &mut Option<PartId>,
        active_material_id: MaterialId,
        loaded_textures: &HashMap<MaterialId, TextureHandle>,
        mut on_before_mutate: impl FnMut(&Project),
    ) {
        self.sync_selection(selected_part, project);

        let avail = ui.available_rect_before_wrap();
        if avail.width() < 50.0 || avail.height() < 50.0 {
            return;
        }

        // Check Tab shortcut to cycle orientation of newly created part
        if ui.input(|i| i.key_pressed(Key::Tab)) && !ui.ctx().egui_wants_keyboard_input() {
            self.create_orientation_override = Some(match self.create_orientation_override {
                None | Some(PartOrientation::HorizontalXY) => PartOrientation::VerticalYZ,
                Some(PartOrientation::VerticalYZ) => PartOrientation::FrontalXZ,
                Some(PartOrientation::FrontalXZ) => PartOrientation::HorizontalXY,
            });
        }

        // Determine active 2D orthogonal viewport for keyboard nudging
        let active_ortho_axis = match self.maximized {
            MaximizedPane::TopXY => Some(OrthoAxis::TopXY),
            MaximizedPane::FrontXZ => Some(OrthoAxis::FrontXZ),
            MaximizedPane::SideYZ => Some(OrthoAxis::SideYZ),
            MaximizedPane::Perspective3D => None,
            MaximizedPane::None => {
                let mid_x = avail.left() + avail.width() * self.split_x;
                let mid_y = avail.top() + avail.height() * self.split_y;
                let gap = 4.0;
                let rect_tl = Rect::from_min_max(
                    avail.min,
                    Pos2::new(mid_x - gap * 0.5, mid_y - gap * 0.5),
                );
                let rect_tr = Rect::from_min_max(
                    Pos2::new(mid_x + gap * 0.5, avail.top()),
                    Pos2::new(avail.right(), mid_y - gap * 0.5),
                );
                let rect_bl = Rect::from_min_max(
                    Pos2::new(avail.left(), mid_y + gap * 0.5),
                    Pos2::new(mid_x - gap * 0.5, avail.bottom()),
                );
                let rect_br = Rect::from_min_max(
                    Pos2::new(mid_x + gap * 0.5, mid_y + gap * 0.5),
                    avail.max,
                );

                if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                    if rect_tl.contains(pos) {
                        Some(OrthoAxis::TopXY)
                    } else if rect_bl.contains(pos) {
                        Some(OrthoAxis::FrontXZ)
                    } else if rect_br.contains(pos) {
                        Some(OrthoAxis::SideYZ)
                    } else if rect_tr.contains(pos) {
                        None // 3D Perspective viewport
                    } else {
                        Some(self.last_active_ortho)
                    }
                } else {
                    Some(self.last_active_ortho)
                }
            }
        };

        if let Some(axis) = active_ortho_axis {
            self.last_active_ortho = axis;
            self.handle_arrow_keys_nudge(ui, axis, project, *selected_part, &mut on_before_mutate);
        }

        match self.maximized {
            MaximizedPane::TopXY => {
                let mut vp = self.top_xy.clone();
                self.render_ortho_pane(
                    ui,
                    avail,
                    &mut vp,
                    project,
                    selected_part,
                    active_material_id,
                    &mut on_before_mutate,
                );
                self.top_xy = vp;
            }
            MaximizedPane::FrontXZ => {
                let mut vp = self.front_xz.clone();
                self.render_ortho_pane(
                    ui,
                    avail,
                    &mut vp,
                    project,
                    selected_part,
                    active_material_id,
                    &mut on_before_mutate,
                );
                self.front_xz = vp;
            }
            MaximizedPane::SideYZ => {
                let mut vp = self.side_yz.clone();
                self.render_ortho_pane(
                    ui,
                    avail,
                    &mut vp,
                    project,
                    selected_part,
                    active_material_id,
                    &mut on_before_mutate,
                );
                self.side_yz = vp;
            }
            MaximizedPane::Perspective3D => {
                self.render_3d_pane(ui, avail, project, selected_part, loaded_textures);
            }
            MaximizedPane::None => {
                let gap = 4.0;
                let mid_x = avail.left() + avail.width() * self.split_x;
                let mid_y = avail.top() + avail.height() * self.split_y;

                // Bounds for the 4 viewports:
                // Top-left: Top (XY) | Top-right: 3D Perspective
                // Bottom-left: Front (XZ) | Bottom-right: Side (YZ)
                let rect_tl = Rect::from_min_max(
                    avail.min,
                    Pos2::new(mid_x - gap * 0.5, mid_y - gap * 0.5),
                );
                let rect_tr = Rect::from_min_max(
                    Pos2::new(mid_x + gap * 0.5, avail.top()),
                    Pos2::new(avail.right(), mid_y - gap * 0.5),
                );
                let rect_bl = Rect::from_min_max(
                    Pos2::new(avail.left(), mid_y + gap * 0.5),
                    Pos2::new(mid_x - gap * 0.5, avail.bottom()),
                );
                let rect_br = Rect::from_min_max(
                    Pos2::new(mid_x + gap * 0.5, mid_y + gap * 0.5),
                    avail.max,
                );

                // Render the 4 viewport panes
                let mut vp_top = self.top_xy.clone();
                self.render_ortho_pane(
                    ui,
                    rect_tl,
                    &mut vp_top,
                    project,
                    selected_part,
                    active_material_id,
                    &mut on_before_mutate,
                );
                self.top_xy = vp_top;

                self.render_3d_pane(ui, rect_tr, project, selected_part, loaded_textures);

                let mut vp_front = self.front_xz.clone();
                self.render_ortho_pane(
                    ui,
                    rect_bl,
                    &mut vp_front,
                    project,
                    selected_part,
                    active_material_id,
                    &mut on_before_mutate,
                );
                self.front_xz = vp_front;

                let mut vp_side = self.side_yz.clone();
                self.render_ortho_pane(
                    ui,
                    rect_br,
                    &mut vp_side,
                    project,
                    selected_part,
                    active_material_id,
                    &mut on_before_mutate,
                );
                self.side_yz = vp_side;

                // Dragging the 4-viewport splitters
                let v_split_rect = Rect::from_min_max(
                    Pos2::new(mid_x - 4.0, avail.top()),
                    Pos2::new(mid_x + 4.0, avail.bottom()),
                );
                let h_split_rect = Rect::from_min_max(
                    Pos2::new(avail.left(), mid_y - 4.0),
                    Pos2::new(avail.right(), mid_y + 4.0),
                );

                let v_resp = ui.interact(v_split_rect, ui.scope_id().with("quad_v_split"), Sense::drag());
                if v_resp.hovered() || v_resp.dragged() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                }
                if v_resp.dragged() {
                    self.split_x = ((mid_x + v_resp.drag_delta().x - avail.left()) / avail.width())
                        .clamp(0.2, 0.8);
                }

                let h_resp = ui.interact(h_split_rect, ui.scope_id().with("quad_h_split"), Sense::drag());
                if h_resp.hovered() || h_resp.dragged() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
                }
                if h_resp.dragged() {
                    self.split_y = ((mid_y + h_resp.drag_delta().y - avail.top()) / avail.height())
                        .clamp(0.2, 0.8);
                }

                ui.painter().line_segment(
                    [Pos2::new(mid_x, avail.top()), Pos2::new(mid_x, avail.bottom())],
                    Stroke::new(2.0, Color32::from_gray(65)),
                );
                ui.painter().line_segment(
                    [Pos2::new(avail.left(), mid_y), Pos2::new(avail.right(), mid_y)],
                    Stroke::new(2.0, Color32::from_gray(65)),
                );
            }
        }
    }

    /// Rendering and interaction for a single 2D orthogonal viewport (Top XY / Front XZ / Side YZ) with scrollbars
    fn render_ortho_pane(
        &mut self,
        ui: &mut egui::Ui,
        rect: Rect,
        vp: &mut OrthoViewportState,
        project: &mut Project,
        selected_part: &mut Option<PartId>,
        active_material_id: MaterialId,
        on_before_mutate: &mut impl FnMut(&Project),
    ) {
        let pane_idx = match vp.axis {
            OrthoAxis::TopXY => 0,
            OrthoAxis::FrontXZ => 1,
            OrthoAxis::SideYZ => 2,
        };
        self.last_pane_sizes[pane_idx] = rect.size();

        // Allocate space for horizontal (bottom) and vertical (right) scrollbars
        let sb_thick = 14.0_f32;
        let canvas_rect = Rect::from_min_max(
            rect.min,
            Pos2::new(
                (rect.right() - sb_thick).max(rect.left() + 24.0),
                (rect.bottom() - sb_thick).max(rect.top() + 24.0),
            ),
        );
        let horiz_sb_rect = Rect::from_min_max(
            Pos2::new(rect.left(), canvas_rect.bottom()),
            Pos2::new(canvas_rect.right(), rect.bottom()),
        );
        let vert_sb_rect = Rect::from_min_max(
            Pos2::new(canvas_rect.right(), rect.top()),
            Pos2::new(rect.right(), canvas_rect.bottom()),
        );
        let corner_rect = Rect::from_min_max(canvas_rect.max, rect.max);

        let id = ui.scope_id().with(("ortho_pane", vp.axis as u8));
        let resp = ui.interact(canvas_rect, id, Sense::click_and_drag());
        if resp.hovered() || resp.clicked() || resp.dragged() {
            self.last_active_ortho = vp.axis;
        }
        let painter = ui.painter_at(canvas_rect);

        // Dark drafting background
        painter.rect_filled(canvas_rect, 0.0, Color32::from_rgb(18, 22, 26));

        // Mouse wheel zooming relative to cursor and RMB / MMB panning
        if resp.hovered() {
            let scroll_y = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll_y.abs() > 0.1 {
                if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                    let before_uv = vp.screen_to_world(canvas_rect, mouse_pos);
                    let factor = (1.0 + scroll_y * 0.0018).clamp(0.75, 1.35);
                    vp.zoom_px_per_mm = (vp.zoom_px_per_mm * factor).clamp(0.03, 8.0);
                    let after_uv = vp.screen_to_world(canvas_rect, mouse_pos);
                    vp.pan_uv_mm += before_uv - after_uv;
                }
            }
        }

        // Pan using middle or right mouse button
        if resp.dragged_by(egui::PointerButton::Middle)
            || resp.dragged_by(egui::PointerButton::Secondary)
        {
            let delta = resp.drag_delta();
            vp.pan_uv_mm.x -= delta.x / vp.zoom_px_per_mm;
            vp.pan_uv_mm.y += delta.y / vp.zoom_px_per_mm;
        }

        // 1. Render millimeter grid
        self.draw_ortho_grid(&painter, canvas_rect, vp);

        // 2. Render all visible parts in 2D projection
        for part in &project.parts {
            if !project.is_part_visible(part) {
                continue;
            }
            let is_sel = self.is_part_selected(*selected_part, part.id);
            self.draw_part_in_ortho(&painter, canvas_rect, vp, project, part, is_sel);
        }

        // 3. Render resize handles (for single selection) or common bounding box for group selection
        if self.selected_parts.len() > 1 {
            if let Some((gmin_3d, gmax_3d)) = self.compute_selected_bounds(project) {
                let min_uv = vp.axis.project(gmin_3d);
                let max_uv = vp.axis.project(gmax_3d);
                let p_min_scr = vp.world_to_screen(canvas_rect, Vec2::new(min_uv.x, max_uv.y));
                let p_max_scr = vp.world_to_screen(canvas_rect, Vec2::new(max_uv.x, min_uv.y));
                let grp_rect = Rect::from_min_max(p_min_scr, p_max_scr).expand(3.0);
                let grp_col = Color32::from_rgb(90, 215, 255);
                painter.rect_stroke(grp_rect, 2, Stroke::new(1.5, grp_col), egui::StrokeKind::Inside);
                let badge_text = self.lang.tf(
                    "fmt.viewport_group_badge",
                    &[
                        &self.selected_parts.len(),
                        &self.lang.dict().unit_pcs,
                        &format!("{:.0}", max_uv.x - min_uv.x),
                        &format!("{:.0}", max_uv.y - min_uv.y),
                        &self.lang.dict().unit_mm,
                    ],
                );
                painter.text(
                    Pos2::new(grp_rect.center().x, grp_rect.top() - 8.0),
                    egui::Align2::CENTER_BOTTOM,
                    badge_text,
                    FontId::proportional(11.0),
                    grp_col,
                );
            }
        } else if let Some(sel_id) = *selected_part {
            if let Some(sel_part) = project.parts.iter().find(|p| p.id == sel_id) {
                if project.is_part_visible(sel_part) {
                    if self.show_distance_callouts {
                        let callouts = compute_neighbor_distances_2d(project, vp.axis, sel_part);
                        for c in callouts {
                            self.draw_distance_callout(
                                &painter,
                                canvas_rect,
                                vp,
                                &c.start_uv,
                                &c.end_uv,
                                c.distance_mm,
                            );
                        }
                    }
                    self.draw_selection_handles_2d(&painter, canvas_rect, vp, project, sel_part);
                }
            }
        }

        // 4. Handle LMB interaction: Selection (single and marquee box), Drag/Resize, Block creation, Tape measure
        self.handle_ortho_lmb_interaction(
            ui,
            &resp,
            &painter,
            canvas_rect,
            vp,
            project,
            selected_part,
            active_material_id,
            on_before_mutate,
        );

        // 5. Viewport header with fullscreen maximize button
        self.draw_ortho_header_overlay(ui, &painter, canvas_rect, vp);

        // 6. Scrollbars at the bottom and right of the 2D window and outer border
        self.draw_and_handle_ortho_scrollbars(
            ui,
            canvas_rect,
            horiz_sb_rect,
            vert_sb_rect,
            corner_rect,
            vp,
            project,
        );
        ui.painter_at(rect)
            .rect_stroke(rect, 0, Stroke::new(1.0, Color32::from_gray(55)), egui::StrokeKind::Inside);
    }

    fn draw_ortho_grid(&self, painter: &Painter, rect: Rect, vp: &OrthoViewportState) {
        let min_uv = vp.screen_to_world(rect, Pos2::new(rect.left(), rect.bottom()));
        let max_uv = vp.screen_to_world(rect, Pos2::new(rect.right(), rect.top()));

        // Choose adaptive visual grid step so lines don't blend together when zoomed out
        let mut visual_step = self.grid_step_mm.max(1.0);
        while visual_step * vp.zoom_px_per_mm < 10.0 {
            visual_step *= 2.0;
        }

        let start_u = (min_uv.x / visual_step).floor() * visual_step;
        let end_u = (max_uv.x / visual_step).ceil() * visual_step;
        let mut u = start_u;
        while u <= end_u {
            let sx = vp.world_to_screen(rect, Vec2::new(u, 0.0)).x;
            let is_major = (u % (visual_step * 5.0)).abs() < 0.01;
            let color = if u.abs() < 0.01 {
                Color32::from_rgb(95, 45, 45)
            } else if is_major {
                Color32::from_rgb(36, 44, 52)
            } else {
                Color32::from_rgb(25, 31, 37)
            };
            painter.line_segment(
                [Pos2::new(sx, rect.top()), Pos2::new(sx, rect.bottom())],
                Stroke::new(1.0, color),
            );
            u += visual_step;
        }

        let start_v = (min_uv.y / visual_step).floor() * visual_step;
        let end_v = (max_uv.y / visual_step).ceil() * visual_step;
        let mut v = start_v;
        while v <= end_v {
            let sy = vp.world_to_screen(rect, Vec2::new(0.0, v)).y;
            let is_major = (v % (visual_step * 5.0)).abs() < 0.01;
            let color = if v.abs() < 0.01 {
                Color32::from_rgb(45, 95, 45)
            } else if is_major {
                Color32::from_rgb(36, 44, 52)
            } else {
                Color32::from_rgb(25, 31, 37)
            };
            painter.line_segment(
                [Pos2::new(rect.left(), sy), Pos2::new(rect.right(), sy)],
                Stroke::new(1.0, color),
            );
            v += visual_step;
        }

        // Origin point (0, 0) with millimeter axis labels
        let origin_scr = vp.world_to_screen(rect, Vec2::ZERO);
        if rect.contains(origin_scr) {
            painter.circle_filled(origin_scr, 3.5, Color32::from_rgb(220, 180, 60));
            painter.text(
                origin_scr + EguiVec2::new(5.0, -5.0),
                egui::Align2::LEFT_BOTTOM,
                format!("(0, 0 {})", self.lang.dict().unit_mm),
                FontId::monospace(10.0),
                Color32::from_rgb(180, 180, 140),
            );
        }
    }

    fn draw_part_in_ortho(
        &self,
        painter: &Painter,
        rect: Rect,
        vp: &OrthoViewportState,
        project: &Project,
        part: &PartBlock,
        is_selected: bool,
    ) {
        let mat = project.get_material(part.material_id);
        let thick = mat.map(|m| m.thickness_mm).unwrap_or(18.0);
        let rgb = mat.map(|m| m.color_rgb).unwrap_or([200, 200, 200]);

        let fill_color = if is_selected {
            Color32::from_rgba_unmultiplied(255, 195, 60, 75)
        } else {
            Color32::from_rgba_unmultiplied(rgb[0], rgb[1], rgb[2], 42)
        };
        let stroke_color = if is_selected {
            Color32::from_rgb(255, 210, 50)
        } else {
            Color32::from_rgb(rgb[0], rgb[1], rgb[2])
        };

        // If part is rotated around Z axis and we are in TopXY, draw a rotated quadrilateral
        if vp.axis == OrthoAxis::TopXY && part.yaw_deg.abs() > 0.05 {
            let verts = part.world_vertices(thick);
            let poly: Vec<Pos2> = (0..4)
                .map(|i| vp.world_to_screen(rect, Vec2::new(verts[i].x, verts[i].y)))
                .collect();
            draw_solid_polygon(painter, &poly, fill_color);
            let stroke = Stroke::new(if is_selected { 2.0 } else { 1.2 }, stroke_color);
            for i in 0..poly.len() {
                painter.line_segment([poly[i], poly[(i + 1) % poly.len()]], stroke);
            }
            let center_scr = vp.world_to_screen(
                rect,
                Vec2::new(part.world_center(thick).x, part.world_center(thick).y),
            );
            painter.text(
                center_scr,
                egui::Align2::CENTER_CENTER,
                format!("{} ({:.0}°)", part.name, part.yaw_deg),
                FontId::proportional(11.0),
                Color32::WHITE,
            );
            return;
        }

        let (min_3d, max_3d) = part.world_aabb(thick);
        let min_uv = vp.axis.project(min_3d);
        let max_uv = vp.axis.project(max_3d);

        let p_min_scr = vp.world_to_screen(rect, Vec2::new(min_uv.x, max_uv.y));
        let p_max_scr = vp.world_to_screen(rect, Vec2::new(max_uv.x, min_uv.y));
        let part_rect = Rect::from_min_max(p_min_scr, p_max_scr);

        if !rect.intersects(part_rect) {
            return;
        }

        painter.rect_filled(part_rect, 0.0, fill_color);
        let stroke_w = if is_selected { 1.8 } else { 1.0 };
        let stroke_rect = if part_rect.width() > stroke_w && part_rect.height() > stroke_w {
            part_rect.shrink(stroke_w * 0.5)
        } else {
            part_rect
        };
        painter.rect_stroke(
            stroke_rect,
            0,
            Stroke::new(stroke_w, stroke_color),
            egui::StrokeKind::Inside,
        );

        // Display edge banding on part faces in 2D projections
        self.draw_part_edges_in_ortho(painter, part_rect, vp.axis, project, part);

        // If part is viewed flat and is large enough on screen, draw texture direction arrow and dimensions
        if part_rect.width() > 48.0 && part_rect.height() > 20.0 {
            let dim_u = max_uv.x - min_uv.x;
            let dim_v = max_uv.y - min_uv.y;
            let label = if is_selected || (part_rect.width() > 90.0 && part_rect.height() > 34.0) {
                format!("{}\n{:.0}×{:.0}", part.name, dim_u, dim_v)
            } else {
                format!("{:.0}×{:.0}", dim_u, dim_v)
            };
            painter.text(
                part_rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                FontId::proportional(11.0),
                if is_selected {
                    Color32::from_rgb(255, 240, 150)
                } else {
                    Color32::from_gray(225)
                },
            );
        }
    }

    /// Highlights edge-banded sides of a part with colored lines on the 2D drawing
    fn draw_part_edges_in_ortho(
        &self,
        painter: &Painter,
        part_rect: Rect,
        axis: OrthoAxis,
        project: &Project,
        part: &PartBlock,
    ) {
        let edge_stroke = |eid: Option<EdgeId>| -> Option<Stroke> {
            let e = project.get_edge(eid?)?;
            let c = Color32::from_rgb(e.color_rgb[0], e.color_rgb[1], e.color_rgb[2]);
            let w = if e.thickness_mm >= 1.5 { 3.2 } else { 2.0 };
            Some(Stroke::new(w, c))
        };

        // Map part sides (L1=Front/Bottom, L2=Back/Top, W1=Left/Bottom, W2=Right/Top) to the screen rectangle,
        // insetting edge banding lines inside part_rect so they never bleed into adjacent parts.
        match (axis, part.orientation) {
            (OrthoAxis::TopXY, PartOrientation::HorizontalXY) => {
                // Shelf in XY plan: Length along X, Width along Y.
                // L1 = front face (top of screen +Y -> top), L2 = rear face (bottom of screen -Y -> bottom)
                // W1 = left edge (left), W2 = right edge (right)
                if let Some(s) = edge_stroke(part.edges.l1) {
                    let inset = (s.width * 0.5).min(part_rect.height() * 0.5);
                    let y = part_rect.top() + inset;
                    painter.line_segment([Pos2::new(part_rect.left(), y), Pos2::new(part_rect.right(), y)], s);
                }
                if let Some(s) = edge_stroke(part.edges.l2) {
                    let inset = (s.width * 0.5).min(part_rect.height() * 0.5);
                    let y = part_rect.bottom() - inset;
                    painter.line_segment([Pos2::new(part_rect.left(), y), Pos2::new(part_rect.right(), y)], s);
                }
                if let Some(s) = edge_stroke(part.edges.w1) {
                    let inset = (s.width * 0.5).min(part_rect.width() * 0.5);
                    let x = part_rect.left() + inset;
                    painter.line_segment([Pos2::new(x, part_rect.top()), Pos2::new(x, part_rect.bottom())], s);
                }
                if let Some(s) = edge_stroke(part.edges.w2) {
                    let inset = (s.width * 0.5).min(part_rect.width() * 0.5);
                    let x = part_rect.right() - inset;
                    painter.line_segment([Pos2::new(x, part_rect.top()), Pos2::new(x, part_rect.bottom())], s);
                }
            }
            (OrthoAxis::SideYZ, PartOrientation::VerticalYZ) => {
                // Side upright in YZ profile: Width along Y (horizontal), Length along Z (vertical).
                // L1 = front vertical face (+Y -> right), L2 = rear vertical face (-Y -> left)
                // W1 = bottom edge (bottom), W2 = top edge (top)
                if let Some(s) = edge_stroke(part.edges.l1) {
                    let inset = (s.width * 0.5).min(part_rect.width() * 0.5);
                    let x = part_rect.right() - inset;
                    painter.line_segment([Pos2::new(x, part_rect.top()), Pos2::new(x, part_rect.bottom())], s);
                }
                if let Some(s) = edge_stroke(part.edges.l2) {
                    let inset = (s.width * 0.5).min(part_rect.width() * 0.5);
                    let x = part_rect.left() + inset;
                    painter.line_segment([Pos2::new(x, part_rect.top()), Pos2::new(x, part_rect.bottom())], s);
                }
                if let Some(s) = edge_stroke(part.edges.w1) {
                    let inset = (s.width * 0.5).min(part_rect.height() * 0.5);
                    let y = part_rect.bottom() - inset;
                    painter.line_segment([Pos2::new(part_rect.left(), y), Pos2::new(part_rect.right(), y)], s);
                }
                if let Some(s) = edge_stroke(part.edges.w2) {
                    let inset = (s.width * 0.5).min(part_rect.height() * 0.5);
                    let y = part_rect.top() + inset;
                    painter.line_segment([Pos2::new(part_rect.left(), y), Pos2::new(part_rect.right(), y)], s);
                }
            }
            (OrthoAxis::FrontXZ, PartOrientation::FrontalXZ) => {
                // Front/Back panel in XZ front view: Width along X (horizontal), Length along Z (vertical).
                // L1 = left vertical face, L2 = right vertical face
                // W1 = bottom edge, W2 = top edge
                if let Some(s) = edge_stroke(part.edges.l1) {
                    let inset = (s.width * 0.5).min(part_rect.width() * 0.5);
                    let x = part_rect.left() + inset;
                    painter.line_segment([Pos2::new(x, part_rect.top()), Pos2::new(x, part_rect.bottom())], s);
                }
                if let Some(s) = edge_stroke(part.edges.l2) {
                    let inset = (s.width * 0.5).min(part_rect.width() * 0.5);
                    let x = part_rect.right() - inset;
                    painter.line_segment([Pos2::new(x, part_rect.top()), Pos2::new(x, part_rect.bottom())], s);
                }
                if let Some(s) = edge_stroke(part.edges.w1) {
                    let inset = (s.width * 0.5).min(part_rect.height() * 0.5);
                    let y = part_rect.bottom() - inset;
                    painter.line_segment([Pos2::new(part_rect.left(), y), Pos2::new(part_rect.right(), y)], s);
                }
                if let Some(s) = edge_stroke(part.edges.w2) {
                    let inset = (s.width * 0.5).min(part_rect.height() * 0.5);
                    let y = part_rect.top() + inset;
                    painter.line_segment([Pos2::new(part_rect.left(), y), Pos2::new(part_rect.right(), y)], s);
                }
            }
            _ => {}
        }
    }

    /// Draws resize handle markers around the selected part (all 8 for face-on parts, 2 along length/width for edge-on parts)
    fn draw_selection_handles_2d(
        &self,
        painter: &Painter,
        rect: Rect,
        vp: &OrthoViewportState,
        project: &Project,
        part: &PartBlock,
    ) {
        let thick = project.get_material_thickness(part.material_id);
        let (min_3d, max_3d) = part.world_aabb(thick);
        let min_uv = vp.axis.project(min_3d);
        let max_uv = vp.axis.project(max_3d);

        let p_min_scr = vp.world_to_screen(rect, Vec2::new(min_uv.x, max_uv.y));
        let p_max_scr = vp.world_to_screen(rect, Vec2::new(max_uv.x, min_uv.y));
        let r = Rect::from_min_max(p_min_scr, p_max_scr);

        let mode = part_view_mode_2d(vp.axis, part, thick);
        for (hp, _handle) in get_allowed_handles_2d(r, mode) {
            let hrect = Rect::from_center_size(hp, EguiVec2::splat(7.0));
            painter.rect_filled(hrect, 1.0, Color32::WHITE);
            painter.rect_stroke(hrect, 1, Stroke::new(1.2, Color32::from_rgb(20, 20, 20)), egui::StrokeKind::Inside);
        }
    }

    /// Draws a dimension callout arrow (distance in mm to the neighboring part)
    fn draw_distance_callout(
        &self,
        painter: &Painter,
        rect: Rect,
        vp: &OrthoViewportState,
        start_uv: &Vec2,
        end_uv: &Vec2,
        distance_mm: f32,
    ) {
        let p1 = vp.world_to_screen(rect, *start_uv);
        let p2 = vp.world_to_screen(rect, *end_uv);
        let color = Color32::from_rgb(80, 220, 255);

        painter.line_segment([p1, p2], Stroke::new(1.3, color));
        painter.circle_filled(p1, 2.5, color);
        painter.circle_filled(p2, 2.5, color);

        let mid = Pos2::new((p1.x + p2.x) * 0.5, (p1.y + p2.y) * 0.5);
        let text = format!("{:.0} {}", distance_mm, self.lang.dict().unit_mm);
        let bg_rect = Rect::from_center_size(mid, EguiVec2::new(48.0, 16.0));
        painter.rect_filled(bg_rect, 3.0, Color32::from_rgba_unmultiplied(10, 25, 35, 220));
        painter.rect_stroke(bg_rect, 3, Stroke::new(1.0, color), egui::StrokeKind::Inside);
        painter.text(
            mid,
            egui::Align2::CENTER_CENTER,
            text,
            FontId::monospace(10.5),
            color,
        );
    }

    /// Handle LMB in 2D pane: part selection, dragging handles with snap, new part creation, and tape measure
    fn handle_ortho_lmb_interaction(
        &mut self,
        ui: &mut egui::Ui,
        resp: &egui::Response,
        painter: &Painter,
        rect: Rect,
        vp: &OrthoViewportState,
        project: &mut Project,
        selected_part: &mut Option<PartId>,
        active_material_id: MaterialId,
        on_before_mutate: &mut impl FnMut(&Project),
    ) {
        let mouse_pos = ui.input(|i| i.pointer.interact_pos());

        // --- MODE 1: Tape Measure (MeasureTape) ---
        if self.tool == EditorTool::MeasureTape {
            if resp.drag_started_by(egui::PointerButton::Primary) {
                if let Some(mp) = mouse_pos {
                    let mut uv = vp.screen_to_world(rect, mp);
                    if self.grid_snap_enabled {
                        uv.x = snap_to_grid(uv.x, self.grid_step_mm);
                        uv.y = snap_to_grid(uv.y, self.grid_step_mm);
                    }
                    self.measure_points = Some((vp.axis, uv, uv));
                }
            }
            if resp.dragged_by(egui::PointerButton::Primary) {
                if let (Some(mp), Some((ax, start_uv, _))) = (mouse_pos, self.measure_points) {
                    if ax == vp.axis {
                        let mut uv = vp.screen_to_world(rect, mp);
                        let (u_cands, v_cands) = collect_snap_candidates_2d(project, vp.axis, None);
                        let snap_thresh = 20.0 / vp.zoom_px_per_mm;
                        let ctrl_cmd_held = ui.input(|i| i.modifiers.ctrl || i.modifiers.command);
                        let edge_snap = self.edge_snap_enabled && !ctrl_cmd_held;
                        uv.x = snap_1d_edge(
                            uv.x,
                            &u_cands,
                            snap_thresh,
                            edge_snap,
                            self.grid_snap_enabled,
                            self.grid_step_mm,
                        )
                        .snapped_pos;
                        uv.y = snap_1d_edge(
                            uv.y,
                            &v_cands,
                            snap_thresh,
                            edge_snap,
                            self.grid_snap_enabled,
                            self.grid_step_mm,
                        )
                        .snapped_pos;
                        self.measure_points = Some((ax, start_uv, uv));
                    }
                }
            }
            if let Some((ax, p1, p2)) = self.measure_points {
                if ax == vp.axis {
                    let s1 = vp.world_to_screen(rect, p1);
                    let s2 = vp.world_to_screen(rect, p2);
                    let dist = p1.distance(p2);
                    let dx = (p2.x - p1.x).abs();
                    let dy = (p2.y - p1.y).abs();
                    let col = Color32::from_rgb(120, 255, 140);
                    painter.line_segment([s1, s2], Stroke::new(2.0, col));
                    painter.circle_filled(s1, 4.0, col);
                    painter.circle_filled(s2, 4.0, col);
                    let mid = Pos2::new((s1.x + s2.x) * 0.5, (s1.y + s2.y) * 0.5 - 14.0);
                    painter.text(
                        mid,
                        egui::Align2::CENTER_CENTER,
                        format!("L = {:.1} {} (ΔU={:.1}, ΔV={:.1})", dist, self.lang.dict().unit_mm, dx, dy),
                        FontId::monospace(11.5),
                        col,
                    );
                }
            }
            return;
        }

        // --- MODE 2: Create new part via box (CreateBlock) ---
        if self.tool == EditorTool::CreateBlock {
            if resp.drag_started_by(egui::PointerButton::Primary) {
                if let Some(mp) = mouse_pos {
                    let mut uv = vp.screen_to_world(rect, mp);
                    let (u_cands, v_cands) = collect_snap_candidates_2d(project, vp.axis, None);
                    let snap_thresh = 20.0 / vp.zoom_px_per_mm;
                    let ctrl_cmd_held = ui.input(|i| i.modifiers.ctrl || i.modifiers.command);
                    let edge_snap = self.edge_snap_enabled && !ctrl_cmd_held;
                    uv.x = snap_1d_edge(
                        uv.x,
                        &u_cands,
                        snap_thresh,
                        edge_snap,
                        self.grid_snap_enabled,
                        self.grid_step_mm,
                    )
                    .snapped_pos;
                    uv.y = snap_1d_edge(
                        uv.y,
                        &v_cands,
                        snap_thresh,
                        edge_snap,
                        self.grid_snap_enabled,
                        self.grid_step_mm,
                    )
                    .snapped_pos;
                    self.creating_drag = Some((vp.axis, uv, uv));
                }
            }

            if resp.dragged_by(egui::PointerButton::Primary) {
                if let (Some(mp), Some((ax, start_uv, _))) = (mouse_pos, self.creating_drag) {
                    if ax == vp.axis {
                        let mut uv = vp.screen_to_world(rect, mp);
                        let (u_cands, v_cands) = collect_snap_candidates_2d(project, vp.axis, None);
                        let snap_thresh = 20.0 / vp.zoom_px_per_mm;
                        let ctrl_cmd_held = ui.input(|i| i.modifiers.ctrl || i.modifiers.command);
                        let edge_snap = self.edge_snap_enabled && !ctrl_cmd_held;
                        uv.x = snap_1d_edge(
                            uv.x,
                            &u_cands,
                            snap_thresh,
                            edge_snap,
                            self.grid_snap_enabled,
                            self.grid_step_mm,
                        )
                        .snapped_pos;
                        uv.y = snap_1d_edge(
                            uv.y,
                            &v_cands,
                            snap_thresh,
                            edge_snap,
                            self.grid_snap_enabled,
                            self.grid_step_mm,
                        )
                        .snapped_pos;
                        self.creating_drag = Some((ax, start_uv, uv));
                    }
                }
            }

            // Render preview of the created block
            if let Some((ax, p1, p2)) = self.creating_drag {
                if ax == vp.axis {
                    let min_uv = p1.min(p2);
                    let max_uv = p1.max(p2);
                    let s1 = vp.world_to_screen(rect, Vec2::new(min_uv.x, max_uv.y));
                    let s2 = vp.world_to_screen(rect, Vec2::new(max_uv.x, min_uv.y));
                    let preview_rect = Rect::from_min_max(s1, s2);
                    let col = Color32::from_rgb(80, 240, 140);
                    painter.rect_filled(
                        preview_rect,
                        0.0,
                        Color32::from_rgba_unmultiplied(80, 240, 140, 45),
                    );
                    painter.rect_stroke(preview_rect, 0, Stroke::new(1.8, col), egui::StrokeKind::Inside);
                    let preview_msg = self.lang.tf(
                        "fmt.viewport_new_part_preview",
                        &[
                            &format!("{:.0}", max_uv.x - min_uv.x),
                            &format!("{:.0}", max_uv.y - min_uv.y),
                            &self.lang.dict().unit_mm,
                        ],
                    );
                    painter.text(
                        preview_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        preview_msg,
                        FontId::proportional(11.5),
                        col,
                    );
                }
            }

            if resp.drag_stopped_by(egui::PointerButton::Primary) {
                if let Some((ax, p1, p2)) = self.creating_drag.take() {
                    if ax == vp.axis {
                        let min_uv = p1.min(p2);
                        let max_uv = p1.max(p2);
                        let du = (max_uv.x - min_uv.x).max(18.0);
                        let dv = (max_uv.y - min_uv.y).max(18.0);
                        on_before_mutate(project);
                        let new_id = self.spawn_part_from_2d_box(
                            project,
                            vp.axis,
                            min_uv,
                            Vec2::new(du, dv),
                            active_material_id,
                            *selected_part,
                        );
                        self.set_single_selection(selected_part, Some(new_id));
                        self.tool = EditorTool::SelectAndMove;
                    }
                }
            }
            return;
        }

        // --- MODE 3: Selection (single and marquee box), Group Dragging and Handle Resizing (SelectAndMove) ---
        let modifiers = ui.input(|i| i.modifiers);
        let multi_mod = modifiers.shift || modifiers.ctrl || modifiers.command;

        // Update mouse cursor when hovering over resize handle squares or inside the selected block/group
        if let Some(drag) = &self.active_drag {
            if drag.axis == vp.axis {
                ui.ctx().set_cursor_icon(drag.handle.cursor_icon());
            }
        } else if resp.hovered() {
            if let Some(hp) = ui.input(|i| i.pointer.hover_pos()) {
                let hover_uv = vp.screen_to_world(rect, hp);
                if self.selected_parts.len() == 1 {
                    if let Some(sel_id) = *selected_part {
                        if let Some(sel_part) = project.parts.iter().find(|p| p.id == sel_id) {
                            if project.is_part_visible(sel_part) && !project.is_part_locked(sel_part) {
                                if let Some(h) = self.hit_test_handles_2d(rect, vp, project, sel_part, hp) {
                                    ui.ctx().set_cursor_icon(h.cursor_icon());
                                } else if self.is_point_inside_part_2d(vp, project, sel_part, hover_uv) {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::Move);
                                }
                            }
                        }
                    }
                } else if self.selected_parts.len() > 1 {
                    let inside_any_selected = self.selected_parts.iter().any(|&sid| {
                        project
                            .parts
                            .iter()
                            .find(|p| p.id == sid)
                            .map(|p| self.is_point_inside_part_2d(vp, project, p, hover_uv))
                            .unwrap_or(false)
                    });
                    if inside_any_selected {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Move);
                    }
                }
            }
        }

        if resp.drag_started_by(egui::PointerButton::Primary) {
            // Use the exact mouse press origin (press_origin) so fast mouse movements don't miss the handle square!
            let press_pos = ui.input(|i| i.pointer.press_origin()).or(mouse_pos);
            if let Some(mp) = press_pos {
                let click_uv = vp.screen_to_world(rect, mp);

                let mut picked_handle: Option<(PartId, DragHandle2D)> = None;

                // 1. If exactly 1 part is selected, check its 8 resize handles and its inner area first
                if self.selected_parts.len() == 1 {
                    if let Some(sel_id) = *selected_part {
                        if let Some(sel_part) = project.parts.iter().find(|p| p.id == sel_id) {
                            if project.is_part_visible(sel_part) && !project.is_part_locked(sel_part) {
                                if let Some(h) = self.hit_test_handles_2d(rect, vp, project, sel_part, mp) {
                                    picked_handle = Some((sel_id, h));
                                } else if !multi_mod && self.is_point_inside_part_2d(vp, project, sel_part, click_uv) {
                                    picked_handle = Some((sel_id, DragHandle2D::MoveWholeBlock));
                                }
                            }
                        }
                    }
                } else if self.selected_parts.len() > 1 && !multi_mod {
                    // 2. If a group of parts is selected and the user drags any selected part (or inside the group bounds), move the whole group!
                    let hit_selected = self.selected_parts.iter().copied().find(|&sid| {
                        project
                            .parts
                            .iter()
                            .find(|p| p.id == sid)
                            .map(|p| {
                                project.is_part_visible(p)
                                    && !project.is_part_locked(p)
                                    && self.is_point_inside_part_2d(vp, project, p, click_uv)
                            })
                            .unwrap_or(false)
                    });
                    if let Some(sid) = hit_selected {
                        picked_handle = Some((sid, DragHandle2D::MoveWholeBlock));
                    }
                }

                // 3. If missed an already selected part/handle, test click on another part or begin marquee selection
                if picked_handle.is_none() {
                    if let Some(hit_id) = self.hit_test_parts_2d(vp, project, click_uv) {
                        if multi_mod {
                            if !self.selected_parts.contains(&hit_id) {
                                self.selected_parts.push(hit_id);
                            }
                            *selected_part = self.selected_parts.first().copied();
                        } else if !self.selected_parts.contains(&hit_id) {
                            self.set_single_selection(selected_part, Some(hit_id));
                        }
                        if let Some(p) = project.parts.iter().find(|p| p.id == hit_id) {
                            if !project.is_part_locked(p) {
                                picked_handle = Some((hit_id, DragHandle2D::MoveWholeBlock));
                            }
                        }
                    } else {
                        // Started dragging in empty space -> initiate marquee box selection!
                        let base_sel = if multi_mod {
                            self.selected_parts.clone()
                        } else {
                            self.set_single_selection(selected_part, None);
                            Vec::new()
                        };
                        self.box_select_drag = Some((vp.axis, click_uv, click_uv, multi_mod, base_sel));
                    }
                }

                // Initiate drag operation (single part or whole selected group)
                if let Some((pid, handle)) = picked_handle {
                    if handle == DragHandle2D::MoveWholeBlock && self.selected_parts.len() > 1 {
                        if let Some((gmin, gmax)) = self.compute_selected_bounds(project) {
                            let group_orig: Vec<(PartId, Vec3)> = self
                                .selected_parts
                                .iter()
                                .filter_map(|&sid| {
                                    let p = project.parts.iter().find(|p| p.id == sid)?;
                                    if project.is_part_visible(p) && !project.is_part_locked(p) {
                                        Some((p.id, p.position_mm))
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            if !group_orig.is_empty() {
                                on_before_mutate(project);
                                self.active_drag = Some(ActiveDrag2D {
                                    axis: vp.axis,
                                    part_id: pid,
                                    handle: DragHandle2D::MoveWholeBlock,
                                    start_mouse_uv: click_uv,
                                    orig_min_3d: gmin,
                                    orig_max_3d: gmax,
                                    group_orig_positions: group_orig,
                                    active_guides_u: Vec::new(),
                                    active_guides_v: Vec::new(),
                                });
                            }
                        }
                    } else if let Some(p) = project.parts.iter().find(|p| p.id == pid) {
                        on_before_mutate(project);
                        let thick = project.get_material_thickness(p.material_id);
                        let size = p.local_box_size(thick);
                        self.active_drag = Some(ActiveDrag2D {
                            axis: vp.axis,
                            part_id: pid,
                            handle,
                            start_mouse_uv: click_uv,
                            orig_min_3d: p.position_mm,
                            orig_max_3d: p.position_mm + size,
                            group_orig_positions: vec![(pid, p.position_mm)],
                            active_guides_u: Vec::new(),
                            active_guides_v: Vec::new(),
                        });
                    }
                }
            }
        }

        // Plain click without drag: select part (with Shift/Ctrl support for multi-selection)
        if resp.clicked_by(egui::PointerButton::Primary) {
            if let Some(mp) = mouse_pos {
                let mut hit_handle = false;
                if self.selected_parts.len() == 1 {
                    if let Some(sel_id) = *selected_part {
                        if let Some(sel_part) = project.parts.iter().find(|p| p.id == sel_id) {
                            if self.hit_test_handles_2d(rect, vp, project, sel_part, mp).is_some() {
                                hit_handle = true;
                            }
                        }
                    }
                }
                if !hit_handle {
                    let click_uv = vp.screen_to_world(rect, mp);
                    if let Some(hit_id) = self.hit_test_parts_2d(vp, project, click_uv) {
                        if multi_mod {
                            self.toggle_part_in_selection(selected_part, hit_id);
                        } else {
                            self.set_single_selection(selected_part, Some(hit_id));
                        }
                    } else if !multi_mod {
                        self.set_single_selection(selected_part, None);
                    }
                }
            }
        }

        // Update marquee box selection
        if let Some((ax, start_uv, ref mut cur_uv, _append, ref base_sel)) = self.box_select_drag {
            if ax == vp.axis {
                if resp.dragged_by(egui::PointerButton::Primary) {
                    if let Some(mp) = mouse_pos {
                        *cur_uv = vp.screen_to_world(rect, mp);
                    }
                }
                let bmin = start_uv.min(*cur_uv);
                let bmax = start_uv.max(*cur_uv);
                let s1 = vp.world_to_screen(rect, Vec2::new(bmin.x, bmax.y));
                let s2 = vp.world_to_screen(rect, Vec2::new(bmax.x, bmin.y));
                let box_rect = Rect::from_min_max(s1, s2);

                // Find all visible unlocked parts intersecting the selection box
                let mut new_sel = base_sel.clone();
                for p in &project.parts {
                    if !project.is_part_visible(p) || project.is_part_locked(p) {
                        continue;
                    }
                    let thick = project.get_material_thickness(p.material_id);
                    let (pmin_3d, pmax_3d) = p.world_aabb(thick);
                    let pmin_uv = vp.axis.project(pmin_3d);
                    let pmax_uv = vp.axis.project(pmax_3d);
                    let intersects = pmax_uv.x >= bmin.x
                        && pmin_uv.x <= bmax.x
                        && pmax_uv.y >= bmin.y
                        && pmin_uv.y <= bmax.y;
                    if intersects && !new_sel.contains(&p.id) {
                        new_sel.push(p.id);
                    }
                }
                self.selected_parts = new_sel;
                *selected_part = self.selected_parts.first().copied();

                let col = Color32::from_rgb(90, 200, 255);
                painter.rect_filled(
                    box_rect,
                    0.0,
                    Color32::from_rgba_unmultiplied(70, 175, 255, 38),
                );
                painter.rect_stroke(box_rect, 0, Stroke::new(1.5, col), egui::StrokeKind::Inside);
                if box_rect.width() > 40.0 && box_rect.height() > 18.0 {
                    let count_msg = self.lang.tf(
                        "fmt.viewport_selected_count",
                        &[&self.selected_parts.len(), &self.lang.dict().unit_pcs],
                    );
                    painter.text(
                        box_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        count_msg,
                        FontId::proportional(11.0),
                        col,
                    );
                }
            }
        }

        // Update active dragging (single or group) / resizing
        if resp.dragged_by(egui::PointerButton::Primary) {
            if let (Some(mp), Some(drag)) = (mouse_pos, self.active_drag.as_mut()) {
                if drag.axis == vp.axis {
                    let cur_uv = vp.screen_to_world(rect, mp);
                    let mut delta_uv = cur_uv - drag.start_mouse_uv;
                    let exclude_ids: Vec<PartId> = if drag.group_orig_positions.len() > 1 {
                        drag.group_orig_positions.iter().map(|(id, _)| *id).collect()
                    } else {
                        vec![drag.part_id]
                    };
                    let (u_cands, v_cands) =
                        collect_snap_candidates_2d_excluding(project, vp.axis, &exclude_ids);
                    let snap_thresh = 20.0 / vp.zoom_px_per_mm;

                    let shift_held = ui.input(|i| i.modifiers.shift);
                    let ctrl_cmd_held = ui.input(|i| i.modifiers.ctrl || i.modifiers.command);
                    let alt_held = ui.input(|i| i.modifiers.alt);
                    let edge_snap = self.edge_snap_enabled && !ctrl_cmd_held;

                    if drag.handle == DragHandle2D::MoveWholeBlock {
                        if alt_held {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::Copy);
                        }
                        if shift_held {
                            if delta_uv.x.abs() >= delta_uv.y.abs() {
                                delta_uv.y = 0.0;
                            } else {
                                delta_uv.x = 0.0;
                            }
                        }
                    }

                    drag.active_guides_u.clear();
                    drag.active_guides_v.clear();

                    let orig_min_uv = vp.axis.project(drag.orig_min_3d);
                    let orig_max_uv = vp.axis.project(drag.orig_max_3d);
                    let orig_size_uv = orig_max_uv - orig_min_uv;

                    let mut new_min_uv = orig_min_uv;
                    let mut new_max_uv = orig_max_uv;

                    match drag.handle {
                        DragHandle2D::MoveWholeBlock => {
                            let target_min_u = orig_min_uv.x + delta_uv.x;
                            let target_min_v = orig_min_uv.y + delta_uv.y;

                            if shift_held && delta_uv.y == 0.0 && delta_uv.x != 0.0 {
                                // Locked to horizontal axis: snap U only, V stays at orig_min_uv.y
                                let su = snap_interval_move(
                                    target_min_u,
                                    orig_size_uv.x,
                                    &u_cands,
                                    snap_thresh,
                                    edge_snap,
                                    self.grid_snap_enabled,
                                    self.grid_step_mm,
                                );
                                new_min_uv.x = su.snapped_pos;
                                new_max_uv.x = su.snapped_pos + orig_size_uv.x;
                                new_min_uv.y = orig_min_uv.y;
                                new_max_uv.y = orig_max_uv.y;
                                if let Some(gu) = su.guide_line_pos {
                                    drag.active_guides_u.push(gu);
                                }
                            } else if shift_held && delta_uv.x == 0.0 && delta_uv.y != 0.0 {
                                // Locked to vertical axis: snap V only, U stays at orig_min_uv.x
                                let sv = snap_interval_move(
                                    target_min_v,
                                    orig_size_uv.y,
                                    &v_cands,
                                    snap_thresh,
                                    edge_snap,
                                    self.grid_snap_enabled,
                                    self.grid_step_mm,
                                );
                                new_min_uv.x = orig_min_uv.x;
                                new_max_uv.x = orig_max_uv.x;
                                new_min_uv.y = sv.snapped_pos;
                                new_max_uv.y = sv.snapped_pos + orig_size_uv.y;
                                if let Some(gv) = sv.guide_line_pos {
                                    drag.active_guides_v.push(gv);
                                }
                            } else {
                                let su = snap_interval_move(
                                    target_min_u,
                                    orig_size_uv.x,
                                    &u_cands,
                                    snap_thresh,
                                    edge_snap,
                                    self.grid_snap_enabled,
                                    self.grid_step_mm,
                                );
                                let sv = snap_interval_move(
                                    target_min_v,
                                    orig_size_uv.y,
                                    &v_cands,
                                    snap_thresh,
                                    edge_snap,
                                    self.grid_snap_enabled,
                                    self.grid_step_mm,
                                );

                                new_min_uv.x = su.snapped_pos;
                                new_max_uv.x = su.snapped_pos + orig_size_uv.x;
                                new_min_uv.y = sv.snapped_pos;
                                new_max_uv.y = sv.snapped_pos + orig_size_uv.y;

                                if let Some(gu) = su.guide_line_pos {
                                    drag.active_guides_u.push(gu);
                                }
                                if let Some(gv) = sv.guide_line_pos {
                                    drag.active_guides_v.push(gv);
                                }
                            }
                        }
                        _ => {
                            // Resize individual edges of a single part
                            let resize_left = matches!(
                                drag.handle,
                                DragHandle2D::ResizeLeft
                                    | DragHandle2D::ResizeBottomLeft
                                    | DragHandle2D::ResizeTopLeft
                            );
                            let resize_right = matches!(
                                drag.handle,
                                DragHandle2D::ResizeRight
                                    | DragHandle2D::ResizeBottomRight
                                    | DragHandle2D::ResizeTopRight
                            );
                            let resize_bottom = matches!(
                                drag.handle,
                                DragHandle2D::ResizeBottom
                                    | DragHandle2D::ResizeBottomLeft
                                    | DragHandle2D::ResizeBottomRight
                            );
                            let resize_top = matches!(
                                drag.handle,
                                DragHandle2D::ResizeTop
                                    | DragHandle2D::ResizeTopLeft
                                    | DragHandle2D::ResizeTopRight
                            );

                            if resize_left {
                                let su = snap_1d_edge(
                                    orig_min_uv.x + delta_uv.x,
                                    &u_cands,
                                    snap_thresh,
                                    edge_snap,
                                    self.grid_snap_enabled,
                                    self.grid_step_mm,
                                );
                                new_min_uv.x = su.snapped_pos.min(new_max_uv.x - 10.0);
                                if let Some(gu) = su.guide_line_pos {
                                    drag.active_guides_u.push(gu);
                                }
                            }
                            if resize_right {
                                let su = snap_1d_edge(
                                    orig_max_uv.x + delta_uv.x,
                                    &u_cands,
                                    snap_thresh,
                                    edge_snap,
                                    self.grid_snap_enabled,
                                    self.grid_step_mm,
                                );
                                new_max_uv.x = su.snapped_pos.max(new_min_uv.x + 10.0);
                                if let Some(gu) = su.guide_line_pos {
                                    drag.active_guides_u.push(gu);
                                }
                            }
                            if resize_bottom {
                                let sv = snap_1d_edge(
                                    orig_min_uv.y + delta_uv.y,
                                    &v_cands,
                                    snap_thresh,
                                    edge_snap,
                                    self.grid_snap_enabled,
                                    self.grid_step_mm,
                                );
                                new_min_uv.y = sv.snapped_pos.min(new_max_uv.y - 10.0);
                                if let Some(gv) = sv.guide_line_pos {
                                    drag.active_guides_v.push(gv);
                                }
                            }
                            if resize_top {
                                let sv = snap_1d_edge(
                                    orig_max_uv.y + delta_uv.y,
                                    &v_cands,
                                    snap_thresh,
                                    edge_snap,
                                    self.grid_snap_enabled,
                                    self.grid_step_mm,
                                );
                                new_max_uv.y = sv.snapped_pos.max(new_min_uv.y + 10.0);
                                if let Some(gv) = sv.guide_line_pos {
                                    drag.active_guides_v.push(gv);
                                }
                            }
                        }
                    }

                    // Apply new 2D coordinates to 3D block (or group of blocks)
                    let new_min_3d = vp.axis.unproject_into(new_min_uv, drag.orig_min_3d);
                    let new_max_3d = vp.axis.unproject_into(new_max_uv, drag.orig_max_3d);

                    if drag.handle == DragHandle2D::MoveWholeBlock && drag.group_orig_positions.len() > 1 {
                        let delta_3d = new_min_3d - drag.orig_min_3d;
                        for &(pid, orig_pos) in &drag.group_orig_positions {
                            if let Some(part) = project.parts.iter_mut().find(|p| p.id == pid) {
                                part.position_mm = orig_pos + delta_3d;
                            }
                        }
                    } else if let Some(part) = project.parts.iter_mut().find(|p| p.id == drag.part_id) {
                        let new_size_3d = new_max_3d - new_min_3d;
                        part.position_mm = new_min_3d;
                        if drag.handle != DragHandle2D::MoveWholeBlock {
                            part.set_from_local_box_size(new_size_3d);
                        }
                    }

                    // If Alt is held while moving parts, draw ghost silhouette of the original parts
                    if alt_held && drag.handle == DragHandle2D::MoveWholeBlock {
                        for &(orig_id, orig_pos) in &drag.group_orig_positions {
                            if let Some(p) = project.parts.iter().find(|p| p.id == orig_id) {
                                let thick = project.get_material_thickness(p.material_id);
                                let size = p.local_box_size(thick);
                                let omin_uv = vp.axis.project(orig_pos);
                                let omax_uv = vp.axis.project(orig_pos + size);
                                let s1 = vp.world_to_screen(rect, Vec2::new(omin_uv.x, omax_uv.y));
                                let s2 = vp.world_to_screen(rect, Vec2::new(omax_uv.x, omin_uv.y));
                                let o_rect = Rect::from_min_max(s1, s2);
                                painter.rect_filled(o_rect, 0.0, Color32::from_rgba_unmultiplied(80, 180, 255, 30));
                                painter.rect_stroke(o_rect, 0, Stroke::new(1.2, Color32::from_rgba_unmultiplied(90, 200, 255, 180)), egui::StrokeKind::Inside);
                            }
                        }
                    }
                }
            }
        }

        // Render magnetic snap guide lines
        if let Some(drag) = &self.active_drag {
            if drag.axis == vp.axis {
                let guide_col = Color32::from_rgb(255, 90, 220);
                for &gu in &drag.active_guides_u {
                    let sx = vp.world_to_screen(rect, Vec2::new(gu, 0.0)).x;
                    painter.line_segment(
                        [Pos2::new(sx, rect.top()), Pos2::new(sx, rect.bottom())],
                        Stroke::new(1.4, guide_col),
                    );
                }
                for &gv in &drag.active_guides_v {
                    let sy = vp.world_to_screen(rect, Vec2::new(0.0, gv)).y;
                    painter.line_segment(
                        [Pos2::new(rect.left(), sy), Pos2::new(rect.right(), sy)],
                        Stroke::new(1.4, guide_col),
                    );
                }
            }
        }

        if resp.drag_stopped_by(egui::PointerButton::Primary) {
            if let Some(drag) = &self.active_drag {
                let alt_held = ui.input(|i| i.modifiers.alt);
                if alt_held && drag.handle == DragHandle2D::MoveWholeBlock {
                    let cur_pos_opt = project.parts.iter().find(|p| p.id == drag.part_id).map(|p| p.position_mm);
                    let orig_pos_opt = drag.group_orig_positions.iter().find(|(id, _)| *id == drag.part_id).map(|(_, pos)| *pos);
                    if let (Some(cur_pos), Some(orig_pos)) = (cur_pos_opt, orig_pos_opt) {
                        let delta_3d = cur_pos - orig_pos;
                        if delta_3d.length() >= 1.0 {
                            let mut new_clones = Vec::new();
                            let copy_word = self.lang.t("copy");
                            for &(pid, orig_pos) in &drag.group_orig_positions {
                                if let Some(idx) = project.parts.iter().position(|p| p.id == pid) {
                                    let orig_name = project.parts[idx].name.clone();
                                    let copy_name = project.generate_copy_name(&orig_name, copy_word);
                                    let part = &mut project.parts[idx];
                                    let mut clone = part.clone();
                                    clone.name = copy_name;
                                    clone.position_mm = part.position_mm;
                                    part.position_mm = orig_pos; // Restore original part
                                    new_clones.push(clone);
                                }
                            }
                            let mut new_ids = Vec::new();
                            for mut clone in new_clones {
                                clone.id = project.alloc_id();
                                new_ids.push(clone.id);
                                project.parts.push(clone);
                            }
                            self.selected_parts = new_ids;
                            *selected_part = self.selected_parts.first().copied();
                        }
                    }
                }
            }
            self.active_drag = None;
            self.box_select_drag = None;
        }
    }

    /// Rendering and interaction for scrollbars (horizontal and vertical) in 2D views
    fn draw_and_handle_ortho_scrollbars(
        &self,
        ui: &mut egui::Ui,
        canvas_rect: Rect,
        horiz_rect: Rect,
        vert_rect: Rect,
        corner_rect: Rect,
        vp: &mut OrthoViewportState,
        project: &Project,
    ) {
        let (pmin_3d, pmax_3d) = Self::compute_project_bounds(project)
            .unwrap_or((Vec3::ZERO, Vec3::new(800.0, 500.0, 720.0)));
        let pmin_uv = vp.axis.project(pmin_3d);
        let pmax_uv = vp.axis.project(pmax_3d);

        let vis_w_mm = (canvas_rect.width() / vp.zoom_px_per_mm).max(20.0);
        let vis_h_mm = (canvas_rect.height() / vp.zoom_px_per_mm).max(20.0);

        let margin_u = ((pmax_uv.x - pmin_uv.x) * 1.2).max(1000.0).max(vis_w_mm * 0.8);
        let margin_v = ((pmax_uv.y - pmin_uv.y) * 1.2).max(1000.0).max(vis_h_mm * 0.8);

        let view_min_u = vp.pan_uv_mm.x - vis_w_mm * 0.5;
        let view_max_u = vp.pan_uv_mm.x + vis_w_mm * 0.5;
        let world_min_u = (pmin_uv.x - margin_u).min(view_min_u);
        let world_max_u = (pmax_uv.x + margin_u).max(view_max_u);
        let total_u = (world_max_u - world_min_u).max(vis_w_mm + 1.0);

        let view_min_v = vp.pan_uv_mm.y - vis_h_mm * 0.5;
        let view_max_v = vp.pan_uv_mm.y + vis_h_mm * 0.5;
        let world_min_v = (pmin_uv.y - margin_v).min(view_min_v);
        let world_max_v = (pmax_uv.y + margin_v).max(view_max_v);
        let total_v = (world_max_v - world_min_v).max(vis_h_mm + 1.0);

        let track_bg = Color32::from_rgb(28, 32, 38);
        let btn_bg = Color32::from_rgb(52, 58, 68);
        let btn_hover_bg = Color32::from_rgb(72, 82, 96);
        let thumb_bg = Color32::from_rgb(84, 94, 108);
        let thumb_hover_bg = Color32::from_rgb(108, 122, 140);
        let border_stroke = Stroke::new(1.0, Color32::from_gray(42));
        let grip_stroke = Stroke::new(1.0, Color32::from_gray(170));

        // Corner square between scrollbars
        ui.painter_at(corner_rect).rect_filled(corner_rect, 0.0, btn_bg);

        // --- 1. Horizontal scrollbar ---
        let h_painter = ui.painter_at(horiz_rect);
        h_painter.rect_filled(horiz_rect, 0.0, track_bg);
        let btn_sz = horiz_rect.height();

        let left_btn = Rect::from_min_size(horiz_rect.min, EguiVec2::splat(btn_sz));
        let right_btn = Rect::from_min_size(
            Pos2::new(horiz_rect.right() - btn_sz, horiz_rect.top()),
            EguiVec2::splat(btn_sz),
        );
        let h_track = Rect::from_min_max(
            Pos2::new(left_btn.right(), horiz_rect.top()),
            Pos2::new(right_btn.left(), horiz_rect.bottom()),
        );

        let l_resp = ui.interact(left_btn, ui.scope_id().with(("sb_hl", vp.axis as u8)), Sense::click());
        h_painter.rect_filled(left_btn, 0.0, if l_resp.hovered() { btn_hover_bg } else { btn_bg });
        h_painter.rect_stroke(left_btn, 0, border_stroke, egui::StrokeKind::Inside);
        h_painter.text(left_btn.center(), egui::Align2::CENTER_CENTER, "◄", FontId::proportional(9.0), Color32::LIGHT_GRAY);
        if l_resp.clicked() {
            vp.pan_uv_mm.x -= vis_w_mm * 0.18;
        }

        let r_resp = ui.interact(right_btn, ui.scope_id().with(("sb_hr", vp.axis as u8)), Sense::click());
        h_painter.rect_filled(right_btn, 0.0, if r_resp.hovered() { btn_hover_bg } else { btn_bg });
        h_painter.rect_stroke(right_btn, 0, border_stroke, egui::StrokeKind::Inside);
        h_painter.text(right_btn.center(), egui::Align2::CENTER_CENTER, "►", FontId::proportional(9.0), Color32::LIGHT_GRAY);
        if r_resp.clicked() {
            vp.pan_uv_mm.x += vis_w_mm * 0.18;
        }

        if h_track.width() > 16.0 {
            let frac_w = (vis_w_mm / total_u).clamp(0.08, 1.0);
            let thumb_w = (h_track.width() * frac_w).max(22.0).min(h_track.width());
            let avail_slide = (h_track.width() - thumb_w).max(1.0);
            let norm_pos = ((view_min_u - world_min_u) / (total_u - vis_w_mm).max(1.0)).clamp(0.0, 1.0);
            let thumb_x = h_track.left() + norm_pos * (h_track.width() - thumb_w);
            let h_thumb = Rect::from_min_size(
                Pos2::new(thumb_x, h_track.top() + 1.0),
                EguiVec2::new(thumb_w, h_track.height() - 2.0),
            );

            let ht_resp = ui.interact(
                h_track,
                ui.scope_id().with(("sb_htrack", vp.axis as u8)),
                Sense::click_and_drag(),
            );
            let thumb_active = ht_resp.hovered() || ht_resp.dragged();
            h_painter.rect_filled(
                h_thumb,
                2.0,
                if thumb_active { thumb_hover_bg } else { thumb_bg },
            );
            h_painter.rect_stroke(h_thumb, 2, Stroke::new(1.0, Color32::from_gray(135)), egui::StrokeKind::Inside);

            // 3 grip marks in the middle of the slider (|||)
            let cx = h_thumb.center().x;
            for dx in [-3.0_f32, 0.0, 3.0] {
                h_painter.line_segment(
                    [
                        Pos2::new(cx + dx, h_thumb.top() + 3.0),
                        Pos2::new(cx + dx, h_thumb.bottom() - 3.0),
                    ],
                    grip_stroke,
                );
            }

            if ht_resp.dragged_by(egui::PointerButton::Primary) {
                let d_px = ht_resp.drag_delta().x;
                vp.pan_uv_mm.x += (d_px / avail_slide) * (total_u - vis_w_mm);
            } else if ht_resp.clicked_by(egui::PointerButton::Primary) {
                if let Some(cp) = ht_resp.interact_pointer_pos() {
                    if cp.x < h_thumb.left() {
                        vp.pan_uv_mm.x -= vis_w_mm * 0.6;
                    } else if cp.x > h_thumb.right() {
                        vp.pan_uv_mm.x += vis_w_mm * 0.6;
                    }
                }
            }
        }

        // --- 2. Vertical scrollbar ---
        let v_painter = ui.painter_at(vert_rect);
        v_painter.rect_filled(vert_rect, 0.0, track_bg);
        let v_btn_sz = vert_rect.width();

        let top_btn = Rect::from_min_size(vert_rect.min, EguiVec2::splat(v_btn_sz));
        let bot_btn = Rect::from_min_size(
            Pos2::new(vert_rect.left(), vert_rect.bottom() - v_btn_sz),
            EguiVec2::splat(v_btn_sz),
        );
        let v_track = Rect::from_min_max(
            Pos2::new(vert_rect.left(), top_btn.bottom()),
            Pos2::new(vert_rect.right(), bot_btn.top()),
        );

        let t_resp = ui.interact(top_btn, ui.scope_id().with(("sb_vt", vp.axis as u8)), Sense::click());
        v_painter.rect_filled(top_btn, 0.0, if t_resp.hovered() { btn_hover_bg } else { btn_bg });
        v_painter.rect_stroke(top_btn, 0, border_stroke, egui::StrokeKind::Inside);
        v_painter.text(top_btn.center(), egui::Align2::CENTER_CENTER, "▲", FontId::proportional(9.0), Color32::LIGHT_GRAY);
        if t_resp.clicked() {
            vp.pan_uv_mm.y += vis_h_mm * 0.18;
        }

        let b_resp = ui.interact(bot_btn, ui.scope_id().with(("sb_vb", vp.axis as u8)), Sense::click());
        v_painter.rect_filled(bot_btn, 0.0, if b_resp.hovered() { btn_hover_bg } else { btn_bg });
        v_painter.rect_stroke(bot_btn, 0, border_stroke, egui::StrokeKind::Inside);
        v_painter.text(bot_btn.center(), egui::Align2::CENTER_CENTER, "▼", FontId::proportional(9.0), Color32::LIGHT_GRAY);
        if b_resp.clicked() {
            vp.pan_uv_mm.y -= vis_h_mm * 0.18;
        }

        if v_track.height() > 16.0 {
            let frac_h = (vis_h_mm / total_v).clamp(0.08, 1.0);
            let thumb_h = (v_track.height() * frac_h).max(22.0).min(v_track.height());
            let avail_slide = (v_track.height() - thumb_h).max(1.0);
            // Up on screen is +V (world_max_v), down on screen is -V (world_min_v)
            let norm_pos = ((world_max_v - view_max_v) / (total_v - vis_h_mm).max(1.0)).clamp(0.0, 1.0);
            let thumb_y = v_track.top() + norm_pos * (v_track.height() - thumb_h);
            let v_thumb = Rect::from_min_size(
                Pos2::new(v_track.left() + 1.0, thumb_y),
                EguiVec2::new(v_track.width() - 2.0, thumb_h),
            );

            let vt_resp = ui.interact(
                v_track,
                ui.scope_id().with(("sb_vtrack", vp.axis as u8)),
                Sense::click_and_drag(),
            );
            let thumb_active = vt_resp.hovered() || vt_resp.dragged();
            v_painter.rect_filled(
                v_thumb,
                2.0,
                if thumb_active { thumb_hover_bg } else { thumb_bg },
            );
            v_painter.rect_stroke(v_thumb, 2, Stroke::new(1.0, Color32::from_gray(135)), egui::StrokeKind::Inside);

            // 3 horizontal grip marks in the middle of the slider (≡)
            let cy = v_thumb.center().y;
            for dy in [-3.0_f32, 0.0, 3.0] {
                v_painter.line_segment(
                    [
                        Pos2::new(v_thumb.left() + 3.0, cy + dy),
                        Pos2::new(v_thumb.right() - 3.0, cy + dy),
                    ],
                    grip_stroke,
                );
            }

            if vt_resp.dragged_by(egui::PointerButton::Primary) {
                let d_px = vt_resp.drag_delta().y;
                vp.pan_uv_mm.y -= (d_px / avail_slide) * (total_v - vis_h_mm);
            } else if vt_resp.clicked_by(egui::PointerButton::Primary) {
                if let Some(cp) = vt_resp.interact_pointer_pos() {
                    if cp.y < v_thumb.top() {
                        vp.pan_uv_mm.y += vis_h_mm * 0.6;
                    } else if cp.y > v_thumb.bottom() {
                        vp.pan_uv_mm.y -= vis_h_mm * 0.6;
                    }
                }
            }
        }
    }

    fn hit_test_handles_2d(
        &self,
        rect: Rect,
        vp: &OrthoViewportState,
        project: &Project,
        part: &PartBlock,
        mouse_pos: Pos2,
    ) -> Option<DragHandle2D> {
        let thick = project.get_material_thickness(part.material_id);
        let (min_3d, max_3d) = part.world_aabb(thick);
        let min_uv = vp.axis.project(min_3d);
        let max_uv = vp.axis.project(max_3d);

        let p_min_scr = vp.world_to_screen(rect, Vec2::new(min_uv.x, max_uv.y));
        let p_max_scr = vp.world_to_screen(rect, Vec2::new(max_uv.x, min_uv.y));
        let r = Rect::from_min_max(p_min_scr, p_max_scr);

        let mode = part_view_mode_2d(vp.axis, part, thick);
        for (pt, handle) in get_allowed_handles_2d(r, mode) {
            if pt.distance(mouse_pos) <= 10.0 {
                return Some(handle);
            }
        }
        None
    }

    fn is_point_inside_part_2d(
        &self,
        vp: &OrthoViewportState,
        project: &Project,
        part: &PartBlock,
        uv: Vec2,
    ) -> bool {
        let thick = project.get_material_thickness(part.material_id);
        let (min_3d, max_3d) = part.world_aabb(thick);
        let min_uv = vp.axis.project(min_3d);
        let max_uv = vp.axis.project(max_3d);
        uv.x >= min_uv.x - 2.0
            && uv.x <= max_uv.x + 2.0
            && uv.y >= min_uv.y - 2.0
            && uv.y <= max_uv.y + 2.0
    }

    fn hit_test_parts_2d(
        &self,
        vp: &OrthoViewportState,
        project: &Project,
        click_uv: Vec2,
    ) -> Option<PartId> {
        // Pick the part with the smallest projected area (so thin shelf/upright edges are easily selected over the back panel)
        let mut best_id = None;
        let mut best_area = f32::MAX;

        for part in &project.parts {
            if !project.is_part_visible(part) {
                continue;
            }
            let thick = project.get_material_thickness(part.material_id);
            let (min_3d, max_3d) = part.world_aabb(thick);
            let min_uv = vp.axis.project(min_3d);
            let max_uv = vp.axis.project(max_3d);

            if click_uv.x >= min_uv.x - 2.0
                && click_uv.x <= max_uv.x + 2.0
                && click_uv.y >= min_uv.y - 2.0
                && click_uv.y <= max_uv.y + 2.0
            {
                let area = (max_uv.x - min_uv.x).max(1.0) * (max_uv.y - min_uv.y).max(1.0);
                if area < best_area {
                    best_area = area;
                    best_id = Some(part.id);
                }
            }
        }
        best_id
    }

    /// Creates a new part from a drawn 2D rectangle, automatically applying active material thickness (e.g. 18 mm)
    fn spawn_part_from_2d_box(
        &self,
        project: &mut Project,
        axis: OrthoAxis,
        min_uv: Vec2,
        size_uv: Vec2,
        active_material_id: MaterialId,
        reference_part_id: Option<PartId>,
    ) -> PartId {
        let mat = project.get_material(active_material_id);
        let thick = mat.map(|m| m.thickness_mm).unwrap_or(18.0);
        let grain_fixed = mat.map(|m| m.default_grain_fixed).unwrap_or(true);

        // Use reference part's 3rd coordinate (or 0.0)
        let ref_part = reference_part_id.and_then(|id| project.parts.iter().find(|p| p.id == id));
        let default_depth_y = ref_part.map(|p| p.width_mm).unwrap_or(500.0);

        let orientation = self.create_orientation_override.unwrap_or_else(|| {
            match axis {
                OrthoAxis::TopXY => PartOrientation::HorizontalXY,
                OrthoAxis::FrontXZ => {
                    if size_uv.x <= thick * 2.2 && size_uv.y > size_uv.x * 2.0 {
                        PartOrientation::VerticalYZ // Narrow vertical box -> Side upright
                    } else if size_uv.y <= thick * 2.2 && size_uv.x > size_uv.y * 2.0 {
                        PartOrientation::HorizontalXY // Narrow horizontal box -> Shelf
                    } else {
                        PartOrientation::FrontalXZ // Wide box -> Front panel / Backing
                    }
                }
                OrthoAxis::SideYZ => {
                    if size_uv.y <= thick * 2.2 {
                        PartOrientation::HorizontalXY
                    } else {
                        PartOrientation::VerticalYZ
                    }
                }
            }
        });

        let (pos_3d, length_mm, width_mm) = match (axis, orientation) {
            (OrthoAxis::TopXY, PartOrientation::HorizontalXY) => (
                Vec3::new(min_uv.x, min_uv.y, 0.0),
                size_uv.x,
                size_uv.y,
            ),
            (OrthoAxis::TopXY, PartOrientation::VerticalYZ) => (
                Vec3::new(min_uv.x, min_uv.y, 0.0),
                720.0,
                size_uv.y,
            ),
            (OrthoAxis::TopXY, PartOrientation::FrontalXZ) => (
                Vec3::new(min_uv.x, min_uv.y, 0.0),
                720.0,
                size_uv.x,
            ),
            (OrthoAxis::FrontXZ, PartOrientation::HorizontalXY) => (
                Vec3::new(min_uv.x, 0.0, min_uv.y),
                size_uv.x,
                default_depth_y,
            ),
            (OrthoAxis::FrontXZ, PartOrientation::VerticalYZ) => (
                Vec3::new(min_uv.x, 0.0, min_uv.y),
                size_uv.y,
                default_depth_y,
            ),
            (OrthoAxis::FrontXZ, PartOrientation::FrontalXZ) => (
                Vec3::new(min_uv.x, 0.0, min_uv.y),
                size_uv.y,
                size_uv.x,
            ),
            (OrthoAxis::SideYZ, PartOrientation::VerticalYZ) => (
                Vec3::new(0.0, min_uv.x, min_uv.y),
                size_uv.y,
                size_uv.x,
            ),
            (OrthoAxis::SideYZ, PartOrientation::HorizontalXY) => (
                Vec3::new(0.0, min_uv.x, min_uv.y),
                600.0,
                size_uv.x,
            ),
            (OrthoAxis::SideYZ, PartOrientation::FrontalXZ) => (
                Vec3::new(0.0, min_uv.x, min_uv.y),
                size_uv.y,
                600.0,
            ),
        };
        let name_prefix = orientation.short_label_i18n(self.lang);

        let new_id = project.alloc_id();
        let count = project.parts.len() + 1;
        project.parts.push(PartBlock {
            id: new_id,
            name: format!("{} {}", name_prefix, count),
            group_id: project.groups.first().map(|g| g.id),
            material_id: active_material_id,
            orientation,
            position_mm: pos_3d,
            length_mm,
            width_mm,
            yaw_deg: 0.0,
            grain_fixed,
            grain_along_width: false,
            edges: PartEdges::default(),
            visible: true,
            locked: false,
            drill_holes: Vec::new(),
        });
        new_id
    }

    fn draw_ortho_header_overlay(
        &mut self,
        ui: &mut egui::Ui,
        painter: &Painter,
        rect: Rect,
        vp: &OrthoViewportState,
    ) {
        let header_rect = Rect::from_min_size(
            rect.min + EguiVec2::new(6.0, 6.0),
            EguiVec2::new((rect.width() - 12.0).min(360.0), 22.0),
        );
        painter.rect_filled(
            header_rect,
            4.0,
            Color32::from_rgba_unmultiplied(28, 34, 42, 215),
        );
        let (ax_u, ax_v) = vp.axis.axis_labels_i18n(self.lang);
        painter.text(
            header_rect.left_center() + EguiVec2::new(8.0, 0.0),
            egui::Align2::LEFT_CENTER,
            format!(
                "{} | {}×{} | {:.0}%",
                vp.axis.title_i18n(self.lang),
                ax_u,
                ax_v,
                vp.zoom_px_per_mm * 100.0
            ),
            FontId::proportional(11.5),
            Color32::from_rgb(190, 215, 240),
        );

        // Window maximize button
        let btn_rect = Rect::from_min_size(
            Pos2::new(rect.right() - 78.0, rect.top() + 6.0),
            EguiVec2::new(72.0, 22.0),
        );
        let target_pane = match vp.axis {
            OrthoAxis::TopXY => MaximizedPane::TopXY,
            OrthoAxis::FrontXZ => MaximizedPane::FrontXZ,
            OrthoAxis::SideYZ => MaximizedPane::SideYZ,
        };
        let is_max = self.maximized == target_pane;
        let btn_resp = ui.interact(
            btn_rect,
            ui.scope_id().with(("max_btn", vp.axis as u8)),
            Sense::click(),
        );
        painter.rect_filled(
            btn_rect,
            4.0,
            if btn_resp.hovered() {
                Color32::from_rgb(65, 90, 120)
            } else {
                Color32::from_rgba_unmultiplied(35, 45, 58, 220)
            },
        );
        painter.text(
            btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            if is_max {
                self.lang.t("[4 Views]")
            } else {
                self.lang.t("[Maximize]")
            },
            FontId::proportional(11.0),
            Color32::WHITE,
        );
        if btn_resp.clicked() {
            self.maximized = if is_max {
                MaximizedPane::None
            } else {
                target_pane
            };
        }
    }

    /// Interactive 3D viewport rendering (Perspective, perspective-correct textures/colors, edge banding, Orbit controls, and first-person Z-Noclip)
    fn render_3d_pane(
        &mut self,
        ui: &mut egui::Ui,
        rect: Rect,
        project: &Project,
        selected_part: &mut Option<PartId>,
        loaded_textures: &HashMap<MaterialId, TextureHandle>,
    ) {
        self.last_pane_sizes[3] = rect.size();

        let id = ui.scope_id().with("pane_3d_view");
        let resp = ui.interact(rect, id, Sense::click_and_drag());
        let painter = ui.painter().with_clip_rect(rect);

        // 3D scene background
        painter.rect_filled(rect, 0.0, Color32::from_rgb(22, 26, 32));

        // Toggle first-person Noclip on unmodified `Z` key (WITHOUT Ctrl/Alt) only when cursor is over the 3D window!
        let text_input_busy = ui.ctx().egui_wants_keyboard_input();
        let pointer_in_3d = ui
            .input(|i| i.pointer.hover_pos())
            .map(|p| rect.contains(p))
            .unwrap_or(false)
            || resp.hovered();

        if !text_input_busy && (pointer_in_3d || self.cam_3d.noclip_active) {
            if self.cam_3d.noclip_active && crate::web::check_and_clear_pointer_lock_lost() {
                self.cam_3d.noclip_active = false;
                self.cam_3d.sync_orbit_from_eye();
                Self::set_noclip_cursor_grab(ui.ctx(), false, rect.center());
            }

            if self.cam_3d.noclip_active && resp.clicked() && !crate::web::is_web_pointer_locked() {
                crate::web::request_web_pointer_lock();
            }

            let web_z = crate::web::check_and_clear_web_noclip();
            let pure_z_pressed = ui.input(|i| {
                (i.key_pressed(Key::Z)
                    || i.events.iter().any(|e| match e {
                        egui::Event::Text(t) => t == "я" || t == "Я" || t == "z" || t == "Z",
                        _ => false,
                    }))
                    && !i.modifiers.ctrl
                    && !i.modifiers.alt
                    && !i.modifiers.command
            }) || (web_z && !ui.input(|i| i.modifiers.ctrl || i.modifiers.alt || i.modifiers.command));
            let cur_time = ui.input(|i| i.time);
            if pure_z_pressed && (cur_time - self.cam_3d.last_toggle_time).abs() > 0.25 {
                self.cam_3d.last_toggle_time = cur_time;
                self.cam_3d.noclip_active = !self.cam_3d.noclip_active;
                if self.cam_3d.noclip_active {
                    Self::set_noclip_cursor_grab(ui.ctx(), true, rect.center());
                } else {
                    self.cam_3d.sync_orbit_from_eye();
                    Self::set_noclip_cursor_grab(ui.ctx(), false, rect.center());
                }
            }
            if self.cam_3d.noclip_active && ui.input(|i| i.key_pressed(Key::Escape)) {
                self.cam_3d.last_toggle_time = cur_time;
                self.cam_3d.noclip_active = false;
                self.cam_3d.sync_orbit_from_eye();
                Self::set_noclip_cursor_grab(ui.ctx(), false, rect.center());
            }
        }

        // --- 3D Camera Controls ---
        if self.cam_3d.noclip_active {
            // Hide cursor and lock it to the center of the 3D window for continuous look rotation
            ui.ctx().set_cursor_icon(egui::CursorIcon::None);
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::CursorVisible(false));
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::CursorPosition(rect.center()));

            // First-person Z-Noclip mode: take raw mouse delta from DeviceEvent::MouseMotion (pointer.motion)
            // or accumulated HTML5 Pointer Lock API motion in web mode
            let mut raw_motion = ui
                .input(|i| i.pointer.motion())
                .unwrap_or_else(|| ui.input(|i| i.pointer.delta()));

            let web_delta = crate::web::get_and_clear_web_mouse_delta();
            if web_delta.length_squared() > 0.0 {
                raw_motion = egui::Vec2::new(web_delta.x, web_delta.y);
            }

            if raw_motion.length_sq() > 0.0 {
                let sensitivity = 0.0032;
                self.cam_3d.yaw_rad -= raw_motion.x * sensitivity;
                self.cam_3d.pitch_rad =
                    (self.cam_3d.pitch_rad - raw_motion.y * sensitivity).clamp(-1.52, 1.52);
            }

            // Mouse wheel adjusts fly speed in Noclip
            let scroll_y = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll_y.abs() > 0.1 {
                self.cam_3d.fly_speed_mm_per_sec =
                    (self.cam_3d.fly_speed_mm_per_sec * (1.0 + scroll_y * 0.002)).clamp(150.0, 8000.0);
            }

            let dt = ui.input(|i| i.stable_dt).min(0.1);
            let mut move_vec = Vec3::ZERO;
            let fwd = self.cam_3d.forward_dir();
            let right = self.cam_3d.right_dir();

            ui.input(|i| {
                if i.key_down(Key::W) || i.key_down(Key::ArrowUp) || crate::web::is_web_key_w_down() {
                    move_vec += fwd;
                }
                if i.key_down(Key::S) || i.key_down(Key::ArrowDown) || crate::web::is_web_key_s_down() {
                    move_vec -= fwd;
                }
                if i.key_down(Key::D) || i.key_down(Key::ArrowRight) || crate::web::is_web_key_d_down() {
                    move_vec += right;
                }
                if i.key_down(Key::A) || i.key_down(Key::ArrowLeft) || crate::web::is_web_key_a_down() {
                    move_vec -= right;
                }
                if i.key_down(Key::Space) || i.key_down(Key::E) || crate::web::is_web_key_space_down() || crate::web::is_web_key_e_down() {
                    move_vec += Vec3::Z;
                }
                if i.key_down(Key::Q) || i.modifiers.ctrl || crate::web::is_web_key_q_down() {
                    move_vec -= Vec3::Z;
                }
            });

            if move_vec.length_squared() > 0.0 {
                let boost = if ui.input(|i| i.modifiers.shift) {
                    2.5
                } else {
                    1.0
                };
                self.cam_3d.eye_mm +=
                    move_vec.normalize() * self.cam_3d.fly_speed_mm_per_sec * boost * dt;
            }

            // Request frame repaint for smooth WASD flight
            ui.ctx().request_repaint();
        } else {
            // Standard CAD Orbit mode:
            // Camera rotation via dragging RMB, LMB on background, or MMB
            if resp.dragged_by(egui::PointerButton::Secondary)
                || (resp.dragged_by(egui::PointerButton::Primary) && !ui.input(|i| i.modifiers.shift))
            {
                let d = resp.drag_delta();
                self.cam_3d.yaw_rad -= d.x * 0.007;
                self.cam_3d.pitch_rad = (self.cam_3d.pitch_rad - d.y * 0.007).clamp(-1.52, 1.52);
                self.cam_3d.sync_eye_from_orbit();
            }

            // Camera panning (MMB or Shift + RMB/LMB)
            if resp.dragged_by(egui::PointerButton::Middle)
                || (resp.dragged() && ui.input(|i| i.modifiers.shift))
            {
                let d = resp.drag_delta();
                let right = self.cam_3d.right_dir();
                let up = self.cam_3d.up_dir();
                let pan_scale = self.cam_3d.orbit_distance_mm * 0.0012;
                self.cam_3d.orbit_target_mm -= right * (d.x * pan_scale);
                self.cam_3d.orbit_target_mm += up * (d.y * pan_scale);
                self.cam_3d.sync_eye_from_orbit();
            }

            // Mouse wheel zoom in 3D
            if resp.hovered() {
                let scroll_y = ui.input(|i| i.smooth_scroll_delta.y);
                if scroll_y.abs() > 0.1 {
                    self.cam_3d.orbit_distance_mm = (self.cam_3d.orbit_distance_mm
                        * (1.0 - scroll_y * 0.0015))
                        .clamp(120.0, 15_000.0);
                    self.cam_3d.sync_eye_from_orbit();
                }
            }
        }

        let aspect = rect.width() / rect.height().max(1.0);
        let vp_mat = self.cam_3d.view_proj_matrix(aspect);

        // 1. Render floor grid Z=0 in 3D
        self.draw_3d_floor_grid(&painter, rect, &vp_mat);

        // 2. Collect 3D block faces grouped by part with internal abutment culling,
        // near-plane 3D clipping, and screen-rect 2D clipping.
        let mut parts_to_draw: Vec<PartDrawData> = Vec::new();
        let light_dir = Vec3::new(-0.45, -0.65, 0.62).normalize();
        let eye = self.cam_3d.eye_mm;
        let fwd = self.cam_3d.forward_dir();

        // Pre-collect AABB bounds of all visible axis-aligned parts for internal abutment culling
        let part_aabbs: Vec<(PartId, Vec3, Vec3)> = project
            .parts
            .iter()
            .filter(|p| project.is_part_visible(p) && p.yaw_deg.abs() < 0.05)
            .map(|p| {
                let t = project.get_material_thickness(p.material_id);
                let (bmin, bmax) = p.world_aabb(t);
                (p.id, bmin, bmax)
            })
            .collect();

        // Indices of the 6 cube faces with uniform vertex winding:
        // 0 (-Z) and 1 (+Z): edge 0->1 along +X, edge 1->2 along +Y
        // 2 (-Y) and 3 (+Y): edge 0->1 along +X, edge 1->2 along +Z
        // 4 (-X) and 5 (+X): edge 0->1 along +Y, edge 1->2 along +Z
        let cube_faces: [([usize; 4], Vec3); 6] = [
            ([0, 1, 2, 3], Vec3::new(0.0, 0.0, -1.0)), // Bottom (-Z)
            ([4, 5, 6, 7], Vec3::new(0.0, 0.0, 1.0)),  // Top (+Z)
            ([0, 1, 5, 4], Vec3::new(0.0, -1.0, 0.0)), // Back (-Y)
            ([3, 2, 6, 7], Vec3::new(0.0, 1.0, 0.0)),  // Front (+Y)
            ([0, 3, 7, 4], Vec3::new(-1.0, 0.0, 0.0)), // Left (-X)
            ([1, 2, 6, 5], Vec3::new(1.0, 0.0, 0.0)),  // Right (+X)
        ];

        for part in &project.parts {
            if !project.is_part_visible(part) {
                continue;
            }
            let mat = project.get_material(part.material_id);
            let thick = mat.map(|m| m.thickness_mm).unwrap_or(18.0);
            let base_rgb = mat.map(|m| m.color_rgb).unwrap_or([210, 210, 210]);
            let is_sel = self.is_part_selected(*selected_part, part.id);

            let (bmin, bmax) = part.world_aabb(thick);
            let verts = part.world_vertices(thick);
            let rot_mat = part.rotation_matrix();

            let mut part_faces: Vec<Face3DToDraw> = Vec::with_capacity(6);

            for (f_idx, (v_indices, local_norm)) in cube_faces.iter().enumerate() {
                let world_norm = (rot_mat * (*local_norm)).normalize_or_zero();
                let q0 = verts[v_indices[0]];
                let q1 = verts[v_indices[1]];
                let q2 = verts[v_indices[2]];
                let q3 = verts[v_indices[3]];
                let face_center = (q0 + q1 + q2 + q3) * 0.25;

                // 1) Backface Culling
                let to_cam = eye - face_center;
                if world_norm.dot(to_cam) <= 0.0 {
                    continue;
                }

                // 2) Internal Abutment Culling:
                // Only cull narrow edge faces (thickness edges) if they are strictly flush against another part.
                // Never cull the main face of a part (front/back/top/bottom of wide panels) to avoid accidental holes!
                let is_edge_face = !self.is_main_face(part.orientation, f_idx);
                if is_edge_face && !is_sel && part.yaw_deg.abs() < 0.05 {
                    let probe_pt = face_center + world_norm * 0.4;
                    let sample_pts = [
                        probe_pt,
                        q0 + world_norm * 0.4,
                        q1 + world_norm * 0.4,
                        q2 + world_norm * 0.4,
                        q3 + world_norm * 0.4,
                    ];
                    let occluded_by_neighbor = part_aabbs.iter().any(|&(other_id, bmin, bmax)| {
                        if other_id == part.id {
                            return false;
                        }
                        sample_pts.iter().all(|pt| {
                            pt.x >= bmin.x - 0.1
                                && pt.x <= bmax.x + 0.1
                                && pt.y >= bmin.y - 0.1
                                && pt.y <= bmax.y + 0.1
                                && pt.z >= bmin.z - 0.1
                                && pt.z <= bmax.z + 0.1
                        })
                    });
                    if occluded_by_neighbor {
                        continue;
                    }
                }

                let diffuse = (world_norm.dot(light_dir) * 0.35 + 0.72).clamp(0.42, 1.12);
                let (r, g, b) = if is_sel {
                    (
                        ((base_rgb[0] as f32 * diffuse) * 0.7 + 255.0 * 0.3).min(255.0) as u8,
                        ((base_rgb[1] as f32 * diffuse) * 0.7 + 205.0 * 0.3).min(255.0) as u8,
                        ((base_rgb[2] as f32 * diffuse) * 0.5).min(255.0) as u8,
                    )
                } else {
                    (
                        (base_rgb[0] as f32 * diffuse).min(255.0) as u8,
                        (base_rgb[1] as f32 * diffuse).min(255.0) as u8,
                        (base_rgb[2] as f32 * diffuse).min(255.0) as u8,
                    )
                };

                let stroke_col = if is_sel {
                    Color32::from_rgb(255, 215, 40)
                } else {
                    Color32::from_rgb(
                        (r as u16 * 65 / 100) as u8,
                        (g as u16 * 65 / 100) as u8,
                        (b as u16 * 65 / 100) as u8,
                    )
                };

                let mut base_edge_strokes = [None, None, None, None];
                if self.cam_3d.show_edge_banding_3d {
                    if let Some(eid) = self.face_edge_id_for_part(part, f_idx) {
                        if let Some(edef) = project.get_edge(eid) {
                            let ecol = Color32::from_rgb(
                                edef.color_rgb[0],
                                edef.color_rgb[1],
                                edef.color_rgb[2],
                            );
                            let st =
                                Stroke::new(if edef.thickness_mm >= 1.5 { 2.6 } else { 1.8 }, ecol);
                            base_edge_strokes = [Some(st), Some(st), Some(st), Some(st)];
                        }
                    }
                }

                let len_u = q0.distance(q1);
                let len_v = q1.distance(q2);
                let (div_u, div_v) = (1, 1);

                // Compute physical UV scale on this face (so textures on narrow 18 mm edges aren't compressed 40x)
                // and orient texture U axis strictly along the part grain direction!
                let ref_len = part.length_mm.max(120.0);
                let ref_wid = part.width_mm.max(120.0);
                let u_scale = if len_u <= thick * 1.5 {
                    (len_u / ref_wid).clamp(0.04, 1.0)
                } else {
                    1.0
                };
                let v_scale = if len_v <= thick * 1.5 {
                    (len_v / ref_wid).clamp(0.04, 1.0)
                } else {
                    1.0
                };
                let _ = ref_len;

                // Determine whether grain runs along the V axis of this face (if so, rotate UV by 90 degrees)
                let grain_along_v = match part.orientation {
                    PartOrientation::HorizontalXY => part.grain_along_width,
                    PartOrientation::VerticalYZ | PartOrientation::FrontalXZ => !part.grain_along_width,
                };

                for iu in 0..div_u {
                    let u0 = iu as f32 / div_u as f32;
                    let u1 = (iu + 1) as f32 / div_u as f32;
                    for iv in 0..div_v {
                        let v0 = iv as f32 / div_v as f32;
                        let v1 = (iv + 1) as f32 / div_v as f32;

                        let bilerp = |u: f32, v: f32| -> Vec3 {
                            let top = q0.lerp(q1, u);
                            let bot = q3.lerp(q2, u);
                            top.lerp(bot, v)
                        };

                        let w_verts = [
                            bilerp(u0, v0),
                            bilerp(u1, v0),
                            bilerp(u1, v1),
                            bilerp(u0, v1),
                        ];

                        // Clip the 3D tile against the camera near plane so faces extending partly behind
                        // the camera are smoothly clipped instead of disappearing or projecting to infinity!
                        let clipped_world_verts =
                            clip_3d_polygon_near_plane(&w_verts, eye, fwd, NEAR_CLIP_MM);
                        if clipped_world_verts.len() < 3 {
                            continue;
                        }

                        let mut unclipped_scr_pts = Vec::with_capacity(clipped_world_verts.len());
                        let mut valid = true;
                        for &wv in &clipped_world_verts {
                            if let Some((sp, _)) = self.cam_3d.project_point(&vp_mat, rect, wv) {
                                unclipped_scr_pts.push(sp);
                            } else {
                                valid = false;
                                break;
                            }
                        }
                        if !valid || unclipped_scr_pts.len() < 3 {
                            continue;
                        }

                        // 2D clip against screen rect (with 10px tolerance) so coordinates never explode offscreen
                        let scr_pts = clip_2d_polygon_rect(&unclipped_scr_pts, rect.expand(10.0));
                        if scr_pts.len() < 3 {
                            continue;
                        }

                        let mut min_sx = f32::MAX;
                        let mut min_sy = f32::MAX;
                        let mut max_sx = f32::MIN;
                        let mut max_sy = f32::MIN;
                        for sp in &scr_pts {
                            min_sx = min_sx.min(sp.x);
                            min_sy = min_sy.min(sp.y);
                            max_sx = max_sx.max(sp.x);
                            max_sy = max_sy.max(sp.y);
                        }

                        let mut sub_center = Vec3::ZERO;
                        for &wv in &clipped_world_verts {
                            sub_center += wv;
                        }
                        sub_center /= clipped_world_verts.len() as f32;
                        let dist_to_cam = eye.distance(sub_center);
                        let _ = dist_to_cam;
                        let z_depth: f32 = (sub_center - eye).dot(fwd);

                        let outer_edges = [
                            iv == 0,         // edge 0..1 (top along V)
                            iu + 1 == div_u, // edge 1..2 (right along U)
                            iv + 1 == div_v, // edge 2..3 (bottom along V)
                            iu == 0,         // edge 3..0 (left along U)
                        ];

                        // Draw grain direction arrow only on the center tile of the main face
                        let is_center_tile = iu == div_u / 2 && iv == div_v / 2;
                        let grain_segment_3d = if is_center_tile
                            && part.grain_fixed
                            && self.is_main_face(part.orientation, f_idx)
                        {
                            let (mid_a, mid_b) = if !grain_along_v {
                                (
                                    (w_verts[0] + w_verts[3]) * 0.5,
                                    (w_verts[1] + w_verts[2]) * 0.5,
                                )
                            } else {
                                (
                                    (w_verts[0] + w_verts[1]) * 0.5,
                                    (w_verts[3] + w_verts[2]) * 0.5,
                                )
                            };
                            Some((mid_a.lerp(mid_b, 0.25), mid_a.lerp(mid_b, 0.75)))
                        } else {
                            None
                        };

                        let tu0 = u0 * u_scale;
                        let tu1 = u1 * u_scale;
                        let tv0 = v0 * v_scale;
                        let tv1 = v1 * v_scale;
                        let uv_corners = if !grain_along_v {
                            [
                                Pos2::new(tu0, tv0),
                                Pos2::new(tu1, tv0),
                                Pos2::new(tu1, tv1),
                                Pos2::new(tu0, tv1),
                            ]
                        } else {
                            [
                                Pos2::new(tv0, tu0),
                                Pos2::new(tv0, tu1),
                                Pos2::new(tv1, tu1),
                                Pos2::new(tv1, tu0),
                            ]
                        };

                        part_faces.push(Face3DToDraw {
                            part_id: part.id,
                            material_id: part.material_id,
                            world_verts: w_verts,
                            screen_pts: scr_pts,
                            screen_bbox: Rect::from_min_max(
                                Pos2::new(min_sx, min_sy),
                                Pos2::new(max_sx, max_sy),
                            ),
                            z_depth,
                            fill_color: Color32::from_rgb(r, g, b),
                            stroke_color: stroke_col,
                            is_selected: is_sel,
                            uv_corners,
                            outer_edges,
                            edge_strokes: base_edge_strokes,
                            grain_segment_3d,
                        });
                    }
                }
            }

            if !part_faces.is_empty() {
                let mut p_min_sx = f32::MAX;
                let mut p_min_sy = f32::MAX;
                let mut p_max_sx = f32::MIN;
                let mut p_max_sy = f32::MIN;
                let mut max_depth = f32::MIN;

                for f in &part_faces {
                    p_min_sx = p_min_sx.min(f.screen_bbox.min.x);
                    p_min_sy = p_min_sy.min(f.screen_bbox.min.y);
                    p_max_sx = p_max_sx.max(f.screen_bbox.max.x);
                    p_max_sy = p_max_sy.max(f.screen_bbox.max.y);
                    max_depth = max_depth.max(f.z_depth);
                }

                parts_to_draw.push(PartDrawData {
                    part_id: part.id,
                    bmin,
                    bmax,
                    yaw_deg: part.yaw_deg,
                    screen_bbox: Rect::from_min_max(
                        Pos2::new(p_min_sx, p_min_sy),
                        Pos2::new(p_max_sx, p_max_sy),
                    ),
                    max_depth,
                    faces: part_faces,
                });
            }
        }

        // Topological sort parts from back to front using Kahn's algorithm
        let n_parts = parts_to_draw.len();
        let mut graph = vec![Vec::new(); n_parts];
        let mut in_degree = vec![0usize; n_parts];

        for i in 0..n_parts {
            for j in (i + 1)..n_parts {
                if !parts_to_draw[i].screen_bbox.intersects(parts_to_draw[j].screen_bbox) {
                    continue;
                }
                if let Some(i_in_front) = does_part_a_occlude_part_b(
                    &parts_to_draw[i],
                    &parts_to_draw[j],
                    eye,
                    &self.cam_3d,
                    rect,
                ) {
                    if i_in_front {
                        // Part j is behind Part i -> Part j must be drawn BEFORE Part i (j -> i)
                        graph[j].push(i);
                        in_degree[i] += 1;
                    } else {
                        // Part i is behind Part j -> Part i must be drawn BEFORE Part j (i -> j)
                        graph[i].push(j);
                        in_degree[j] += 1;
                    }
                }
            }
        }

        // Ready queue holds parts with in_degree == 0.
        // Sorted ascending so pop() retrieves the part with largest max_depth (farthest from camera).
        let mut ready: Vec<usize> = (0..n_parts).filter(|&i| in_degree[i] == 0).collect();
        ready.sort_by(|&a, &b| {
            parts_to_draw[a]
                .max_depth
                .partial_cmp(&parts_to_draw[b].max_depth)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut sorted_parts = Vec::with_capacity(n_parts);
        let mut visited = vec![false; n_parts];

        while sorted_parts.len() < n_parts {
            if let Some(curr) = ready.pop() {
                if visited[curr] {
                    continue;
                }
                visited[curr] = true;
                sorted_parts.push(curr);

                for &dep in &graph[curr] {
                    if in_degree[dep] > 0 {
                        in_degree[dep] -= 1;
                        if in_degree[dep] == 0 && !visited[dep] {
                            ready.push(dep);
                            ready.sort_by(|&a, &b| {
                                parts_to_draw[a]
                                    .max_depth
                                    .partial_cmp(&parts_to_draw[b].max_depth)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            });
                        }
                    }
                }
            } else {
                // Break cycle if any: pick unvisited part with largest max_depth
                let mut best = None;
                let mut best_depth = f32::MIN;
                for i in 0..n_parts {
                    if !visited[i] && parts_to_draw[i].max_depth > best_depth {
                        best_depth = parts_to_draw[i].max_depth;
                        best = Some(i);
                    }
                }
                if let Some(i) = best {
                    visited[i] = true;
                    sorted_parts.push(i);
                    for &dep in &graph[i] {
                        if in_degree[dep] > 0 {
                            in_degree[dep] -= 1;
                            if in_degree[dep] == 0 && !visited[dep] {
                                ready.push(dep);
                            }
                        }
                    }
                } else {
                    break;
                }
            }
        }

        // 3. Render parts from back to front
        let mut all_drawn_faces: Vec<Face3DToDraw> = Vec::new();

        for &part_idx in &sorted_parts {
            let part_data = &parts_to_draw[part_idx];
            // Sort faces within part descending by view depth (farthest first)
            let mut faces = part_data.faces.clone();
            faces.sort_by(|a, b| {
                b.z_depth
                    .partial_cmp(&a.z_depth)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            for face in faces {
                let is_enhanced = self.cam_3d.z_order;
                if is_enhanced {
                    if let Some(tex) = loaded_textures.get(&face.material_id) {
                        self.draw_perspective_correct_textured_quad(
                            &painter,
                            rect,
                            &vp_mat,
                            tex.id(),
                            &face.world_verts,
                            &face.uv_corners,
                            face.screen_bbox,
                            face.fill_color,
                        );
                    } else {
                        draw_solid_polygon(&painter, &face.screen_pts, face.fill_color);
                    }
                } else {
                    // Normal mode: fast flat solid polygons with lighting, zero texture fetches
                    draw_solid_polygon(&painter, &face.screen_pts, face.fill_color);
                }

                if self.cam_3d.wireframe_overlay || face.is_selected {
                    for i in 0..4 {
                        if !face.outer_edges[i] {
                            continue;
                        }
                        let w1 = face.world_verts[i];
                        let w2 = face.world_verts[(i + 1) % 4];
                        if let Some((p1, p2)) =
                            self.project_clipped_segment_3d(&vp_mat, rect, eye, fwd, w1, w2)
                        {
                            let st = face.edge_strokes[i].unwrap_or_else(|| {
                                Stroke::new(
                                    if face.is_selected { 2.0 } else { 1.0 },
                                    face.stroke_color,
                                )
                            });
                            painter.line_segment([p1, p2], st);
                        }
                    }
                }

                if is_enhanced {
                    if let Some((gs_w, ge_w)) = face.grain_segment_3d {
                        if let Some((gs, ge)) =
                            self.project_clipped_segment_3d(&vp_mat, rect, eye, fwd, gs_w, ge_w)
                        {
                            let dist: f32 = gs.distance(ge);
                            if dist > 12.0 {
                                painter.line_segment(
                                    [gs, ge],
                                    Stroke::new(1.1, Color32::from_rgba_unmultiplied(60, 40, 20, 120)),
                                );
                            }
                        }
                    }
                }

                all_drawn_faces.push(face);
            }
        }

        // 4. Select part with LMB in 3D viewport (with Shift/Ctrl support for multi-selection):
        // - In Z-Noclip fly mode: LMB selects the part right under the central crosshair!
        // - In standard Orbit mode: LMB selects the part under the mouse cursor
        let modifiers = ui.input(|i| i.modifiers);
        let multi_mod = modifiers.shift || modifiers.ctrl || modifiers.command;

        if self.cam_3d.noclip_active {
            if ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary)) {
                let crosshair_pos = rect.center();
                let mut hit_part = None;
                for face in all_drawn_faces.iter().rev() {
                    if point_in_convex_polygon(crosshair_pos, &face.screen_pts) {
                        hit_part = Some(face.part_id);
                        break;
                    }
                }
                if let Some(hid) = hit_part {
                    if multi_mod {
                        self.toggle_part_in_selection(selected_part, hid);
                    } else {
                        self.set_single_selection(selected_part, Some(hid));
                    }
                } else if !multi_mod {
                    self.set_single_selection(selected_part, None);
                }
            }
        } else if resp.clicked_by(egui::PointerButton::Primary) {
            if let Some(click_pos) = ui.input(|i| i.pointer.interact_pos()) {
                let mut hit_part = None;
                for face in all_drawn_faces.iter().rev() {
                    if point_in_convex_polygon(click_pos, &face.screen_pts) {
                        hit_part = Some(face.part_id);
                        break;
                    }
                }
                if let Some(hid) = hit_part {
                    if multi_mod {
                        self.toggle_part_in_selection(selected_part, hid);
                    } else {
                        self.set_single_selection(selected_part, Some(hid));
                    }
                } else if !multi_mod {
                    self.set_single_selection(selected_part, None);
                }
            }
        }

        // Crosshair in the center of the 3D screen in first-person Z-Noclip mode
        if self.cam_3d.noclip_active {
            let c = rect.center();
            let ch_col = Color32::from_rgb(90, 255, 120);
            painter.line_segment(
                [Pos2::new(c.x - 10.0, c.y), Pos2::new(c.x + 10.0, c.y)],
                Stroke::new(1.5, ch_col),
            );
            painter.line_segment(
                [Pos2::new(c.x, c.y - 10.0), Pos2::new(c.x, c.y + 10.0)],
                Stroke::new(1.5, ch_col),
            );
        }

        // 5. Frame border and info header for 3D viewport
        let border_col = if self.cam_3d.noclip_active {
            Color32::from_rgb(80, 230, 110)
        } else {
            Color32::from_gray(55)
        };
        painter.rect_stroke(rect, 0, Stroke::new(if self.cam_3d.noclip_active { 2.0 } else { 1.0 }, border_col), egui::StrokeKind::Inside);

        let header_rect = Rect::from_min_size(
            rect.min + EguiVec2::new(6.0, 6.0),
            EguiVec2::new((rect.width() - 90.0).min(480.0), 22.0),
        );
        painter.rect_filled(
            header_rect,
            4.0,
            Color32::from_rgba_unmultiplied(28, 34, 42, 220),
        );
        let mode_text = if self.cam_3d.noclip_active {
            self.lang
                .t("3D VIEW [Z: FLY ON — WASD + Mouse | LMB — Pick at Crosshair | Z/Esc — Exit]")
                .to_string()
        } else {
            self.lang
                .t("3D VIEW [LMB — Select | RMB/MMB — Orbit | Press Z over pane for Fly mode]")
                .to_string()
        };
        painter.text(
            header_rect.left_center() + EguiVec2::new(8.0, 0.0),
            egui::Align2::LEFT_CENTER,
            mode_text,
            FontId::proportional(11.5),
            if self.cam_3d.noclip_active {
                Color32::from_rgb(110, 255, 140)
            } else {
                Color32::from_rgb(255, 215, 130)
            },
        );

        // Fullscreen maximize button for 3D viewport
        let btn_rect = Rect::from_min_size(
            Pos2::new(rect.right() - 78.0, rect.top() + 6.0),
            EguiVec2::new(72.0, 22.0),
        );
        let is_max = self.maximized == MaximizedPane::Perspective3D;
        let btn_resp = ui.interact(btn_rect, ui.scope_id().with("max_btn_3d"), Sense::click());
        painter.rect_filled(
            btn_rect,
            4.0,
            if btn_resp.hovered() {
                Color32::from_rgb(65, 90, 120)
            } else {
                Color32::from_rgba_unmultiplied(35, 45, 58, 220)
            },
        );
        painter.text(
            btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            if is_max {
                self.lang.t("[4 Views]")
            } else {
                self.lang.t("[Maximize]")
            },
            FontId::proportional(11.0),
            Color32::WHITE,
        );
        if btn_resp.clicked() {
            self.maximized = if is_max {
                MaximizedPane::None
            } else {
                MaximizedPane::Perspective3D
            };
        }

        // Bottom-right "Enhanced / Normal" quality toggle checkbox
        let zorder_rect = Rect::from_min_size(
            Pos2::new(rect.right() - 92.0, rect.bottom() - 28.0),
            EguiVec2::new(86.0, 22.0),
        );
        let z_resp = ui.interact(zorder_rect, ui.scope_id().with("hq_render_toggle_3d"), Sense::click());
        if z_resp.clicked() {
            self.cam_3d.z_order = !self.cam_3d.z_order;
        }
        let z_hovered = z_resp.hovered();
        painter.rect_filled(
            zorder_rect,
            4.0,
            if z_hovered {
                Color32::from_rgb(50, 65, 85)
            } else {
                Color32::from_rgba_unmultiplied(28, 34, 42, 220)
            },
        );
        painter.rect_stroke(
            zorder_rect,
            4,
            Stroke::new(
                1.0,
                if self.cam_3d.z_order {
                    Color32::from_rgb(80, 170, 255)
                } else {
                    Color32::from_gray(70)
                },
            ),
            egui::StrokeKind::Inside,
        );
        let box_size = 12.0;
        let box_rect = Rect::from_min_size(
            Pos2::new(zorder_rect.left() + 6.0, zorder_rect.center().y - box_size * 0.5),
            EguiVec2::new(box_size, box_size),
        );
        painter.rect_filled(
            box_rect,
            2.0,
            if self.cam_3d.z_order {
                Color32::from_rgb(60, 130, 246)
            } else {
                Color32::from_gray(40)
            },
        );
        painter.rect_stroke(box_rect, 2, Stroke::new(1.0, Color32::from_gray(120)), egui::StrokeKind::Inside);
        if self.cam_3d.z_order {
            painter.line_segment(
                [
                    Pos2::new(box_rect.left() + 2.5, box_rect.center().y),
                    Pos2::new(box_rect.center().x - 0.5, box_rect.bottom() - 3.0),
                ],
                Stroke::new(1.5, Color32::WHITE),
            );
            painter.line_segment(
                [
                    Pos2::new(box_rect.center().x - 0.5, box_rect.bottom() - 3.0),
                    Pos2::new(box_rect.right() - 2.5, box_rect.top() + 3.0),
                ],
                Stroke::new(1.5, Color32::WHITE),
            );
        }
        let label_text = if self.cam_3d.z_order {
            self.lang.t("viewport.render_enhanced")
        } else {
            self.lang.t("viewport.render_normal")
        };
        painter.text(
            Pos2::new(box_rect.right() + 6.0, zorder_rect.center().y),
            egui::Align2::LEFT_CENTER,
            label_text,
            FontId::proportional(11.0),
            if self.cam_3d.z_order {
                Color32::WHITE
            } else {
                Color32::from_gray(210)
            },
        );
        z_resp.on_hover_text(self.lang.t(
            "Display mode:\n• Enhanced: photorealistic textures with perspective correction and anti-aliasing.\n• Normal: fast flat solid shading without textures for weak GPUs and maximum FPS.",
        ));
    }

    /// Renders a textured 3D quadrilateral with perspective-correct 3D tessellation and symmetric center X-split.
    /// Completely eliminates PlayStation 1 affine texture warping in screen-space `egui::Mesh`.
    #[allow(clippy::too_many_arguments)]
    fn draw_perspective_correct_textured_quad(
        &self,
        painter: &Painter,
        rect: Rect,
        vp_mat: &Mat4,
        tex_id: egui::TextureId,
        w_verts: &[Vec3; 4],
        uv_corners: &[Pos2; 4],
        screen_bbox: Rect,
        tint_color: Color32,
    ) {
        // Choose perspective-correct grid density based on screen size and depth range W
        let mut min_w = f32::MAX;
        let mut max_w = f32::MIN;
        for v in w_verts {
            let clip = *vp_mat * v.extend(1.0);
            min_w = min_w.min(clip.w.max(NEAR_CLIP_MM));
            max_w = max_w.max(clip.w.max(NEAR_CLIP_MM));
        }
        let (sub_u, sub_v) = if self.cam_3d.z_order {
            let depth_ratio = (max_w / min_w.max(NEAR_CLIP_MM)).clamp(1.0, 4.0);
            (
                (((screen_bbox.width() * depth_ratio) / 26.0).ceil() as usize).clamp(3, 8),
                (((screen_bbox.height() * depth_ratio) / 26.0).ceil() as usize).clamp(3, 8),
            )
        } else {
            // Fast mode for lower-end hardware: single quad per face (2 triangles)
            (1, 1)
        };

        let bilerp_3d = |s: f32, t: f32| -> Vec3 {
            let top = w_verts[0].lerp(w_verts[1], s);
            let bot = w_verts[3].lerp(w_verts[2], s);
            top.lerp(bot, t)
        };
        let bilerp_uv = |s: f32, t: f32| -> Pos2 {
            let top_x = uv_corners[0].x + (uv_corners[1].x - uv_corners[0].x) * s;
            let top_y = uv_corners[0].y + (uv_corners[1].y - uv_corners[0].y) * s;
            let bot_x = uv_corners[3].x + (uv_corners[2].x - uv_corners[3].x) * s;
            let bot_y = uv_corners[3].y + (uv_corners[2].y - uv_corners[3].y) * s;
            Pos2::new(top_x + (bot_x - top_x) * t, top_y + (bot_y - top_y) * t)
        };

        let eye = self.cam_3d.eye_mm;
        let fwd = self.cam_3d.forward_dir();

        // For each micro-cell, clip against near-plane and build triangles
        let mut mesh = egui::Mesh::with_texture(tex_id);

        for iv in 0..sub_v {
            let t0 = iv as f32 / sub_v as f32;
            let t1 = (iv + 1) as f32 / sub_v as f32;
            let t_mid = (iv as f32 + 0.5) / sub_v as f32;

            for iu in 0..sub_u {
                let s0 = iu as f32 / sub_u as f32;
                let s1 = (iu + 1) as f32 / sub_u as f32;
                let s_mid = (iu as f32 + 0.5) / sub_u as f32;

                let cell_3d = [
                    bilerp_3d(s0, t0),
                    bilerp_3d(s1, t0),
                    bilerp_3d(s1, t1),
                    bilerp_3d(s0, t1),
                ];
                let cell_uv = [
                    bilerp_uv(s0, t0),
                    bilerp_uv(s1, t0),
                    bilerp_uv(s1, t1),
                    bilerp_uv(s0, t1),
                ];

                // Check how many vertices of this micro-cell are in front of near-plane
                let in_front = cell_3d
                    .iter()
                    .all(|&p| (p - eye).dot(fwd) >= NEAR_CLIP_MM);

                if in_front {
                    let mut scr = [Pos2::ZERO; 4];
                    let mut ok = true;
                    for k in 0..4 {
                        if let Some((p, _)) = self.cam_3d.project_point(vp_mat, rect, cell_3d[k]) {
                            scr[k] = p;
                        } else {
                            ok = false;
                            break;
                        }
                    }
                    let p_mid_3d = bilerp_3d(s_mid, t_mid);
                    let uv_mid = bilerp_uv(s_mid, t_mid);
                    let scr_mid = self.cam_3d.project_point(vp_mat, rect, p_mid_3d);

                    if ok {
                        if let Some((sm, _)) = scr_mid {
                            let base_idx = mesh.vertices.len() as u32;
                            mesh.vertices.push(egui::epaint::Vertex {
                                pos: scr[0],
                                uv: cell_uv[0],
                                color: tint_color,
                            });
                            mesh.vertices.push(egui::epaint::Vertex {
                                pos: scr[1],
                                uv: cell_uv[1],
                                color: tint_color,
                            });
                            mesh.vertices.push(egui::epaint::Vertex {
                                pos: scr[2],
                                uv: cell_uv[2],
                                color: tint_color,
                            });
                            mesh.vertices.push(egui::epaint::Vertex {
                                pos: scr[3],
                                uv: cell_uv[3],
                                color: tint_color,
                            });
                            mesh.vertices.push(egui::epaint::Vertex {
                                pos: sm,
                                uv: uv_mid,
                                color: tint_color,
                            });

                            mesh.add_triangle(base_idx, base_idx + 1, base_idx + 4);
                            mesh.add_triangle(base_idx + 1, base_idx + 2, base_idx + 4);
                            mesh.add_triangle(base_idx + 2, base_idx + 3, base_idx + 4);
                            mesh.add_triangle(base_idx + 3, base_idx, base_idx + 4);
                            continue;
                        }
                    }
                }

                // If cell intersects near-plane, clip the cell polygon and interpolate UVs
                let mut poly_3d_uv = Vec::with_capacity(6);
                let dist = |p: Vec3| -> f32 { (p - eye).dot(fwd) - NEAR_CLIP_MM };
                for k in 0..4 {
                    let cur_p = cell_3d[k];
                    let cur_uv = cell_uv[k];
                    let prev_idx = if k == 0 { 3 } else { k - 1 };
                    let prev_p = cell_3d[prev_idx];
                    let prev_uv = cell_uv[prev_idx];

                    let d_cur = dist(cur_p);
                    let d_prev = dist(prev_p);

                    if d_cur >= 0.0 {
                        if d_prev < 0.0 {
                            let t = d_prev / (d_prev - d_cur);
                            let inter_p = prev_p.lerp(cur_p, t);
                            let inter_uv = Pos2::new(
                                prev_uv.x + t * (cur_uv.x - prev_uv.x),
                                prev_uv.y + t * (cur_uv.y - prev_uv.y),
                            );
                            poly_3d_uv.push((inter_p, inter_uv));
                        }
                        poly_3d_uv.push((cur_p, cur_uv));
                    } else if d_prev >= 0.0 {
                        let t = d_prev / (d_prev - d_cur);
                        let inter_p = prev_p.lerp(cur_p, t);
                        let inter_uv = Pos2::new(
                            prev_uv.x + t * (cur_uv.x - prev_uv.x),
                            prev_uv.y + t * (cur_uv.y - prev_uv.y),
                        );
                        poly_3d_uv.push((inter_p, inter_uv));
                    }
                }

                if poly_3d_uv.len() >= 3 {
                    let mut scr_uv = Vec::with_capacity(poly_3d_uv.len());
                    let mut ok = true;
                    for (p3d, uv) in &poly_3d_uv {
                        if let Some((sp, _)) = self.cam_3d.project_point(vp_mat, rect, *p3d) {
                            scr_uv.push((sp, *uv));
                        } else {
                            ok = false;
                            break;
                        }
                    }
                    if ok && scr_uv.len() >= 3 {
                        let base_idx = mesh.vertices.len() as u32;
                        for (pos, uv) in &scr_uv {
                            mesh.vertices.push(egui::epaint::Vertex {
                                pos: *pos,
                                uv: *uv,
                                color: tint_color,
                            });
                        }
                        for i in 1..(scr_uv.len() - 1) {
                            mesh.add_triangle(base_idx, base_idx + i as u32, base_idx + (i + 1) as u32);
                        }
                    }
                }
            }
        }

        if !mesh.is_empty() {
            painter.add(Shape::mesh(mesh));
        }
    }

    fn draw_3d_floor_grid(&self, painter: &Painter, rect: Rect, vp_mat: &Mat4) {
        let step: f32 = 100.0;
        let min_c: f32 = -200.0;
        let max_c: f32 = 1200.0;

        let mut c: f32 = min_c;
        while c <= max_c {
            let col = if c.abs() < 0.1 {
                Color32::from_rgb(70, 80, 95)
            } else {
                Color32::from_rgb(35, 42, 50)
            };
            if let (Some((p1, _)), Some((p2, _))) = (
                self.cam_3d.project_point(vp_mat, rect, Vec3::new(c, min_c, 0.0)),
                self.cam_3d.project_point(vp_mat, rect, Vec3::new(c, max_c, 0.0)),
            ) {
                painter.line_segment([p1, p2], Stroke::new(1.0, col));
            }
            if let (Some((p1, _)), Some((p2, _))) = (
                self.cam_3d.project_point(vp_mat, rect, Vec3::new(min_c, c, 0.0)),
                self.cam_3d.project_point(vp_mat, rect, Vec3::new(max_c, c, 0.0)),
            ) {
                painter.line_segment([p1, p2], Stroke::new(1.0, col));
            }
            c += step;
        }

        // Axes X (red), Y (green), Z (blue) from (0,0,0) with length 250 mm
        if let Some((o, _)) = self.cam_3d.project_point(vp_mat, rect, Vec3::ZERO) {
            if let Some((px, _)) = self.cam_3d.project_point(vp_mat, rect, Vec3::new(250.0, 0.0, 0.0)) {
                painter.line_segment([o, px], Stroke::new(2.2, Color32::from_rgb(230, 70, 70)));
            }
            if let Some((py, _)) = self.cam_3d.project_point(vp_mat, rect, Vec3::new(0.0, 250.0, 0.0)) {
                painter.line_segment([o, py], Stroke::new(2.2, Color32::from_rgb(70, 210, 90)));
            }
            if let Some((pz, _)) = self.cam_3d.project_point(vp_mat, rect, Vec3::new(0.0, 0.0, 250.0)) {
                painter.line_segment([o, pz], Stroke::new(2.2, Color32::from_rgb(80, 150, 250)));
            }
        }
    }

    /// Clips a 3D line segment against the camera near plane and projects it to 2D screen coordinates.
    pub fn project_clipped_segment_3d(
        &self,
        vp_mat: &Mat4,
        rect: Rect,
        eye: Vec3,
        fwd: Vec3,
        mut p1: Vec3,
        mut p2: Vec3,
    ) -> Option<(Pos2, Pos2)> {
        let d1 = (p1 - eye).dot(fwd) - NEAR_CLIP_MM;
        let d2 = (p2 - eye).dot(fwd) - NEAR_CLIP_MM;
        if d1 < 0.0 && d2 < 0.0 {
            return None;
        }
        if d1 < 0.0 {
            let t = d1 / (d1 - d2);
            p1 = p1.lerp(p2, t);
        } else if d2 < 0.0 {
            let t = d2 / (d2 - d1);
            p2 = p2.lerp(p1, t);
        }
        let (s1, _) = self.cam_3d.project_point(vp_mat, rect, p1)?;
        let (s2, _) = self.cam_3d.project_point(vp_mat, rect, p2)?;
        Some((s1, s2))
    }

    /// Checks if face `f_idx` (0:-Z, 1:+Z, 2:-Y, 3:+Y, 4:-X, 5:+X) matches one of the 4 edge-banded sides of the part
    fn face_edge_id_for_part(&self, part: &PartBlock, f_idx: usize) -> Option<EdgeId> {
        match part.orientation {
            PartOrientation::HorizontalXY => match f_idx {
                3 => part.edges.l1, // Front (+Y) = L1
                2 => part.edges.l2, // Back (-Y) = L2
                4 => part.edges.w1, // Left (-X) = W1
                5 => part.edges.w2, // Right (+X) = W2
                _ => None,
            },
            PartOrientation::VerticalYZ => match f_idx {
                3 => part.edges.l1, // Front (+Y) = L1
                2 => part.edges.l2, // Back (-Y) = L2
                0 => part.edges.w1, // Bottom (-Z) = W1
                1 => part.edges.w2, // Top (+Z) = W2
                _ => None,
            },
            PartOrientation::FrontalXZ => match f_idx {
                4 => part.edges.l1, // Left (-X) = L1
                5 => part.edges.l2, // Right (+X) = L2
                0 => part.edges.w1, // Bottom (-Z) = W1
                1 => part.edges.w2, // Top (+Z) = W2
                _ => None,
            },
        }
    }

    fn is_main_face(&self, orientation: PartOrientation, f_idx: usize) -> bool {
        match orientation {
            PartOrientation::HorizontalXY => f_idx == 0 || f_idx == 1,
            PartOrientation::VerticalYZ => f_idx == 4 || f_idx == 5,
            PartOrientation::FrontalXZ => f_idx == 2 || f_idx == 3,
        }
    }

    /// Manages grab and visibility of the system cursor in first-person Z-Noclip mode
    pub fn set_noclip_cursor_grab(ctx: &egui::Context, active: bool, center_pos: Pos2) {
        crate::web::set_web_noclip_active(active);
        if active {
            ctx.send_viewport_cmd(egui::ViewportCommand::CursorGrab(
                egui::CursorGrab::Locked,
            ));
            ctx.send_viewport_cmd(egui::ViewportCommand::CursorVisible(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::CursorPosition(center_pos));
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::CursorGrab(
                egui::CursorGrab::None,
            ));
            ctx.send_viewport_cmd(egui::ViewportCommand::CursorVisible(true));
        }
    }
}

/// Clips a 3D convex polygon against the camera near plane `(P - eye) . fwd >= near_d`
/// using the Sutherland-Hodgman polygon clipping algorithm.
fn clip_3d_polygon_near_plane(verts: &[Vec3], eye: Vec3, fwd: Vec3, near_d: f32) -> Vec<Vec3> {
    if verts.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(verts.len() + 2);
    let dist = |p: Vec3| -> f32 { (p - eye).dot(fwd) - near_d };

    for i in 0..verts.len() {
        let cur = verts[i];
        let prev = if i == 0 { verts[verts.len() - 1] } else { verts[i - 1] };
        let d_cur = dist(cur);
        let d_prev = dist(prev);

        if d_cur >= 0.0 {
            if d_prev < 0.0 {
                let t = d_prev / (d_prev - d_cur);
                out.push(prev.lerp(cur, t));
            }
            out.push(cur);
        } else if d_prev >= 0.0 {
            let t = d_prev / (d_prev - d_cur);
            out.push(prev.lerp(cur, t));
        }
    }
    out
}

/// Renders a solid 2D polygon with a triangle fan using `Mesh::default()`.
/// Avoids `Shape::convex_polygon` whose stroke feathering can explode into spikes.
fn draw_solid_polygon(painter: &Painter, pts: &[Pos2], color: Color32) {
    if pts.len() < 3 {
        return;
    }
    let mut mesh = egui::Mesh::default();
    mesh.reserve_vertices(pts.len());
    mesh.reserve_triangles(pts.len() - 2);

    for &pos in pts {
        mesh.vertices.push(egui::epaint::Vertex {
            pos,
            uv: Pos2::ZERO,
            color,
        });
    }
    for i in 1..(pts.len() - 1) {
        mesh.add_triangle(0, i as u32, (i + 1) as u32);
    }
    painter.add(Shape::mesh(mesh));
}

/// Checks if point `pt` is inside a 2D convex polygon
fn point_in_convex_polygon(pt: Pos2, poly: &[Pos2]) -> bool {
    if poly.len() < 3 {
        return false;
    }
    let mut has_pos = false;
    let mut has_neg = false;
    for i in 0..poly.len() {
        let a = poly[i];
        let b = poly[(i + 1) % poly.len()];
        let cross = (b.x - a.x) * (pt.y - a.y) - (b.y - a.y) * (pt.x - a.x);
        if cross > 1e-4 {
            has_pos = true;
        } else if cross < -1e-4 {
            has_neg = true;
        }
        if has_pos && has_neg {
            return false;
        }
    }
    true
}

#[derive(Clone)]
pub struct Face3DToDraw {
    pub part_id: PartId,
    pub material_id: MaterialId,
    pub world_verts: [Vec3; 4],
    pub screen_pts: Vec<Pos2>,
    pub screen_bbox: Rect,
    pub z_depth: f32,
    pub fill_color: Color32,
    pub stroke_color: Color32,
    pub is_selected: bool,
    pub uv_corners: [Pos2; 4],
    pub outer_edges: [bool; 4],
    pub edge_strokes: [Option<Stroke>; 4],
    pub grain_segment_3d: Option<(Vec3, Vec3)>,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct PartDrawData {
    #[allow(dead_code)]
    pub part_id: PartId,
    pub bmin: Vec3,
    pub bmax: Vec3,
    pub yaw_deg: f32,
    pub screen_bbox: Rect,
    pub max_depth: f32,
    pub faces: Vec<Face3DToDraw>,
}

/// Tests whether box A is separated from box B along a single axis, and if so,
/// whether box A is on the camera's side of the separating plane (closer to eye).
/// Returns `Some(true)` if A is closer to eye (A in front of B).
/// Returns `Some(false)` if B is closer to eye (B in front of A).
/// Returns `None` if not separated along this axis or camera is between the boxes.
#[allow(dead_code)]
pub fn test_axis_separation(
    min_a: f32,
    max_a: f32,
    min_b: f32,
    max_b: f32,
    eye_c: f32,
    eps: f32,
) -> Option<bool> {
    if max_a <= min_b + eps {
        // A is below/left/behind B along this axis
        if eye_c <= max_a {
            Some(true)
        } else if eye_c >= min_b {
            Some(false)
        } else {
            None
        }
    } else if max_b <= min_a + eps {
        // B is below/left/behind A along this axis
        if eye_c <= max_b {
            Some(false)
        } else if eye_c >= min_a {
            Some(true)
        } else {
            None
        }
    } else {
        None
    }
}

/// Computes the intersection distance `t >= 0` along ray `eye + t * dir` into an AABB.
/// Returns `None` if the ray misses the AABB or the box is behind the ray origin.
#[allow(dead_code)]
pub fn ray_aabb_distance(eye: Vec3, dir: Vec3, bmin: Vec3, bmax: Vec3) -> Option<f32> {
    let mut tmin = f32::MIN;
    let mut tmax = f32::MAX;
    for i in 0..3 {
        let (e, d, min_v, max_v) = match i {
            0 => (eye.x, dir.x, bmin.x, bmax.x),
            1 => (eye.y, dir.y, bmin.y, bmax.y),
            _ => (eye.z, dir.z, bmin.z, bmax.z),
        };
        if d.abs() > 1e-6 {
            let mut t1 = (min_v - e) / d;
            let mut t2 = (max_v - e) / d;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmin > tmax {
                return None;
            }
        } else if e < min_v || e > max_v {
            return None;
        }
    }
    if tmax < 0.0 {
        return None;
    }
    Some(tmin.max(0.0))
}

/// Computes the intersection distance `t >= 0` along ray `eye + t * dir` into a planar 3D quad.
/// Returns `None` if the ray is parallel or the intersection point is behind the ray origin.
pub fn ray_quad_distance(eye: Vec3, dir: Vec3, quad: &[Vec3; 4]) -> Option<f32> {
    let edge1 = quad[1] - quad[0];
    let edge2 = quad[3] - quad[0];
    let norm = edge1.cross(edge2);
    let denom = norm.dot(dir);
    if denom.abs() < 1e-6 {
        return None;
    }
    let t = (quad[0] - eye).dot(norm) / denom;
    if t < 0.0 {
        return None;
    }
    Some(t)
}

/// Determines whether Part A occludes Part B on screen.
/// Returns:
/// - `Some(true)` if Part A is in front of Part B (Part A occludes Part B, so B must be drawn before A).
/// - `Some(false)` if Part B is in front of Part A (Part B occludes Part A, so A must be drawn before B).
/// - `None` if the parts do not occlude each other on screen (no dependency needed).
pub fn does_part_a_occlude_part_b(
    a: &PartDrawData,
    b: &PartDrawData,
    eye: Vec3,
    cam_3d: &Camera3D,
    rect: Rect,
) -> Option<bool> {
    // 1. If screen bounding boxes don't overlap by at least 1.5 pixels, they cannot occlude each other
    let overlap = a.screen_bbox.intersect(b.screen_bbox);
    if overlap.width() < 1.5 || overlap.height() < 1.5 {
        return None;
    }

    // 2. Test visible faces of Part A against visible faces of Part B using the exact
    // 2D convex intersection polygon of their screen projections.
    for fa in &a.faces {
        for fb in &b.faces {
            let f_overlap = fa.screen_bbox.intersect(fb.screen_bbox);
            if f_overlap.width() < 1.0 || f_overlap.height() < 1.0 {
                continue;
            }

            let inter = clip_convex_polygon(&fa.screen_pts, &fb.screen_pts);
            if inter.len() < 3 || polygon_area_abs(&inter) < 2.0 {
                // Faces only touch along an edge or a corner -> no real occlusion
                continue;
            }

            // Sample the centroid (always strictly inside a convex polygon) plus points pulled
            // from each vertex towards the centroid, so the ray hits both faces' interiors.
            let mut centroid = Pos2::ZERO;
            for p in &inter {
                centroid.x += p.x;
                centroid.y += p.y;
            }
            centroid.x /= inter.len() as f32;
            centroid.y /= inter.len() as f32;

            let mut sample_pts = Vec::with_capacity(inter.len() + 1);
            sample_pts.push(centroid);
            for p in &inter {
                sample_pts.push(centroid.lerp(*p, 0.6));
            }

            for pt in sample_pts {
                let ray_dir = cam_3d.ray_from_screen_point(pt, rect);
                let dist_a = ray_quad_distance(eye, ray_dir, &fa.world_verts);
                let dist_b = ray_quad_distance(eye, ray_dir, &fb.world_verts);
                if let (Some(da), Some(db)) = (dist_a, dist_b) {
                    if (da - db).abs() > 0.05 {
                        return Some(da < db);
                    }
                }
            }
        }
    }

    // No true geometric occlusion on screen -> no dependency edge needed!
    None
}

/// Signed-area-agnostic absolute area of a 2D polygon (shoelace formula).
fn polygon_area_abs(pts: &[Pos2]) -> f32 {
    let mut s = 0.0;
    for i in 0..pts.len() {
        let a = pts[i];
        let b = pts[(i + 1) % pts.len()];
        s += a.x * b.y - b.x * a.y;
    }
    (s * 0.5).abs()
}

/// Intersects two convex 2D polygons (Sutherland-Hodgman, `subject` clipped by `clip`).
/// Works for either winding order of `clip`.
pub fn clip_convex_polygon(subject: &[Pos2], clip: &[Pos2]) -> Vec<Pos2> {
    if subject.len() < 3 || clip.len() < 3 {
        return Vec::new();
    }
    // Orientation of the clip polygon: +1 for CCW (in screen coords), -1 for CW
    let mut signed = 0.0;
    for i in 0..clip.len() {
        let a = clip[i];
        let b = clip[(i + 1) % clip.len()];
        signed += a.x * b.y - b.x * a.y;
    }
    if signed.abs() < 1e-6 {
        return Vec::new();
    }
    let orient = signed.signum();

    let side = |a: Pos2, b: Pos2, p: Pos2| -> f32 {
        ((b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)) * orient
    };

    let mut output = subject.to_vec();
    for i in 0..clip.len() {
        if output.len() < 3 {
            return Vec::new();
        }
        let ca = clip[i];
        let cb = clip[(i + 1) % clip.len()];
        let input = std::mem::take(&mut output);
        for j in 0..input.len() {
            let cur = input[j];
            let prev = input[(j + input.len() - 1) % input.len()];
            let s_cur = side(ca, cb, cur);
            let s_prev = side(ca, cb, prev);
            if s_cur >= 0.0 {
                if s_prev < 0.0 {
                    let t = s_prev / (s_prev - s_cur);
                    output.push(prev.lerp(cur, t));
                }
                output.push(cur);
            } else if s_prev >= 0.0 {
                let t = s_prev / (s_prev - s_cur);
                output.push(prev.lerp(cur, t));
            }
        }
    }
    output
}

/// Clips a 2D convex polygon against an axis-aligned `Rect` using Sutherland-Hodgman clipping.
pub fn clip_2d_polygon_rect(pts: &[Pos2], rect: Rect) -> Vec<Pos2> {
    if pts.len() < 3 {
        return Vec::new();
    }
    let mut poly = pts.to_vec();
    poly = clip_2d_polygon_axis(&poly, rect.min.x, true, true);
    if poly.len() < 3 {
        return Vec::new();
    }
    poly = clip_2d_polygon_axis(&poly, rect.max.x, false, true);
    if poly.len() < 3 {
        return Vec::new();
    }
    poly = clip_2d_polygon_axis(&poly, rect.min.y, true, false);
    if poly.len() < 3 {
        return Vec::new();
    }
    poly = clip_2d_polygon_axis(&poly, rect.max.y, false, false);
    poly
}

fn clip_2d_polygon_axis(pts: &[Pos2], bound: f32, is_min: bool, is_x: bool) -> Vec<Pos2> {
    let mut out = Vec::with_capacity(pts.len() + 2);
    let is_inside = |p: Pos2| -> bool {
        let v = if is_x { p.x } else { p.y };
        if is_min {
            v >= bound
        } else {
            v <= bound
        }
    };
    for i in 0..pts.len() {
        let cur = pts[i];
        let prev = if i == 0 {
            pts[pts.len() - 1]
        } else {
            pts[i - 1]
        };
        let in_cur = is_inside(cur);
        let in_prev = is_inside(prev);

        if in_cur {
            if !in_prev {
                let (v_prev, v_cur) = if is_x { (prev.x, cur.x) } else { (prev.y, cur.y) };
                let denom = v_cur - v_prev;
                let t = if denom.abs() > 1e-6 {
                    (bound - v_prev) / denom
                } else {
                    0.5
                };
                out.push(prev.lerp(cur, t.clamp(0.0, 1.0)));
            }
            out.push(cur);
        } else if in_prev {
            let (v_prev, v_cur) = if is_x { (prev.x, cur.x) } else { (prev.y, cur.y) };
            let denom = v_cur - v_prev;
            let t = if denom.abs() > 1e-6 {
                (bound - v_prev) / denom
            } else {
                0.5
            };
            out.push(prev.lerp(cur, t.clamp(0.0, 1.0)));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_arrow_nudge_delta_top_xy() {
        // TopXY: U is X, V is Y
        let left = QuadViewportController::compute_arrow_nudge_delta(
            OrthoAxis::TopXY,
            true,
            false,
            false,
            false,
            5.0,
            false,
        );
        assert_eq!(left, Some(Vec3::new(-5.0, 0.0, 0.0)));

        let right = QuadViewportController::compute_arrow_nudge_delta(
            OrthoAxis::TopXY,
            false,
            true,
            false,
            false,
            5.0,
            false,
        );
        assert_eq!(right, Some(Vec3::new(5.0, 0.0, 0.0)));

        let up = QuadViewportController::compute_arrow_nudge_delta(
            OrthoAxis::TopXY,
            false,
            false,
            true,
            false,
            5.0,
            false,
        );
        assert_eq!(up, Some(Vec3::new(0.0, 5.0, 0.0)));

        let down = QuadViewportController::compute_arrow_nudge_delta(
            OrthoAxis::TopXY,
            false,
            false,
            false,
            true,
            5.0,
            false,
        );
        assert_eq!(down, Some(Vec3::new(0.0, -5.0, 0.0)));

        // With Shift (step doubled: 5.0 * 2.0 = 10.0)
        let right_shift = QuadViewportController::compute_arrow_nudge_delta(
            OrthoAxis::TopXY,
            false,
            true,
            false,
            false,
            5.0,
            true,
        );
        assert_eq!(right_shift, Some(Vec3::new(10.0, 0.0, 0.0)));

        let up_shift = QuadViewportController::compute_arrow_nudge_delta(
            OrthoAxis::TopXY,
            false,
            false,
            true,
            false,
            5.0,
            true,
        );
        assert_eq!(up_shift, Some(Vec3::new(0.0, 10.0, 0.0)));
    }

    #[test]
    fn test_compute_arrow_nudge_delta_front_xz() {
        // FrontXZ: U is X, V is Z
        let left = QuadViewportController::compute_arrow_nudge_delta(
            OrthoAxis::FrontXZ,
            true,
            false,
            false,
            false,
            18.0,
            false,
        );
        assert_eq!(left, Some(Vec3::new(-18.0, 0.0, 0.0)));

        let right = QuadViewportController::compute_arrow_nudge_delta(
            OrthoAxis::FrontXZ,
            false,
            true,
            false,
            false,
            18.0,
            false,
        );
        assert_eq!(right, Some(Vec3::new(18.0, 0.0, 0.0)));

        let up = QuadViewportController::compute_arrow_nudge_delta(
            OrthoAxis::FrontXZ,
            false,
            false,
            true,
            false,
            18.0,
            false,
        );
        assert_eq!(up, Some(Vec3::new(0.0, 0.0, 18.0)));

        let down = QuadViewportController::compute_arrow_nudge_delta(
            OrthoAxis::FrontXZ,
            false,
            false,
            false,
            true,
            18.0,
            false,
        );
        assert_eq!(down, Some(Vec3::new(0.0, 0.0, -18.0)));

        // With Shift (step doubled: 18.0 * 2.0 = 36.0)
        let up_shift = QuadViewportController::compute_arrow_nudge_delta(
            OrthoAxis::FrontXZ,
            false,
            false,
            true,
            false,
            18.0,
            true,
        );
        assert_eq!(up_shift, Some(Vec3::new(0.0, 0.0, 36.0)));
    }

    #[test]
    fn test_compute_arrow_nudge_delta_side_yz() {
        // SideYZ: U is Y, V is Z
        let left = QuadViewportController::compute_arrow_nudge_delta(
            OrthoAxis::SideYZ,
            true,
            false,
            false,
            false,
            32.0,
            false,
        );
        assert_eq!(left, Some(Vec3::new(0.0, -32.0, 0.0)));

        let right = QuadViewportController::compute_arrow_nudge_delta(
            OrthoAxis::SideYZ,
            false,
            true,
            false,
            false,
            32.0,
            false,
        );
        assert_eq!(right, Some(Vec3::new(0.0, 32.0, 0.0)));

        let up = QuadViewportController::compute_arrow_nudge_delta(
            OrthoAxis::SideYZ,
            false,
            false,
            true,
            false,
            32.0,
            false,
        );
        assert_eq!(up, Some(Vec3::new(0.0, 0.0, 32.0)));

        let down = QuadViewportController::compute_arrow_nudge_delta(
            OrthoAxis::SideYZ,
            false,
            false,
            false,
            true,
            32.0,
            false,
        );
        assert_eq!(down, Some(Vec3::new(0.0, 0.0, -32.0)));

        // With Shift (step doubled: 32.0 * 2.0 = 64.0)
        let down_shift = QuadViewportController::compute_arrow_nudge_delta(
            OrthoAxis::SideYZ,
            false,
            false,
            false,
            true,
            32.0,
            true,
        );
        assert_eq!(down_shift, Some(Vec3::new(0.0, 0.0, -64.0)));
    }

    #[test]
    fn test_compute_arrow_nudge_delta_none_when_no_keys() {
        let none = QuadViewportController::compute_arrow_nudge_delta(
            OrthoAxis::TopXY,
            false,
            false,
            false,
            false,
            5.0,
            false,
        );
        assert_eq!(none, None);
    }

    #[test]
    fn test_part_view_mode_2d_allowed_handles() {
        let mut p = PartBlock {
            id: 1,
            name: "Test".to_string(),
            group_id: None,
            material_id: 1,
            orientation: PartOrientation::HorizontalXY,
            position_mm: Vec3::ZERO,
            length_mm: 800.0,
            width_mm: 500.0,
            yaw_deg: 0.0,
            grain_fixed: false,
            grain_along_width: false,
            edges: PartEdges::default(),
            visible: true,
            locked: false,
            drill_holes: Vec::new(),
        };
        let thick = 18.0;

        // 1. HorizontalXY:
        // TopXY: Face-on -> 8 handles
        assert_eq!(part_view_mode_2d(OrthoAxis::TopXY, &p, thick), PartViewMode2D::FaceOn);
        // FrontXZ: Edge-on (vertical thickness Z) -> Only left & right handles
        assert_eq!(part_view_mode_2d(OrthoAxis::FrontXZ, &p, thick), PartViewMode2D::EdgeOnVerticalThickness);
        // SideYZ: Edge-on (vertical thickness Z) -> Only left & right handles
        assert_eq!(part_view_mode_2d(OrthoAxis::SideYZ, &p, thick), PartViewMode2D::EdgeOnVerticalThickness);

        // 2. VerticalYZ:
        p.orientation = PartOrientation::VerticalYZ;
        // SideYZ: Face-on -> 8 handles
        assert_eq!(part_view_mode_2d(OrthoAxis::SideYZ, &p, thick), PartViewMode2D::FaceOn);
        // TopXY: Edge-on (horizontal thickness X) -> Only top & bottom handles
        assert_eq!(part_view_mode_2d(OrthoAxis::TopXY, &p, thick), PartViewMode2D::EdgeOnHorizontalThickness);
        // FrontXZ: Edge-on (horizontal thickness X) -> Only top & bottom handles
        assert_eq!(part_view_mode_2d(OrthoAxis::FrontXZ, &p, thick), PartViewMode2D::EdgeOnHorizontalThickness);

        // 3. FrontalXZ:
        p.orientation = PartOrientation::FrontalXZ;
        // FrontXZ: Face-on -> 8 handles
        assert_eq!(part_view_mode_2d(OrthoAxis::FrontXZ, &p, thick), PartViewMode2D::FaceOn);
        // TopXY: Edge-on (vertical thickness Y) -> Only left & right handles
        assert_eq!(part_view_mode_2d(OrthoAxis::TopXY, &p, thick), PartViewMode2D::EdgeOnVerticalThickness);
        // SideYZ: Edge-on (horizontal thickness Y) -> Only top & bottom handles
        assert_eq!(part_view_mode_2d(OrthoAxis::SideYZ, &p, thick), PartViewMode2D::EdgeOnHorizontalThickness);

        // Verify handle counts from get_allowed_handles_2d:
        let dummy_r = Rect::from_min_size(Pos2::ZERO, EguiVec2::new(100.0, 100.0));
        assert_eq!(get_allowed_handles_2d(dummy_r, PartViewMode2D::FaceOn).len(), 8);
        assert_eq!(get_allowed_handles_2d(dummy_r, PartViewMode2D::EdgeOnHorizontalThickness).len(), 2);
        assert_eq!(get_allowed_handles_2d(dummy_r, PartViewMode2D::EdgeOnVerticalThickness).len(), 2);
    }

    #[test]
    fn test_shift_drag_axis_constraint() {
        // Shift axis constraint: major displacement preserves movement axis, minor displacement is zeroed out
        let constrain = |mut delta: Vec2, shift_held: bool| -> Vec2 {
            if shift_held {
                if delta.x.abs() >= delta.y.abs() {
                    delta.y = 0.0;
                } else {
                    delta.x = 0.0;
                }
            }
            delta
        };

        // Horizontal dominant
        assert_eq!(constrain(Vec2::new(45.0, 12.0), true), Vec2::new(45.0, 0.0));
        assert_eq!(constrain(Vec2::new(-30.0, -10.0), true), Vec2::new(-30.0, 0.0));

        // Vertical dominant
        assert_eq!(constrain(Vec2::new(15.0, 60.0), true), Vec2::new(0.0, 60.0));
        assert_eq!(constrain(Vec2::new(-5.0, -40.0), true), Vec2::new(0.0, -40.0));

        // Equal displacement -> defaults to horizontal
        assert_eq!(constrain(Vec2::new(20.0, 20.0), true), Vec2::new(20.0, 0.0));

        // Shift released -> free movement preserved
        assert_eq!(constrain(Vec2::new(35.0, 22.0), false), Vec2::new(35.0, 22.0));
    }

    #[test]
    fn test_ctrl_cmd_bypass_edge_snapping() {
        let candidates = [100.0, 200.0, 300.0];
        let snap_thresh = 20.0;

        // Without Ctrl/Cmd: snaps to candidate at 105.0 -> 100.0
        let normal_snap = snap_1d_edge(105.0, &candidates, snap_thresh, true, false, 10.0);
        assert_eq!(normal_snap.snapped_pos, 100.0);
        assert_eq!(normal_snap.guide_line_pos, Some(100.0));

        // With Ctrl/Cmd: edge snap bypassed, stays at exact floating coordinate 105.0
        let ctrl_bypass = snap_1d_edge(105.0, &candidates, snap_thresh, false, false, 10.0);
        assert_eq!(ctrl_bypass.snapped_pos, 105.0);
        assert_eq!(ctrl_bypass.guide_line_pos, None);
    }

    #[test]
    fn test_inner_stroke_no_overlap_between_abutting_parts() {
        // Two parts sharing seam at x = 782.0
        let part_a = Rect::from_min_max(Pos2::new(18.0, 350.0), Pos2::new(782.0, 368.0));
        let part_b = Rect::from_min_max(Pos2::new(782.0, 0.0), Pos2::new(800.0, 720.0));

        let stroke_w = 1.0;
        let stroke_a = part_a.shrink(stroke_w * 0.5);
        let stroke_b = part_b.shrink(stroke_w * 0.5);

        // Outer bounds of stroke for part A: right edge is at 782.0 - 0.5 + 0.5 = 782.0
        assert_eq!(stroke_a.right(), 781.5);
        let outer_right_a = stroke_a.right() + stroke_w * 0.5;
        assert_eq!(outer_right_a, 782.0);

        // Outer bounds of stroke for part B: left edge is at 782.0 + 0.5 - 0.5 = 782.0
        assert_eq!(stroke_b.left(), 782.5);
        let outer_left_b = stroke_b.left() - stroke_w * 0.5;
        assert_eq!(outer_left_b, 782.0);

        // Neither stroke extends past x = 782.0 -> zero overlap!
        assert!(outer_right_a <= outer_left_b);
    }

    #[test]
    fn test_z_order_camera_view_depth_ordering() {
        // Camera looking at left side of cabinet from above (negative X looking towards +X)
        let eye = Vec3::new(-1200.0, -200.0, 2600.0);
        let target = Vec3::new(300.0, 275.0, 2000.0);
        let fwd = (target - eye).normalize();

        // Upright panel top tile at x = 9.0 (between x=0 and x=18)
        let upright_tile_center = Vec3::new(9.0, 275.0, 2535.0);
        // Shelf center at x = 300.0 (between x=18 and x=582)
        let shelf_center = Vec3::new(300.0, 275.0, 2661.0);

        let z_upright = (upright_tile_center - eye).dot(fwd);
        let z_shelf = (shelf_center - eye).dot(fwd);

        // Upright is closer along camera forward view direction than the shelf behind it
        assert!(z_upright < z_shelf, "Upright must have smaller view depth than shelf: {} < {}", z_upright, z_shelf);
    }

    #[test]
    fn test_clip_3d_polygon_near_plane_preserves_visible_geometry() {
        let eye = Vec3::new(0.0, 0.0, 0.0);
        let fwd = Vec3::new(0.0, 1.0, 0.0);
        let near_d = 10.0;

        // Quad straddling near plane (y goes from -5.0 behind camera to +20.0 in front)
        let quad = [
            Vec3::new(-10.0, -5.0, 0.0),
            Vec3::new(10.0, -5.0, 0.0),
            Vec3::new(10.0, 20.0, 0.0),
            Vec3::new(-10.0, 20.0, 0.0),
        ];

        let clipped = clip_3d_polygon_near_plane(&quad, eye, fwd, near_d);
        assert_eq!(clipped.len(), 4);
        for pt in &clipped {
            assert!((pt - eye).dot(fwd) >= near_d - 1e-4);
        }
    }

    #[test]
    fn test_point_in_convex_polygon_inside_and_outside() {
        let poly = [
            Pos2::new(10.0, 10.0),
            Pos2::new(50.0, 10.0),
            Pos2::new(50.0, 50.0),
            Pos2::new(10.0, 50.0),
        ];
        assert!(point_in_convex_polygon(Pos2::new(30.0, 30.0), &poly));
        assert!(!point_in_convex_polygon(Pos2::new(5.0, 30.0), &poly));
        assert!(!point_in_convex_polygon(Pos2::new(60.0, 30.0), &poly));
    }

    #[test]
    fn test_axis_separation_shelf_and_back_panel_from_front_and_rear() {
        // Back panel: Y from 0.0 to 18.0
        let back_min_y = 0.0;
        let back_max_y = 18.0;
        // Shelf in front: Y from 18.0 to 550.0
        let shelf_min_y = 18.0;
        let shelf_max_y = 550.0;

        // From front: eye_y = 1500.0 (in front of shelf)
        let eye_y_front = 1500.0;
        let shelf_in_front = test_axis_separation(
            shelf_min_y,
            shelf_max_y,
            back_min_y,
            back_max_y,
            eye_y_front,
            0.5,
        );
        // Shelf should be in front of back panel when camera is in front
        assert_eq!(shelf_in_front, Some(true));

        // From rear: eye_y = -1000.0 (behind back panel)
        let eye_y_rear = -1000.0;
        let shelf_in_front_rear = test_axis_separation(
            shelf_min_y,
            shelf_max_y,
            back_min_y,
            back_max_y,
            eye_y_rear,
            0.5,
        );
        // Shelf should be behind back panel (false) when camera is at the rear
        assert_eq!(shelf_in_front_rear, Some(false));
    }

    #[test]
    fn test_clip_convex_polygon_overlap_and_edge_touch() {
        let a = [
            Pos2::new(0.0, 0.0),
            Pos2::new(10.0, 0.0),
            Pos2::new(10.0, 10.0),
            Pos2::new(0.0, 10.0),
        ];
        // Overlapping square (opposite winding)
        let b = [
            Pos2::new(5.0, 5.0),
            Pos2::new(5.0, 15.0),
            Pos2::new(15.0, 15.0),
            Pos2::new(15.0, 5.0),
        ];
        let inter = clip_convex_polygon(&a, &b);
        assert!((polygon_area_abs(&inter) - 25.0).abs() < 1e-3);

        // Square touching only along the edge x = 10 -> zero area
        let c = [
            Pos2::new(10.0, 0.0),
            Pos2::new(20.0, 0.0),
            Pos2::new(20.0, 10.0),
            Pos2::new(10.0, 10.0),
        ];
        let touch = clip_convex_polygon(&a, &c);
        assert!(touch.len() < 3 || polygon_area_abs(&touch) < 1e-3);
    }

    #[test]
    fn test_clip_2d_polygon_rect_constrains_extreme_coordinates() {
        let rect = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(800.0, 600.0));
        // Quad with vertices extending to 20,000 pixels
        let extreme_poly = [
            Pos2::new(-20000.0, 300.0),
            Pos2::new(400.0, -15000.0),
            Pos2::new(25000.0, 300.0),
            Pos2::new(400.0, 20000.0),
        ];

        let clipped = clip_2d_polygon_rect(&extreme_poly, rect);
        assert!(clipped.len() >= 3);
        for pt in &clipped {
            assert!(pt.x >= rect.min.x - 1e-3 && pt.x <= rect.max.x + 1e-3);
            assert!(pt.y >= rect.min.y - 1e-3 && pt.y <= rect.max.y + 1e-3);
        }
    }

    #[test]
    fn test_ray_aabb_distance_hits_closer_box_first() {
        let eye = Vec3::new(300.0, 1500.0, 500.0);
        let dir = Vec3::new(0.0, -1.0, 0.0); // pointing straight in -Y direction

        // Box A (front door/shelf) at Y in [500, 518]
        let a_min = Vec3::new(200.0, 500.0, 400.0);
        let a_max = Vec3::new(400.0, 518.0, 600.0);

        // Box B (back wall) at Y in [0, 18]
        let b_min = Vec3::new(200.0, 0.0, 400.0);
        let b_max = Vec3::new(400.0, 18.0, 600.0);

        let dist_a = ray_aabb_distance(eye, dir, a_min, a_max).expect("Ray should hit box A");
        let dist_b = ray_aabb_distance(eye, dir, b_min, b_max).expect("Ray should hit box B");

        assert!(dist_a < dist_b, "Front box must have smaller hit distance than back box: {} < {}", dist_a, dist_b);
    }
}

