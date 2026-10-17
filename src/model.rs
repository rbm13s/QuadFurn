use glam::{Mat3, Vec2, Vec3};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Уникальные идентификаторы сущностей
pub type MaterialId = u64;
pub type EdgeId = u64;
pub type GroupId = u64;
pub type PartId = u64;

/// Справочник листового материала (ЛДСП, МДФ, ХДФ/ДВП)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Material {
    pub id: MaterialId,
    pub name: String,
    /// Толщина листа в миллиметрах (по умолчанию 18.0 мм)
    pub thickness_mm: f32,
    /// Цвет отображения в 2D/3D [R, G, B]
    pub color_rgb: [u8; 3],
    /// Опциональный путь к файлу текстуры (PNG/JPG)
    pub texture_path: Option<PathBuf>,
    /// Стандартная длина листа для раскроя (мм), например 2800.0
    pub sheet_length_mm: f32,
    /// Стандартная ширина листа для раскроя (мм), например 2070.0
    pub sheet_width_mm: f32,
    /// По умолчанию фиксировать направление волокон (текстуры) для новых деталей из этого материала
    pub default_grain_fixed: bool,
}

/// Справочник кромочного материала (ПВХ, АБС, Меламин)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EdgeBanding {
    pub id: EdgeId,
    pub name: String,
    /// Толщина кромки в миллиметрах (например 0.4, 0.8, 1.0, 2.0)
    pub thickness_mm: f32,
    /// Цвет индикации кромки на чертежах и в 3D [R, G, B]
    pub color_rgb: [u8; 3],
}

/// Базовая ориентация детали в пространстве шкафа
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartOrientation {
    /// Горизонтальная панель (Полка, Крышка, Дно): плоскость в XY, толщина по Z
    HorizontalXY,
    /// Профильная вертикальная панель (Боковая стойка, Перегородка): плоскость в YZ, толщина по X
    VerticalYZ,
    /// Фронтальная вертикальная панель (Фасад, Задняя стенка, Царга): плоскость в XZ, толщина по Y
    FrontalXZ,
}

impl PartOrientation {
    pub fn label(&self) -> &'static str {
        match self {
            PartOrientation::HorizontalXY => "Горизонтальная (Полка/Дно XY)",
            PartOrientation::VerticalYZ => "Боковая стойка (Профиль YZ)",
            PartOrientation::FrontalXZ => "Фронтальная (Фасад/Стенка XZ)",
        }
    }

    pub fn label_i18n(&self, lang: crate::i18n::AppLanguage) -> &'static str {
        match (self, lang) {
            (PartOrientation::HorizontalXY, crate::i18n::AppLanguage::Ru) => {
                "Горизонтальная (Полка/Дно XY)"
            }
            (PartOrientation::VerticalYZ, crate::i18n::AppLanguage::Ru) => {
                "Боковая стойка (Профиль YZ)"
            }
            (PartOrientation::FrontalXZ, crate::i18n::AppLanguage::Ru) => {
                "Фронтальная (Фасад/Стенка XZ)"
            }
            (PartOrientation::HorizontalXY, crate::i18n::AppLanguage::Uk) => {
                "Горизонтальна (Полиця/Дно XY)"
            }
            (PartOrientation::VerticalYZ, crate::i18n::AppLanguage::Uk) => {
                "Бокова стійка (Профіль YZ)"
            }
            (PartOrientation::FrontalXZ, crate::i18n::AppLanguage::Uk) => {
                "Фронтальна (Фасад/Стінка XZ)"
            }
            (PartOrientation::HorizontalXY, _) => "Horizontal (Shelf/Bottom XY)",
            (PartOrientation::VerticalYZ, _) => "Side Panel (Profile YZ)",
            (PartOrientation::FrontalXZ, _) => "Frontal (Door/Back XZ)",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            PartOrientation::HorizontalXY => "Полка (XY)",
            PartOrientation::VerticalYZ => "Стойка (YZ)",
            PartOrientation::FrontalXZ => "Фасад (XZ)",
        }
    }
}

