use crate::model::{EdgeId, MaterialId, PartBlock, PartEdges, PartId, PartOrientation, Project};
use crate::snapping::{
    collect_snap_candidates_2d, collect_snap_candidates_2d_excluding,
    compute_neighbor_distances_2d, snap_1d_edge, snap_interval_move, snap_to_grid, OrthoAxis,
};
use egui::{Color32, FontId, Key, Painter, Pos2, Rect, Sense, Shape, Stroke, TextureHandle, Vec2 as EguiVec2};
use glam::{Mat4, Vec2, Vec3, Vec4};
use std::collections::HashMap;

/// Инструмент в редакторе
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorTool {
    /// Выбор, перемещение и ресайз блоков за ручки
    SelectAndMove,
    /// Создание новой детали рамкой в любом 2D-окне
    CreateBlock,
    /// Измерительная рулетка между двумя точками в 2D
    MeasureTape,
}

/// Какую ручку ресайза или сам блок тянет пользователь в 2D-окне
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
    /// Возвращает иконку двусторонней стрелки для курсора мыши при наведении/перетаскивании маркера
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

/// Состояние активного перетаскивания (одиночного или группового) в 2D-окне
#[derive(Clone, Debug)]
pub struct ActiveDrag2D {
    pub axis: OrthoAxis,
    pub part_id: PartId,
    pub handle: DragHandle2D,
    pub start_mouse_uv: Vec2,
    pub orig_min_3d: Vec3,
    pub orig_max_3d: Vec3,
    /// Исходные мировые координаты `position_mm` всех деталей, перемещаемых вместе группой
    pub group_orig_positions: Vec<(PartId, Vec3)>,
    pub active_guides_u: Vec<f32>,
    pub active_guides_v: Vec<f32>,
}

/// Состояние одной 2D ортогональной проекции
#[derive(Clone, Debug)]
pub struct OrthoViewportState {
    pub axis: OrthoAxis,
    /// Центр камеры в мировых миллиметрах (U, V)
    pub pan_uv_mm: Vec2,
    /// Масштаб: сколько пикселей экрана в 1 миллиметре (например 0.45 px/mm)
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

    /// Перевод из мировых миллиметров (U, V) в экранные пиксели `Pos2`
    /// Ось U направлена вправо (+X), ось V направлена вверх (-Y на экране)
    pub fn world_to_screen(&self, rect: Rect, uv_mm: Vec2) -> Pos2 {
        let center = rect.center();
        let dx = (uv_mm.x - self.pan_uv_mm.x) * self.zoom_px_per_mm;
        let dy = -(uv_mm.y - self.pan_uv_mm.y) * self.zoom_px_per_mm;
        Pos2::new(center.x + dx, center.y + dy)
    }

    /// Перевод из экранных пикселей `Pos2` в мировые миллиметры (U, V)
    pub fn screen_to_world(&self, rect: Rect, screen_pos: Pos2) -> Vec2 {
        let center = rect.center();
        let u = self.pan_uv_mm.x + (screen_pos.x - center.x) / self.zoom_px_per_mm;
        let v = self.pan_uv_mm.y - (screen_pos.y - center.y) / self.zoom_px_per_mm;
        Vec2::new(u, v)
    }
}

/// Состояние 3D-камеры с поддержкой двух режимов:
/// 1) CAD Orbit (вращение вокруг цели)
/// 2) Hammer Editor Noclip Fly по клавише `Z` (WASD + свободный обзор мышью)
#[derive(Clone, Debug)]
pub struct Camera3D {
    /// Включен ли режим полета Hammer Noclip (по клавише Z)
    pub noclip_active: bool,
    /// Позиция глаза камеры в миллиметрах (X, Y — горизонталь, Z — высота вверх)
    pub eye_mm: Vec3,
    /// Точка прицела для режима Orbit (мм)
    pub orbit_target_mm: Vec3,
    /// Дистанция до цели в режиме Orbit (мм)
    pub orbit_distance_mm: f32,
    /// Горизонтальный угол обзора (рыскание вокруг Z в радианах)
    pub yaw_rad: f32,
    /// Вертикальный угол обзора (тангаж в радианах, от -1.5 до +1.5)
    pub pitch_rad: f32,
    /// Базовая скорость полета в режиме Z-Noclip (мм/сек)
    pub fly_speed_mm_per_sec: f32,
    /// Показывать ли полупрозрачный каркас (X-Ray) или плотную заливку
    pub wireframe_overlay: bool,
    /// Показывать ли маркеры кромок в 3D
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
    /// Единичный вектор направления взгляда камеры в системе координат (X вправо, Y вглубь, Z вверх)
    pub fn forward_dir(&self) -> Vec3 {
        let cp = self.pitch_rad.cos();
        let sp = self.pitch_rad.sin();
        let cy = self.yaw_rad.cos();
        let sy = self.yaw_rad.sin();
        Vec3::new(cp * cy, cp * sy, sp).normalize_or_zero()
    }

