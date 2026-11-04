use crate::i18n::AppLanguage;
use egui::{Color32, CornerRadius, Stroke, Visuals};
use serde::{Deserialize, Serialize};

// Font Awesome 6 Free Solid Unicode Private Use Area codepoints (`fa-solid-900.ttf`)
pub const FA_POINTER: &str = "\u{f245}"; // fa-arrow-pointer (Select tool)
pub const FA_CUBE: &str = "\u{f1b2}"; // fa-cube (Create block / part)
pub const FA_CUBES: &str = "\u{f1b3}"; // fa-cubes (Product / groups)
pub const FA_RULER: &str = "\u{f545}"; // fa-ruler (Measure tape)
pub const FA_PLUS: &str = "\u{f067}"; // fa-plus
pub const FA_FOLDER: &str = "\u{f07c}"; // fa-folder-open
pub const FA_SAVE: &str = "\u{f0c7}"; // fa-floppy-disk
pub const FA_UNDO: &str = "\u{f2ea}"; // fa-rotate-left
pub const FA_COPY: &str = "\u{f0c5}"; // fa-copy
pub const FA_CLIPBOARD: &str = "\u{f0ea}"; // fa-paste
pub const FA_PASTE: &str = "\u{f0ea}"; // fa-paste
pub const FA_DUPLICATE: &str = "\u{f24d}"; // fa-clone
pub const FA_CLONE: &str = "\u{f24d}"; // fa-clone
pub const FA_TRASH: &str = "\u{f2ed}"; // fa-trash-can
pub const FA_CLOSE: &str = "\u{f00d}"; // fa-xmark
pub const FA_EXPORT: &str = "\u{f56e}"; // fa-file-export
pub const FA_FILE_PDF: &str = "\u{f1c1}"; // fa-file-pdf
pub const FA_FILE_CSV: &str = "\u{f6dd}"; // fa-file-csv
pub const FA_PALETTE: &str = "\u{f53f}"; // fa-palette
pub const FA_EYE: &str = "\u{f06e}"; // fa-eye
pub const FA_EYE_SLASH: &str = "\u{f070}"; // fa-eye-slash
pub const FA_LOCK: &str = "\u{f023}"; // fa-lock
pub const FA_LOCK_OPEN: &str = "\u{f3c1}"; // fa-lock-open
pub const FA_ARROW_UP: &str = "\u{f062}"; // fa-arrow-up
pub const FA_ARROW_DOWN: &str = "\u{f063}"; // fa-arrow-down
pub const FA_ROTATE_90: &str = "\u{f01e}"; // fa-rotate-right
pub const FA_ROTATE: &str = "\u{f01e}"; // fa-rotate-right
pub const FA_FIT: &str = "\u{f065}"; // fa-expand
pub const FA_EXPAND: &str = "\u{f065}"; // fa-expand
pub const FA_GRID_TILED: &str = "\u{f00a}"; // fa-table-cells
pub const FA_TABLE_CELLS_LARGE: &str = "\u{f009}"; // fa-table-cells-large
pub const FA_LIST: &str = "\u{f03a}"; // fa-list
pub const FA_CUT: &str = "\u{f0c4}"; // fa-scissors
pub const FA_LAYER_GROUP: &str = "\u{f5fd}"; // fa-layer-group
pub const FA_QUESTION: &str = "\u{f059}"; // fa-circle-question
pub const FA_INFO: &str = "\u{f05a}"; // fa-circle-info
pub const FA_CHECK: &str = "\u{f00c}"; // fa-check
pub const FA_WARN: &str = "\u{f071}"; // fa-triangle-exclamation
pub const FA_GLOBE: &str = "\u{f0ac}"; // fa-globe
pub const FA_MAGNET: &str = "\u{f076}"; // fa-magnet
pub const FA_BORDER_ALL: &str = "\u{f84c}"; // fa-border-all

/// Color theme selection (Standard themes, 10 Dark themes, 10 Light themes).
/// Note: 2D and 3D CAD viewports always remain dark across all themes for contrast.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemeChoice {
    // Standard Defaults
    #[default]
    Dark,
    Light,
    System,

    // 10 Dark Themes
    NordFrost,
    Dracula,
    TokyoNight,
    CatppuccinMocha,
    OneDark,
    GruvboxDark,
    MonokaiPro,
    AyuDark,
    SolarizedDark,
    RosePine,

    // 10 Light Themes
    SolarizedLight,
    CatppuccinLatte,
    NordSnow,
    GithubLight,
    OneLight,
    GruvboxLight,
    AyuLight,
    RosePineDawn,
    EverforestLight,
    PaperColorLight,
}

