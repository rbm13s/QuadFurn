use std::path::PathBuf;
use glam::Vec3;
use crate::i18n::AppLanguage;
use crate::model::*;
use crate::{FurnitureDocument, QuadFurnApp, ThemeChoice};
use crate::ui_panels::AppTab;

pub const BATCH_LANGS: [(&str, AppLanguage); 22] = [
    ("ru", AppLanguage::Ru),
    ("en", AppLanguage::En),
    ("uk", AppLanguage::Uk),
    ("de", AppLanguage::De),
    ("pl", AppLanguage::Pl),
    ("es", AppLanguage::Es),
    ("fr", AppLanguage::Fr),
    ("it", AppLanguage::It),
    ("zh-cn", AppLanguage::ZhCn),
    ("ja", AppLanguage::Ja),
    ("ko", AppLanguage::Ko),
    ("pt", AppLanguage::Pt),
    ("cs", AppLanguage::Cs),
    ("sk", AppLanguage::Sk),
    ("tr", AppLanguage::Tr),
    ("nl", AppLanguage::Nl),
    ("da", AppLanguage::Da),
    ("no", AppLanguage::No),
    ("sv", AppLanguage::Sv),
    ("hu", AppLanguage::Hu),
    ("ro", AppLanguage::Ro),
    ("sl", AppLanguage::Sl),
];

pub struct LocalizedCabinetNames {
    pub proj_name: &'static str,
    pub mat_white: &'static str,
    pub edge_2: &'static str,
    pub edge_1: &'static str,
    pub edge_05: &'static str,
    pub grp_carcass: &'static str,
    pub grp_fronts: &'static str,
    pub p2000: &'static str,
    pub p2001: &'static str,
    pub p2002: &'static str,
    pub p2003: &'static str,
    pub p2004: &'static str,
    pub p2005: &'static str,
    pub p2006: &'static str,
    pub p2007: &'static str,
    pub p2008: &'static str,
    pub p2009: &'static str,
}