    /// Единичный вектор «вправо» от взгляда камеры (в горизонтальной плоскости XY)
    pub fn right_dir(&self) -> Vec3 {
        let f = self.forward_dir();
        f.cross(Vec3::Z).normalize_or_zero()
    }

    /// Единичный вектор «вверх» камеры
    pub fn up_dir(&self) -> Vec3 {
        let r = self.right_dir();
        let f = self.forward_dir();
        r.cross(f).normalize_or_zero()
    }

    /// Пересчитывает `eye_mm` из параметров `orbit_target_mm`, `orbit_distance_mm`, `yaw_rad`, `pitch_rad`
    pub fn sync_eye_from_orbit(&mut self) {
        let fwd = self.forward_dir();
        self.eye_mm = self.orbit_target_mm - fwd * self.orbit_distance_mm;
    }

    /// При выходе из Noclip обновляет `orbit_target_mm` перед камерой для бесшовного переключения
    pub fn sync_orbit_from_eye(&mut self) {
        let fwd = self.forward_dir();
        self.orbit_target_mm = self.eye_mm + fwd * self.orbit_distance_mm;
    }

    /// Матрица View-Projection для перспективной 3D проекции
    pub fn view_proj_matrix(&self, aspect: f32) -> Mat4 {
        let target = self.eye_mm + self.forward_dir() * 100.0;
        let view = Mat4::look_at_rh(self.eye_mm, target, Vec3::Z);
        let proj = Mat4::perspective_rh(45.0_f32.to_radians(), aspect.max(0.1), 10.0, 50_000.0);
        proj * view
    }

    /// Проецирует 3D-точку в экранные координаты `Pos2` и возвращает `(Pos2, depth_w)`
    /// Если точка позади камеры (`w <= 10.0`), возвращает `None`
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

/// Какое из 4 окон развернуто на весь экран (по клавише Space или кнопке `[Maximize]`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaximizedPane {
    None,
    TopXY,
    Perspective3D,
    FrontXZ,
    SideYZ,
}

/// Состояние 4-экранного рабочего стола (Hammer Editor style)
pub struct QuadViewportController {
    pub top_xy: OrthoViewportState,
    pub front_xz: OrthoViewportState,
    pub side_yz: OrthoViewportState,
    pub cam_3d: Camera3D,
    pub maximized: MaximizedPane,
    /// Положение вертикального и горизонтального разделителя 4 экранов (от 0.2 до 0.8, по умолчанию 0.5)
    pub split_x: f32,
    pub split_y: f32,
    /// Последние известные размеры 4 окон в пикселях (TopXY, FrontXZ, SideYZ, 3D) для точного вписывания модели в окно
    pub last_pane_sizes: [EguiVec2; 4],

    /// Активный инструмент
    pub tool: EditorTool,
    /// Список всех выделенных деталей (для группового выделения рамкой или через Shift/Ctrl+клик)
    pub selected_parts: Vec<PartId>,
    /// Начало рамки группового выделения в режиме SelectAndMove: (OrthoAxis, start_uv_mm, current_uv_mm, append_mode)
    pub box_select_drag: Option<(OrthoAxis, Vec2, Vec2, bool, Vec<PartId>)>,
    /// Ориентация по умолчанию для создания новой детали (можно переключать клавишей Tab)
    pub create_orientation_override: Option<PartOrientation>,
    /// Начало рамки создания новой детали в 2D: (OrthoAxis, start_uv_mm, current_uv_mm)
    pub creating_drag: Option<(OrthoAxis, Vec2, Vec2)>,
    /// Активное перетаскивание или ресайз существующего блока (или группы блоков)
    pub active_drag: Option<ActiveDrag2D>,
    /// Точки рулетки измерения: (OrthoAxis, start_uv_mm, end_uv_mm)
    pub measure_points: Option<(OrthoAxis, Vec2, Vec2)>,

    /// Настройки сетки и привязки
    pub grid_snap_enabled: bool,
    pub edge_snap_enabled: bool,
    pub show_distance_callouts: bool,
    pub grid_step_mm: f32,
    /// Текущий язык интерфейса для заголовков проекций и 3D-окна
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
            grid_step_mm: 18.0, // По умолчанию шаг сетки 18 мм (под базовый ЛДСП 18 мм)
            lang: crate::i18n::AppLanguage::Ru,
        }
    }
}

impl QuadViewportController {
    /// Синхронизирует список группового выделения `selected_parts` с основным `selected_part`
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

    /// Вычисляет общий 3D AABB всех видимых деталей проекта
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

    /// Вычисляет общий 3D AABB для текущих выделенных деталей
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

    /// Центрирует все 4 вида на центре текущего проекта (без принудительного сброса масштаба)
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

