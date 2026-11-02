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
    /// Price per full sheet (numeric value, currency set elsewhere).
    #[serde(default)]
    pub price_per_sheet: f64,
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
            price_per_sheet: 0.0,
        }
    }
}

/// Extracts the base filename from a path string, properly handling both '/' and '\'
/// cross-platform on Windows, Linux, macOS, and WebAssembly.
pub fn extract_cross_platform_filename(path_str: &str) -> String {
    path_str
        .rsplit(['/', '\\'])
        .next()
        .filter(|part| !part.is_empty())
        .unwrap_or(path_str)
        .to_string()
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
    /// Price per linear meter (numeric value, currency set elsewhere).
    #[serde(default)]
    pub price_per_m: f64,
}

impl Default for EdgeBanding {
    fn default() -> Self {
        Self {
            id: 1,
            name: "PVC 2.0 mm".to_string(),
            thickness_mm: 2.0,
            color_rgb: [235, 85, 45],
            price_per_m: 0.0,
        }
    }
}

/// Curated harmonious furniture decor colors (wood tones, stone, matte decors).
pub const MATERIAL_PALETTE: [[u8; 3]; 16] = [
    [205, 160, 115], // Oak / Honey Wood
    [75, 80, 85],    // Graphite / Anthracite
    [230, 222, 208], // Cashmere / Sand Beige
    [130, 95, 65],   // Classic Walnut
    [145, 160, 135], // Muted Sage Olive
    [120, 140, 155], // Dusty Blue / Slate
    [185, 110, 85],  // Terracotta / Clay
    [65, 50, 42],    // Wenge / Dark Chocolate
    [225, 205, 175], // Bleached Birch
    [160, 162, 165], // Ash Grey
    [68, 92, 78],    // Deep Forest Green
    [185, 130, 80],  // Caramel Alder
    [140, 125, 120], // Smoked Taupe
    [190, 145, 75],  // Mustard / Ochre
    [85, 95, 115],   // Deep Navy Slate
    [242, 238, 230], // Warm Off-White / Alabaster
];

/// Curated high-contrast marker colors for edge bandings in 3D and cutting diagrams.
pub const EDGE_BANDING_PALETTE: [[u8; 3]; 16] = [
    [235, 85, 45],   // Vivid Orange
    [40, 190, 95],   // Emerald Green
    [35, 135, 245],  // Vivid Azure Blue
    [175, 70, 230],  // Purple / Violet
    [245, 170, 25],  // Amber Gold
    [235, 55, 75],   // Coral Red
    [20, 190, 210],  // Cyan / Turquoise
    [135, 205, 40],  // Lime
    [235, 60, 150],  // Hot Pink
    [110, 65, 225],  // Indigo
    [25, 160, 150],  // Teal
    [245, 215, 30],  // Bright Yellow
    [195, 30, 60],   // Ruby Crimson
    [50, 80, 240],   // Electric Cobalt
    [190, 115, 235], // Orchid Lavender
    [210, 95, 25],   // Burnt Orange
];

/// Human-perception weighted squared color distance (Red: 2, Green: 4, Blue: 3).
#[inline]
pub fn color_dist_sq(c1: [u8; 3], c2: [u8; 3]) -> u32 {
    let dr = (c1[0] as i32) - (c2[0] as i32);
    let dg = (c1[1] as i32) - (c2[1] as i32);
    let db = (c1[2] as i32) - (c2[2] as i32);
    (2 * dr * dr + 4 * dg * dg + 3 * db * db) as u32
}