/// Кромки по 4 сторонам детали:
/// Д1 и Д2 — вдоль длины детали (Length)
/// Ш1 и Ш2 — вдоль ширины детали (Width)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartEdges {
    /// Д1 (первая длинная сторона, напр. Перед / Фронт)
    pub l1: Option<EdgeId>,
    /// Д2 (вторая длинная сторона, напр. Зад / Тыл)
    pub l2: Option<EdgeId>,
    /// Ш1 (первая короткая сторона / торец 1)
    pub w1: Option<EdgeId>,
    /// Ш2 (вторая короткая сторона / торец 2)
    pub w2: Option<EdgeId>,
}

/// Задел второго этапа: тип крепежа / отверстия карты присадки
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FastenerType {
    Confirmate5x50,
    Dowel8x30,
    Minifix15,
    HingeCup35,
    ShelfPin5,
}

/// Задел второго этапа: отверстие присадки в локальных координатах детали
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DrillHole {
    pub fastener: FastenerType,
    pub local_pos_mm: Vec2,
    pub diameter_mm: f32,
    pub depth_mm: f32,
    pub on_edge: bool,
}

/// Группа деталей (папка в правом инспекторе)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartGroup {
    pub id: GroupId,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub expanded: bool,
}

/// Основной строительный блок — деталь корпусной мебели
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartBlock {
    pub id: PartId,
    pub name: String,
    pub group_id: Option<GroupId>,
    pub material_id: MaterialId,
    pub orientation: PartOrientation,
    /// Позиция базового угла детали (X, Y, Z) в миллиметрах от нуля сцены (0,0,0)
    /// При yaw_deg == 0 это минимальный угол (min_x, min_y, min_z)
    pub position_mm: Vec3,
    /// Чистовая (габаритная) Длина детали ВМЕСТЕ с кромкой (мм)
    /// Для Полки(XY): вдоль X; для Стойки(YZ): вдоль Z (высота); для Фасада(XZ): вдоль Z (высота)
    pub length_mm: f32,
    /// Чистовая (габаритная) Ширина детали ВМЕСТЕ с кромкой (мм)
    /// Для Полки(XY): вдоль Y (глубина); для Стойки(YZ): вдоль Y (глубина); для Фасада(XZ): вдоль X (ширина)
    pub width_mm: f32,
    /// Поворот детали вокруг вертикальной оси Z (в градусах, для угловых шкафов и скошенных полок)
    pub yaw_deg: f32,
    /// Фиксировать ли направление волокон (текстуры) при раскрое (не вращать на 90°)
    pub grain_fixed: bool,
    /// Если true, волокна идут вдоль Ширины вместо Длины (или можно просто поменять Д и Ш местами)
    pub grain_along_width: bool,
    /// Оклеивание кромкой по 4 граням (Д1, Д2, Ш1, Ш2)
    pub edges: PartEdges,
    /// Индивидуальная видимость и блокировка
    pub visible: bool,
    pub locked: bool,
    /// Задел под карту присадки (отверстия)
    #[serde(default)]
    pub drill_holes: Vec<DrillHole>,
}

impl PartBlock {
    /// Возвращает локальные габариты параллелепипеда (dX, dY, dZ) в мм до поворота yaw_deg
    pub fn local_box_size(&self, thickness_mm: f32) -> Vec3 {
        match self.orientation {
            PartOrientation::HorizontalXY => Vec3::new(self.length_mm, self.width_mm, thickness_mm),
            PartOrientation::VerticalYZ => Vec3::new(thickness_mm, self.width_mm, self.length_mm),
            PartOrientation::FrontalXZ => Vec3::new(self.width_mm, thickness_mm, self.length_mm),
        }
    }

    /// Устанавливает габариты детали из локального размера (dX, dY, dZ)
    pub fn set_from_local_box_size(&mut self, size: Vec3) {
        match self.orientation {
            PartOrientation::HorizontalXY => {
                self.length_mm = size.x.max(10.0);
                self.width_mm = size.y.max(10.0);
            }
            PartOrientation::VerticalYZ => {
                self.width_mm = size.y.max(10.0);
                self.length_mm = size.z.max(10.0);
            }
            PartOrientation::FrontalXZ => {
                self.width_mm = size.x.max(10.0);
                self.length_mm = size.z.max(10.0);
            }
        }
    }