impl ThemeChoice {
    pub const DEFAULT_THEMES: [ThemeChoice; 3] = [
        ThemeChoice::Dark,
        ThemeChoice::Light,
        ThemeChoice::System,
    ];

    pub const DARK_THEMES: [ThemeChoice; 10] = [
        ThemeChoice::NordFrost,
        ThemeChoice::Dracula,
        ThemeChoice::TokyoNight,
        ThemeChoice::CatppuccinMocha,
        ThemeChoice::OneDark,
        ThemeChoice::GruvboxDark,
        ThemeChoice::MonokaiPro,
        ThemeChoice::AyuDark,
        ThemeChoice::SolarizedDark,
        ThemeChoice::RosePine,
    ];

    pub const LIGHT_THEMES: [ThemeChoice; 10] = [
        ThemeChoice::SolarizedLight,
        ThemeChoice::CatppuccinLatte,
        ThemeChoice::NordSnow,
        ThemeChoice::GithubLight,
        ThemeChoice::OneLight,
        ThemeChoice::GruvboxLight,
        ThemeChoice::AyuLight,
        ThemeChoice::RosePineDawn,
        ThemeChoice::EverforestLight,
        ThemeChoice::PaperColorLight,
    ];

    pub fn label(self) -> &'static str {
        self.label_i18n(AppLanguage::Ru)
    }

    pub fn label_i18n(self, lang: AppLanguage) -> &'static str {
        match self {
            Self::Dark => lang.t("theme.dark"),
            Self::Light => lang.t("theme.light"),
            Self::System => lang.dict().theme_system,

            Self::NordFrost => lang.t("theme.nord_frost"),
            Self::Dracula => lang.t("theme.dracula"),
            Self::TokyoNight => lang.t("theme.tokyo_night"),
            Self::CatppuccinMocha => lang.t("theme.catppuccin_mocha"),
            Self::OneDark => lang.t("theme.one_dark"),
            Self::GruvboxDark => lang.t("theme.gruvbox_dark"),
            Self::MonokaiPro => lang.t("theme.monokai_pro"),
            Self::AyuDark => lang.t("theme.ayu_dark"),
            Self::SolarizedDark => lang.t("theme.solarized_dark"),
            Self::RosePine => lang.t("theme.rose_pine"),

            Self::SolarizedLight => lang.t("theme.solarized_light"),
            Self::CatppuccinLatte => lang.t("theme.catppuccin_latte"),
            Self::NordSnow => lang.t("theme.nord_snow"),
            Self::GithubLight => lang.t("theme.github_light"),
            Self::OneLight => lang.t("theme.one_light"),
            Self::GruvboxLight => lang.t("theme.gruvbox_light"),
            Self::AyuLight => lang.t("theme.ayu_light"),
            Self::RosePineDawn => lang.t("theme.rose_pine_dawn"),
            Self::EverforestLight => lang.t("theme.everforest_light"),
            Self::PaperColorLight => lang.t("theme.papercolor_light"),
        }
    }

    pub fn apply(self, ctx: &egui::Context) {
        apply_app_theme(ctx, self);
    }
}

struct ThemeSpec {
    dark_mode: bool,
    panel_bg: Color32,
    window_bg: Color32,
    extreme_bg: Color32,
    faint_bg: Color32,
    code_bg: Color32,
    widget_noninteractive_bg: Color32,
    widget_inactive_bg: Color32,
    widget_hovered_bg: Color32,
    widget_active_bg: Color32,
    text_primary: Color32,
    text_bright: Color32,
    accent: Color32,
    accent_fg: Color32,
    border: Color32,
    rounding: f32,
}