pub fn get_localized_cabinet_names(lang: AppLanguage) -> LocalizedCabinetNames {
    match lang {
        AppLanguage::Ru => LocalizedCabinetNames {
            proj_name: "Тумба в ванную",
            mat_white: "ЛДСП Белый",
            edge_2: "ПВХ 2.0 мм",
            edge_1: "ПВХ 1 мм",
            edge_05: "ПВХ 0.5 мм",
            grp_carcass: "Корпус",
            grp_fronts: "Фасады и наполнение",
            p2000: "Стенка правая нижняя",
            p2001: "Стенка левая",
            p2002: "Дно",
            p2003: "Крышка",
            p2004: "Полка 1",
            p2005: "Полка 2",
            p2006: "Полка 3",
            p2007: "Стенка правая верхняя",
            p2008: "Фасад левый",
            p2009: "Фасад правый",
        },
        AppLanguage::En => LocalizedCabinetNames {
            proj_name: "Bathroom Vanity",
            mat_white: "Chipboard White",
            edge_2: "PVC 2.0 mm",
            edge_1: "PVC 1.0 mm",
            edge_05: "PVC 0.5 mm",
            grp_carcass: "Carcass",
            grp_fronts: "Fronts and infill",
            p2000: "Right lower side",
            p2001: "Left side",
            p2002: "Bottom",
            p2003: "Top",
            p2004: "Shelf 1",
            p2005: "Shelf 2",
            p2006: "Shelf 3",
            p2007: "Right upper side",
            p2008: "Left front",
            p2009: "Right front",
        },
        AppLanguage::Uk => LocalizedCabinetNames {
            proj_name: "Тумба у ванну",
            mat_white: "ЛДСП Білий",
            edge_2: "ПВХ 2.0 мм",
            edge_1: "ПВХ 1 мм",
            edge_05: "ПВХ 0.5 мм",
            grp_carcass: "Корпус",
            grp_fronts: "Фасади та наповнення",
            p2000: "Стінка права нижня",
            p2001: "Стінка ліва",
            p2002: "Дно",
            p2003: "Кришка",
            p2004: "Полиця 1",
            p2005: "Полиця 2",
            p2006: "Полиця 3",
            p2007: "Стінка права верхня",
            p2008: "Фасад лівий",
            p2009: "Фасад правий",
        },
        AppLanguage::De => LocalizedCabinetNames {
            proj_name: "Badezimmerschrank",
            mat_white: "Spanplatte Weiß",
            edge_2: "PVC 2.0 mm",
            edge_1: "PVC 1.0 mm",
            edge_05: "PVC 0.5 mm",
            grp_carcass: "Korpus",
            grp_fronts: "Fronten und Fächer",
            p2000: "Rechte untere Seitenwand",
            p2001: "Linke Seitenwand",
            p2002: "Unterboden",
            p2003: "Oberboden",
            p2004: "Fachboden 1",
            p2005: "Fachboden 2",
            p2006: "Fachboden 3",
            p2007: "Rechte obere Seitenwand",
            p2008: "Front links",
            p2009: "Front rechts",
        },
        AppLanguage::Pl => LocalizedCabinetNames {
            proj_name: "Szafka łazienkowa",
            mat_white: "Płyta wiórowa Biała",
            edge_2: "PCV 2.0 mm",
            edge_1: "PCV 1.0 mm",
            edge_05: "PCV 0.5 mm",
            grp_carcass: "Korpus",
            grp_fronts: "Fronty i półki",
            p2000: "Bok prawy dolny",
            p2001: "Bok lewy",
            p2002: "Dno",
            p2003: "Wieniec górny",
            p2004: "Półka 1",
            p2005: "Półka 2",
            p2006: "Półka 3",
            p2007: "Bok prawy górny",
            p2008: "Front lewy",
            p2009: "Front prawy",
        },
        AppLanguage::Es => LocalizedCabinetNames {
            proj_name: "Mueble de baño",
            mat_white: "Aglomerado Blanco",
            edge_2: "PVC 2.0 mm",
            edge_1: "PVC 1.0 mm",
            edge_05: "PVC 0.5 mm",
            grp_carcass: "Estructura",
            grp_fronts: "Frentes y baldas",
            p2000: "Costado derecho inferior",
            p2001: "Costado izquierdo",
            p2002: "Fondo inferior",
            p2003: "Tapa superior",
            p2004: "Balda 1",
            p2005: "Balda 2",
            p2006: "Balda 3",
            p2007: "Costado derecho superior",
            p2008: "Frente izquierdo",
            p2009: "Frente derecho",
        },
        AppLanguage::Fr => LocalizedCabinetNames {
            proj_name: "Meuble de salle de bain",
            mat_white: "Aggloméré Blanc",
            edge_2: "PVC 2.0 mm",
            edge_1: "PVC 1.0 mm",
            edge_05: "PVC 0.5 mm",
            grp_carcass: "Caisson",
            grp_fronts: "Façades et étagères",
            p2000: "Côté droit inférieur",
            p2001: "Côté gauche",
            p2002: "Fond",
            p2003: "Dessus",
            p2004: "Étagère 1",
            p2005: "Étagère 2",
            p2006: "Étagère 3",
            p2007: "Côté droit supérieur",
            p2008: "Façade gauche",
            p2009: "Façade droite",
        },
        AppLanguage::It => LocalizedCabinetNames {
            proj_name: "Mobile da bagno",
            mat_white: "Truciolare Bianco",
            edge_2: "PVC 2.0 mm",
            edge_1: "PVC 1.0 mm",
            edge_05: "PVC 0.5 mm",
            grp_carcass: "Struttura",
            grp_fronts: "Ante e ripiani",
            p2000: "Fianco destro inferiore",
            p2001: "Fianco sinistro",
            p2002: "Fondo",
            p2003: "Cielo",
            p2004: "Ripiano 1",
            p2005: "Ripiano 2",
            p2006: "Ripiano 3",
            p2007: "Fianco destro superiore",
            p2008: "Anta sinistra",
            p2009: "Anta destra",
        },
        AppLanguage::ZhCn => LocalizedCabinetNames {
            proj_name: "浴室地柜",
            mat_white: "白色刨花板",
            edge_2: "PVC 2.0 mm",
            edge_1: "PVC 1.0 mm",
            edge_05: "PVC 0.5 mm",
            grp_carcass: "柜身",
            grp_fronts: "门板与隔板",
            p2000: "右下侧板",
            p2001: "左侧板",
            p2002: "底板",
            p2003: "顶板",
            p2004: "层板 1",
            p2005: "层板 2",
            p2006: "层板 3",
            p2007: "右上侧板",
            p2008: "左门板",
            p2009: "右门板",
        },
        AppLanguage::Ja => LocalizedCabinetNames {
            proj_name: "洗面台収納キャビネット",
            mat_white: "ホワイトパーティクルボード",
            edge_2: "PVC 2.0 mm",
            edge_1: "PVC 1.0 mm",
            edge_05: "PVC 0.5 mm",
            grp_carcass: "キャビネット本体",
            grp_fronts: "扉と棚板",
            p2000: "右下側板",
            p2001: "左側板",
            p2002: "底板",
            p2003: "天板",
            p2004: "棚板 1",
            p2005: "棚板 2",
            p2006: "棚板 3",
            p2007: "右上側板",
            p2008: "左扉",
            p2009: "右扉",
        },
        AppLanguage::Ko => LocalizedCabinetNames {
            proj_name: "욕실 수납장",
            mat_white: "화이트 파티클보드",
            edge_2: "PVC 2.0 mm",
            edge_1: "PVC 1.0 mm",
            edge_05: "PVC 0.5 mm",
            grp_carcass: "몸체",
            grp_fronts: "도어 및 선반",
            p2000: "우측 하부 측판",
            p2001: "좌측 측판",
            p2002: "밑판",
            p2003: "상판",
            p2004: "선반 1",
            p2005: "선반 2",
            p2006: "선반 3",
            p2007: "우측 상부 측판",
            p2008: "좌측 도어",
            p2009: "우측 도어",
        },
        AppLanguage::Pt => LocalizedCabinetNames {
            proj_name: "Armário de casa de banho",
            mat_white: "Aglomerado Branco",
            edge_2: "PVC 2.0 mm",
            edge_1: "PVC 1.0 mm",
            edge_05: "PVC 0.5 mm",
            grp_carcass: "Estrutura",
            grp_fronts: "Frentes e prateleiras",
            p2000: "Lateral direita inferior",
            p2001: "Lateral esquerda",
            p2002: "Fundo",
            p2003: "Tampo",
            p2004: "Prateleira 1",
            p2005: "Prateleira 2",
            p2006: "Prateleira 3",
            p2007: "Lateral direita superior",
            p2008: "Porta esquerda",
            p2009: "Porta direita",
        },
        AppLanguage::Cs => LocalizedCabinetNames {
            proj_name: "Koupelnová skříňka",
            mat_white: "Lamino Bílé",
            edge_2: "PVC 2.0 mm",
            edge_1: "PVC 1.0 mm",
            edge_05: "PVC 0.5 mm",
            grp_carcass: "Korpus",
            grp_fronts: "Čela a police",
            p2000: "Pravý spodní bok",
            p2001: "Levý bok",
            p2002: "Dno",
            p2003: "Půda",
            p2004: "Police 1",
            p2005: "Police 2",
            p2006: "Police 3",
            p2007: "Pravý horní bok",
            p2008: "Levé čelo",
            p2009: "Pravé čelo",
        },
        AppLanguage::Sk => LocalizedCabinetNames {
            proj_name: "Kúpeľňová skrinka",
            mat_white: "Lamino Biele",
            edge_2: "PVC 2.0 mm",
            edge_1: "PVC 1.0 mm",
            edge_05: "PVC 0.5 mm",
            grp_carcass: "Korpus",
            grp_fronts: "Dvierka a police",
            p2000: "Pravý spodný bok",
            p2001: "Ľavý bok",
            p2002: "Dno",
            p2003: "Vrch",
            p2004: "Polica 1",
            p2005: "Polica 2",
            p2006: "Polica 3",
            p2007: "Pravý horný bok",
            p2008: "Ľavé dvierka",
            p2009: "Pravé dvierka",
        },
        AppLanguage::Tr => LocalizedCabinetNames {
            proj_name: "Banyo Dolabı",
            mat_white: "Beyaz Suntalam",
            edge_2: "PVC 2.0 mm",
            edge_1: "PVC 1.0 mm",
            edge_05: "PVC 0.5 mm",
            grp_carcass: "Gövde",
            grp_fronts: "Kapaklar ve raflar",
            p2000: "Sağ alt yan panel",
            p2001: "Sol yan panel",
            p2002: "Alt taban",
            p2003: "Üst tabla",
            p2004: "Raf 1",
            p2005: "Raf 2",
            p2006: "Raf 3",
            p2007: "Sağ üst yan panel",
            p2008: "Sol kapak",
            p2009: "Sağ kapak",
        },
        AppLanguage::Nl => LocalizedCabinetNames {
            proj_name: "Badkamermeubel",
            mat_white: "Spaanplaat Wit",
            edge_2: "PVC 2.0 mm",
            edge_1: "PVC 1.0 mm",
            edge_05: "PVC 0.5 mm",
            grp_carcass: "Romp",
            grp_fronts: "Fronten en legplanken",
            p2000: "Rechter onderzijde",
            p2001: "Linkerzijde",
            p2002: "Bodem",
            p2003: "Bovenpaneel",
            p2004: "Legplank 1",
            p2005: "Legplank 2",
            p2006: "Legplank 3",
            p2007: "Rechter bovenzijde",
            p2008: "Linkerdeur",
            p2009: "Rechterdeur",
        },
        AppLanguage::Da => LocalizedCabinetNames {
            proj_name: "Badeværelsesmøbel",
            mat_white: "Spånplade Hvid",
            edge_2: "PVC 2.0 mm",
            edge_1: "PVC 1.0 mm",
            edge_05: "PVC 0.5 mm",
            grp_carcass: "Korpus",
            grp_fronts: "Låger og hylder",
            p2000: "Højre nederste side",
            p2001: "Venstre side",
            p2002: "Bund",
            p2003: "Top",
            p2004: "Hylde 1",
            p2005: "Hylde 2",
            p2006: "Hylde 3",
            p2007: "Højre øverste side",
            p2008: "Venstre låge",
            p2009: "Højre låge",
        },
        AppLanguage::No => LocalizedCabinetNames {
            proj_name: "Baderomsmøbel",
            mat_white: "Sponplate Hvit",
            edge_2: "PVC 2.0 mm",
            edge_1: "PVC 1.0 mm",
            edge_05: "PVC 0.5 mm",
            grp_carcass: "Skrog",
            grp_fronts: "Dører og hyller",
            p2000: "Høyre nedre side",
            p2001: "Venstre side",
            p2002: "Bunn",
            p2003: "Topp",
            p2004: "Hylle 1",
            p2005: "Hylle 2",
            p2006: "Hylle 3",
            p2007: "Høyre øvre side",
            p2008: "Venstre dør",
            p2009: "Høyre dør",
        },
        AppLanguage::Sv => LocalizedCabinetNames {
            proj_name: "Badrumsskåp",
            mat_white: "Spånskiva Vit",
            edge_2: "PVC 2.0 mm",
            edge_1: "PVC 1.0 mm",
            edge_05: "PVC 0.5 mm",
            grp_carcass: "Stomme",
            grp_fronts: "Luckor och hyllor",
            p2000: "Höger nedre sida",
            p2001: "Vänster sida",
            p2002: "Botten",
            p2003: "Toppskiva",
            p2004: "Hylla 1",
            p2005: "Hylla 2",
            p2006: "Hylla 3",
            p2007: "Höger övre sida",
            p2008: "Vänster dörr",
            p2009: "Höger dörr",
        },
        AppLanguage::Hu => LocalizedCabinetNames {
            proj_name: "Fürdőszoba szekrény",
            mat_white: "Bútorlap Fehér",
            edge_2: "PVC 2.0 mm",
            edge_1: "PVC 1.0 mm",
            edge_05: "PVC 0.5 mm",
            grp_carcass: "Korpusz",
            grp_fronts: "Frontok és polcok",
            p2000: "Jobb alsó oldal",
            p2001: "Bal oldal",
            p2002: "Alsó lap",
            p2003: "Felső lap",
            p2004: "Polc 1",
            p2005: "Polc 2",
            p2006: "Polc 3",
            p2007: "Jobb felső oldal",
            p2008: "Bal front",
            p2009: "Jobb front",
        },
        AppLanguage::Ro => LocalizedCabinetNames {
            proj_name: "Dulap de baie",
            mat_white: "PAL Alb",
            edge_2: "PVC 2.0 mm",
            edge_1: "PVC 1.0 mm",
            edge_05: "PVC 0.5 mm",
            grp_carcass: "Corp",
            grp_fronts: "Uși și polițe",
            p2000: "Laterală dreapta inferioară",
            p2001: "Laterală stânga",
            p2002: "Fund",
            p2003: "Capac",
            p2004: "Poliță 1",
            p2005: "Poliță 2",
            p2006: "Poliță 3",
            p2007: "Laterală dreapta superioară",
            p2008: "Ușă stânga",
            p2009: "Ușă dreapta",
        },
        AppLanguage::Sl => LocalizedCabinetNames {
            proj_name: "Kopalniška omarica",
            mat_white: "Iveral Bela",
            edge_2: "PVC 2.0 mm",
            edge_1: "PVC 1.0 mm",
            edge_05: "PVC 0.5 mm",
            grp_carcass: "Korpus",
            grp_fronts: "Ličnice in police",
            p2000: "Desna spodnja stranica",
            p2001: "Leva stranica",
            p2002: "Dno",
            p2003: "Zgornja plošča",
            p2004: "Polica 1",
            p2005: "Polica 2",
            p2006: "Polica 3",
            p2007: "Desna zgornja stranica",
            p2008: "Leva vrata",
            p2009: "Desna vrata",
        },
    }
}

