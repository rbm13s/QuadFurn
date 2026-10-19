use glam::{Mat3, Vec2, Vec3};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::i18n::AppLanguage;

/// Unique entity identifiers
pub type MaterialId = u64;
pub type EdgeId = u64;
pub type GroupId = u64;
pub type PartId = u64;

/// Sheet material specification (Particleboard, MDF, HDF / Hardboard).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Material {
    pub id: MaterialId,
    pub name: String,
    /// Sheet thickness in millimeters (default 18.0 mm).
    pub thickness_mm: f32,
    /// Display color in 2D/3D [R, G, B].
    pub color_rgb: [u8; 3],
    /// Optional path to texture image file (PNG/JPG).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture_path: Option<PathBuf>,
    /// Relative path to texture relative to project directory (Option A).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture_relative_path: Option<String>,
    /// Packed compressed texture bytes (PNG/JPG/WEBP) in Base64 format (Option B).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture_base64: Option<String>,
    /// Standard sheet length for cutting (mm), e.g. 2800.0.
    pub sheet_length_mm: f32,
    /// Standard sheet width for cutting (mm), e.g. 2070.0.
    pub sheet_width_mm: f32,
    /// Default fixed grain direction for new parts made from this material.
    pub default_grain_fixed: bool,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            id: 1,
            name: "Default Material".to_string(),
            thickness_mm: 18.0,
            color_rgb: [210, 180, 140],
            texture_path: None,
            texture_relative_path: None,
            texture_base64: None,
            sheet_length_mm: 2800.0,
            sheet_width_mm: 2070.0,
            default_grain_fixed: false,
        }
    }
}

/// Edge banding material specification (PVC, ABS, Melamine).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EdgeBanding {
    pub id: EdgeId,
    pub name: String,
    /// Edge thickness in millimeters (e.g. 0.4, 0.8, 1.0, 2.0).
    pub thickness_mm: f32,
    /// Edge marker color in drawings and 3D [R, G, B].
    pub color_rgb: [u8; 3],
}

/// Base spatial orientation of the part within the cabinet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartOrientation {
    /// Horizontal panel (Shelf, Top, Bottom): XY plane, thickness along Z.
    HorizontalXY,
    /// Profile vertical panel (Side panel, Partition): YZ plane, thickness along X.
    VerticalYZ,
    /// Frontal vertical panel (Door, Back panel, Plinth): XZ plane, thickness along Y.
    FrontalXZ,
}

impl PartOrientation {
    pub fn label(&self) -> &'static str {
        self.label_i18n(crate::i18n::AppLanguage::Ru)
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
        self.short_label_i18n(crate::i18n::AppLanguage::Ru)
    }

    pub fn short_label_i18n(&self, lang: crate::i18n::AppLanguage) -> &'static str {
        match (self, lang) {
            (PartOrientation::HorizontalXY, crate::i18n::AppLanguage::Ru) => "Полка (XY)",
            (PartOrientation::VerticalYZ, crate::i18n::AppLanguage::Ru) => "Стойка (YZ)",
            (PartOrientation::FrontalXZ, crate::i18n::AppLanguage::Ru) => "Фасад (XZ)",
            (PartOrientation::HorizontalXY, crate::i18n::AppLanguage::Uk) => "Полиця (XY)",
            (PartOrientation::VerticalYZ, crate::i18n::AppLanguage::Uk) => "Стійка (YZ)",
            (PartOrientation::FrontalXZ, crate::i18n::AppLanguage::Uk) => "Фасад (XZ)",
            (PartOrientation::HorizontalXY, _) => "Shelf (XY)",
            (PartOrientation::VerticalYZ, _) => "Side (YZ)",
            (PartOrientation::FrontalXZ, _) => "Door (XZ)",
        }
    }
}

/// Edge banding on 4 sides of the part:
/// L1 and L2 — along part length
/// W1 and W2 — along part width
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartEdges {
    /// L1 (first long side, e.g. Front)
    pub l1: Option<EdgeId>,
    /// L2 (second long side, e.g. Back)
    pub l2: Option<EdgeId>,
    /// W1 (first short edge / butt 1)
    pub w1: Option<EdgeId>,
    /// W2 (second short edge / butt 2)
    pub w2: Option<EdgeId>,
}

/// Fastener / hardware type for drilling map.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FastenerType {
    Confirmate5x50,
    Dowel8x30,
    Minifix15,
    HingeCup35,
    ShelfPin5,
}

