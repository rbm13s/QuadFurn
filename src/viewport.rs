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
        };
        cam.sync_eye_from_orbit();
        cam
    }
}

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
        if ui.ctx().wants_keyboard_input() {
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
        if ui.input(|i| i.key_pressed(Key::Tab)) && !ui.ctx().wants_keyboard_input() {
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

                let v_resp = ui.interact(v_split_rect, ui.id().with("quad_v_split"), Sense::drag());
                if v_resp.hovered() || v_resp.dragged() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                }
                if v_resp.dragged() {
                    self.split_x = ((mid_x + v_resp.drag_delta().x - avail.left()) / avail.width())
                        .clamp(0.2, 0.8);
                }

                let h_resp = ui.interact(h_split_rect, ui.id().with("quad_h_split"), Sense::drag());
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

        let id = ui.id().with(("ortho_pane", vp.axis as u8));
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
                painter.rect_stroke(grp_rect, 2.0, Stroke::new(1.5, grp_col));
                let badge_text = match self.lang {
                    crate::i18n::AppLanguage::Ru => format!(
                        "Группа ({} шт.): {:.0}×{:.0} мм",
                        self.selected_parts.len(),
                        max_uv.x - min_uv.x,
                        max_uv.y - min_uv.y
                    ),
                    _ => format!(
                        "{} ({} {}): {:.0}×{:.0} {}",
                        self.lang.dict().pdf_col_group,
                        self.selected_parts.len(),
                        self.lang.dict().unit_pcs,
                        max_uv.x - min_uv.x,
                        max_uv.y - min_uv.y,
                        self.lang.dict().unit_mm
                    ),
                };
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
            .rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_gray(55)));
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
            painter.add(Shape::convex_polygon(
                poly.clone(),
                fill_color,
                Stroke::new(if is_selected { 2.0 } else { 1.2 }, stroke_color),
            ));
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
        painter.rect_stroke(
            part_rect,
            0.0,
            Stroke::new(if is_selected { 2.0 } else { 1.2 }, stroke_color),
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

        // Map part sides (L1=Front/Bottom, L2=Back/Top, W1=Left/Bottom, W2=Right/Top) to the screen rectangle
        match (axis, part.orientation) {
            (OrthoAxis::TopXY, PartOrientation::HorizontalXY) => {
                // Shelf in XY plan: Length along X, Width along Y.
                // L1 = front face (top of screen +Y -> top), L2 = rear face (bottom of screen -Y -> bottom)
                // W1 = left edge (left), W2 = right edge (right)
                if let Some(s) = edge_stroke(part.edges.l1) {
                    painter.line_segment([part_rect.left_top(), part_rect.right_top()], s);
                }
                if let Some(s) = edge_stroke(part.edges.l2) {
                    painter.line_segment([part_rect.left_bottom(), part_rect.right_bottom()], s);
                }
                if let Some(s) = edge_stroke(part.edges.w1) {
                    painter.line_segment([part_rect.left_top(), part_rect.left_bottom()], s);
                }
                if let Some(s) = edge_stroke(part.edges.w2) {
                    painter.line_segment([part_rect.right_top(), part_rect.right_bottom()], s);
                }
            }
            (OrthoAxis::SideYZ, PartOrientation::VerticalYZ) => {
                // Side upright in YZ profile: Width along Y (horizontal), Length along Z (vertical).
                // L1 = front vertical face (+Y -> right), L2 = rear vertical face (-Y -> left)
                // W1 = bottom edge (bottom), W2 = top edge (top)
                if let Some(s) = edge_stroke(part.edges.l1) {
                    painter.line_segment([part_rect.right_top(), part_rect.right_bottom()], s);
                }
                if let Some(s) = edge_stroke(part.edges.l2) {
                    painter.line_segment([part_rect.left_top(), part_rect.left_bottom()], s);
                }
                if let Some(s) = edge_stroke(part.edges.w1) {
                    painter.line_segment([part_rect.left_bottom(), part_rect.right_bottom()], s);
                }
                if let Some(s) = edge_stroke(part.edges.w2) {
                    painter.line_segment([part_rect.left_top(), part_rect.right_top()], s);
                }
            }
            (OrthoAxis::FrontXZ, PartOrientation::FrontalXZ) => {
                // Front/Back panel in XZ front view: Width along X (horizontal), Length along Z (vertical).
                // L1 = left vertical face, L2 = right vertical face
                // W1 = bottom edge, W2 = top edge
                if let Some(s) = edge_stroke(part.edges.l1) {
                    painter.line_segment([part_rect.left_top(), part_rect.left_bottom()], s);
                }
                if let Some(s) = edge_stroke(part.edges.l2) {
                    painter.line_segment([part_rect.right_top(), part_rect.right_bottom()], s);
                }
                if let Some(s) = edge_stroke(part.edges.w1) {
                    painter.line_segment([part_rect.left_bottom(), part_rect.right_bottom()], s);
                }
                if let Some(s) = edge_stroke(part.edges.w2) {
                    painter.line_segment([part_rect.left_top(), part_rect.right_top()], s);
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
            painter.rect_stroke(hrect, 1.0, Stroke::new(1.2, Color32::from_rgb(20, 20, 20)));
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
        painter.rect_stroke(bg_rect, 3.0, Stroke::new(1.0, color));
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
                        let snap_thresh = 10.0 / vp.zoom_px_per_mm;
                        uv.x = snap_1d_edge(
                            uv.x,
                            &u_cands,
                            snap_thresh,
                            self.edge_snap_enabled,
                            self.grid_snap_enabled,
                            self.grid_step_mm,
                        )
                        .snapped_pos;
                        uv.y = snap_1d_edge(
                            uv.y,
                            &v_cands,
                            snap_thresh,
                            self.edge_snap_enabled,
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
                    let snap_thresh = 10.0 / vp.zoom_px_per_mm;
                    uv.x = snap_1d_edge(
                        uv.x,
                        &u_cands,
                        snap_thresh,
                        self.edge_snap_enabled,
                        self.grid_snap_enabled,
                        self.grid_step_mm,
                    )
                    .snapped_pos;
                    uv.y = snap_1d_edge(
                        uv.y,
                        &v_cands,
                        snap_thresh,
                        self.edge_snap_enabled,
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
                        let snap_thresh = 10.0 / vp.zoom_px_per_mm;
                        uv.x = snap_1d_edge(
                            uv.x,
                            &u_cands,
                            snap_thresh,
                            self.edge_snap_enabled,
                            self.grid_snap_enabled,
                            self.grid_step_mm,
                        )
                        .snapped_pos;
                        uv.y = snap_1d_edge(
                            uv.y,
                            &v_cands,
                            snap_thresh,
                            self.edge_snap_enabled,
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
                    painter.rect_stroke(preview_rect, 0.0, Stroke::new(1.8, col));
                    let preview_msg = match self.lang {
                        crate::i18n::AppLanguage::Ru => format!(
                            "Новая деталь: {:.0} × {:.0} мм\n(Tab — сменить тип)",
                            max_uv.x - min_uv.x,
                            max_uv.y - min_uv.y
                        ),
                        crate::i18n::AppLanguage::Uk => format!(
                            "Нова деталь: {:.0} × {:.0} мм\n(Tab — змінити тип)",
                            max_uv.x - min_uv.x,
                            max_uv.y - min_uv.y
                        ),
                        _ => format!(
                            "{}: {:.0} × {:.0} {}\n(Tab — switch type)",
                            self.lang.tr("Новая деталь", "New part"),
                            max_uv.x - min_uv.x,
                            max_uv.y - min_uv.y,
                            self.lang.dict().unit_mm
                        ),
                    };
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
                painter.rect_stroke(box_rect, 0.0, Stroke::new(1.5, col));
                if box_rect.width() > 40.0 && box_rect.height() > 18.0 {
                    let count_msg = match self.lang {
                        crate::i18n::AppLanguage::Ru => format!("Выделено: {} шт.", self.selected_parts.len()),
                        crate::i18n::AppLanguage::Uk => format!("Виділено: {} шт.", self.selected_parts.len()),
                        _ => format!("Selected: {} {}", self.selected_parts.len(), self.lang.dict().unit_pcs),
                    };
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
                    let delta_uv = cur_uv - drag.start_mouse_uv;
                    let exclude_ids: Vec<PartId> = if drag.group_orig_positions.len() > 1 {
                        drag.group_orig_positions.iter().map(|(id, _)| *id).collect()
                    } else {
                        vec![drag.part_id]
                    };
                    let (u_cands, v_cands) =
                        collect_snap_candidates_2d_excluding(project, vp.axis, &exclude_ids);
                    let snap_thresh = 10.0 / vp.zoom_px_per_mm;

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

                            let su = snap_interval_move(
                                target_min_u,
                                orig_size_uv.x,
                                &u_cands,
                                snap_thresh,
                                self.edge_snap_enabled,
                                self.grid_snap_enabled,
                                self.grid_step_mm,
                            );
                            let sv = snap_interval_move(
                                target_min_v,
                                orig_size_uv.y,
                                &v_cands,
                                snap_thresh,
                                self.edge_snap_enabled,
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
                                    self.edge_snap_enabled,
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
                                    self.edge_snap_enabled,
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
                                    self.edge_snap_enabled,
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
                                    self.edge_snap_enabled,
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

        let l_resp = ui.interact(left_btn, ui.id().with(("sb_hl", vp.axis as u8)), Sense::click());
        h_painter.rect_filled(left_btn, 0.0, if l_resp.hovered() { btn_hover_bg } else { btn_bg });
        h_painter.rect_stroke(left_btn, 0.0, border_stroke);
        h_painter.text(left_btn.center(), egui::Align2::CENTER_CENTER, "◄", FontId::proportional(9.0), Color32::LIGHT_GRAY);
        if l_resp.clicked() {
            vp.pan_uv_mm.x -= vis_w_mm * 0.18;
        }

        let r_resp = ui.interact(right_btn, ui.id().with(("sb_hr", vp.axis as u8)), Sense::click());
        h_painter.rect_filled(right_btn, 0.0, if r_resp.hovered() { btn_hover_bg } else { btn_bg });
        h_painter.rect_stroke(right_btn, 0.0, border_stroke);
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
                ui.id().with(("sb_htrack", vp.axis as u8)),
                Sense::click_and_drag(),
            );
            let thumb_active = ht_resp.hovered() || ht_resp.dragged();
            h_painter.rect_filled(
                h_thumb,
                2.0,
                if thumb_active { thumb_hover_bg } else { thumb_bg },
            );
            h_painter.rect_stroke(h_thumb, 2.0, Stroke::new(1.0, Color32::from_gray(135)));

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

        let t_resp = ui.interact(top_btn, ui.id().with(("sb_vt", vp.axis as u8)), Sense::click());
        v_painter.rect_filled(top_btn, 0.0, if t_resp.hovered() { btn_hover_bg } else { btn_bg });
        v_painter.rect_stroke(top_btn, 0.0, border_stroke);
        v_painter.text(top_btn.center(), egui::Align2::CENTER_CENTER, "▲", FontId::proportional(9.0), Color32::LIGHT_GRAY);
        if t_resp.clicked() {
            vp.pan_uv_mm.y += vis_h_mm * 0.18;
        }

        let b_resp = ui.interact(bot_btn, ui.id().with(("sb_vb", vp.axis as u8)), Sense::click());
        v_painter.rect_filled(bot_btn, 0.0, if b_resp.hovered() { btn_hover_bg } else { btn_bg });
        v_painter.rect_stroke(bot_btn, 0.0, border_stroke);
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
                ui.id().with(("sb_vtrack", vp.axis as u8)),
                Sense::click_and_drag(),
            );
            let thumb_active = vt_resp.hovered() || vt_resp.dragged();
            v_painter.rect_filled(
                v_thumb,
                2.0,
                if thumb_active { thumb_hover_bg } else { thumb_bg },
            );
            v_painter.rect_stroke(v_thumb, 2.0, Stroke::new(1.0, Color32::from_gray(135)));

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
            ui.id().with(("max_btn", vp.axis as u8)),
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
                self.lang.tr("[4 окна]", "[4 Views]")
            } else {
                self.lang.tr("[На весь]", "[Maximize]")
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

        let id = ui.id().with("pane_3d_view");
        let resp = ui.interact(rect, id, Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        // 3D scene background
        painter.rect_filled(rect, 0.0, Color32::from_rgb(22, 26, 32));

        // Toggle first-person Noclip on unmodified `Z` key (WITHOUT Ctrl/Alt) only when cursor is over the 3D window!
        let text_input_busy = ui.ctx().wants_keyboard_input();
        let pointer_in_3d = ui
            .input(|i| i.pointer.hover_pos())
            .map(|p| rect.contains(p))
            .unwrap_or(false)
            || resp.hovered();

        if !text_input_busy && (pointer_in_3d || self.cam_3d.noclip_active) {
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
            if pure_z_pressed {
                self.cam_3d.noclip_active = !self.cam_3d.noclip_active;
                if self.cam_3d.noclip_active {
                    Self::set_noclip_cursor_grab(ui.ctx(), true, rect.center());
                } else {
                    self.cam_3d.sync_orbit_from_eye();
                    Self::set_noclip_cursor_grab(ui.ctx(), false, rect.center());
                }
            }
            if self.cam_3d.noclip_active && ui.input(|i| i.key_pressed(Key::Escape)) {
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
            let raw_motion = ui
                .input(|i| i.pointer.motion())
                .unwrap_or_else(|| ui.input(|i| i.pointer.delta()));
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

        // 2. Collect all 3D block faces with internal abutment culling,
        // adaptive subdivision, and Newell's separating plane visibility sort
        // to completely eliminate Z-fighting and overlap artifacts between shelves, uprights, and back panels!
        #[derive(Clone)]
        struct Face3DToDraw {
            part_id: PartId,
            material_id: MaterialId,
            world_verts: [Vec3; 4],
            world_norm: Vec3,
            plane_d: f32,
            screen_pts: [Pos2; 4],
            screen_bbox: Rect,
            avg_depth: f32,
            min_depth: f32,
            max_depth: f32,
            fill_color: Color32,
            stroke_color: Color32,
            is_selected: bool,
            /// Exact UV coordinates of the 4 tile corners (taking grain orientation and physical edge scale into account)
            uv_corners: [Pos2; 4],
            /// Which of the 4 edges of this tile lie on the outer boundary of the part (for outline and edge banding)
            outer_edges: [bool; 4],
            edge_strokes: [Option<Stroke>; 4],
            grain_segment: Option<(Pos2, Pos2)>,
        }

        let mut faces_to_draw: Vec<Face3DToDraw> = Vec::new();
        let light_dir = Vec3::new(-0.45, -0.65, 0.62).normalize();
        let eye = self.cam_3d.eye_mm;

        // Pre-collect AABB bounds of all visible parts for internal abutment culling (e.g. shelf edge touching an upright)
        let part_aabbs: Vec<(PartId, Vec3, Vec3)> = project
            .parts
            .iter()
            .filter(|p| project.is_part_visible(p))
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

            let verts = part.world_vertices(thick);
            let rot_mat = part.rotation_matrix();

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
                // If this face is flush against another part (and the part is not selected),
                // skip rendering this internal edge to prevent Z-fighting through uprights or tops!
                if !is_sel && part.yaw_deg.abs() < 0.05 {
                    let probe_pt = face_center + world_norm * 0.4;
                    let sample_pts = [
                        probe_pt,
                        (q0 + face_center) * 0.5 + world_norm * 0.4,
                        (q1 + face_center) * 0.5 + world_norm * 0.4,
                        (q2 + face_center) * 0.5 + world_norm * 0.4,
                        (q3 + face_center) * 0.5 + world_norm * 0.4,
                    ];
                    let occluded_by_neighbor = part_aabbs.iter().any(|&(other_id, bmin, bmax)| {
                        if other_id == part.id {
                            return false;
                        }
                        sample_pts.iter().all(|pt| {
                            pt.x >= bmin.x - 0.5
                                && pt.x <= bmax.x + 0.5
                                && pt.y >= bmin.y - 0.5
                                && pt.y <= bmax.y + 0.5
                                && pt.z >= bmin.z - 0.5
                                && pt.z <= bmax.z + 0.5
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

                // Adaptive grid subdivision of large faces (e.g. side panel / shelf / backing),
                // so T-junctions (shelf meeting middle of an upright) sort correctly per tile!
                let len_u = q0.distance(q1);
                let len_v = q1.distance(q2);
                let div_u = ((len_u / 260.0).ceil() as usize).clamp(1, 4);
                let div_v = ((len_v / 260.0).ceil() as usize).clamp(1, 4);

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

                let plane_d = world_norm.dot(q0);

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

                        let mut scr_pts = [Pos2::ZERO; 4];
                        let mut valid = true;
                        let mut min_sx = f32::MAX;
                        let mut min_sy = f32::MAX;
                        let mut max_sx = f32::MIN;
                        let mut max_sy = f32::MIN;

                        for i in 0..4 {
                            if let Some((sp, _)) =
                                self.cam_3d.project_point(&vp_mat, rect, w_verts[i])
                            {
                                scr_pts[i] = sp;
                                min_sx = min_sx.min(sp.x);
                                min_sy = min_sy.min(sp.y);
                                max_sx = max_sx.max(sp.x);
                                max_sy = max_sy.max(sp.y);
                            } else {
                                valid = false;
                                break;
                            }
                        }
                        if !valid {
                            continue;
                        }

                        let sub_center =
                            (w_verts[0] + w_verts[1] + w_verts[2] + w_verts[3]) * 0.25;
                        let dist_to_cam = eye.distance(sub_center);
                        let d0 = eye.distance(w_verts[0]);
                        let d1 = eye.distance(w_verts[1]);
                        let d2 = eye.distance(w_verts[2]);
                        let d3 = eye.distance(w_verts[3]);
                        let min_depth = d0.min(d1).min(d2).min(d3);
                        let max_depth = d0.max(d1).max(d2).max(d3);

                        let outer_edges = [
                            iv == 0,         // edge 0..1 (top along V)
                            iu + 1 == div_u, // edge 1..2 (right along U)
                            iv + 1 == div_v, // edge 2..3 (bottom along V)
                            iu == 0,         // edge 3..0 (left along U)
                        ];

                        // Draw grain direction arrow only on the center tile of the main face
                        let is_center_tile = iu == div_u / 2 && iv == div_v / 2;
                        let grain_segment = if is_center_tile
                            && part.grain_fixed
                            && self.is_main_face(part.orientation, f_idx)
                        {
                            let (mid_a, mid_b) = if !grain_along_v {
                                (
                                    Pos2::new(
                                        (scr_pts[0].x + scr_pts[3].x) * 0.5,
                                        (scr_pts[0].y + scr_pts[3].y) * 0.5,
                                    ),
                                    Pos2::new(
                                        (scr_pts[1].x + scr_pts[2].x) * 0.5,
                                        (scr_pts[1].y + scr_pts[2].y) * 0.5,
                                    ),
                                )
                            } else {
                                (
                                    Pos2::new(
                                        (scr_pts[0].x + scr_pts[1].x) * 0.5,
                                        (scr_pts[0].y + scr_pts[1].y) * 0.5,
                                    ),
                                    Pos2::new(
                                        (scr_pts[3].x + scr_pts[2].x) * 0.5,
                                        (scr_pts[3].y + scr_pts[2].y) * 0.5,
                                    ),
                                )
                            };
                            Some((
                                Pos2::new(
                                    mid_a.x * 0.75 + mid_b.x * 0.25,
                                    mid_a.y * 0.75 + mid_b.y * 0.25,
                                ),
                                Pos2::new(
                                    mid_a.x * 0.25 + mid_b.x * 0.75,
                                    mid_a.y * 0.25 + mid_b.y * 0.75,
                                ),
                            ))
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

                        faces_to_draw.push(Face3DToDraw {
                            part_id: part.id,
                            material_id: part.material_id,
                            world_verts: w_verts,
                            world_norm,
                            plane_d,
                            screen_pts: scr_pts,
                            screen_bbox: Rect::from_min_max(
                                Pos2::new(min_sx, min_sy),
                                Pos2::new(max_sx, max_sy),
                            ),
                            avg_depth: dist_to_cam,
                            min_depth,
                            max_depth,
                            fill_color: Color32::from_rgb(r, g, b),
                            stroke_color: stroke_col,
                            is_selected: is_sel,
                            uv_corners,
                            outer_edges,
                            edge_strokes: base_edge_strokes,
                            grain_segment,
                        });
                    }
                }
            }
        }

        // Sort descending by distance to camera first (far to near)
        faces_to_draw.sort_by(|a, b| {
            b.avg_depth
                .partial_cmp(&a.avg_depth)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Refine order using Topological Depth Sorting:
        // Detects exact 2D screen overlap between quad faces, casts ray through overlap point,
        // and establishes directed dependency edges (B -> A means B must be drawn before A).
        let n_faces = faces_to_draw.len();
        if n_faces > 1 {
            let inv_vp = vp_mat.inverse();
            let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n_faces];
            let mut in_degree: Vec<usize> = vec![0; n_faces];

            for i in 0..n_faces {
                let fa = &faces_to_draw[i];
                for j in (i + 1)..n_faces {
                    let fb = &faces_to_draw[j];
                    if fa.part_id == fb.part_id {
                        continue;
                    }

                    // Quick test 1: If depth ranges do not overlap, the one strictly farther must be drawn first
                    if fa.min_depth >= fb.max_depth - 0.2 {
                        adj[i].push(j);
                        in_degree[j] += 1;
                        continue;
                    }
                    if fb.min_depth >= fa.max_depth - 0.2 {
                        adj[j].push(i);
                        in_degree[i] += 1;
                        continue;
                    }

                    // Quick test 2: Screen bounding boxes
                    if !fa.screen_bbox.intersects(fb.screen_bbox) {
                        continue;
                    }

                    // Detailed test 3: Find point in 2D quad overlap and test ray depths
                    if let Some(inter_pt) = find_point_in_quad_intersection(&fa.screen_pts, &fb.screen_pts) {
                        if let Some(ray_dir) = ray_from_screen_point(inter_pt, rect, &inv_vp) {
                            let t_a = ray_plane_distance(eye, ray_dir, fa.world_norm, fa.plane_d);
                            let t_b = ray_plane_distance(eye, ray_dir, fb.world_norm, fb.plane_d);
                            if let (Some(ta), Some(tb)) = (t_a, t_b) {
                                if ta + 0.3 < tb {
                                    // A is closer than B at this screen pixel -> B must be drawn BEFORE A (B -> A)
                                    adj[j].push(i);
                                    in_degree[i] += 1;
                                } else if tb + 0.3 < ta {
                                    // B is closer than A at this screen pixel -> A must be drawn BEFORE B (A -> B)
                                    adj[i].push(j);
                                    in_degree[j] += 1;
                                }
                            }
                        }
                    }
                }
            }

            // Kahn's algorithm with priority queue ordered by descending avg_depth (farthest first)
            use std::collections::BinaryHeap;
            #[derive(PartialEq)]
            struct HeapItem {
                depth: f32,
                idx: usize,
            }
            impl Eq for HeapItem {}
            impl PartialOrd for HeapItem {
                fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    Some(self.cmp(other))
                }
            }
            impl Ord for HeapItem {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    self.depth.partial_cmp(&other.depth).unwrap_or(std::cmp::Ordering::Equal)
                }
            }

            let mut heap = BinaryHeap::new();
            for (idx, &deg) in in_degree.iter().enumerate() {
                if deg == 0 {
                    heap.push(HeapItem {
                        depth: faces_to_draw[idx].avg_depth,
                        idx,
                    });
                }
            }

            let mut sorted_indices = Vec::with_capacity(n_faces);
            let mut visited = vec![false; n_faces];

            while let Some(HeapItem { idx: u, .. }) = heap.pop() {
                if visited[u] {
                    continue;
                }
                visited[u] = true;
                sorted_indices.push(u);

                for &v in &adj[u] {
                    if in_degree[v] > 0 {
                        in_degree[v] -= 1;
                        if in_degree[v] == 0 && !visited[v] {
                            heap.push(HeapItem {
                                depth: faces_to_draw[v].avg_depth,
                                idx: v,
                            });
                        }
                    }
                }
            }

            if sorted_indices.len() < n_faces {
                let mut remaining: Vec<usize> = (0..n_faces).filter(|&idx| !visited[idx]).collect();
                remaining.sort_by(|&a, &b| {
                    faces_to_draw[b]
                        .avg_depth
                        .partial_cmp(&faces_to_draw[a].avg_depth)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                sorted_indices.extend(remaining);
            }

            let mut reordered = Vec::with_capacity(n_faces);
            for idx in sorted_indices {
                reordered.push(faces_to_draw[idx].clone());
            }
            faces_to_draw = reordered;
        }

        // 3. Render faces (with perspective-correct 3D subdivision for textures without PS1 affine distortion, or solid material color)
        for face in &faces_to_draw {
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
                painter.add(Shape::convex_polygon(
                    face.screen_pts.to_vec(),
                    face.fill_color,
                    Stroke::new(0.4, face.fill_color), // Hides anti-aliasing seams between adjacent tiles of the same face
                ));
            }

            if self.cam_3d.wireframe_overlay || face.is_selected {
                for i in 0..4 {
                    // Draw outlines and edge banding only on outer part boundaries (skip internal tile subdivision edges)
                    if !face.outer_edges[i] {
                        continue;
                    }
                    let p1 = face.screen_pts[i];
                    let p2 = face.screen_pts[(i + 1) % 4];
                    let st = face.edge_strokes[i].unwrap_or_else(|| {
                        Stroke::new(if face.is_selected { 2.0 } else { 1.0 }, face.stroke_color)
                    });
                    painter.line_segment([p1, p2], st);
                }
            }

            if let Some((gs, ge)) = face.grain_segment {
                if gs.distance(ge) > 12.0 {
                    painter.line_segment(
                        [gs, ge],
                        Stroke::new(1.1, Color32::from_rgba_unmultiplied(60, 40, 20, 120)),
                    );
                }
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
                for face in faces_to_draw.iter().rev() {
                    if point_in_quad(crosshair_pos, &face.screen_pts) {
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
                for face in faces_to_draw.iter().rev() {
                    if point_in_quad(click_pos, &face.screen_pts) {
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
        painter.rect_stroke(rect, 0.0, Stroke::new(if self.cam_3d.noclip_active { 2.0 } else { 1.0 }, border_col));

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
                .tr(
                    "3D ВИД [Z: ПОЛЕТ ВКЛ — WASD + Мышь | ЛКМ — Выбор по прицелу | Z/Esc — Выход]",
                    "3D VIEW [Z: FLY ON — WASD + Mouse | LMB — Pick at Crosshair | Z/Esc — Exit]",
                )
                .to_string()
        } else {
            self.lang
                .tr(
                    "3D ВИД [ЛКМ — Выбор | ПКМ/СКМ — Вращать | Нажмите Z над окном для полета]",
                    "3D VIEW [LMB — Select | RMB/MMB — Orbit | Press Z over pane for Fly mode]",
                )
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
        let btn_resp = ui.interact(btn_rect, ui.id().with("max_btn_3d"), Sense::click());
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
                self.lang.tr("[4 окна]", "[4 Views]")
            } else {
                self.lang.tr("[На весь]", "[Maximize]")
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
            min_w = min_w.min(clip.w.max(10.0));
            max_w = max_w.max(clip.w.max(10.0));
        }
        let depth_ratio = (max_w / min_w.max(10.0)).clamp(1.0, 4.0);
        let sub_u = (((screen_bbox.width() * depth_ratio) / 26.0).ceil() as usize).clamp(3, 8);
        let sub_v = (((screen_bbox.height() * depth_ratio) / 26.0).ceil() as usize).clamp(3, 8);

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

        let mut mesh = egui::Mesh::with_texture(tex_id);
        let cols = sub_u + 1;
        let mut grid_scr = Vec::with_capacity(cols * (sub_v + 1));

        for iv in 0..=sub_v {
            let t = iv as f32 / sub_v as f32;
            for iu in 0..=sub_u {
                let s = iu as f32 / sub_u as f32;
                let p3d = bilerp_3d(s, t);
                let uv = bilerp_uv(s, t);
                let Some((scr, _)) = self.cam_3d.project_point(vp_mat, rect, p3d) else {
                    return;
                };
                grid_scr.push((scr, uv));
            }
        }

        // Add grid nodes to vertex buffer
        for &(pos, uv) in &grid_scr {
            mesh.vertices.push(egui::epaint::Vertex {
                pos,
                uv,
                color: tint_color,
            });
        }

        // For each micro-cell, add its true 3D-projected center and build 4 symmetric triangles (X-split)
        // to eliminate diagonal texture creasing!
        for iv in 0..sub_v {
            let t_mid = (iv as f32 + 0.5) / sub_v as f32;
            for iu in 0..sub_u {
                let s_mid = (iu as f32 + 0.5) / sub_u as f32;
                let p_mid_3d = bilerp_3d(s_mid, t_mid);
                let uv_mid = bilerp_uv(s_mid, t_mid);
                let Some((scr_mid, _)) = self.cam_3d.project_point(vp_mat, rect, p_mid_3d) else {
                    continue;
                };

                let i00 = (iv * cols + iu) as u32;
                let i10 = (iv * cols + iu + 1) as u32;
                let i11 = ((iv + 1) * cols + iu + 1) as u32;
                let i01 = ((iv + 1) * cols + iu) as u32;
                let i_mid = mesh.vertices.len() as u32;

                mesh.vertices.push(egui::epaint::Vertex {
                    pos: scr_mid,
                    uv: uv_mid,
                    color: tint_color,
                });

                mesh.add_triangle(i00, i10, i_mid);
                mesh.add_triangle(i10, i11, i_mid);
                mesh.add_triangle(i11, i01, i_mid);
                mesh.add_triangle(i01, i00, i_mid);
            }
        }

        painter.add(Shape::Mesh(mesh));
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

/// Test if a 2D point is inside a convex quad
fn point_in_quad(pt: Pos2, quad: &[Pos2; 4]) -> bool {
    let mut pos = 0;
    let mut neg = 0;
    for i in 0..4 {
        let a = quad[i];
        let b = quad[(i + 1) % 4];
        let cross = (b.x - a.x) * (pt.y - a.y) - (b.y - a.y) * (pt.x - a.x);
        if cross > 0.0 {
            pos += 1;
        } else if cross < 0.0 {
            neg += 1;
        }
    }
    pos == 0 || neg == 0
}

fn line_segment_intersection_2d(p1: Pos2, p2: Pos2, q1: Pos2, q2: Pos2) -> Option<Pos2> {
    let d = (p2.x - p1.x) * (q2.y - q1.y) - (p2.y - p1.y) * (q2.x - q1.x);
    if d.abs() < 1e-5 {
        return None;
    }
    let t = ((q1.x - p1.x) * (q2.y - q1.y) - (q1.y - p1.y) * (q2.x - q1.x)) / d;
    let u = ((q1.x - p1.x) * (p2.y - p1.y) - (q1.y - p1.y) * (p2.x - p1.x)) / d;
    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
        Some(Pos2::new(
            p1.x + t * (p2.x - p1.x),
            p1.y + t * (p2.y - p1.y),
        ))
    } else {
        None
    }
}

fn find_point_in_quad_intersection(quad_a: &[Pos2; 4], quad_b: &[Pos2; 4]) -> Option<Pos2> {
    for &pa in quad_a {
        if point_in_quad(pa, quad_b) {
            return Some(pa);
        }
    }
    for &pb in quad_b {
        if point_in_quad(pb, quad_a) {
            return Some(pb);
        }
    }
    for i in 0..4 {
        let a1 = quad_a[i];
        let a2 = quad_a[(i + 1) % 4];
        for j in 0..4 {
            let b1 = quad_b[j];
            let b2 = quad_b[(j + 1) % 4];
            if let Some(inter) = line_segment_intersection_2d(a1, a2, b1, b2) {
                return Some(inter);
            }
        }
    }
    None
}

fn ray_from_screen_point(scr_pt: Pos2, rect: Rect, inv_vp: &Mat4) -> Option<Vec3> {
    let ndc_x = (scr_pt.x - rect.center().x) / (rect.width() * 0.5).max(1.0);
    let ndc_y = -(scr_pt.y - rect.center().y) / (rect.height() * 0.5).max(1.0);
    let clip_near = Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
    let world_near_h = *inv_vp * clip_near;
    if world_near_h.w.abs() < 1e-6 {
        return None;
    }
    let world_near = world_near_h.truncate() / world_near_h.w;
    let clip_far = Vec4::new(ndc_x, ndc_y, 1.0, 1.0);
    let world_far_h = *inv_vp * clip_far;
    if world_far_h.w.abs() < 1e-6 {
        return None;
    }
    let world_far = world_far_h.truncate() / world_far_h.w;
    Some((world_far - world_near).normalize())
}

fn ray_plane_distance(eye: Vec3, ray_dir: Vec3, norm: Vec3, plane_d: f32) -> Option<f32> {
    let denom = norm.dot(ray_dir);
    if denom.abs() < 1e-5 {
        return None;
    }
    let t = (plane_d - norm.dot(eye)) / denom;
    if t > 0.0 {
        Some(t)
    } else {
        None
    }
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
}