/// Converts HSV color representation (H: [0.0..1.0], S: [0.0..1.0], V: [0.0..1.0]) to RGB `[u8; 3]`.
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let h = (h.fract() + 1.0).fract();
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);

    let c = v * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r_prime, g_prime, b_prime) = match (h * 6.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    [
        ((r_prime + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((g_prime + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((b_prime + m) * 255.0).round().clamp(0.0, 255.0) as u8,
    ]
}

/// Selects the best distinct furniture material decor color given existing materials.
/// Picks the curated palette color with maximum distance from existing materials,
/// or falls back to golden-ratio HSV generation if palette is exhausted.
pub fn generate_next_material_color(existing: &[Material], seed: u64) -> [u8; 3] {
    if existing.is_empty() {
        return MATERIAL_PALETTE[(seed as usize) % MATERIAL_PALETTE.len()];
    }

    let mut best_color = MATERIAL_PALETTE[0];
    let mut max_min_dist = 0;

    for &candidate in &MATERIAL_PALETTE {
        let min_dist = existing
            .iter()
            .map(|m| color_dist_sq(m.color_rgb, candidate))
            .min()
            .unwrap_or(u32::MAX);

        if min_dist > max_min_dist {
            max_min_dist = min_dist;
            best_color = candidate;
        }
    }

    if max_min_dist < 600 {
        let h = ((existing.len() as f32 + (seed as f32 * 0.17)) * 0.618_034).fract();
        let s = 0.35 + ((seed % 3) as f32) * 0.1;
        let v = 0.65 + (((seed / 3) % 3) as f32) * 0.1;
        hsv_to_rgb(h, s, v)
    } else {
        best_color
    }
}

/// Selects the best distinct high-contrast marker color for a newly created edge banding type.
pub fn generate_next_edge_color(existing: &[EdgeBanding], seed: u64) -> [u8; 3] {
    if existing.is_empty() {
        return EDGE_BANDING_PALETTE[(seed as usize) % EDGE_BANDING_PALETTE.len()];
    }

    let mut best_color = EDGE_BANDING_PALETTE[0];
    let mut max_min_dist = 0;

    for &candidate in &EDGE_BANDING_PALETTE {
        let min_dist = existing
            .iter()
            .map(|e| color_dist_sq(e.color_rgb, candidate))
            .min()
            .unwrap_or(u32::MAX);

        if min_dist > max_min_dist {
            max_min_dist = min_dist;
            best_color = candidate;
        }
    }

    if max_min_dist < 600 {
        let h = ((existing.len() as f32 + (seed as f32 * 0.23)) * 0.618_034).fract();
        let s = 0.85 + ((seed % 2) as f32) * 0.1;
        let v = 0.90 + (((seed / 2) % 2) as f32) * 0.05;
        hsv_to_rgb(h, s, v)
    } else {
        best_color
    }
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
        match self {
            PartOrientation::HorizontalXY => lang.t("orient.horizontal_xy"),
            PartOrientation::VerticalYZ => lang.t("orient.vertical_yz"),
            PartOrientation::FrontalXZ => lang.t("orient.frontal_xz"),
        }
    }

    pub fn short_label(&self) -> &'static str {
        self.short_label_i18n(crate::i18n::AppLanguage::Ru)
    }

    pub fn short_label_i18n(&self, lang: crate::i18n::AppLanguage) -> &'static str {
        match self {
            PartOrientation::HorizontalXY => lang.t("orient.short_xy"),
            PartOrientation::VerticalYZ => lang.t("orient.short_yz"),
            PartOrientation::FrontalXZ => lang.t("orient.short_xz"),
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

/// Optimization criteria / strategy for 2D guillotine sheet cutting
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CuttingAlgorithm {
    /// Maximizes overall sheet utilization and reduces waste (Best Short Side Fit)
    #[default]
    MinimizeRemnants,
    /// Groups parts into continuous lengthwise cuts, reducing cutting passes and tool travel
    MinimizeToolpath,
    /// Packs parts tightly into one corner to leave the largest single usable rectangular offcut
    LargeUsableOffcut,
    /// Standard 2-stage strip cutting: rips sheet into strips first, then cross-cuts (ideal for panel saws)
    StripCutting,
}

impl CuttingAlgorithm {
    pub const ALL: [CuttingAlgorithm; 4] = [
        CuttingAlgorithm::MinimizeRemnants,
        CuttingAlgorithm::MinimizeToolpath,
        CuttingAlgorithm::LargeUsableOffcut,
        CuttingAlgorithm::StripCutting,
    ];

    pub fn label_i18n(self, lang: AppLanguage) -> &'static str {
        match self {
            Self::MinimizeRemnants => lang.t("cutting.algo_min_remnants"),
            Self::MinimizeToolpath => lang.t("cutting.algo_min_toolpath"),
            Self::LargeUsableOffcut => lang.t("cutting.algo_large_offcut"),
            Self::StripCutting => lang.t("cutting.algo_strip_cutting"),
        }
    }
}

/// Guillotine sheet cutting settings.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CuttingSettings {
    /// Blade / router cutter kerf width (mm). Default is 6.0 mm (industry standard).
    pub kerf_mm: f32,
    /// Left trim margin of sheet (mm).
    pub trim_left_mm: f32,
    /// Right trim margin of sheet (mm).
    pub trim_right_mm: f32,
    /// Top trim margin of sheet (mm).
    pub trim_top_mm: f32,
    /// Bottom trim margin of sheet (mm).
    pub trim_bottom_mm: f32,
    /// Whether sheet trim margins are configured separately for each edge (L/R/T/B).
    /// Defaults to false (single unified trim margin for all 4 edges).
    #[serde(default)]
    pub separate_trim_margins: bool,
    /// Active 2D guillotine cutting algorithm / optimization strategy.
    #[serde(default)]
    pub algorithm: CuttingAlgorithm,
    /// Price per linear meter of cut.
    #[serde(default)]
    pub cut_price_per_m: f64,
    /// Whether to group identical parts in specification and cutting report.
    pub group_identical_parts: bool,
    /// Whether to display material textures on cutting diagram panels.
    #[serde(default)]
    pub show_textures_on_sheet: bool,
}

impl Default for CuttingSettings {
    fn default() -> Self {
        Self {
            kerf_mm: 6.0,
            trim_left_mm: 10.0,
            trim_right_mm: 10.0,
            trim_top_mm: 10.0,
            trim_bottom_mm: 10.0,
            separate_trim_margins: false,
            algorithm: CuttingAlgorithm::default(),
            cut_price_per_m: 0.0,
            group_identical_parts: true,
            show_textures_on_sheet: false,
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

/// Strips trailing copy suffixes such as "(copy)", "(copy 2)", "(копия)", "(копия 3)", etc.
pub fn strip_copy_suffixes(s: &str) -> &str {
    let copy_patterns = [
        "copy", "копия", "копія", "Kopie", "copie", "copia", "cópia", "kopia",
    ];
    let mut current = s.trim_end();
    loop {
        if !current.ends_with(')') {
            break;
        }
        if let Some(open_idx) = current.rfind('(') {
            let inside = current[open_idx + 1..current.len() - 1].trim();
            let mut matches_copy = false;
            for pattern in &copy_patterns {
                if inside.eq_ignore_ascii_case(pattern) {
                    matches_copy = true;
                    break;
                }
                let inside_lower = inside.to_lowercase();
                let pat_lower = pattern.to_lowercase();
                if let Some(rest) = inside_lower.strip_prefix(&pat_lower) {
                    if let Some(stripped) = rest.strip_prefix(' ') {
                        if !stripped.is_empty() && stripped.chars().all(|c| c.is_ascii_digit()) {
                            matches_copy = true;
                            break;
                        }
                    }
                }
            }
            if matches_copy {
                current = current[..open_idx].trim_end();
                continue;
            }
        }
        break;
    }
    if current.is_empty() {
        s
    } else {
        current
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

    /// Generates a sequenced copy name: "Name (copy)", "Name (copy 2)", "Name (copy 3)", etc.,
    /// preventing exponential accumulation of repeated "(copy)" tags.
    pub fn generate_copy_name(&self, orig_name: &str, copy_word: &str) -> String {
        let base = strip_copy_suffixes(orig_name);
        let mut max_idx = 0usize;

        let copy_patterns = [
            "copy", "копия", "копія", "Kopie", "copie", "copia", "cópia", "kopia", copy_word,
        ];

        for part in &self.parts {
            let name = part.name.trim();
            if let Some(rest) = name.strip_prefix(base) {
                let rest = rest.trim();
                if rest.is_empty() {
                    continue;
                }
                if rest.starts_with('(') && rest.ends_with(')') {
                    let inside = rest[1..rest.len() - 1].trim();
                    for pat in &copy_patterns {
                        if inside.eq_ignore_ascii_case(pat) {
                            max_idx = max_idx.max(1);
                            break;
                        }
                        let inside_lower = inside.to_lowercase();
                        let pat_lower = pat.to_lowercase();
                        if let Some(num_str) = inside_lower.strip_prefix(&pat_lower) {
                            if let Some(digits) = num_str.strip_prefix(' ') {
                                if let Ok(n) = digits.trim().parse::<usize>() {
                                    max_idx = max_idx.max(n);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        if max_idx == 0 {
            format!("{} ({})", base, copy_word)
        } else {
            format!("{} ({} {})", base, copy_word, max_idx + 1)
        }
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
            price_per_sheet: 0.0,
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
            price_per_sheet: 0.0,
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
            price_per_sheet: 0.0,
        };

        let edge_pvc_2 = EdgeBanding {
            id: 10,
            name: l10n.edge_pvc_2.to_string(),
            thickness_mm: 2.0,
            color_rgb: [235, 85, 45], // Bright orange/red marker for 2 mm
            price_per_m: 0.0,
        };
        let edge_pvc_08 = EdgeBanding {
            id: 11,
            name: l10n.edge_pvc_08.to_string(),
            thickness_mm: 0.8,
            color_rgb: [50, 195, 95], // Green marker for 0.8 mm
            price_per_m: 0.0,
        };
        let edge_pvc_04 = EdgeBanding {
            id: 12,
            name: l10n.edge_pvc_04.to_string(),
            thickness_mm: 0.4,
            color_rgb: [65, 150, 245], // Blue marker for 0.4 mm
            price_per_m: 0.0,
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
pub struct DefaultProjectL10n {
    pub mat_white_18: &'static str,
    pub mat_oak_18: &'static str,
    pub mat_hdf_3: &'static str,
    pub edge_pvc_2: &'static str,
    pub edge_pvc_08: &'static str,
    pub edge_pvc_04: &'static str,
    pub grp_carcass: &'static str,
    pub grp_fronts: &'static str,
    pub grp_main: &'static str,
    pub proj_name: &'static str,
    pub part_left_side: &'static str,
    pub part_right_side: &'static str,
    pub part_bottom: &'static str,
    pub part_top: &'static str,
    pub part_shelf: &'static str,
    pub part_back: &'static str,
    pub part_door_left: &'static str,
    pub part_door_right: &'static str,
}

fn default_project_l10n(lang: AppLanguage) -> DefaultProjectL10n {
    crate::glossary::default_project_l10n(lang)
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
        assert_eq!(proj_ru.materials[0].name, "ЛДСП Белый Альпийский");
        assert_eq!(proj_ru.materials[1].name, "ЛДСП Дуб Вотан");
        assert_eq!(proj_ru.edge_bandings[0].name, "ПВХ");
        assert_eq!(proj_ru.groups[0].name, "Корпус шкафа (800x500x720)");
        assert_eq!(proj_ru.parts[0].name, "Боковина левая");

        // English localization
        let proj_en = Project::new_demo_cabinet_i18n(AppLanguage::En);
        assert_eq!(proj_en.materials[0].name, "Chipboard Alpine White");
        assert_eq!(proj_en.materials[1].name, "Chipboard Wotan Oak");
        assert_eq!(proj_en.edge_bandings[0].name, "PVC");
        assert_eq!(proj_en.groups[0].name, "Cabinet Carcass (800x500x720)");
        assert_eq!(proj_en.parts[0].name, "Left Side");

        // Ukrainian localization
        let proj_uk = Project::new_demo_cabinet_i18n(AppLanguage::Uk);
        assert_eq!(proj_uk.materials[0].name, "ЛДСП Білий Альпійський");
        assert_eq!(proj_uk.groups[0].name, "Корпус шафи (800x500x720)");
        assert_eq!(proj_uk.parts[0].name, "Боковина ліва");

        // German localization
        let proj_de = Project::new_demo_cabinet_i18n(AppLanguage::De);
        assert_eq!(proj_de.materials[0].name, "Spanplatte Alpinweiß");
        assert_eq!(proj_de.groups[0].name, "Schrankkorpus (800x500x720)");

        // Empty project localization
        let empty_en = Project::new_empty_i18n("Custom Desk", AppLanguage::En);
        assert_eq!(empty_en.name, "Custom Desk");
        assert_eq!(empty_en.groups[0].name, "Main Carcass");
        assert_eq!(empty_en.materials[0].name, "Chipboard Alpine White");
    }

    #[test]
    fn test_extract_cross_platform_filename() {
        assert_eq!(
            extract_cross_platform_filename("C:\\NonExistent\\dark_walnut.jpg"),
            "dark_walnut.jpg"
        );
        assert_eq!(
            extract_cross_platform_filename("/home/user/textures/oak.png"),
            "oak.png"
        );
        assert_eq!(
            extract_cross_platform_filename("textures/sub/pine.bmp"),
            "pine.bmp"
        );
        assert_eq!(
            extract_cross_platform_filename("textures\\sub\\pine.bmp"),
            "pine.bmp"
        );
        assert_eq!(
            extract_cross_platform_filename("single_file.jpg"),
            "single_file.jpg"
        );
        assert_eq!(extract_cross_platform_filename(""), "");
    }

    #[test]
    fn test_copy_name_generation() {
        assert_eq!(strip_copy_suffixes("Полка"), "Полка");
        assert_eq!(strip_copy_suffixes("Полка (copy)"), "Полка");
        assert_eq!(strip_copy_suffixes("Полка (copy 2)"), "Полка");
        assert_eq!(strip_copy_suffixes("Полка (copy 15)"), "Полка");
        assert_eq!(strip_copy_suffixes("Полка (копия)"), "Полка");
        assert_eq!(strip_copy_suffixes("Полка (копия 3)"), "Полка");
        assert_eq!(strip_copy_suffixes("Полка (copy) (copy) (copy)"), "Полка");
        assert_eq!(strip_copy_suffixes("Полка (18мм)"), "Полка (18мм)");

        let mut proj = Project::new_demo_cabinet();
        proj.parts.clear();

        // 1st copy -> "Полка (copy)"
        let name1 = proj.generate_copy_name("Полка", "copy");
        assert_eq!(name1, "Полка (copy)");

        // Add 1st copy
        let p1 = PartBlock {
            id: 1,
            name: "Полка (copy)".to_string(),
            group_id: None,
            material_id: 1,
            orientation: PartOrientation::HorizontalXY,
            position_mm: glam::Vec3::ZERO,
            length_mm: 500.0,
            width_mm: 300.0,
            yaw_deg: 0.0,
            grain_fixed: false,
            grain_along_width: false,
            edges: PartEdges::default(),
            drill_holes: Vec::new(),
            visible: true,
            locked: false,
        };
        proj.parts.push(p1);

        // 2nd copy from original or from copy -> "Полка (copy 2)"
        let name2 = proj.generate_copy_name("Полка", "copy");
        assert_eq!(name2, "Полка (copy 2)");
        let name2_from_copy = proj.generate_copy_name("Полка (copy)", "copy");
        assert_eq!(name2_from_copy, "Полка (copy 2)");

        // Add 2nd copy
        let p2 = PartBlock {
            id: 2,
            name: "Полка (copy 2)".to_string(),
            group_id: None,
            material_id: 1,
            orientation: PartOrientation::HorizontalXY,
            position_mm: glam::Vec3::ZERO,
            length_mm: 500.0,
            width_mm: 300.0,
            yaw_deg: 0.0,
            grain_fixed: false,
            grain_along_width: false,
            edges: PartEdges::default(),
            drill_holes: Vec::new(),
            visible: true,
            locked: false,
        };
        proj.parts.push(p2);

        // 3rd copy -> "Полка (copy 3)"
        let name3 = proj.generate_copy_name("Полка (copy 2)", "copy");
        assert_eq!(name3, "Полка (copy 3)");

        // Legacy repeated copy name -> collapses to "Полка (copy 3)"
        let name3_legacy = proj.generate_copy_name("Полка (copy) (copy)", "copy");
        assert_eq!(name3_legacy, "Полка (copy 3)");
    }

    #[test]
    fn test_cutting_settings_default_kerf_6() {
        let cs = CuttingSettings::default();
        assert_eq!(cs.kerf_mm, 6.0);

        // Deserializing empty JSON object should default to 6.0 mm kerf
        let empty_json = "{}";
        let deserialized: CuttingSettings = serde_json::from_str(empty_json).expect("Must deserialize default settings");
        assert_eq!(deserialized.kerf_mm, 6.0);
        assert_eq!(deserialized.trim_left_mm, 10.0);
    }

    #[test]
    fn test_color_generation_distinct() {
        let mut materials = Vec::new();
        // Add 5 materials sequentially and check that each gets a distinct color
        for i in 1..=5 {
            let color = generate_next_material_color(&materials, i);
            assert!(
                !materials.iter().any(|m: &Material| m.color_rgb == color),
                "Sequential material additions must generate distinct colors"
            );
            materials.push(Material {
                id: i,
                color_rgb: color,
                ..Default::default()
            });
        }

        let mut edges = Vec::new();
        // Add 5 edge bandings sequentially and check that each gets a distinct color
        for i in 1..=5 {
            let color = generate_next_edge_color(&edges, i);
            assert!(
                !edges.iter().any(|e: &EdgeBanding| e.color_rgb == color),
                "Sequential edge additions must generate distinct colors"
            );
            edges.push(EdgeBanding {
                id: i,
                color_rgb: color,
                ..Default::default()
            });
        }
    }

    #[test]
    fn test_export_sample_materials_project() {
        let mut proj = Project::new_demo_cabinet();
        for i in 1..=3 {
            let id = proj.alloc_id();
            let col = generate_next_material_color(&proj.materials, id);
            proj.materials.push(Material {
                id,
                name: format!("Материал декор {}", i),
                color_rgb: col,
                price_per_sheet: (3500 + i * 800) as f64,
                ..Default::default()
            });
        }
        for i in 1..=4 {
            let id = proj.alloc_id();
            let col = generate_next_edge_color(&proj.edge_bandings, id);
            proj.edge_bandings.push(EdgeBanding {
                id,
                name: format!("Кромка тип {}", i),
                color_rgb: col,
                thickness_mm: 1.0,
                price_per_m: (30 + i * 15) as f64,
            });
        }
        let json = serde_json::to_string_pretty(&proj).unwrap();
        let _ = std::fs::write("target/sample_materials.qfurn", json);
    }
}