/// Drill hole in part local coordinates.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DrillHole {
    pub fastener: FastenerType,
    pub local_pos_mm: Vec2,
    pub diameter_mm: f32,
    pub depth_mm: f32,
    pub on_edge: bool,
}

/// Part group (folder in inspector).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartGroup {
    pub id: GroupId,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub expanded: bool,
}

/// Main building block — furniture part block.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartBlock {
    pub id: PartId,
    pub name: String,
    pub group_id: Option<GroupId>,
    pub material_id: MaterialId,
    pub orientation: PartOrientation,
    /// Position of base corner (X, Y, Z) in millimeters from world origin (0,0,0).
    /// When yaw_deg == 0, this is the minimum corner (min_x, min_y, min_z).
    pub position_mm: Vec3,
    /// Finished (overall) length of part including edge banding (mm).
    /// For Shelf(XY): along X; for Side(YZ): along Z (height); for Door(XZ): along Z (height).
    pub length_mm: f32,
    /// Finished (overall) width of part including edge banding (mm).
    /// For Shelf(XY): along Y (depth); for Side(YZ): along Y (depth); for Door(XZ): along X (width).
    pub width_mm: f32,
    /// Rotation around vertical axis Z (in degrees, for corner cabinets and angled parts).
    pub yaw_deg: f32,
    /// Whether grain direction is fixed during cutting (do not rotate 90 degrees).
    pub grain_fixed: bool,
    /// If true, grain runs along Width instead of Length.
    pub grain_along_width: bool,
    /// Edge banding on 4 edges (L1, L2, W1, W2).
    pub edges: PartEdges,
    /// Individual visibility and lock state.
    pub visible: bool,
    pub locked: bool,
    /// Drill holes list.
    #[serde(default)]
    pub drill_holes: Vec<DrillHole>,
}

impl PartBlock {
    /// Returns local cuboid dimensions (dX, dY, dZ) in mm before yaw_deg rotation.
    pub fn local_box_size(&self, thickness_mm: f32) -> Vec3 {
        match self.orientation {
            PartOrientation::HorizontalXY => Vec3::new(self.length_mm, self.width_mm, thickness_mm),
            PartOrientation::VerticalYZ => Vec3::new(thickness_mm, self.width_mm, self.length_mm),
            PartOrientation::FrontalXZ => Vec3::new(self.width_mm, thickness_mm, self.length_mm),
        }
    }

    /// Sets part dimensions from local cuboid size (dX, dY, dZ).
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

    /// Center of part in world coordinates (mm).
    /// Rotation yaw_deg occurs around the vertical axis passing through the part XY center.
    pub fn world_center(&self, thickness_mm: f32) -> Vec3 {
        let size = self.local_box_size(thickness_mm);
        self.position_mm + size * 0.5
    }

    /// Moves the part so its world center matches `new_center`.
    pub fn set_world_center(&mut self, new_center: Vec3, thickness_mm: f32) {
        let size = self.local_box_size(thickness_mm);
        self.position_mm = new_center - size * 0.5;
    }

    /// Rotation matrix around Z axis by yaw_deg.
    pub fn rotation_matrix(&self) -> Mat3 {
        Mat3::from_rotation_z(self.yaw_deg.to_radians())
    }

    /// Returns 8 vertices of the part cuboid in world coordinates (mm).
    /// Indices:
    /// 0: (0,0,0), 1: (dx,0,0), 2: (dx,dy,0), 3: (0,dy,0) — bottom face (Z min)
    /// 4: (0,0,dz), 5: (dx,0,dz), 6: (dx,dy,dz), 7: (0,dy,dz) — top face (Z max)
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

    /// Axis-aligned bounding box (AABB) in world coordinates: (min, max).
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

    /// Edge banding thicknesses on 4 sides (L1, L2, W1, W2) in millimeters.
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

    /// Saw cutting dimensions (Length cut, Width cut) in millimeters.
    /// Subtracts edge banding thickness from overall dimensions:
    /// - L1 and L2 run along Length -> their thickness is subtracted from Width.
    /// - W1 and W2 run along Width -> their thickness is subtracted from Length.
    pub fn cut_dimensions_mm(&self, edges_lib: &[EdgeBanding]) -> (f32, f32) {
        let (t_l1, t_l2, t_w1, t_w2) = self.edge_thicknesses(edges_lib);
        let cut_length = (self.length_mm - t_w1 - t_w2).max(1.0);
        let cut_width = (self.width_mm - t_l1 - t_l2).max(1.0);
        (cut_length, cut_width)
    }
}