    /// Центр детали в мировых координатах (мм)
    /// Вращение yaw_deg происходит вокруг вертикальной оси, проходящей через центр детали в XY
    pub fn world_center(&self, thickness_mm: f32) -> Vec3 {
        let size = self.local_box_size(thickness_mm);
        self.position_mm + size * 0.5
    }

    /// Переместить деталь так, чтобы ее мировой центр оказался в `new_center`
    pub fn set_world_center(&mut self, new_center: Vec3, thickness_mm: f32) {
        let size = self.local_box_size(thickness_mm);
        self.position_mm = new_center - size * 0.5;
    }

    /// Матрица вращения вокруг оси Z на угол yaw_deg
    pub fn rotation_matrix(&self) -> Mat3 {
        Mat3::from_rotation_z(self.yaw_deg.to_radians())
    }

    /// Возвращает 8 вершин параллелепипеда детали в мировых координатах (мм)
    /// Индексы:
    /// 0: (0,0,0), 1: (dx,0,0), 2: (dx,dy,0), 3: (0,dy,0) — нижняя грань (Z min)
    /// 4: (0,0,dz), 5: (dx,0,dz), 6: (dx,dy,dz), 7: (0,dy,dz) — верхняя грань (Z max)
    pub fn world_vertices(&self, thickness_mm: f32) -> [Vec3; 8] {
        let size = self.local_box_size(thickness_mm);
        let center = self.position_mm + size * 0.5;
        let half = size * 0.5;
        let rot = self.rotation_matrix();

        let local_corners = [
            Vec3::new(-half.x, -half.y, -half.z),
            Vec3::new(half.x, -half.y, -half.z),
            Vec3::new(half.x, half.y, -half.z),
            Vec3::new(-half.x, half.y, -half.z),
            Vec3::new(-half.x, -half.y, half.z),
            Vec3::new(half.x, -half.y, half.z),
            Vec3::new(half.x, half.y, half.z),
            Vec3::new(-half.x, half.y, half.z),
        ];

        let mut out = [Vec3::ZERO; 8];
        for (i, c) in local_corners.iter().enumerate() {
            out[i] = center + rot * (*c);
        }
        out
    }

    /// Осевой ограничивающий параллелепипед (AABB) детали в мировых координатах: (min, max)
    pub fn world_aabb(&self, thickness_mm: f32) -> (Vec3, Vec3) {
        if self.yaw_deg.abs() < 1e-3 {
            let size = self.local_box_size(thickness_mm);
            return (self.position_mm, self.position_mm + size);
        }
        let verts = self.world_vertices(thickness_mm);
        let mut min_v = verts[0];
        let mut max_v = verts[0];
        for v in &verts[1..] {
            min_v = min_v.min(*v);
            max_v = max_v.max(*v);
        }
        (min_v, max_v)
    }

    /// Толщины кромок по 4 сторонам (Д1, Д2, Ш1, Ш2) в миллиметрах
    pub fn edge_thicknesses(&self, edges_lib: &[EdgeBanding]) -> (f32, f32, f32, f32) {
        let find_t = |eid: Option<EdgeId>| -> f32 {
            eid.and_then(|id| edges_lib.iter().find(|e| e.id == id))
                .map(|e| e.thickness_mm)
                .unwrap_or(0.0)
        };
        (
            find_t(self.edges.l1),
            find_t(self.edges.l2),
            find_t(self.edges.w1),
            find_t(self.edges.w2),
        )
    }