fn build_visuals(spec: &ThemeSpec) -> Visuals {
    let mut v = if spec.dark_mode {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    let r = CornerRadius::same(spec.rounding.round() as u8);

    v.panel_fill = spec.panel_bg;
    v.window_fill = spec.window_bg;
    v.extreme_bg_color = spec.extreme_bg;
    v.faint_bg_color = spec.faint_bg;
    v.code_bg_color = spec.code_bg;

    v.window_corner_radius = CornerRadius::same(((spec.rounding + 2.0).min(8.0).round()) as u8);
    v.menu_corner_radius = CornerRadius::same(((spec.rounding + 1.0).min(6.0).round()) as u8);
    v.window_stroke = Stroke::new(1.0, spec.border);

    v.widgets.noninteractive.bg_fill = spec.widget_noninteractive_bg;
    v.widgets.noninteractive.weak_bg_fill = spec.widget_noninteractive_bg;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, spec.border);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, spec.text_primary);
    v.widgets.noninteractive.corner_radius = r;

    v.widgets.inactive.bg_fill = spec.widget_inactive_bg;
    v.widgets.inactive.weak_bg_fill = spec.widget_inactive_bg;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, spec.border);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, spec.text_primary);
    v.widgets.inactive.corner_radius = r;

    v.widgets.hovered.bg_fill = spec.widget_hovered_bg;
    v.widgets.hovered.weak_bg_fill = spec.widget_hovered_bg;
    v.widgets.hovered.bg_stroke = Stroke::new(1.2, spec.accent);
    v.widgets.hovered.fg_stroke = Stroke::new(1.5, spec.text_bright);
    v.widgets.hovered.corner_radius = r;

    v.widgets.active.bg_fill = spec.widget_active_bg;
    v.widgets.active.weak_bg_fill = spec.widget_active_bg;
    v.widgets.active.bg_stroke = Stroke::new(1.5, spec.accent);
    v.widgets.active.fg_stroke = Stroke::new(2.0, spec.text_bright);
    v.widgets.active.corner_radius = r;

    v.widgets.open.bg_fill = spec.widget_active_bg;
    v.widgets.open.weak_bg_fill = spec.widget_active_bg;
    v.widgets.open.bg_stroke = Stroke::new(1.2, spec.accent);
    v.widgets.open.fg_stroke = Stroke::new(1.5, spec.text_bright);
    v.widgets.open.corner_radius = r;

    v.selection.bg_fill = spec.accent;
    v.selection.stroke = Stroke::new(1.0, spec.accent_fg);
    v.hyperlink_color = spec.accent;

    v
}