pub fn get_localized_cabinet_project(lang: AppLanguage) -> Project {
    let names = get_localized_cabinet_names(lang);
    let texture_path = PathBuf::from(r"C:\Users\liqui\Desktop\ldsp_sosna_belenaya_r55031_ln_2800x2100x18_mm_pfleiderer_1_30.webp");
    let actual_tex = if texture_path.exists() {
        Some(texture_path)
    } else {
        None
    };

    Project {
        name: names.proj_name.to_string(),
        materials: vec![
            Material {
                id: 1,
                name: names.mat_white.to_string(),
                thickness_mm: 18.0,
                color_rgb: [235, 238, 242],
                texture_path: actual_tex,
                texture_relative_path: None,
                texture_base64: None,
                sheet_length_mm: 2800.0,
                sheet_width_mm: 2070.0,
                default_grain_fixed: false,
            }
        ],
        edge_bandings: vec![
            EdgeBanding { id: 10, name: names.edge_2.to_string(), thickness_mm: 2.0, color_rgb: [235, 85, 45] },
            EdgeBanding { id: 11, name: names.edge_1.to_string(), thickness_mm: 1.0, color_rgb: [50, 195, 95] },
            EdgeBanding { id: 12, name: names.edge_05.to_string(), thickness_mm: 0.5, color_rgb: [65, 150, 245] },
        ],
        groups: vec![
            PartGroup { id: 100, name: names.grp_carcass.to_string(), visible: true, locked: false, expanded: true },
            PartGroup { id: 101, name: names.grp_fronts.to_string(), visible: true, locked: false, expanded: true },
        ],
        parts: vec![
            PartBlock {
                id: 2000,
                name: names.p2000.to_string(),
                group_id: Some(100),
                material_id: 1,
                orientation: PartOrientation::VerticalYZ,
                position_mm: Vec3::new(702.0, 0.0, 18.0),
                length_mm: 680.0,
                width_mm: 200.0,
                yaw_deg: 0.0,
                grain_fixed: true,
                grain_along_width: false,
                edges: PartEdges { l1: Some(10), l2: Some(12), w1: Some(12), w2: Some(12) },
                visible: true,
                locked: false,
                drill_holes: Vec::new(),
            },
            PartBlock {
                id: 2001,
                name: names.p2001.to_string(),
                group_id: Some(100),
                material_id: 1,
                orientation: PartOrientation::VerticalYZ,
                position_mm: Vec3::new(0.0, 0.0, 18.0),
                length_mm: 1364.0,
                width_mm: 315.0,
                yaw_deg: 0.0,
                grain_fixed: true,
                grain_along_width: false,
                edges: PartEdges { l1: Some(10), l2: Some(12), w1: Some(12), w2: Some(12) },
                visible: true,
                locked: false,
                drill_holes: Vec::new(),
            },
            PartBlock {
                id: 2002,
                name: names.p2002.to_string(),
                group_id: Some(100),
                material_id: 1,
                orientation: PartOrientation::HorizontalXY,
                position_mm: Vec3::new(0.0, 0.0, 0.0),
                length_mm: 720.0,
                width_mm: 315.0,
                yaw_deg: 0.0,
                grain_fixed: true,
                grain_along_width: false,
                edges: PartEdges { l1: Some(10), l2: Some(12), w1: Some(12), w2: Some(12) },
                visible: true,
                locked: false,
                drill_holes: Vec::new(),
            },
            PartBlock {
                id: 2003,
                name: names.p2003.to_string(),
                group_id: Some(100),
                material_id: 1,
                orientation: PartOrientation::HorizontalXY,
                position_mm: Vec3::new(0.0, 0.0, 1382.0),
                length_mm: 720.0,
                width_mm: 315.0,
                yaw_deg: 0.0,
                grain_fixed: true,
                grain_along_width: false,
                edges: PartEdges { l1: Some(10), l2: Some(12), w1: Some(12), w2: Some(12) },
                visible: true,
                locked: false,
                drill_holes: Vec::new(),
            },
            PartBlock {
                id: 2004,
                name: names.p2004.to_string(),
                group_id: Some(101),
                material_id: 1,
                orientation: PartOrientation::HorizontalXY,
                position_mm: Vec3::new(18.0, 20.0, 350.0),
                length_mm: 684.0,
                width_mm: 180.0,
                yaw_deg: 0.0,
                grain_fixed: true,
                grain_along_width: false,
                edges: PartEdges { l1: Some(10), l2: Some(11), w1: Some(12), w2: Some(12) },
                visible: true,
                locked: false,
                drill_holes: Vec::new(),
            },
            PartBlock {
                id: 2005,
                name: names.p2005.to_string(),
                group_id: Some(101),
                material_id: 1,
                orientation: PartOrientation::HorizontalXY,
                position_mm: Vec3::new(18.0, 20.0, 680.0),
                length_mm: 684.0,
                width_mm: 295.0,
                yaw_deg: 0.0,
                grain_fixed: true,
                grain_along_width: false,
                edges: PartEdges { l1: Some(10), l2: Some(11), w1: Some(12), w2: Some(12) },
                visible: true,
                locked: false,
                drill_holes: Vec::new(),
            },
            PartBlock {
                id: 2006,
                name: names.p2006.to_string(),
                group_id: Some(101),
                material_id: 1,
                orientation: PartOrientation::HorizontalXY,
                position_mm: Vec3::new(18.0, 20.0, 984.0),
                length_mm: 684.0,
                width_mm: 295.0,
                yaw_deg: 0.0,
                grain_fixed: true,
                grain_along_width: false,
                edges: PartEdges { l1: Some(10), l2: Some(11), w1: Some(12), w2: Some(12) },
                visible: true,
                locked: false,
                drill_holes: Vec::new(),
            },
            PartBlock {
                id: 2007,
                name: names.p2007.to_string(),
                group_id: Some(100),
                material_id: 1,
                orientation: PartOrientation::VerticalYZ,
                position_mm: Vec3::new(702.0, 0.0, 698.0),
                length_mm: 684.0,
                width_mm: 315.0,
                yaw_deg: 0.0,
                grain_fixed: true,
                grain_along_width: false,
                edges: PartEdges { l1: Some(10), l2: Some(12), w1: Some(12), w2: Some(12) },
                visible: true,
                locked: false,
                drill_holes: Vec::new(),
            },
            PartBlock {
                id: 2008,
                name: names.p2008.to_string(),
                group_id: Some(101),
                material_id: 1,
                orientation: PartOrientation::FrontalXZ,
                position_mm: Vec3::new(-161.0, -182.0, 20.0),
                length_mm: 1360.0,
                width_mm: 340.0,
                yaw_deg: 90.0,
                grain_fixed: true,
                grain_along_width: false,
                edges: PartEdges { l1: Some(10), l2: Some(10), w1: Some(10), w2: Some(10) },
                visible: true,
                locked: false,
                drill_holes: Vec::new(),
            },
            PartBlock {
                id: 2009,
                name: names.p2009.to_string(),
                group_id: Some(101),
                material_id: 1,
                orientation: PartOrientation::FrontalXZ,
                position_mm: Vec3::new(419.0, -135.57211, 20.0),
                length_mm: 1360.0,
                width_mm: 338.0,
                yaw_deg: 45.0,
                grain_fixed: true,
                grain_along_width: false,
                edges: PartEdges { l1: Some(10), l2: Some(10), w1: Some(10), w2: Some(10) },
                visible: true,
                locked: false,
                drill_holes: Vec::new(),
            },
        ],
        cutting_settings: CuttingSettings {
            kerf_mm: 4.0,
            trim_left_mm: 10.0,
            trim_right_mm: 10.0,
            trim_top_mm: 10.0,
            trim_bottom_mm: 10.0,
            group_identical_parts: true,
        },
        next_id: 2011,
        embed_textures: false,
    }
}