    /// Вписывает всю модель целиком в каждое из 4 окон (3 ортогональные 2D-проекции и 3D-вид)
    pub fn fit_all_to_window(&mut self, project: &Project) {
        let (min_pt, max_pt) = Self::compute_project_bounds(project)
            .unwrap_or((Vec3::ZERO, Vec3::new(800.0, 500.0, 720.0)));
        let center = (min_pt + max_pt) * 0.5;
        let span = (max_pt - min_pt).max(Vec3::splat(60.0));

        // 1. Вписываем во все три 2D проекции с учетом размеров их окон, заголовков и полос прокрутки
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

        // 2. Вписываем 3D-модель в пирамиду видимости 3D-камеры
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

    /// Отрисовка всех 4 экранов (или одного развернутого) в центральном рабочем поле
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

        // Проверяем горячую клавишу Tab для переключения ориентации создаваемой детали
        if ui.input(|i| i.key_pressed(Key::Tab)) && !ui.ctx().wants_keyboard_input() {
            self.create_orientation_override = Some(match self.create_orientation_override {
                None | Some(PartOrientation::HorizontalXY) => PartOrientation::VerticalYZ,
                Some(PartOrientation::VerticalYZ) => PartOrientation::FrontalXZ,
                Some(PartOrientation::FrontalXZ) => PartOrientation::HorizontalXY,
            });
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

                // Рамки 4 вьюпортов:
                // Верхний левый: Top (XY) | Верхний правый: 3D Perspective
                // Нижний левый: Front (XZ) | Нижний правый: Side (YZ)
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

                // Отрисовка 4 экранов
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

                // Перетаскивание разделителей 4 экранов
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

    /// Отрисовка и интерактивность одного 2D ортогонального экрана (Top XY / Front XZ / Side YZ) с полосами прокрутки в стиле Hammer Editor
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

        // Выделяем место под горизонтальную (внизу) и вертикальную (справа) полосы прокрутки как в Hammer Editor
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
        let painter = ui.painter_at(canvas_rect);

        // Темный чертежный фон в стиле WorldCraft / Hammer Editor
        painter.rect_filled(canvas_rect, 0.0, Color32::from_rgb(18, 22, 26));

        // Масштабирование колесиком мыши относительно курсора и панорамирование ПКМ / СКМ
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

        // Панорамирование средней или правой кнопкой мыши
        if resp.dragged_by(egui::PointerButton::Middle)
            || resp.dragged_by(egui::PointerButton::Secondary)
        {
            let delta = resp.drag_delta();
            vp.pan_uv_mm.x -= delta.x / vp.zoom_px_per_mm;
            vp.pan_uv_mm.y += delta.y / vp.zoom_px_per_mm;
        }

        // 1. Отрисовка миллиметровой сетки
        self.draw_ortho_grid(&painter, canvas_rect, vp);

        // 2. Отрисовка всех видимых деталей в 2D проекции
        for part in &project.parts {
            if !project.is_part_visible(part) {
                continue;
            }
            let is_sel = self.is_part_selected(*selected_part, part.id);
            self.draw_part_in_ortho(&painter, canvas_rect, vp, project, part, is_sel);
        }

        // 3. Отрисовка ручек ресайза (для одиночного выделения) или общей рамки группового выделения
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

        // 4. Обработка левой кнопки мыши: Выделение (одиночное и групповое рамкой), Перетаскивание/Ресайз, Создание блока, Рулетка
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

        // 5. Заголовок вьюпорта с кнопкой разворота на весь экран
        self.draw_ortho_header_overlay(ui, &painter, canvas_rect, vp);

        // 6. Полосы прокрутки внизу и справа 2D-окна (как в Hammer Editor) и внешняя рамка
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

        // Выбираем адаптивный шаг визуальной сетки, чтобы линии не сливались при отдалении
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

        // Точка (0, 0) с подписями миллиметровых осей
        let origin_scr = vp.world_to_screen(rect, Vec2::ZERO);
        if rect.contains(origin_scr) {
            painter.circle_filled(origin_scr, 3.5, Color32::from_rgb(220, 180, 60));
            painter.text(
                origin_scr + EguiVec2::new(5.0, -5.0),
                egui::Align2::LEFT_BOTTOM,
                "(0, 0 мм)",
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

        // Если деталь повернута вокруг оси Z и мы в TopXY — рисуем повернутый 4-угольник
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

        // Отображение кромок на гранях детали в 2D проекциях
        self.draw_part_edges_in_ortho(painter, part_rect, vp.axis, project, part);

        // Если деталь видна пластью и достаточно крупная на экране — рисуем стрелку направления текстуры и размер
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

    /// Подсвечивает оклеенные кромкой стороны детали цветными линиями на 2D-чертеже
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

        // Сопоставляем стороны детали (Д1=Перед/Низ, Д2=Зад/Верх, Ш1=Лев/Низ, Ш2=Прав/Верх) с экранным прямоугольником
        match (axis, part.orientation) {
            (OrthoAxis::TopXY, PartOrientation::HorizontalXY) => {
                // Полка в плане XY: Длина по X, Ширина по Y.
                // Д1 = передняя грань (верх экрана +Y -> top), Д2 = задняя грань (низ экрана -Y -> bottom)
                // Ш1 = левый торец (left), Ш2 = правый торец (right)
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
                // Боковая стойка в профиле YZ: Ширина по Y (горизонталь), Длина по Z (вертикаль).
                // Д1 = передняя вертикальная грань (+Y -> right), Д2 = задняя вертикальная грань (-Y -> left)
                // Ш1 = нижний торец (bottom), Ш2 = верхний торец (top)
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
                // Фасад/Стенка во фронтальном виде XZ: Ширина по X (горизонталь), Длина по Z (вертикаль).
                // Д1 = левая вертикальная грань, Д2 = правая вертикальная грань
                // Ш1 = нижний торец, Ш2 = верхний торец
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

    /// Рисует 8 квадратных маркеров-ручек (как в Hammer Editor и CorelDRAW) вокруг выбранной детали
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

        let handle_pts = [
            r.left_top(),
            Pos2::new(r.center().x, r.top()),
            r.right_top(),
            Pos2::new(r.right(), r.center().y),
            r.right_bottom(),
            Pos2::new(r.center().x, r.bottom()),
            r.left_bottom(),
            Pos2::new(r.left(), r.center().y),
        ];

        for hp in handle_pts {
            let hrect = Rect::from_center_size(hp, EguiVec2::splat(7.0));
            painter.rect_filled(hrect, 1.0, Color32::WHITE);
            painter.rect_stroke(hrect, 1.0, Stroke::new(1.2, Color32::from_rgb(20, 20, 20)));
        }
    }

    /// Рисует размерную стрелку-выноску (расстояние в мм до соседней детали)
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
        let text = format!("{:.0} мм", distance_mm);
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

    /// Обработка ЛКМ в 2D-окне: выбор детали, перетаскивание за ручки с магнитной привязкой, создание новой детали и рулетка
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

        // --- РЕЖИМ 1: Рулетка (MeasureTape) ---
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
                        format!("L = {:.1} мм (ΔU={:.1}, ΔV={:.1})", dist, dx, dy),
                        FontId::monospace(11.5),
                        col,
                    );
                }
            }
            return;
        }

        // --- РЕЖИМ 2: Создание новой детали рамкой (CreateBlock) ---
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

            // Отрисовка превью создаваемого блока
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
                    painter.text(
                        preview_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        format!(
                            "Новая деталь: {:.0} × {:.0} мм\n(Tab — сменить тип)",
                            max_uv.x - min_uv.x,
                            max_uv.y - min_uv.y
                        ),
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

        // --- РЕЖИМ 3: Выделение (одиночное и групповое рамкой), Перетаскивание группы и Ресайз за ручки (SelectAndMove) ---
        let modifiers = ui.input(|i| i.modifiers);
        let multi_mod = modifiers.shift || modifiers.ctrl || modifiers.command;

        // Меняем курсор мыши при наведении на квадратики (маркеры ресайза) или внутрь выделенного блока/группы
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
            // Берем точку ИМЕННО начала нажатия кнопки мыши (press_origin), чтобы при рывке мыши не промахнуться мимо квадратика!
            let press_pos = ui.input(|i| i.pointer.press_origin()).or(mouse_pos);
            if let Some(mp) = press_pos {
                let click_uv = vp.screen_to_world(rect, mp);

                let mut picked_handle: Option<(PartId, DragHandle2D)> = None;

                // 1. Если выделена ровно 1 деталь — сначала проверяем её 8 квадратиков ресайза и её внутреннюю область
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
                    // 2. Если выделена группа деталей и пользователь тянет за любую из выделенных деталей (или внутри габарита группы) — двигаем всю группу целиком!
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

                // 3. Если не попали в уже выделенную деталь/ручку — проверяем клик по другой детали или начало рамки группового выделения
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
                        // Начали тянуть на пустом месте -> запускаем рамку группового выделения (Marquee Box Selection)!
                        let base_sel = if multi_mod {
                            self.selected_parts.clone()
                        } else {
                            self.set_single_selection(selected_part, None);
                            Vec::new()
                        };
                        self.box_select_drag = Some((vp.axis, click_uv, click_uv, multi_mod, base_sel));
                    }
                }

                // Запускаем перетаскивание (одиночное или всей выделенной группы)
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

        // Обычный клик без перетаскивания — выбор детали (с поддержкой Shift/Ctrl для группового выделения)
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

        // Обновление рамки группового выделения (Marquee Box Selection)
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

                // Ищем все видимые незаблокированные детали, попадающие в рамку выделения
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
                    painter.text(
                        box_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        format!("Выделено: {} шт.", self.selected_parts.len()),
                        FontId::proportional(11.0),
                        col,
                    );
                }
            }
        }

        // Обновление активного перетаскивания (одиночного или группового) / ресайза
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
                            // Ресайз отдельных сторон одиночной детали
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

                    // Применяем новые 2D-координаты к 3D-блоку (или группе блоков)
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

        // Отрисовка магнитных направляющих линий привязки (Snap Guides)
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

    /// Отрисовка и интерактивность полос прокрутки (горизонтальной и вертикальной) в 2D-окнах в стиле Hammer Editor / J.A.C.K.
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

        // Угловой квадратик между полосами прокрутки
        ui.painter_at(corner_rect).rect_filled(corner_rect, 0.0, btn_bg);

        // --- 1. Горизонтальная полоса прокрутки ---
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

            // 3 насечки посередине ползунка как в Hammer Editor (|||)
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

        // --- 2. Вертикальная полоса прокрутки ---
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
            // Вверх на экране — это +V (world_max_v), вниз на экране — это -V (world_min_v)
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

            // 3 горизонтальные насечки посередине ползунка (≡)
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

        let candidates = [
            (r.left_top(), DragHandle2D::ResizeTopLeft),
            (Pos2::new(r.center().x, r.top()), DragHandle2D::ResizeTop),
            (r.right_top(), DragHandle2D::ResizeTopRight),
            (Pos2::new(r.right(), r.center().y), DragHandle2D::ResizeRight),
            (r.right_bottom(), DragHandle2D::ResizeBottomRight),
            (Pos2::new(r.center().x, r.bottom()), DragHandle2D::ResizeBottom),
            (r.left_bottom(), DragHandle2D::ResizeBottomLeft),
            (Pos2::new(r.left(), r.center().y), DragHandle2D::ResizeLeft),
        ];

        for (pt, handle) in candidates {
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
        // Ищем деталь с наименьшей площадью в проекции (чтобы тонкие торцы полок/стоек легко выбирались поверх задней стенки)
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

    /// Создает новую деталь по нарисованной рамке в 2D проекции, автоматически подставляя толщину активного материала (напр. 18 мм)
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

        // Берем базовую 3-ю координату от выделенной детали (или 0.0)
        let ref_part = reference_part_id.and_then(|id| project.parts.iter().find(|p| p.id == id));
        let default_depth_y = ref_part.map(|p| p.width_mm).unwrap_or(500.0);

        let orientation = self.create_orientation_override.unwrap_or_else(|| {
            match axis {
                OrthoAxis::TopXY => PartOrientation::HorizontalXY,
                OrthoAxis::FrontXZ => {
                    if size_uv.x <= thick * 2.2 && size_uv.y > size_uv.x * 2.0 {
                        PartOrientation::VerticalYZ // Узкая вертикальная рамка -> Боковая стойка
                    } else if size_uv.y <= thick * 2.2 && size_uv.x > size_uv.y * 2.0 {
                        PartOrientation::HorizontalXY // Узкая горизонтальная рамка -> Полка
                    } else {
                        PartOrientation::FrontalXZ // Широкая рамка -> Фасад / Задняя стенка
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

        let (pos_3d, length_mm, width_mm, name_prefix) = match (axis, orientation) {
            (OrthoAxis::TopXY, PartOrientation::HorizontalXY) => (
                Vec3::new(min_uv.x, min_uv.y, 0.0),
                size_uv.x,
                size_uv.y,
                "Полка",
            ),
            (OrthoAxis::TopXY, PartOrientation::VerticalYZ) => (
                Vec3::new(min_uv.x, min_uv.y, 0.0),
                720.0,
                size_uv.y,
                "Стойка",
            ),
            (OrthoAxis::TopXY, PartOrientation::FrontalXZ) => (
                Vec3::new(min_uv.x, min_uv.y, 0.0),
                720.0,
                size_uv.x,
                "Панель",
            ),
            (OrthoAxis::FrontXZ, PartOrientation::HorizontalXY) => (
                Vec3::new(min_uv.x, 0.0, min_uv.y),
                size_uv.x,
                default_depth_y,
                "Полка",
            ),
            (OrthoAxis::FrontXZ, PartOrientation::VerticalYZ) => (
                Vec3::new(min_uv.x, 0.0, min_uv.y),
                size_uv.y,
                default_depth_y,
                "Стойка",
            ),
            (OrthoAxis::FrontXZ, PartOrientation::FrontalXZ) => (
                Vec3::new(min_uv.x, 0.0, min_uv.y),
                size_uv.y,
                size_uv.x,
                "Фасад/Панель",
            ),
            (OrthoAxis::SideYZ, PartOrientation::VerticalYZ) => (
                Vec3::new(0.0, min_uv.x, min_uv.y),
                size_uv.y,
                size_uv.x,
                "Стойка",
            ),
            (OrthoAxis::SideYZ, PartOrientation::HorizontalXY) => (
                Vec3::new(0.0, min_uv.x, min_uv.y),
                600.0,
                size_uv.x,
                "Полка",
            ),
            (OrthoAxis::SideYZ, PartOrientation::FrontalXZ) => (
                Vec3::new(0.0, min_uv.x, min_uv.y),
                size_uv.y,
                600.0,
                "Панель",
            ),
        };

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

        // Кнопка разворота окна на весь экран
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

    /// Отрисовка интерактивного 3D-окна (Перспектива, перспективно-корректные текстуры/цвета, кромки, вращение Orbit и полет Hammer Editor Z-Noclip)
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

        // Фон 3D сцены
        painter.rect_filled(rect, 0.0, Color32::from_rgb(22, 26, 32));

        // Переключение режима Hammer Editor Noclip по нажатию чистой клавиши `Z` (БЕЗ Ctrl/Alt) только когда курсор над 3D-окном!
        let text_input_busy = ui.ctx().wants_keyboard_input();
        let pointer_in_3d = ui
            .input(|i| i.pointer.hover_pos())
            .map(|p| rect.contains(p))
            .unwrap_or(false)
            || resp.hovered();

        if !text_input_busy && (pointer_in_3d || self.cam_3d.noclip_active) {
            let pure_z_pressed = ui.input(|i| {
                i.key_pressed(Key::Z)
                    && !i.modifiers.ctrl
                    && !i.modifiers.alt
                    && !i.modifiers.command
            });
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

        // --- Управление 3D камерой ---
        if self.cam_3d.noclip_active {
            // Скрываем курсор и удерживаем его в центре 3D-окна, чтобы вид вращался бесконечно
            ui.ctx().set_cursor_icon(egui::CursorIcon::None);
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::CursorVisible(false));
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::CursorPosition(rect.center()));

            // Режим Hammer Editor Z-Noclip: берем сырое смещение мыши из DeviceEvent::MouseMotion (pointer.motion)
            let raw_motion = ui
                .input(|i| i.pointer.motion())
                .unwrap_or_else(|| ui.input(|i| i.pointer.delta()));
            if raw_motion.length_sq() > 0.0 {
                let sensitivity = 0.0032;
                self.cam_3d.yaw_rad -= raw_motion.x * sensitivity;
                self.cam_3d.pitch_rad =
                    (self.cam_3d.pitch_rad - raw_motion.y * sensitivity).clamp(-1.52, 1.52);
            }

            // Колесико мыши регулирует скорость полета в Noclip
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
                if i.key_down(Key::W) || i.key_down(Key::ArrowUp) {
                    move_vec += fwd;
                }
                if i.key_down(Key::S) || i.key_down(Key::ArrowDown) {
                    move_vec -= fwd;
                }
                if i.key_down(Key::D) || i.key_down(Key::ArrowRight) {
                    move_vec += right;
                }
                if i.key_down(Key::A) || i.key_down(Key::ArrowLeft) {
                    move_vec -= right;
                }
                if i.key_down(Key::Space) || i.key_down(Key::E) {
                    move_vec += Vec3::Z;
                }
                if i.key_down(Key::Q) || i.modifiers.ctrl {
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

            // Запрашиваем перерисовку кадра для плавного полета WASD
            ui.ctx().request_repaint();
        } else {
            // Обычный режим CAD Orbit:
            // Вращение камеры зажатой ПКМ или ЛКМ по фону (или СКМ)
            if resp.dragged_by(egui::PointerButton::Secondary)
                || (resp.dragged_by(egui::PointerButton::Primary) && !ui.input(|i| i.modifiers.shift))
            {
                let d = resp.drag_delta();
                self.cam_3d.yaw_rad -= d.x * 0.007;
                self.cam_3d.pitch_rad = (self.cam_3d.pitch_rad - d.y * 0.007).clamp(-1.52, 1.52);
                self.cam_3d.sync_eye_from_orbit();
            }

            // Панорамирование камеры (СКМ или Shift + ПКМ/ЛКМ)
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

            // Зум колесиком мыши в 3D
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

        // 1. Отрисовка базовой координатной сетки пола Z=0 в 3D
        self.draw_3d_floor_grid(&painter, rect, &vp_mat);

        // 2. Сбор всех граней 3D-блоков с отсечением скрытых стыков (Internal Abutment Culling),
        // адаптивным разбиением (Subdivision) и топологической сортировкой плоскостей (Newell's Visibility Sort)
        // для полного устранения Z-fighting и ошибок перекрытия полок/стоек/задней стенки!
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
            fill_color: Color32,
            stroke_color: Color32,
            is_selected: bool,
            /// Точные UV-координаты 4 углов тайла (с учетом ориентации волокон и физического масштаба торцов)
            uv_corners: [Pos2; 4],
            /// Какие из 4 ребер этого тайла лежат на внешней границе детали (для отрисовки контура и кромки)
            outer_edges: [bool; 4],
            edge_strokes: [Option<Stroke>; 4],
            grain_segment: Option<(Pos2, Pos2)>,
        }

        let mut faces_to_draw: Vec<Face3DToDraw> = Vec::new();
        let light_dir = Vec3::new(-0.45, -0.65, 0.62).normalize();
        let eye = self.cam_3d.eye_mm;

        // Предварительно собираем AABB всех видимых деталей для отсечения внутренних глухих стыков (например, торец полки, упертый в боковину)
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

        // Индексы 6 граней куба с единообразным обходом сторон прямоугольника:
        // 0 (-Z) и 1 (+Z): ребро 0->1 вдоль +X, ребро 1->2 вдоль +Y
        // 2 (-Y) и 3 (+Y): ребро 0->1 вдоль +X, ребро 1->2 вдоль +Z
        // 4 (-X) и 5 (+X): ребро 0->1 вдоль +Y, ребро 1->2 вдоль +Z
        let cube_faces: [([usize; 4], Vec3); 6] = [
            ([0, 1, 2, 3], Vec3::new(0.0, 0.0, -1.0)), // Нижняя (-Z)
            ([4, 5, 6, 7], Vec3::new(0.0, 0.0, 1.0)),  // Верхняя (+Z)
            ([0, 1, 5, 4], Vec3::new(0.0, -1.0, 0.0)), // Задняя (-Y)
            ([3, 2, 6, 7], Vec3::new(0.0, 1.0, 0.0)),  // Передняя (+Y)
            ([0, 3, 7, 4], Vec3::new(-1.0, 0.0, 0.0)), // Левая (-X)
            ([1, 2, 6, 5], Vec3::new(1.0, 0.0, 0.0)),  // Правая (+X)
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

                // 1) Отсечение нелицевых граней (Backface Culling)
                let to_cam = eye - face_center;
                if world_norm.dot(to_cam) <= 0.0 {
                    continue;
                }

                // 2) Отсечение скрытых торцов на глухих стыках (Internal Abutment Culling):
                // Если эта грань полностью прижата вплотную к другой детали (и деталь не выделена),
                // не рисуем этот внутренний торец, чтобы он никогда не давал Z-fighting сквозь боковину или крышку!
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

                // Адаптивное разбиение крупных граней на сетку (например, боковина/полка/задняя стенка),
                // чтобы при Т-образных примыканиях (полка посередине стойки) каждый участок грани сортировался корректно!
                let len_u = q0.distance(q1);
                let len_v = q1.distance(q2);
                let div_u = ((len_u / 260.0).ceil() as usize).clamp(1, 4);
                let div_v = ((len_v / 260.0).ceil() as usize).clamp(1, 4);

                // Вычисляем физический масштаб UV на этой грани (чтобы на узких торцах 18 мм текстура не сжималась в 40 раз)
                // и ориентируем ось U текстуры строго вдоль направления волокон детали!
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

                // Определяем, идет ли направление волокон вдоль оси V этой грани (тогда поворачиваем UV на 90°)
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

                        let outer_edges = [
                            iv == 0,         // ребро 0..1 (верхнее по V)
                            iu + 1 == div_u, // ребро 1..2 (правое по U)
                            iv + 1 == div_v, // ребро 2..3 (нижнее по V)
                            iu == 0,         // ребро 3..0 (левое по U)
                        ];

                        // Рисуем стрелку волокон только на центральном тайле главной пласти
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

        // Сначала сортируем по убыванию расстояния до камеры (базовый порядок от дальних к ближним)
        faces_to_draw.sort_by(|a, b| {
            b.avg_depth
                .partial_cmp(&a.avg_depth)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Затем уточняем порядок методом разделяющих плоскостей Ньюэлла (Newell's Separating Plane Visibility Sort):
        // Для любых двух граней, чьи 2D-проекции перекрываются на экране, проверяем их взаимное положение относительно плоскостей друг друга и камеры!
        let n_faces = faces_to_draw.len();
        for i in 0..n_faces {
            for j in (i + 1)..n_faces {
                if faces_to_draw[i].part_id == faces_to_draw[j].part_id {
                    continue;
                }
                if !faces_to_draw[i]
                    .screen_bbox
                    .intersects(faces_to_draw[j].screen_bbox)
                {
                    continue;
                }

                // Проверяем: должна ли грань `i` рисоваться ПОЗЖЕ (т.е. ПОВЕРХ) грани `j`?
                let i_in_front_of_j = is_face_a_in_front_of_b(
                    &faces_to_draw[i].world_verts,
                    faces_to_draw[i].world_norm,
                    faces_to_draw[i].plane_d,
                    &faces_to_draw[j].world_verts,
                    faces_to_draw[j].world_norm,
                    faces_to_draw[j].plane_d,
                    eye,
                );
                if i_in_front_of_j {
                    faces_to_draw.swap(i, j);
                }
            }
        }

        // 3. Отрисовка граней (с перспективно-корректной 3D-разбивкой текстур PNG/JPG без искажений PS1, либо заливкой цветом материала)
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
                    Stroke::new(0.4, face.fill_color), // Скрывает микро-щели сглаживания между соседними тайлами одной грани
                ));
            }

            if self.cam_3d.wireframe_overlay || face.is_selected {
                for i in 0..4 {
                    // Рисуем контурные линии и кромки только по внешним ребрам детали (не рисуем внутреннюю сетку разбиения тайлов)
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

        // 4. Выделение детали левым кликом мыши (ЛКМ) в 3D-окне (с поддержкой Shift/Ctrl для группового выделения):
        // - В режиме полета Z-Noclip: ЛКМ выделяет деталь прямо под центральным прицелом (Crosshair)!
        // - В обычном режиме Orbit: ЛКМ выделяет деталь под курсором мыши
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

        // Прицел в центре 3D-экрана в режиме Hammer Editor Z-Noclip
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

        // 5. Рамка и информационный заголовок 3D-окна
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

        // Кнопка разворота 3D-окна на весь экран
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

    /// Отрисовывает текстурированный 3D-четырехугольник с перспективно-корректной 3D-тесселяцией и симметричным X-разбиением через центр.
    /// Полностью устраняет эффект «плавающей/ломаной текстуры PlayStation 1» (аффинного искажения в экранном пространстве `egui::Mesh`).
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
        // Выбираем плотность перспективно-корректной сетки по экранному размеру тайла и перепаду глубины W
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

        // Добавляем узлы сетки в буфер вершин
        for &(pos, uv) in &grid_scr {
            mesh.vertices.push(egui::epaint::Vertex {
                pos,
                uv,
                color: tint_color,
            });
        }

        // Для каждой микро-ячейки добавляем её истинный 3D-спроецированный центр и строим 4 симметричных треугольника (X-разбиение),
        // чтобы убрать диагональный излом текстуры!
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

        // Оси X (красная), Y (зеленая), Z (синяя) из точки (0,0,0) длиной 250 мм
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

    /// Проверяет, соответствует ли грань `f_idx` (0:-Z, 1:+Z, 2:-Y, 3:+Y, 4:-X, 5:+X) одному из 4 кромленных торцов детали
    fn face_edge_id_for_part(&self, part: &PartBlock, f_idx: usize) -> Option<EdgeId> {
        match part.orientation {
            PartOrientation::HorizontalXY => match f_idx {
                3 => part.edges.l1, // Передняя грань (+Y) = Д1
                2 => part.edges.l2, // Задняя грань (-Y) = Д2
                4 => part.edges.w1, // Левый торец (-X) = Ш1
                5 => part.edges.w2, // Правый торец (+X) = Ш2
                _ => None,
            },
            PartOrientation::VerticalYZ => match f_idx {
                3 => part.edges.l1, // Передняя грань (+Y) = Д1
                2 => part.edges.l2, // Задняя грань (-Y) = Д2
                0 => part.edges.w1, // Нижний торец (-Z) = Ш1
                1 => part.edges.w2, // Верхний торец (+Z) = Ш2
                _ => None,
            },
            PartOrientation::FrontalXZ => match f_idx {
                4 => part.edges.l1, // Левая грань (-X) = Д1
                5 => part.edges.l2, // Правая грань (+X) = Д2
                0 => part.edges.w1, // Нижний торец (-Z) = Ш1
                1 => part.edges.w2, // Верхний торец (+Z) = Ш2
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

    /// Управляет захватом и видимостью системного курсора мыши в режиме Hammer Editor Z-Noclip
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

/// Проверка попадания 2D-точки внутрь выпуклого четырехугольника
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

/// Проверяет по критериям разделяющей плоскости Ньюэлла (Newell's Separating Plane Test):
/// перекрывает ли грань `A` грань `B` относительно глаза камеры `eye` (то есть `A` должна быть нарисована ПОВЕРХ `B`).
fn is_face_a_in_front_of_b(
    verts_a: &[Vec3; 4],
    norm_a: Vec3,
    plane_d_a: f32,
    verts_b: &[Vec3; 4],
    norm_b: Vec3,
    plane_d_b: f32,
    _eye: Vec3,
) -> bool {
    let eps = 0.25_f32;

    // 1. Относительно плоскости грани B:
    // Камера находится на лицевой стороне B: norm_b.dot(eye) - plane_d_b > 0.
    // Если ВСЕ вершины A лежат по ту же сторону от плоскости B, что и камера (>= -eps), и хотя бы одна строго ближе к камере (> eps),
    // значит A гарантированно перед B!
    let mut a_all_front_of_b = true;
    let mut a_all_behind_b = true;
    let mut a_strictly_front = false;
    let mut a_strictly_behind = false;

    for va in verts_a {
        let dist = norm_b.dot(*va) - plane_d_b;
        if dist < -eps {
            a_all_front_of_b = false;
        }
        if dist > eps {
            a_strictly_front = true;
            a_all_behind_b = false;
        }
        if dist < -eps {
            a_strictly_behind = true;
        }
    }

    if a_all_front_of_b && a_strictly_front {
        return true; // A полностью спереди плоскости B -> A поверх B
    }
    if a_all_behind_b && a_strictly_behind {
        return false; // A полностью позади плоскости B -> B поверх A
    }

    // 2. Относительно плоскости грани A:
    // Камера находится на лицевой стороне A: norm_a.dot(eye) - plane_d_a > 0.
    // Если ВСЕ вершины B лежат позади плоскости A (<= eps, противоположно от камеры), значит A перед B!
    let mut b_all_behind_a = true;
    let mut b_all_front_of_a = true;
    let mut b_strictly_behind = false;
    let mut b_strictly_front = false;

    for vb in verts_b {
        let dist = norm_a.dot(*vb) - plane_d_a;
        if dist > eps {
            b_all_behind_a = false;
            b_strictly_front = true;
        }
        if dist < -eps {
            b_all_front_of_a = false;
            b_strictly_behind = true;
        }
    }

    if b_all_behind_a && b_strictly_behind {
        return true; // B полностью позади плоскости A -> A поверх B
    }
    if b_all_front_of_a && b_strictly_front {
        return false; // B полностью спереди плоскости A -> B поверх A
    }

    false
}