/// Guillotine sheet cutting settings.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CuttingSettings {
    /// Blade / router cutter kerf width (mm).
    pub kerf_mm: f32,
    /// Left trim margin of sheet (mm).
    pub trim_left_mm: f32,
    /// Right trim margin of sheet (mm).
    pub trim_right_mm: f32,
    /// Top trim margin of sheet (mm).
    pub trim_top_mm: f32,
    /// Bottom trim margin of sheet (mm).
    pub trim_bottom_mm: f32,
    /// Whether to group identical parts in specification and cutting report.
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

/// Complete QuadFurn furniture project.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub materials: Vec<Material>,
    pub edge_bandings: Vec<EdgeBanding>,
    pub groups: Vec<PartGroup>,
    pub parts: Vec<PartBlock>,
    pub cutting_settings: CuttingSettings,
    /// Flag: whether to embed textures as Base64 inside .qfurn file when saving (Option B).
    #[serde(default)]
    pub embed_textures: bool,
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

    /// Creates a demo cabinet project in 18 mm particleboard with doors and 3 mm HDF back,
    /// localized to the specified language.
    pub fn new_demo_cabinet_i18n(lang: AppLanguage) -> Self {
        let l10n = default_project_l10n(lang);
        let mat_white_18 = Material {
            id: 1,
            name: l10n.mat_white_18.to_string(),
            thickness_mm: 18.0,
            color_rgb: [235, 238, 242],
            texture_path: None,
            texture_relative_path: None,
            texture_base64: None,
            sheet_length_mm: 2800.0,
            sheet_width_mm: 2070.0,
            default_grain_fixed: false,
        };
        let mat_oak_18 = Material {
            id: 2,
            name: l10n.mat_oak_18.to_string(),
            thickness_mm: 18.0,
            color_rgb: [196, 148, 98],
            texture_path: None,
            texture_relative_path: None,
            texture_base64: None,
            sheet_length_mm: 2800.0,
            sheet_width_mm: 2070.0,
            default_grain_fixed: true,
        };
        let mat_hdf_3 = Material {
            id: 3,
            name: l10n.mat_hdf_3.to_string(),
            thickness_mm: 3.0,
            color_rgb: [210, 205, 195],
            texture_path: None,
            texture_relative_path: None,
            texture_base64: None,
            sheet_length_mm: 2800.0,
            sheet_width_mm: 2070.0,
            default_grain_fixed: false,
        };

        let edge_pvc_2 = EdgeBanding {
            id: 10,
            name: l10n.edge_pvc_2.to_string(),
            thickness_mm: 2.0,
            color_rgb: [235, 85, 45], // Bright orange/red marker for 2 mm
        };
        let edge_pvc_08 = EdgeBanding {
            id: 11,
            name: l10n.edge_pvc_08.to_string(),
            thickness_mm: 0.8,
            color_rgb: [50, 195, 95], // Green marker for 0.8 mm
        };
        let edge_pvc_04 = EdgeBanding {
            id: 12,
            name: l10n.edge_pvc_04.to_string(),
            thickness_mm: 0.4,
            color_rgb: [65, 150, 245], // Blue marker for 0.4 mm
        };

        let grp_carcass = PartGroup {
            id: 100,
            name: l10n.grp_carcass.to_string(),
            visible: true,
            locked: false,
            expanded: true,
        };
        let grp_fronts = PartGroup {
            id: 101,
            name: l10n.grp_fronts.to_string(),
            visible: true,
            locked: false,
            expanded: true,
        };

        // Carcass dimensions: Width X = 800 mm, Depth Y = 500 mm, Height Z = 720 mm, 18 mm board
        // Sides: 720 x 500 x 18 (inset bottom and top between sides: 800 - 36 = 764 mm)
        let parts = vec![
            PartBlock {
                id: 1001,
                name: l10n.part_left_side.to_string(),
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
                    l1: Some(10), // Front edge PVC 2.0 mm
                    l2: None,
                    w1: Some(12), // Bottom edge 0.4 mm
                    w2: Some(12), // Top edge 0.4 mm
                },
                visible: true,
                locked: false,
                drill_holes: Vec::new(),
            },
            PartBlock {
                id: 1002,
                name: l10n.part_right_side.to_string(),
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
                    l1: Some(10), // Front edge PVC 2.0 mm
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
                name: l10n.part_bottom.to_string(),
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
                    l1: Some(10), // Front PVC 2.0 mm
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
                name: l10n.part_top.to_string(),
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
                    l1: Some(10), // Front PVC 2.0 mm
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
                name: l10n.part_shelf.to_string(),
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
                    l1: Some(10), // Front PVC 2.0 mm
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
                name: l10n.part_back.to_string(),
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
                name: l10n.part_door_left.to_string(),
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
                name: l10n.part_door_right.to_string(),
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
            name: l10n.proj_name.to_string(),
            materials: vec![mat_white_18, mat_oak_18, mat_hdf_3],
            edge_bandings: vec![edge_pvc_2, edge_pvc_08, edge_pvc_04],
            groups: vec![grp_carcass, grp_fronts],
            parts,
            cutting_settings: CuttingSettings::default(),
            embed_textures: false,
            next_id: 2000,
        }
    }

    /// Creates a demo cabinet project in 18 mm particleboard with default Russian localization.
    pub fn new_demo_cabinet() -> Self {
        Self::new_demo_cabinet_i18n(AppLanguage::Ru)
    }

    /// Creates a new empty project with base material library localized to the specified language.
    pub fn new_empty_i18n(name: impl Into<String>, lang: AppLanguage) -> Self {
        let mut p = Self::new_demo_cabinet_i18n(lang);
        p.name = name.into();
        p.parts.clear();
        p.groups.clear();
        let l10n = default_project_l10n(lang);
        p.groups.push(PartGroup {
            id: 100,
            name: l10n.grp_main.to_string(),
            visible: true,
            locked: false,
            expanded: true,
        });
        p
    }

    /// Creates a new empty project with base material library (default Russian localization).
    pub fn new_empty(name: impl Into<String>) -> Self {
        Self::new_empty_i18n(name, AppLanguage::Ru)
    }
}