pub fn apply_app_theme(ctx: &egui::Context, choice: ThemeChoice) {
    match choice {
        ThemeChoice::System => {
            ctx.set_theme(egui::ThemePreference::System);
            return;
        }
        ThemeChoice::Dark => {
            ctx.set_theme(egui::ThemePreference::Dark);
            ctx.set_visuals(Visuals::dark());
            return;
        }
        ThemeChoice::Light => {
            ctx.set_theme(egui::ThemePreference::Light);
            ctx.set_visuals(Visuals::light());
            return;
        }
        _ => {}
    }

    let spec = match choice {
        ThemeChoice::NordFrost => ThemeSpec {
            dark_mode: true,
            panel_bg: Color32::from_rgb(46, 52, 64),             // #2E3440
            window_bg: Color32::from_rgb(59, 66, 82),            // #3B4252
            extreme_bg: Color32::from_rgb(36, 41, 51),           // #242933
            faint_bg: Color32::from_rgb(53, 60, 74),
            code_bg: Color32::from_rgb(40, 45, 56),
            widget_noninteractive_bg: Color32::from_rgb(46, 52, 64),
            widget_inactive_bg: Color32::from_rgb(67, 76, 94),   // #434C5E
            widget_hovered_bg: Color32::from_rgb(76, 86, 106),   // #4C566A
            widget_active_bg: Color32::from_rgb(94, 129, 172),   // #5E81AC
            text_primary: Color32::from_rgb(216, 222, 233),      // #D8DEE9
            text_bright: Color32::from_rgb(236, 239, 244),       // #ECEFF4
            accent: Color32::from_rgb(136, 192, 208),            // #88C0D0
            accent_fg: Color32::from_rgb(36, 41, 51),
            border: Color32::from_rgb(76, 86, 106),
            rounding: 4.0,
        },
        ThemeChoice::Dracula => ThemeSpec {
            dark_mode: true,
            panel_bg: Color32::from_rgb(40, 42, 54),             // #282A36
            window_bg: Color32::from_rgb(49, 51, 66),
            extreme_bg: Color32::from_rgb(30, 31, 41),           // #1E1F29
            faint_bg: Color32::from_rgb(45, 47, 61),
            code_bg: Color32::from_rgb(33, 35, 46),
            widget_noninteractive_bg: Color32::from_rgb(40, 42, 54),
            widget_inactive_bg: Color32::from_rgb(68, 71, 90),   // #44475A
            widget_hovered_bg: Color32::from_rgb(84, 88, 112),
            widget_active_bg: Color32::from_rgb(98, 114, 164),   // #6272A4
            text_primary: Color32::from_rgb(248, 248, 242),      // #F8F8F2
            text_bright: Color32::from_rgb(255, 255, 255),
            accent: Color32::from_rgb(189, 147, 249),            // #BD93F9
            accent_fg: Color32::from_rgb(30, 31, 41),
            border: Color32::from_rgb(68, 71, 90),
            rounding: 4.0,
        },
        ThemeChoice::TokyoNight => ThemeSpec {
            dark_mode: true,
            panel_bg: Color32::from_rgb(26, 27, 38),             // #1A1B26
            window_bg: Color32::from_rgb(36, 40, 59),            // #24283B
            extreme_bg: Color32::from_rgb(21, 22, 30),           // #15161E
            faint_bg: Color32::from_rgb(31, 35, 53),
            code_bg: Color32::from_rgb(23, 24, 34),
            widget_noninteractive_bg: Color32::from_rgb(26, 27, 38),
            widget_inactive_bg: Color32::from_rgb(41, 46, 66),   // #292E42
            widget_hovered_bg: Color32::from_rgb(59, 66, 97),    // #3B4261
            widget_active_bg: Color32::from_rgb(65, 72, 104),    // #414868
            text_primary: Color32::from_rgb(169, 177, 214),      // #A9B1D6
            text_bright: Color32::from_rgb(192, 202, 245),       // #C0CAF5
            accent: Color32::from_rgb(122, 162, 247),            // #7AA2F7
            accent_fg: Color32::from_rgb(21, 22, 30),
            border: Color32::from_rgb(59, 66, 97),
            rounding: 4.0,
        },
        ThemeChoice::CatppuccinMocha => ThemeSpec {
            dark_mode: true,
            panel_bg: Color32::from_rgb(30, 30, 46),             // #1E1E2E
            window_bg: Color32::from_rgb(49, 50, 68),            // #313244
            extreme_bg: Color32::from_rgb(17, 17, 27),           // #11111B
            faint_bg: Color32::from_rgb(24, 24, 37),             // #181825
            code_bg: Color32::from_rgb(24, 24, 37),
            widget_noninteractive_bg: Color32::from_rgb(30, 30, 46),
            widget_inactive_bg: Color32::from_rgb(49, 50, 68),   // #313244
            widget_hovered_bg: Color32::from_rgb(69, 71, 90),    // #45475A
            widget_active_bg: Color32::from_rgb(88, 91, 112),    // #585B70
            text_primary: Color32::from_rgb(205, 214, 244),      // #CDD6F4
            text_bright: Color32::from_rgb(245, 224, 220),       // #F5E0DC
            accent: Color32::from_rgb(203, 166, 247),            // #CBA6F7
            accent_fg: Color32::from_rgb(17, 17, 27),
            border: Color32::from_rgb(69, 71, 90),
            rounding: 5.0,
        },
        ThemeChoice::SolarizedLight => ThemeSpec {
            dark_mode: false,
            panel_bg: Color32::from_rgb(238, 232, 213),          // #EEE8D5
            window_bg: Color32::from_rgb(253, 246, 227),         // #FDF6E3
            extreme_bg: Color32::from_rgb(253, 246, 227),
            faint_bg: Color32::from_rgb(245, 239, 220),
            code_bg: Color32::from_rgb(235, 228, 208),
            widget_noninteractive_bg: Color32::from_rgb(238, 232, 213),
            widget_inactive_bg: Color32::from_rgb(228, 221, 200),
            widget_hovered_bg: Color32::from_rgb(214, 206, 182),
            widget_active_bg: Color32::from_rgb(198, 190, 165),
            text_primary: Color32::from_rgb(88, 110, 117),       // #586E75
            text_bright: Color32::from_rgb(7, 54, 66),           // #073642
            accent: Color32::from_rgb(38, 139, 210),             // #268BD2
            accent_fg: Color32::WHITE,
            border: Color32::from_rgb(195, 187, 164),
            rounding: 4.0,
        },
        ThemeChoice::CatppuccinLatte => ThemeSpec {
            dark_mode: false,
            panel_bg: Color32::from_rgb(230, 233, 239),          // #E6E9EF (Mantle)
            window_bg: Color32::from_rgb(239, 241, 245),         // #EFF1F5 (Base)
            extreme_bg: Color32::from_rgb(252, 253, 255),
            faint_bg: Color32::from_rgb(220, 224, 232),          // #DCE0E8 (Crust)
            code_bg: Color32::from_rgb(220, 224, 232),
            widget_noninteractive_bg: Color32::from_rgb(230, 233, 239),
            widget_inactive_bg: Color32::from_rgb(204, 208, 218), // #CCD0DA (Surface0)
            widget_hovered_bg: Color32::from_rgb(188, 192, 204),  // #BCC0CC (Surface1)
            widget_active_bg: Color32::from_rgb(172, 176, 190),   // #ACB0BE (Surface2)
            text_primary: Color32::from_rgb(76, 79, 105),         // #4C4F69
            text_bright: Color32::from_rgb(30, 32, 48),
            accent: Color32::from_rgb(30, 102, 245),              // #1E66F5
            accent_fg: Color32::WHITE,
            border: Color32::from_rgb(188, 192, 204),
            rounding: 5.0,
        },
        ThemeChoice::NordSnow => ThemeSpec {
            dark_mode: false,
            panel_bg: Color32::from_rgb(229, 233, 240),          // #E5E9F0 (nord5)
            window_bg: Color32::from_rgb(236, 239, 244),         // #ECEFF4 (nord6)
            extreme_bg: Color32::from_rgb(248, 250, 252),
            faint_bg: Color32::from_rgb(216, 222, 233),          // #D8DEE9 (nord4)
            code_bg: Color32::from_rgb(216, 222, 233),
            widget_noninteractive_bg: Color32::from_rgb(229, 233, 240),
            widget_inactive_bg: Color32::from_rgb(216, 222, 233),
            widget_hovered_bg: Color32::from_rgb(198, 207, 222),
            widget_active_bg: Color32::from_rgb(180, 192, 212),
            text_primary: Color32::from_rgb(59, 66, 82),         // #3B4252 (nord1)
            text_bright: Color32::from_rgb(46, 52, 64),          // #2E3440 (nord0)
            accent: Color32::from_rgb(94, 129, 172),             // #5E81AC (nord10)
            accent_fg: Color32::WHITE,
            border: Color32::from_rgb(190, 200, 216),
            rounding: 4.0,
        },
        ThemeChoice::GithubLight => ThemeSpec {
            dark_mode: false,
            panel_bg: Color32::from_rgb(246, 248, 250),          // #F6F8FA
            window_bg: Color32::from_rgb(255, 255, 255),         // #FFFFFF
            extreme_bg: Color32::from_rgb(255, 255, 255),
            faint_bg: Color32::from_rgb(238, 242, 246),
            code_bg: Color32::from_rgb(240, 243, 246),
            widget_noninteractive_bg: Color32::from_rgb(246, 248, 250),
            widget_inactive_bg: Color32::from_rgb(234, 238, 242), // #EAEEF2
            widget_hovered_bg: Color32::from_rgb(218, 224, 231),
            widget_active_bg: Color32::from_rgb(202, 210, 220),
            text_primary: Color32::from_rgb(31, 35, 40),         // #1F2328
            text_bright: Color32::from_rgb(10, 12, 16),
            accent: Color32::from_rgb(9, 105, 218),              // #0969DA
            accent_fg: Color32::WHITE,
            border: Color32::from_rgb(208, 215, 222),            // #D0D7DE
            rounding: 4.0,
        },
        ThemeChoice::OneDark => ThemeSpec {
            dark_mode: true,
            panel_bg: Color32::from_rgb(40, 44, 52),             // #282C34
            window_bg: Color32::from_rgb(33, 37, 43),            // #21252B
            extreme_bg: Color32::from_rgb(30, 34, 39),           // #1E2227
            faint_bg: Color32::from_rgb(44, 49, 60),             // #2C313C
            code_bg: Color32::from_rgb(33, 37, 43),
            widget_noninteractive_bg: Color32::from_rgb(40, 44, 52),
            widget_inactive_bg: Color32::from_rgb(58, 63, 75),   // #3A3F4B
            widget_hovered_bg: Color32::from_rgb(75, 82, 99),    // #4B5263
            widget_active_bg: Color32::from_rgb(92, 99, 112),    // #5C6370
            text_primary: Color32::from_rgb(171, 178, 191),      // #ABB2BF
            text_bright: Color32::from_rgb(236, 239, 244),
            accent: Color32::from_rgb(97, 175, 239),             // #61AFEF
            accent_fg: Color32::from_rgb(33, 37, 43),
            border: Color32::from_rgb(62, 68, 81),               // #3E4451
            rounding: 4.0,
        },
        ThemeChoice::GruvboxDark => ThemeSpec {
            dark_mode: true,
            panel_bg: Color32::from_rgb(40, 40, 40),             // #282828
            window_bg: Color32::from_rgb(50, 48, 47),            // #32302F
            extreme_bg: Color32::from_rgb(29, 32, 33),           // #1D2021
            faint_bg: Color32::from_rgb(60, 56, 54),             // #3C3836
            code_bg: Color32::from_rgb(29, 32, 33),
            widget_noninteractive_bg: Color32::from_rgb(40, 40, 40),
            widget_inactive_bg: Color32::from_rgb(80, 73, 69),   // #504945
            widget_hovered_bg: Color32::from_rgb(102, 92, 84),   // #665C54
            widget_active_bg: Color32::from_rgb(124, 111, 100),  // #7C6F64
            text_primary: Color32::from_rgb(235, 219, 178),      // #EBDBB2
            text_bright: Color32::from_rgb(251, 241, 199),       // #FBF1C7
            accent: Color32::from_rgb(250, 189, 47),             // #FABD2F
            accent_fg: Color32::from_rgb(40, 40, 40),
            border: Color32::from_rgb(80, 73, 69),
            rounding: 3.0,
        },
        ThemeChoice::MonokaiPro => ThemeSpec {
            dark_mode: true,
            panel_bg: Color32::from_rgb(45, 42, 46),             // #2D2A2E
            window_bg: Color32::from_rgb(54, 51, 55),            // #363337
            extreme_bg: Color32::from_rgb(34, 31, 34),           // #221F22
            faint_bg: Color32::from_rgb(64, 62, 65),             // #403E41
            code_bg: Color32::from_rgb(34, 31, 34),
            widget_noninteractive_bg: Color32::from_rgb(45, 42, 46),
            widget_inactive_bg: Color32::from_rgb(74, 71, 75),   // #4A474B
            widget_hovered_bg: Color32::from_rgb(91, 87, 92),    // #5B575C
            widget_active_bg: Color32::from_rgb(114, 112, 114),  // #727072
            text_primary: Color32::from_rgb(252, 252, 250),      // #FCFCFA
            text_bright: Color32::WHITE,
            accent: Color32::from_rgb(255, 216, 102),            // #FFD866
            accent_fg: Color32::from_rgb(45, 42, 46),
            border: Color32::from_rgb(91, 87, 92),
            rounding: 4.0,
        },
        ThemeChoice::AyuDark => ThemeSpec {
            dark_mode: true,
            panel_bg: Color32::from_rgb(15, 20, 28),             // #0F141C
            window_bg: Color32::from_rgb(19, 23, 33),            // #131721
            extreme_bg: Color32::from_rgb(11, 14, 20),           // #0B0E14
            faint_bg: Color32::from_rgb(26, 31, 44),             // #1A1F2C
            code_bg: Color32::from_rgb(11, 14, 20),
            widget_noninteractive_bg: Color32::from_rgb(15, 20, 28),
            widget_inactive_bg: Color32::from_rgb(30, 37, 54),   // #1E2536
            widget_hovered_bg: Color32::from_rgb(39, 50, 75),    // #27324B
            widget_active_bg: Color32::from_rgb(52, 66, 100),   // #344264
            text_primary: Color32::from_rgb(191, 189, 182),      // #BFBDB6
            text_bright: Color32::from_rgb(230, 225, 207),       // #E6E1CF
            accent: Color32::from_rgb(230, 180, 80),             // #E6B450
            accent_fg: Color32::from_rgb(15, 20, 28),
            border: Color32::from_rgb(39, 50, 75),
            rounding: 4.0,
        },
        ThemeChoice::SolarizedDark => ThemeSpec {
            dark_mode: true,
            panel_bg: Color32::from_rgb(0, 43, 54),              // #002B36
            window_bg: Color32::from_rgb(7, 54, 66),             // #073642
            extreme_bg: Color32::from_rgb(0, 33, 43),            // #00212B
            faint_bg: Color32::from_rgb(9, 68, 82),              // #094452
            code_bg: Color32::from_rgb(0, 33, 43),
            widget_noninteractive_bg: Color32::from_rgb(0, 43, 54),
            widget_inactive_bg: Color32::from_rgb(11, 76, 92),   // #0B4C5C
            widget_hovered_bg: Color32::from_rgb(20, 96, 115),   // #146073
            widget_active_bg: Color32::from_rgb(30, 117, 140),   // #1E758C
            text_primary: Color32::from_rgb(131, 148, 150),      // #839496
            text_bright: Color32::from_rgb(147, 161, 161),       // #93A1A1
            accent: Color32::from_rgb(38, 139, 210),             // #268BD2
            accent_fg: Color32::WHITE,
            border: Color32::from_rgb(14, 85, 102),
            rounding: 4.0,
        },
        ThemeChoice::RosePine => ThemeSpec {
            dark_mode: true,
            panel_bg: Color32::from_rgb(25, 23, 36),             // #191724 (Base)
            window_bg: Color32::from_rgb(31, 29, 46),            // #1F1D2E (Surface)
            extreme_bg: Color32::from_rgb(20, 18, 31),           // #14121F
            faint_bg: Color32::from_rgb(38, 35, 58),             // #26233A (Overlay)
            code_bg: Color32::from_rgb(31, 29, 46),
            widget_noninteractive_bg: Color32::from_rgb(25, 23, 36),
            widget_inactive_bg: Color32::from_rgb(49, 45, 75),   // #312D4B
            widget_hovered_bg: Color32::from_rgb(66, 61, 100),   // #423D64
            widget_active_bg: Color32::from_rgb(82, 76, 125),   // #524C7D
            text_primary: Color32::from_rgb(224, 222, 244),      // #E0DEF4
            text_bright: Color32::from_rgb(240, 240, 254),
            accent: Color32::from_rgb(235, 188, 186),            // #EBBCBA (Rose)
            accent_fg: Color32::from_rgb(25, 23, 36),
            border: Color32::from_rgb(66, 61, 100),
            rounding: 5.0,
        },
        ThemeChoice::OneLight => ThemeSpec {
            dark_mode: false,
            panel_bg: Color32::from_rgb(240, 240, 240),          // #F0F0F0
            window_bg: Color32::from_rgb(255, 255, 255),         // #FFFFFF
            extreme_bg: Color32::from_rgb(250, 250, 250),         // #FAFAFA
            faint_bg: Color32::from_rgb(229, 229, 230),          // #E5E5E6
            code_bg: Color32::from_rgb(234, 234, 235),
            widget_noninteractive_bg: Color32::from_rgb(240, 240, 240),
            widget_inactive_bg: Color32::from_rgb(224, 224, 226), // #E0E0E2
            widget_hovered_bg: Color32::from_rgb(208, 208, 212),  // #D0D0D4
            widget_active_bg: Color32::from_rgb(188, 188, 194),   // #BCBCC2
            text_primary: Color32::from_rgb(56, 58, 66),          // #383A42
            text_bright: Color32::from_rgb(30, 31, 36),
            accent: Color32::from_rgb(64, 120, 242),              // #4078F2
            accent_fg: Color32::WHITE,
            border: Color32::from_rgb(208, 208, 212),
            rounding: 4.0,
        },
        ThemeChoice::GruvboxLight => ThemeSpec {
            dark_mode: false,
            panel_bg: Color32::from_rgb(235, 219, 178),          // #EBDBB2
            window_bg: Color32::from_rgb(251, 241, 199),         // #FBF1C7
            extreme_bg: Color32::from_rgb(249, 245, 215),         // #F9F5D7
            faint_bg: Color32::from_rgb(224, 207, 158),          // #E0CF9E
            code_bg: Color32::from_rgb(224, 207, 158),
            widget_noninteractive_bg: Color32::from_rgb(235, 219, 178),
            widget_inactive_bg: Color32::from_rgb(213, 196, 161), // #D5C4A1
            widget_hovered_bg: Color32::from_rgb(189, 174, 141),  // #BDAE8D
            widget_active_bg: Color32::from_rgb(168, 153, 124),   // #A8997C
            text_primary: Color32::from_rgb(60, 56, 54),          // #3C3836
            text_bright: Color32::from_rgb(40, 40, 40),
            accent: Color32::from_rgb(181, 118, 20),             // #B57614
            accent_fg: Color32::WHITE,
            border: Color32::from_rgb(189, 174, 141),
            rounding: 3.0,
        },
        ThemeChoice::AyuLight => ThemeSpec {
            dark_mode: false,
            panel_bg: Color32::from_rgb(243, 244, 245),          // #F3F4F5
            window_bg: Color32::from_rgb(250, 250, 250),         // #FAFAFA
            extreme_bg: Color32::from_rgb(255, 255, 255),
            faint_bg: Color32::from_rgb(231, 232, 235),          // #E7E8EB
            code_bg: Color32::from_rgb(231, 232, 235),
            widget_noninteractive_bg: Color32::from_rgb(243, 244, 245),
            widget_inactive_bg: Color32::from_rgb(223, 224, 228), // #DFE0E4
            widget_hovered_bg: Color32::from_rgb(206, 209, 215),  // #CED1D7
            widget_active_bg: Color32::from_rgb(189, 193, 201),   // #BDC1C9
            text_primary: Color32::from_rgb(92, 97, 102),         // #5C6166
            text_bright: Color32::from_rgb(36, 41, 47),
            accent: Color32::from_rgb(242, 151, 24),              // #F29718
            accent_fg: Color32::WHITE,
            border: Color32::from_rgb(208, 211, 216),
            rounding: 4.0,
        },
        ThemeChoice::RosePineDawn => ThemeSpec {
            dark_mode: false,
            panel_bg: Color32::from_rgb(242, 233, 222),          // #F2E9DE (Overlay)
            window_bg: Color32::from_rgb(250, 244, 237),         // #FAF4ED (Base)
            extreme_bg: Color32::from_rgb(255, 253, 249),
            faint_bg: Color32::from_rgb(232, 222, 200),          // #E8DEC8
            code_bg: Color32::from_rgb(232, 222, 200),
            widget_noninteractive_bg: Color32::from_rgb(242, 233, 222),
            widget_inactive_bg: Color32::from_rgb(223, 208, 191), // #DFD0BF
            widget_hovered_bg: Color32::from_rgb(207, 189, 168),  // #CFBDA8
            widget_active_bg: Color32::from_rgb(191, 170, 148),   // #BFAA94
            text_primary: Color32::from_rgb(87, 82, 121),         // #575279
            text_bright: Color32::from_rgb(38, 35, 58),
            accent: Color32::from_rgb(40, 105, 131),             // #286983 (Pine teal)
            accent_fg: Color32::WHITE,
            border: Color32::from_rgb(207, 189, 168),
            rounding: 5.0,
        },
        ThemeChoice::EverforestLight => ThemeSpec {
            dark_mode: false,
            panel_bg: Color32::from_rgb(239, 235, 212),          // #EFEBD4
            window_bg: Color32::from_rgb(248, 245, 228),         // #F8F5E4
            extreme_bg: Color32::from_rgb(250, 248, 237),
            faint_bg: Color32::from_rgb(227, 221, 186),          // #E3DDBA
            code_bg: Color32::from_rgb(227, 221, 186),
            widget_noninteractive_bg: Color32::from_rgb(239, 235, 212),
            widget_inactive_bg: Color32::from_rgb(221, 214, 171), // #DDD6AB
            widget_hovered_bg: Color32::from_rgb(203, 194, 149),  // #CBC295
            widget_active_bg: Color32::from_rgb(183, 173, 125),   // #B7AD7D
            text_primary: Color32::from_rgb(92, 106, 114),        // #5C6A72
            text_bright: Color32::from_rgb(45, 53, 59),
            accent: Color32::from_rgb(141, 161, 1),              // #8DA101
            accent_fg: Color32::WHITE,
            border: Color32::from_rgb(203, 194, 149),
            rounding: 4.0,
        },
        ThemeChoice::PaperColorLight => ThemeSpec {
            dark_mode: false,
            panel_bg: Color32::from_rgb(228, 228, 228),          // #E4E4E4
            window_bg: Color32::from_rgb(238, 238, 238),         // #EEEEEE
            extreme_bg: Color32::from_rgb(245, 245, 245),
            faint_bg: Color32::from_rgb(216, 216, 216),          // #D8D8D8
            code_bg: Color32::from_rgb(216, 216, 216),
            widget_noninteractive_bg: Color32::from_rgb(228, 228, 228),
            widget_inactive_bg: Color32::from_rgb(208, 208, 208), // #D0D0D0
            widget_hovered_bg: Color32::from_rgb(188, 188, 188),  // #BCBCBC
            widget_active_bg: Color32::from_rgb(168, 168, 168),   // #A8A8A8
            text_primary: Color32::from_rgb(68, 68, 68),          // #444444
            text_bright: Color32::from_rgb(18, 18, 18),
            accent: Color32::from_rgb(0, 95, 175),               // #005FAF
            accent_fg: Color32::WHITE,
            border: Color32::from_rgb(188, 188, 188),
            rounding: 3.0,
        },
        _ => unreachable!(),
    };

    ctx.set_theme(if spec.dark_mode {
        egui::ThemePreference::Dark
    } else {
        egui::ThemePreference::Light
    });
    ctx.set_visuals(build_visuals(&spec));
}