    /// Пилильный размер детали (Длина пила, Ширина пила) в миллиметрах
    /// Вычитает толщину кромок из габаритного размера детали:
    /// L_cut = L_full - t(Д1) - t(Д2) (кромки вдоль длинных сторон вычитаются из Ширины, а торцевые Ш1/Ш2 — из Длины,
    /// ЛИБО по стандарту Cutting 3: кромка по Д1/Д2 наклеивается вдоль стороны Длины, значит уменьшает Ширину детали,
    /// а кромка по Ш1/Ш2 наклеивается на сторону Ширины, значит уменьшает Длину детали).
    /// Давайте сделаем прозрачно и однозначно:
    /// - Стороны Д1 и Д2 имеют длину `Length` (верхняя/нижняя длинные грани на чертеже детали) -> их толщина вычитается из `Width`!
    /// - Стороны Ш1 и Ш2 имеют длину `Width` (левый/правый торцы на чертеже детали) -> их толщина вычитается из `Length`!
    pub fn cut_dimensions_mm(&self, edges_lib: &[EdgeBanding]) -> (f32, f32) {
        let (t_l1, t_l2, t_w1, t_w2) = self.edge_thicknesses(edges_lib);
        let cut_length = (self.length_mm - t_w1 - t_w2).max(1.0);
        let cut_width = (self.width_mm - t_l1 - t_l2).max(1.0);
        (cut_length, cut_width)
    }
}

/// Настройки раскроя в стиле Cutting 3
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CuttingSettings {
    /// Ширина реза фрезы / пильного диска (мм)
    pub kerf_mm: f32,
    /// Отступ (обпил) от левого края листа (мм)
    pub trim_left_mm: f32,
    /// Отступ (обпил) от правого края листа (мм)
    pub trim_right_mm: f32,
    /// Отступ (обпил) от верхнего края листа (мм)
    pub trim_top_mm: f32,
    /// Отступ (обпил) от нижнего края листа (мм)
    pub trim_bottom_mm: f32,
    /// Группировать ли одинаковые детали в спецификации и отчете раскроя
    pub group_identical_parts: bool,
}

impl Default for CuttingSettings {
    fn default() -> Self {
        Self {
            kerf_mm: 4.0,
            trim_left_mm: 10.0,
            trim_right_mm: 10.0,
            trim_top_mm: 10.0,
            trim_bottom_mm: 10.0,
            group_identical_parts: true,
        }
    }
}

/// Полный проект мебели QuadFurn
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub materials: Vec<Material>,
    pub edge_bandings: Vec<EdgeBanding>,
    pub groups: Vec<PartGroup>,
    pub parts: Vec<PartBlock>,
    pub cutting_settings: CuttingSettings,
    pub next_id: u64,
}

impl Default for Project {
    fn default() -> Self {
        Self::new_demo_cabinet()
    }
}

impl Project {
    pub fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn get_material(&self, id: MaterialId) -> Option<&Material> {
        self.materials.iter().find(|m| m.id == id)
    }

    pub fn get_material_thickness(&self, id: MaterialId) -> f32 {
        self.get_material(id).map(|m| m.thickness_mm).unwrap_or(18.0)
    }

    pub fn get_edge(&self, id: EdgeId) -> Option<&EdgeBanding> {
        self.edge_bandings.iter().find(|e| e.id == id)
    }

    pub fn is_part_visible(&self, part: &PartBlock) -> bool {
        if !part.visible {
            return false;
        }
        if let Some(gid) = part.group_id {
            if let Some(g) = self.groups.iter().find(|g| g.id == gid) {
                return g.visible;
            }
        }
        true
    }

    pub fn is_part_locked(&self, part: &PartBlock) -> bool {
        if part.locked {
            return true;
        }
        if let Some(gid) = part.group_id {
            if let Some(g) = self.groups.iter().find(|g| g.id == gid) {
                return g.locked;
            }
        }
        false
    }