/// Localized names of default materials, edges, groups, and parts for new projects.
struct DefaultProjectL10n {
    mat_white_18: &'static str,
    mat_oak_18: &'static str,
    mat_hdf_3: &'static str,
    edge_pvc_2: &'static str,
    edge_pvc_08: &'static str,
    edge_pvc_04: &'static str,
    grp_carcass: &'static str,
    grp_fronts: &'static str,
    grp_main: &'static str,
    proj_name: &'static str,
    part_left_side: &'static str,
    part_right_side: &'static str,
    part_bottom: &'static str,
    part_top: &'static str,
    part_shelf: &'static str,
    part_back: &'static str,
    part_door_left: &'static str,
    part_door_right: &'static str,
}

fn default_project_l10n(lang: AppLanguage) -> DefaultProjectL10n {
    match lang {
        AppLanguage::Ru => DefaultProjectL10n {
            mat_white_18: "ЛДСП 18мм Белый Альпийский",
            mat_oak_18: "ЛДСП 18мм Дуб Вотан",
            mat_hdf_3: "ХДФ 3мм Белый (Задняя стенка)",
            edge_pvc_2: "ПВХ 2.0 мм",
            edge_pvc_08: "ПВХ 0.8 мм",
            edge_pvc_04: "ПВХ 0.4 мм",
            grp_carcass: "Корпус шкафа (800x500x720)",
            grp_fronts: "Фасады и наполнение",
            grp_main: "Основной корпус",
            proj_name: "Тумба распашная 800х500х720 (ЛДСП 18мм)",
            part_left_side: "Боковина левая",
            part_right_side: "Боковина правая",
            part_bottom: "Дно вкладное",
            part_top: "Крышка вкладная",
            part_shelf: "Полка съемная",
            part_back: "Задняя стенка ХДФ",
            part_door_left: "Фасад левый (Дуб)",
            part_door_right: "Фасад правый (Дуб)",
        },
        AppLanguage::Uk => DefaultProjectL10n {
            mat_white_18: "ЛДСП 18мм Білий Альпійський",
            mat_oak_18: "ЛДСП 18мм Дуб Вотан",
            mat_hdf_3: "ХДФ 3мм Білий (Задня стінка)",
            edge_pvc_2: "ПВХ 2.0 мм",
            edge_pvc_08: "ПВХ 0.8 мм",
            edge_pvc_04: "ПВХ 0.4 мм",
            grp_carcass: "Корпус шафи (800x500x720)",
            grp_fronts: "Фасади та наповнення",
            grp_main: "Основний корпус",
            proj_name: "Тумба розпашна 800х500х720 (ЛДСП 18мм)",
            part_left_side: "Боковина ліва",
            part_right_side: "Боковина права",
            part_bottom: "Дно вкладене",
            part_top: "Кришка вкладена",
            part_shelf: "Полиця знімна",
            part_back: "Задня стінка ХДФ",
            part_door_left: "Фасад лівий (Дуб)",
            part_door_right: "Фасад правий (Дуб)",
        },
        AppLanguage::De => DefaultProjectL10n {
            mat_white_18: "18mm Spanplatte Alpinweiß",
            mat_oak_18: "18mm Spanplatte Wotaneiche",
            mat_hdf_3: "3mm HDF Weiß (Rückwand)",
            edge_pvc_2: "PVC 2.0 mm",
            edge_pvc_08: "PVC 0.8 mm",
            edge_pvc_04: "PVC 0.4 mm",
            grp_carcass: "Schrankkorpus (800x500x720)",
            grp_fronts: "Fronten & Fächer",
            grp_main: "Hauptkorpus",
            proj_name: "Drehtürenschrank 800x500x720 (18mm)",
            part_left_side: "Linke Seitenwand",
            part_right_side: "Rechte Seitenwand",
            part_bottom: "Unterboden",
            part_top: "Oberboden",
            part_shelf: "Einlegeboden",
            part_back: "Rückwand HDF",
            part_door_left: "Front links (Eiche)",
            part_door_right: "Front rechts (Eiche)",
        },
        AppLanguage::Pl => DefaultProjectL10n {
            mat_white_18: "Płyta wiórowa 18mm Biały Alpejski",
            mat_oak_18: "Płyta wiórowa 18mm Dąb Wotan",
            mat_hdf_3: "HDF 3mm Biały (Ściana tylna)",
            edge_pvc_2: "PCV 2.0 mm",
            edge_pvc_08: "PCV 0.8 mm",
            edge_pvc_04: "PCV 0.4 mm",
            grp_carcass: "Korpus szafy (800x500x720)",
            grp_fronts: "Fronty i wypełnienie",
            grp_main: "Główny korpus",
            proj_name: "Szafka uchylna 800x500x720 (18mm)",
            part_left_side: "Bok lewy",
            part_right_side: "Bok prawy",
            part_bottom: "Wieniec dolny",
            part_top: "Wieniec górny",
            part_shelf: "Półka wyjmowana",
            part_back: "Ściana tylna HDF",
            part_door_left: "Front lewy (Dąb)",
            part_door_right: "Front prawy (Dąb)",
        },
        _ => DefaultProjectL10n {
            mat_white_18: "18mm Chipboard Alpine White",
            mat_oak_18: "18mm Chipboard Wotan Oak",
            mat_hdf_3: "3mm HDF White (Back panel)",
            edge_pvc_2: "PVC 2.0 mm",
            edge_pvc_08: "PVC 0.8 mm",
            edge_pvc_04: "PVC 0.4 mm",
            grp_carcass: "Cabinet Carcass (800x500x720)",
            grp_fronts: "Fronts & Infill",
            grp_main: "Main Carcass",
            proj_name: "Cabinet 800x500x720 (18mm Chipboard)",
            part_left_side: "Left Side",
            part_right_side: "Right Side",
            part_bottom: "Bottom Panel",
            part_top: "Top Panel",
            part_shelf: "Removable Shelf",
            part_back: "Back Panel HDF",
            part_door_left: "Left Door (Oak)",
            part_door_right: "Right Door (Oak)",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_texture_backward_compatibility() {
        // Legacy JSON format without embed_textures, texture_base64, and texture_relative_path
        let legacy_json = r#"{
            "name": "Тестовый шкаф",
            "materials": [
                {
                    "id": 1,
                    "name": "ЛДСП 18",
                    "thickness_mm": 18.0,
                    "color_rgb": [240, 240, 240],
                    "texture_path": "C:\\textures\\oak.png",
                    "sheet_length_mm": 2800.0,
                    "sheet_width_mm": 2070.0,
                    "default_grain_fixed": true
                }
            ],
            "edge_bandings": [],
            "groups": [],
            "parts": [],
            "cutting_settings": {
                "kerf_mm": 4.0,
                "trim_left_mm": 10.0,
                "trim_right_mm": 10.0,
                "trim_top_mm": 10.0,
                "trim_bottom_mm": 10.0,
                "group_identical_parts": true
            },
            "next_id": 10
        }"#;

        let proj: Result<Project, _> = serde_json::from_str(legacy_json);
        assert!(proj.is_ok(), "Legacy project must deserialize without error");
        let proj = proj.unwrap();
        assert!(!proj.embed_textures, "embed_textures must default to false");
        assert_eq!(proj.materials[0].texture_base64, None);
        assert_eq!(proj.materials[0].texture_relative_path, None);
        assert_eq!(
            proj.materials[0].texture_path.as_deref(),
            Some(std::path::Path::new("C:\\textures\\oak.png"))
        );
    }

    #[test]
    fn test_project_texture_embedding_roundtrip() {
        let mut proj = Project::new_demo_cabinet();
        proj.embed_textures = true;
        proj.materials[0].texture_base64 = Some("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAAAAAA6fptVAAAACklEQVR4nGNiAAAABgADNjd8qAAAAABJRU5ErkJggg==".to_string());
        proj.materials[0].texture_relative_path = Some("textures/oak.png".to_string());

        let json = serde_json::to_string_pretty(&proj).expect("Must serialize project");
        assert!(json.contains("embed_textures"));
        assert!(json.contains("texture_base64"));
        assert!(json.contains("texture_relative_path"));

        let restored: Project = serde_json::from_str(&json).expect("Must deserialize project");
        assert!(restored.embed_textures);
        assert_eq!(
            restored.materials[0].texture_base64.as_deref(),
            Some("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAAAAAA6fptVAAAACklEQVR4nGNiAAAABgADNjd8qAAAAABJRU5ErkJggg==")
        );
        assert_eq!(
            restored.materials[0].texture_relative_path.as_deref(),
            Some("textures/oak.png")
        );
    }

    #[test]
    fn test_project_default_materials_localization() {
        // Russian localization
        let proj_ru = Project::new_demo_cabinet_i18n(AppLanguage::Ru);
        assert_eq!(proj_ru.materials[0].name, "ЛДСП 18мм Белый Альпийский");
        assert_eq!(proj_ru.materials[1].name, "ЛДСП 18мм Дуб Вотан");
        assert_eq!(proj_ru.edge_bandings[0].name, "ПВХ 2.0 мм");
        assert_eq!(proj_ru.groups[0].name, "Корпус шкафа (800x500x720)");
        assert_eq!(proj_ru.parts[0].name, "Боковина левая");

        // English localization
        let proj_en = Project::new_demo_cabinet_i18n(AppLanguage::En);
        assert_eq!(proj_en.materials[0].name, "18mm Chipboard Alpine White");
        assert_eq!(proj_en.materials[1].name, "18mm Chipboard Wotan Oak");
        assert_eq!(proj_en.edge_bandings[0].name, "PVC 2.0 mm");
        assert_eq!(proj_en.groups[0].name, "Cabinet Carcass (800x500x720)");
        assert_eq!(proj_en.parts[0].name, "Left Side");

        // Ukrainian localization
        let proj_uk = Project::new_demo_cabinet_i18n(AppLanguage::Uk);
        assert_eq!(proj_uk.materials[0].name, "ЛДСП 18мм Білий Альпійський");
        assert_eq!(proj_uk.groups[0].name, "Корпус шафи (800x500x720)");
        assert_eq!(proj_uk.parts[0].name, "Боковина ліва");

        // German localization
        let proj_de = Project::new_demo_cabinet_i18n(AppLanguage::De);
        assert_eq!(proj_de.materials[0].name, "18mm Spanplatte Alpinweiß");
        assert_eq!(proj_de.groups[0].name, "Schrankkorpus (800x500x720)");

        // Empty project localization
        let empty_en = Project::new_empty_i18n("Custom Desk", AppLanguage::En);
        assert_eq!(empty_en.name, "Custom Desk");
        assert_eq!(empty_en.groups[0].name, "Main Carcass");
        assert_eq!(empty_en.materials[0].name, "18mm Chipboard Alpine White");
    }
}