pub fn handle_batch_screenshot_step(app: &mut QuadFurnApp, ctx: &egui::Context) {
    if app.batch_screenshots_dir.is_none() {
        return;
    }

    if app.batch_startup_delay < 5 {
        app.batch_startup_delay += 1;
        ctx.request_repaint();
        return;
    }

    if app.batch_lang_idx >= BATCH_LANGS.len() {
        app.batch_screenshots_dir = None;
        app.allow_exit = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        return;
    }

    let (_lang_code, lang) = BATCH_LANGS[app.batch_lang_idx];

    if app.batch_frame_delay == 0 {
        app.language = lang;
        let proj = get_localized_cabinet_project(lang);
        let mut doc = FurnitureDocument::new(proj);
        let virtual_path = PathBuf::from(r"C:\Users\liqui\Desktop\cabinet_18mm.qfurn");
        doc.mark_saved(Some(virtual_path.clone()));

        for m in &mut doc.project.materials {
            if m.texture_path.is_some() || m.texture_base64.is_some() || m.texture_relative_path.is_some() {
                if let Ok(info) = crate::resolve_and_load_material_texture(
                    ctx,
                    m,
                    Some(&virtual_path),
                    app.language,
                ) {
                    doc.loaded_textures.insert(m.id, info.handle);
                }
            }
        }

        doc.viewport_ctrl.set_single_selection(&mut doc.selected_part, Some(2000));
        doc.viewport_ctrl.fit_all_to_window(&doc.project);
        app.documents = vec![doc];
        app.active_doc_idx = 0;

        match app.batch_shot_idx {
            0 => {
                app.theme_choice = ThemeChoice::Dark;
                app.theme_choice.apply(ctx);
                app.active_tab = AppTab::Designer4View;
                app.force_open_lang_menu = false;
            }
            1 => {
                app.theme_choice = ThemeChoice::Dark;
                app.theme_choice.apply(ctx);
                app.active_tab = AppTab::Specification;
                app.force_open_lang_menu = false;
            }
            2 => {
                app.theme_choice = ThemeChoice::Dark;
                app.theme_choice.apply(ctx);
                app.active_tab = AppTab::CuttingLayout;
                app.force_open_lang_menu = false;
            }
            3 => {
                app.theme_choice = ThemeChoice::Dark;
                app.theme_choice.apply(ctx);
                app.active_tab = AppTab::MaterialsManager;
                app.force_open_lang_menu = false;
            }
            4 => {
                app.theme_choice = ThemeChoice::Light;
                app.theme_choice.apply(ctx);
                app.active_tab = AppTab::Designer4View;
                app.force_open_lang_menu = false;
            }
            _ => {
                app.theme_choice = ThemeChoice::Light;
                app.theme_choice.apply(ctx);
                app.active_tab = AppTab::Designer4View;
                app.force_open_lang_menu = true;
            }
        }
    }

    if app.batch_frame_delay < 5 {
        app.batch_frame_delay += 1;
        ctx.request_repaint();
    } else if app.batch_frame_delay == 5 {
        app.batch_frame_delay += 1;
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
        ctx.request_repaint();
    } else {
        ctx.request_repaint();
    }
}