    /// Создает демонстрационный проект тумбы/шкафа из ЛДСП 18 мм с фасадами и задней стенкой ХДФ 3 мм
    pub fn new_demo_cabinet() -> Self {
        let mat_white_18 = Material {
            id: 1,
            name: "ЛДСП 18мм Белый Альпийский".to_string(),
            thickness_mm: 18.0,
            color_rgb: [235, 238, 242],
            texture_path: None,
            sheet_length_mm: 2800.0,
            sheet_width_mm: 2070.0,
            default_grain_fixed: false,
        };
        let mat_oak_18 = Material {
            id: 2,
            name: "ЛДСП 18мм Дуб Вотан".to_string(),
            thickness_mm: 18.0,
            color_rgb: [196, 148, 98],
            texture_path: None,
            sheet_length_mm: 2800.0,
            sheet_width_mm: 2070.0,
            default_grain_fixed: true,
        };
        let mat_hdf_3 = Material {
            id: 3,
            name: "ХДФ 3мм Белый (Задняя стенка)".to_string(),
            thickness_mm: 3.0,
            color_rgb: [210, 205, 195],
            texture_path: None,
            sheet_length_mm: 2800.0,
            sheet_width_mm: 2070.0,
            default_grain_fixed: false,
        };

        let edge_pvc_2 = EdgeBanding {
            id: 10,
            name: "ПВХ 2.0 мм".to_string(),
            thickness_mm: 2.0,
            color_rgb: [235, 85, 45], // Ярко-оранжевый/красный маркер для 2 мм
        };
        let edge_pvc_08 = EdgeBanding {
            id: 11,
            name: "ПВХ 0.8 мм".to_string(),
            thickness_mm: 0.8,
            color_rgb: [50, 195, 95], // Зеленый маркер для 0.8 мм
        };
        let edge_pvc_04 = EdgeBanding {
            id: 12,
            name: "ПВХ 0.4 мм".to_string(),
            thickness_mm: 0.4,
            color_rgb: [65, 150, 245], // Синий маркер для 0.4 мм
        };

        let grp_carcass = PartGroup {
            id: 100,
            name: "Корпус шкафа (800x500x720)".to_string(),
            visible: true,
            locked: false,
            expanded: true,
        };
        let grp_fronts = PartGroup {
            id: 101,
            name: "Фасады и наполнение".to_string(),
            visible: true,
            locked: false,
            expanded: true,
        };

        // Габариты корпуса: Ширина X = 800 мм, Глубина Y = 500 мм, Высота Z = 720 мм, ЛДСП 18 мм
        // Боковины: 720 x 500 x 18 (вкладное дно и крышка между боковинами: 800 - 36 = 764 мм)
        let parts = vec![
            PartBlock {
                id: 1001,
                name: "Боковина левая".to_string(),
                group_id: Some(100),
                material_id: 1,
                orientation: PartOrientation::VerticalYZ,
                position_mm: Vec3::new(0.0, 3.0, 0.0),
                length_mm: 720.0,
                width_mm: 500.0,
                yaw_deg: 0.0,
                grain_fixed: true,
                grain_along_width: false,
                edges: PartEdges {
                    l1: Some(10), // Передняя грань ПВХ 2.0 мм
                    l2: None,
                    w1: Some(12), // Нижний торец 0.4 мм
                    w2: Some(12), // Верхний торец 0.4 мм
                },
                visible: true,
                locked: false,
                drill_holes: Vec::new(),
            },
            PartBlock {
                id: 1002,
                name: "Боковина правая".to_string(),
                group_id: Some(100),
                material_id: 1,
                orientation: PartOrientation::VerticalYZ,
                position_mm: Vec3::new(800.0 - 18.0, 3.0, 0.0),
                length_mm: 720.0,
                width_mm: 500.0,
                yaw_deg: 0.0,
                grain_fixed: true,
                grain_along_width: false,
                edges: PartEdges {
                    l1: Some(10), // Передняя грань ПВХ 2.0 мм
                    l2: None,
                    w1: Some(12),
                    w2: Some(12),
                },
                visible: true,
                locked: false,
                drill_holes: Vec::new(),
            },
            PartBlock {
                id: 1003,
                name: "Дно вкладное".to_string(),
                group_id: Some(100),
                material_id: 1,
                orientation: PartOrientation::HorizontalXY,
                position_mm: Vec3::new(18.0, 3.0, 0.0),
                length_mm: 764.0,
                width_mm: 500.0,
                yaw_deg: 0.0,
                grain_fixed: true,
                grain_along_width: false,
                edges: PartEdges {
                    l1: Some(10), // Перед ПВХ 2.0 мм
                    l2: None,
                    w1: None,
                    w2: None,
                },
                visible: true,
                locked: false,
                drill_holes: Vec::new(),
            },
            PartBlock {
                id: 1004,
                name: "Крышка вкладная".to_string(),
                group_id: Some(100),
                material_id: 1,
                orientation: PartOrientation::HorizontalXY,
                position_mm: Vec3::new(18.0, 3.0, 720.0 - 18.0),
                length_mm: 764.0,
                width_mm: 500.0,
                yaw_deg: 0.0,
                grain_fixed: true,
                grain_along_width: false,
                edges: PartEdges {
                    l1: Some(10), // Перед ПВХ 2.0 мм
                    l2: None,
                    w1: None,
                    w2: None,
                },
                visible: true,
                locked: false,
                drill_holes: Vec::new(),
            },
            PartBlock {
                id: 1005,
                name: "Полка съемная".to_string(),
                group_id: Some(101),
                material_id: 1,
                orientation: PartOrientation::HorizontalXY,
                position_mm: Vec3::new(18.0, 3.0, 350.0),
                length_mm: 764.0,
                width_mm: 480.0,
                yaw_deg: 0.0,
                grain_fixed: true,
                grain_along_width: false,
                edges: PartEdges {
                    l1: Some(10), // Перед ПВХ 2.0 мм
                    l2: Some(12),
                    w1: Some(12),
                    w2: Some(12),
                },
                visible: true,
                locked: false,
                drill_holes: Vec::new(),
            },
            PartBlock {
                id: 1006,
                name: "Задняя стенка ХДФ".to_string(),
                group_id: Some(100),
                material_id: 3,
                orientation: PartOrientation::FrontalXZ,
                position_mm: Vec3::new(0.0, 0.0, 0.0),
                length_mm: 720.0,
                width_mm: 800.0,
                yaw_deg: 0.0,
                grain_fixed: false,
                grain_along_width: false,
                edges: PartEdges::default(),
                visible: true,
                locked: false,
                drill_holes: Vec::new(),
            },
            PartBlock {
                id: 1007,
                name: "Фасад левый (Дуб)".to_string(),
                group_id: Some(101),
                material_id: 2,
                orientation: PartOrientation::FrontalXZ,
                position_mm: Vec3::new(2.0, 503.0, 2.0),
                length_mm: 716.0,
                width_mm: 396.0,
                yaw_deg: 0.0,
                grain_fixed: true,
                grain_along_width: false,
                edges: PartEdges {
                    l1: Some(10),
                    l2: Some(10),
                    w1: Some(10),
                    w2: Some(10),
                },
                visible: true,
                locked: false,
                drill_holes: Vec::new(),
            },
            PartBlock {
                id: 1008,
                name: "Фасад правый (Дуб)".to_string(),
                group_id: Some(101),
                material_id: 2,
                orientation: PartOrientation::FrontalXZ,
                position_mm: Vec3::new(402.0, 503.0, 2.0),
                length_mm: 716.0,
                width_mm: 396.0,
                yaw_deg: 0.0,
                grain_fixed: true,
                grain_along_width: false,
                edges: PartEdges {
                    l1: Some(10),
                    l2: Some(10),
                    w1: Some(10),
                    w2: Some(10),
                },
                visible: true,
                locked: false,
                drill_holes: Vec::new(),
            },
        ];

        Self {
            name: "Тумба распашная 800х500х720 (ЛДСП 18мм)".to_string(),
            materials: vec![mat_white_18, mat_oak_18, mat_hdf_3],
            edge_bandings: vec![edge_pvc_2, edge_pvc_08, edge_pvc_04],
            groups: vec![grp_carcass, grp_fronts],
            parts,
            cutting_settings: CuttingSettings::default(),
            next_id: 2000,
        }
    }

    /// Создает новое пустое изделие с базовым справочником материалов ЛДСП 18 мм, ХДФ 3 мм и кромок ПВХ
    pub fn new_empty(name: impl Into<String>) -> Self {
        let mut p = Self::new_demo_cabinet();
        p.name = name.into();
        p.parts.clear();
        p.groups.clear();
        p.groups.push(PartGroup {
            id: 100,
            name: "Основной корпус".to_string(),
            visible: true,
            locked: false,
            expanded: true,
        });
        p
    }
}
