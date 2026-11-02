use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Supported UI and PDF export languages in QuadFurn CAD (37 languages).
/// All translations are loaded from `assets/locales/<lang>.json`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AppLanguage {
    Az,   // Azərbaycan (Azerbaijani)
    Be,   // Беларуская (Belarusian)
    Bg,   // Български (Bulgarian)
    Cs,   // Čeština (Czech)
    Da,   // Dansk (Danish)
    De,   // Deutsch (German)
    El,   // Ελληνικά (Greek)
    En,   // English
    Es,   // Español (Spanish)
    Et,   // Eesti (Estonian)
    Fi,   // Suomi (Finnish)
    Fr,   // Français (French)
    Hr,   // Hrvatski (Croatian)
    Hu,   // Magyar (Hungarian)
    Hy,   // Հայերեն (Armenian)
    It,   // Italiano (Italian)
    Ja,   // 日本語 (Japanese)
    Ka,   // ქართული (Georgian)
    Kk,   // Қазақша (Kazakh)
    Ko,   // Korean (한국어)
    Lt,   // Lietuvių (Lithuanian)
    Lv,   // Latviešu (Latvian)
    Nl,   // Nederlands (Dutch)
    No,   // Norwegian (Bokmål)
    Pl,   // Polish
    Pt,   // Portuguese
    Ro,   // Romanian
    Ru,   // Russian
    Sk,   // Slovak
    Sl,   // Slovenian
    Sr,   // Српски (Serbian)
    Sv,   // Swedish
    Tr,   // Turkish
    Uk,   // Ukrainian
    Uz,   // Oʻzbekcha (Uzbek)
    ZhCn, // Chinese Simplified
    ZhTw, // Chinese Traditional
}

impl Default for AppLanguage {
    fn default() -> Self {
        Self::detect_system_or_en()
    }
}

struct LangBundle {
    map: HashMap<String, &'static str>,
    l10n: L10n,
}

static LOCALES: LazyLock<HashMap<AppLanguage, LangBundle>> = LazyLock::new(|| {
    let mut all = HashMap::with_capacity(37);
    for lang in AppLanguage::ALL {
        let raw_json = lang.raw_locale_json();
        let parsed: HashMap<String, String> =
            serde_json::from_str(raw_json).expect("Failed to parse embedded locale JSON");
        let mut static_map: HashMap<String, &'static str> = HashMap::with_capacity(parsed.len());
        for (k, v) in parsed {
            let leaked: &'static str = Box::leak(v.into_boxed_str());
            static_map.insert(k, leaked);
        }
        let get = |key: &str| -> &'static str { static_map.get(key).copied().unwrap_or("") };
        let l10n = L10n {
            window_title: get("l10n.window_title"),
            open_in_new_tab: get("l10n.open_in_new_tab"),
            save_current_product: get("l10n.save_current_product"),
            save_current_product_hover: get("l10n.save_current_product_hover"),
            undo_btn: get("l10n.undo_btn"),
            tab_designer: get("l10n.tab_designer"),
            tab_specification: get("l10n.tab_specification"),
            tab_cutting: get("l10n.tab_cutting"),
            tab_materials: get("l10n.tab_materials"),
            theme_label: get("l10n.theme_label"),
            theme_dark: get("l10n.theme_dark"),
            theme_light: get("l10n.theme_light"),
            theme_system: get("l10n.theme_system"),
            products_bar_label: get("l10n.products_bar_label"),
            empty_object_btn: get("l10n.empty_object_btn"),
            empty_object_hover: get("l10n.empty_object_hover"),
            cabinet_template_btn: get("l10n.cabinet_template_btn"),
            cabinet_template_hover: get("l10n.cabinet_template_hover"),
            start_page_subtitle: get("l10n.start_page_subtitle"),
            start_create_section: get("l10n.start_create_section"),
            start_product_name_label: get("l10n.start_product_name_label"),
            start_product_name_hint: get("l10n.start_product_name_hint"),
            start_create_empty_btn: get("l10n.start_create_empty_btn"),
            start_create_empty_hover: get("l10n.start_create_empty_hover"),
            start_create_template_btn: get("l10n.start_create_template_btn"),
            start_create_template_hover: get("l10n.start_create_template_hover"),
            start_open_section: get("l10n.start_open_section"),
            start_open_btn: get("l10n.start_open_btn"),
            start_open_hover: get("l10n.start_open_hover"),
            start_recent_empty: get("l10n.start_recent_empty"),
            start_footer_status: get("l10n.start_footer_status"),
            unit_mm: get("l10n.unit_mm"),
            unit_mm_spaced: get("l10n.unit_mm_spaced"),
            unit_pcs: get("l10n.unit_pcs"),
            unit_m2: get("l10n.unit_m2"),
            unit_mp: get("l10n.unit_mp"),
            unit_sheets: get("l10n.unit_sheets"),
            btn_cancel: get("l10n.btn_cancel"),
            btn_delete: get("l10n.btn_delete"),
            btn_export_pdf: get("l10n.btn_export_pdf"),
            btn_export_csv: get("l10n.btn_export_csv"),
            inspector_heading: get("l10n.inspector_heading"),
            part_properties_heading: get("l10n.part_properties_heading"),
            spec_heading: get("l10n.spec_heading"),
            materials_heading: get("l10n.materials_heading"),
            pdf_sec1_title: get("l10n.pdf_sec1_title"),
            pdf_positions_label: get("l10n.pdf_positions_label"),
            pdf_total_parts_label: get("l10n.pdf_total_parts_label"),
            pdf_materials_in_project_label: get("l10n.pdf_materials_in_project_label"),
            pdf_all_dims_in_mm: get("l10n.pdf_all_dims_in_mm"),
            pdf_col_group: get("l10n.pdf_col_group"),
            pdf_col_name: get("l10n.pdf_col_name"),
            pdf_col_material: get("l10n.pdf_col_material"),
            pdf_col_full_dim: get("l10n.pdf_col_full_dim"),
            pdf_col_cut_dim: get("l10n.pdf_col_cut_dim"),
            pdf_col_qty: get("l10n.pdf_col_qty"),
            pdf_col_grain: get("l10n.pdf_col_grain"),
            pdf_col_edge_l1: get("l10n.pdf_col_edge_l1"),
            pdf_col_edge_l2: get("l10n.pdf_col_edge_l2"),
            pdf_col_edge_w1: get("l10n.pdf_col_edge_w1"),
            pdf_col_edge_w2: get("l10n.pdf_col_edge_w2"),
            pdf_no_parts: get("l10n.pdf_no_parts"),
            pdf_grain_fixed: get("l10n.pdf_grain_fixed"),
            pdf_grain_free: get("l10n.pdf_grain_free"),
            pdf_sec2_title: get("l10n.pdf_sec2_title"),
            pdf_overall_bounds: get("l10n.pdf_overall_bounds"),
            pdf_visible_parts: get("l10n.pdf_visible_parts"),
            pdf_view_top: get("l10n.pdf_view_top"),
            pdf_view_front: get("l10n.pdf_view_front"),
            pdf_view_side: get("l10n.pdf_view_side"),
            pdf_view_3d: get("l10n.pdf_view_3d"),
            pdf_axis_x: get("l10n.pdf_axis_x"),
            pdf_axis_y: get("l10n.pdf_axis_y"),
            pdf_axis_z: get("l10n.pdf_axis_z"),
            pdf_sec3_title: get("l10n.pdf_sec3_title"),
            pdf_sheet_word: get("l10n.pdf_sheet_word"),
            pdf_of_word: get("l10n.pdf_of_word"),
            pdf_no_placed_parts: get("l10n.pdf_no_placed_parts"),
            pdf_utilization: get("l10n.pdf_utilization"),
            pdf_parts_on_sheet: get("l10n.pdf_parts_on_sheet"),
            pdf_kerf: get("l10n.pdf_kerf"),
            pdf_trim: get("l10n.pdf_trim"),
            pdf_sheet_length: get("l10n.pdf_sheet_length"),
            pdf_remnant: get("l10n.pdf_remnant"),
            pdf_sec4_title: get("l10n.pdf_sec4_title"),
            pdf_total_sheets_to_cut: get("l10n.pdf_total_sheets_to_cut"),
            pdf_net_parts_area: get("l10n.pdf_net_parts_area"),
            pdf_avg_utilization: get("l10n.pdf_avg_utilization"),
            pdf_total_cut_length: get("l10n.pdf_total_cut_length"),
            pdf_materials_table_heading: get("l10n.pdf_materials_table_heading"),
            pdf_col_material_name: get("l10n.pdf_col_material_name"),
            pdf_col_thickness: get("l10n.pdf_col_thickness"),
            pdf_col_color_decor: get("l10n.pdf_col_color_decor"),
            pdf_col_sheet_format: get("l10n.pdf_col_sheet_format"),
            pdf_col_default_grain: get("l10n.pdf_col_default_grain"),
            pdf_col_parts_count_area: get("l10n.pdf_col_parts_count_area"),
            pdf_col_sheets_count: get("l10n.pdf_col_sheets_count"),
            pdf_grain_locked: get("l10n.pdf_grain_locked"),
            pdf_grain_none: get("l10n.pdf_grain_none"),
            pdf_edges_table_heading: get("l10n.pdf_edges_table_heading"),
            pdf_col_edge_name: get("l10n.pdf_col_edge_name"),
            pdf_col_edge_thickness: get("l10n.pdf_col_edge_thickness"),
            pdf_col_marker_color: get("l10n.pdf_col_marker_color"),
            pdf_col_banded_edges: get("l10n.pdf_col_banded_edges"),
            pdf_col_net_length: get("l10n.pdf_col_net_length"),
            pdf_col_with_waste: get("l10n.pdf_col_with_waste"),
            pdf_footer_passport: get("l10n.pdf_footer_passport"),
            pdf_page_word: get("l10n.pdf_page_word"),
        };
        all.insert(
            lang,
            LangBundle {
                map: static_map,
                l10n,
            },
        );
    }
    all
});

impl AppLanguage {
    pub const ALL: [AppLanguage; 37] = [
        AppLanguage::Az,
        AppLanguage::Be,
        AppLanguage::Bg,
        AppLanguage::Cs,
        AppLanguage::Da,
        AppLanguage::De,
        AppLanguage::El,
        AppLanguage::En,
        AppLanguage::Es,
        AppLanguage::Et,
        AppLanguage::Fi,
        AppLanguage::Fr,
        AppLanguage::Hr,
        AppLanguage::Hu,
        AppLanguage::Hy,
        AppLanguage::It,
        AppLanguage::Ja,
        AppLanguage::Ka,
        AppLanguage::Kk,
        AppLanguage::Ko,
        AppLanguage::Lt,
        AppLanguage::Lv,
        AppLanguage::Nl,
        AppLanguage::No,
        AppLanguage::Pl,
        AppLanguage::Pt,
        AppLanguage::Ro,
        AppLanguage::Ru,
        AppLanguage::Sk,
        AppLanguage::Sl,
        AppLanguage::Sr,
        AppLanguage::Sv,
        AppLanguage::Tr,
        AppLanguage::Uk,
        AppLanguage::Uz,
        AppLanguage::ZhCn,
        AppLanguage::ZhTw,
    ];

    /// Embedded JSON dictionary for this language.
    pub fn raw_locale_json(self) -> &'static str {
        match self {
            AppLanguage::Az => include_str!("../assets/locales/az.json"),
            AppLanguage::Be => include_str!("../assets/locales/be.json"),
            AppLanguage::Bg => include_str!("../assets/locales/bg.json"),
            AppLanguage::Cs => include_str!("../assets/locales/cs.json"),
            AppLanguage::Da => include_str!("../assets/locales/da.json"),
            AppLanguage::De => include_str!("../assets/locales/de.json"),
            AppLanguage::El => include_str!("../assets/locales/el.json"),
            AppLanguage::En => include_str!("../assets/locales/en.json"),
            AppLanguage::Es => include_str!("../assets/locales/es.json"),
            AppLanguage::Et => include_str!("../assets/locales/et.json"),
            AppLanguage::Fi => include_str!("../assets/locales/fi.json"),
            AppLanguage::Fr => include_str!("../assets/locales/fr.json"),
            AppLanguage::Hr => include_str!("../assets/locales/hr.json"),
            AppLanguage::Hu => include_str!("../assets/locales/hu.json"),
            AppLanguage::Hy => include_str!("../assets/locales/hy.json"),
            AppLanguage::It => include_str!("../assets/locales/it.json"),
            AppLanguage::Ja => include_str!("../assets/locales/ja.json"),
            AppLanguage::Ka => include_str!("../assets/locales/ka.json"),
            AppLanguage::Kk => include_str!("../assets/locales/kk.json"),
            AppLanguage::Ko => include_str!("../assets/locales/ko.json"),
            AppLanguage::Lt => include_str!("../assets/locales/lt.json"),
            AppLanguage::Lv => include_str!("../assets/locales/lv.json"),
            AppLanguage::Nl => include_str!("../assets/locales/nl.json"),
            AppLanguage::No => include_str!("../assets/locales/no.json"),
            AppLanguage::Pl => include_str!("../assets/locales/pl.json"),
            AppLanguage::Pt => include_str!("../assets/locales/pt.json"),
            AppLanguage::Ro => include_str!("../assets/locales/ro.json"),
            AppLanguage::Ru => include_str!("../assets/locales/ru.json"),
            AppLanguage::Sk => include_str!("../assets/locales/sk.json"),
            AppLanguage::Sl => include_str!("../assets/locales/sl.json"),
            AppLanguage::Sr => include_str!("../assets/locales/sr.json"),
            AppLanguage::Sv => include_str!("../assets/locales/sv.json"),
            AppLanguage::Tr => include_str!("../assets/locales/tr.json"),
            AppLanguage::Uk => include_str!("../assets/locales/uk.json"),
            AppLanguage::Uz => include_str!("../assets/locales/uz.json"),
            AppLanguage::ZhCn => include_str!("../assets/locales/zh_cn.json"),
            AppLanguage::ZhTw => include_str!("../assets/locales/zh_tw.json"),
        }
    }

    /// Detects operating system language on first application launch.
    /// If the OS language is not supported, defaults to English (`AppLanguage::En`).
    pub fn detect_system_or_en() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(lang) = crate::web::detect_browser_language() {
                return lang;
            }
            return AppLanguage::En;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            for tag in Self::collect_system_locale_tags() {
                if let Some(lang) = Self::from_locale_tag(&tag) {
                    return lang;
                }
            }
            AppLanguage::En
        }
    }

    /// Converts a locale tag string (e.g., `"ru-RU"`, `"uk_UA"`, `"zh-CN"`, `"zh-TW"`, `"de"`, `"en-US"`) to `AppLanguage`.
    pub fn from_locale_tag(tag: &str) -> Option<Self> {
        let clean = tag.trim().to_ascii_lowercase();
        if clean.is_empty() || clean == "c" || clean == "posix" {
            return None;
        }
        if clean == "zh-tw"
            || clean == "zh_tw"
            || clean == "zh-hk"
            || clean == "zh_hk"
            || clean == "zh-mo"
            || clean == "zh_mo"
            || clean.contains("hant")
            || clean == "cht"
        {
            return Some(AppLanguage::ZhTw);
        }
        let primary = clean.split(['-', '_', '.', '@']).next().unwrap_or("");
        match primary {
            "az" | "aze" => Some(AppLanguage::Az),
            "be" | "bel" => Some(AppLanguage::Be),
            "bg" | "bul" => Some(AppLanguage::Bg),
            "cs" | "ces" | "cze" => Some(AppLanguage::Cs),
            "da" | "dan" => Some(AppLanguage::Da),
            "de" | "deu" | "ger" => Some(AppLanguage::De),
            "el" | "ell" | "gre" | "gr" => Some(AppLanguage::El),
            "en" | "eng" => Some(AppLanguage::En),
            "es" | "spa" => Some(AppLanguage::Es),
            "et" | "est" | "ee" => Some(AppLanguage::Et),
            "fi" | "fin" => Some(AppLanguage::Fi),
            "fr" | "fra" | "fre" => Some(AppLanguage::Fr),
            "hr" | "hrv" => Some(AppLanguage::Hr),
            "hu" | "hun" => Some(AppLanguage::Hu),
            "hy" | "hye" | "arm" | "am" => Some(AppLanguage::Hy),
            "it" | "ita" => Some(AppLanguage::It),
            "ja" | "jpn" | "jp" => Some(AppLanguage::Ja),
            "ka" | "kat" | "geo" | "ge" => Some(AppLanguage::Ka),
            "kk" | "kaz" | "kz" => Some(AppLanguage::Kk),
            "ko" | "kor" => Some(AppLanguage::Ko),
            "lt" | "lit" => Some(AppLanguage::Lt),
            "lv" | "lav" => Some(AppLanguage::Lv),
            "nl" | "nld" | "dut" => Some(AppLanguage::Nl),
            "no" | "nb" | "nn" | "nor" | "nob" => Some(AppLanguage::No),
            "pl" | "pol" => Some(AppLanguage::Pl),
            "pt" | "por" => Some(AppLanguage::Pt),
            "ro" | "ron" | "rum" => Some(AppLanguage::Ro),
            "ru" | "rus" => Some(AppLanguage::Ru),
            "sk" | "slk" | "slo" => Some(AppLanguage::Sk),
            "sl" | "slv" | "svn" => Some(AppLanguage::Sl),
            "sr" | "srp" | "scc" => Some(AppLanguage::Sr),
            "sv" | "swe" => Some(AppLanguage::Sv),
            "tr" | "tur" => Some(AppLanguage::Tr),
            "uk" | "ukr" | "ua" => Some(AppLanguage::Uk),
            "uz" | "uzb" => Some(AppLanguage::Uz),
            "zh" | "chs" | "zho" | "chi" => Some(AppLanguage::ZhCn),
            _ => None,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn collect_system_locale_tags() -> Vec<String> {
        let mut tags = Vec::new();

        for var in ["QUADFURN_LANG", "LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(val) = std::env::var(var) {
                if !val.trim().is_empty() {
                    tags.push(val);
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            #[link(name = "kernel32")]
            extern "system" {
                fn GetUserPreferredUILanguages(
                    dw_flags: u32,
                    pul_num_languages: *mut u32,
                    pwsz_languages_buffer: *mut u16,
                    pcch_languages_buffer: *mut u32,
                ) -> i32;
                fn GetUserDefaultLocaleName(lp_locale_name: *mut u16, cch_locale_name: i32) -> i32;
            }

            let mut num_langs = 0u32;
            let mut buf = [0u16; 256];
            let mut buf_len = buf.len() as u32;
            let ok = unsafe {
                GetUserPreferredUILanguages(0x8, &mut num_langs, buf.as_mut_ptr(), &mut buf_len)
            };
            if ok != 0 && buf_len > 1 {
                let slice = &buf[..(buf_len as usize)];
                for part in slice.split(|&ch| ch == 0) {
                    if !part.is_empty() {
                        tags.push(String::from_utf16_lossy(part));
                    }
                }
            }

            let mut loc_buf = [0u16; 85];
            let len =
                unsafe { GetUserDefaultLocaleName(loc_buf.as_mut_ptr(), loc_buf.len() as i32) };
            if len > 1 {
                tags.push(String::from_utf16_lossy(&loc_buf[..(len as usize - 1)]));
            }
        }

        tags
    }

    /// Short language code for compact button in the top menu bar.
    pub fn code(self) -> &'static str {
        match self {
            AppLanguage::Az => "AZ",
            AppLanguage::Be => "BE",
            AppLanguage::Bg => "BG",
            AppLanguage::Cs => "CS",
            AppLanguage::Da => "DA",
            AppLanguage::De => "DE",
            AppLanguage::El => "EL",
            AppLanguage::En => "EN",
            AppLanguage::Es => "ES",
            AppLanguage::Et => "ET",
            AppLanguage::Fi => "FI",
            AppLanguage::Fr => "FR",
            AppLanguage::Hr => "HR",
            AppLanguage::Hu => "HU",
            AppLanguage::Hy => "HY",
            AppLanguage::It => "IT",
            AppLanguage::Ja => "JA",
            AppLanguage::Ka => "KA",
            AppLanguage::Kk => "KK",
            AppLanguage::Ko => "KO",
            AppLanguage::Lt => "LT",
            AppLanguage::Lv => "LV",
            AppLanguage::Nl => "NL",
            AppLanguage::No => "NO",
            AppLanguage::Pl => "PL",
            AppLanguage::Pt => "PT",
            AppLanguage::Ro => "RO",
            AppLanguage::Ru => "RU",
            AppLanguage::Sk => "SK",
            AppLanguage::Sl => "SL",
            AppLanguage::Sr => "SR",
            AppLanguage::Sv => "SV",
            AppLanguage::Tr => "TR",
            AppLanguage::Uk => "UK",
            AppLanguage::Uz => "UZ",
            AppLanguage::ZhCn => "ZH",
            AppLanguage::ZhTw => "ZH-TW",
        }
    }

    /// Full language label for dropdown menu.
    pub fn label(self) -> &'static str {
        match self {
            AppLanguage::Az => "Azərbaycan (Azerbaijani)",
            AppLanguage::Be => "Беларуская (Belarusian)",
            AppLanguage::Bg => "Български (Bulgarian)",
            AppLanguage::Cs => "Čeština (Czech)",
            AppLanguage::Da => "Dansk (Danish)",
            AppLanguage::De => "Deutsch (German)",
            AppLanguage::El => "Ελληνικά (Greek)",
            AppLanguage::En => "English",
            AppLanguage::Es => "Español (Spanish)",
            AppLanguage::Et => "Eesti (Estonian)",
            AppLanguage::Fi => "Suomi (Finnish)",
            AppLanguage::Fr => "Français (French)",
            AppLanguage::Hr => "Hrvatski (Croatian)",
            AppLanguage::Hu => "Magyar (Hungarian)",
            AppLanguage::Hy => "Հայերեն (Armenian)",
            AppLanguage::It => "Italiano (Italian)",
            AppLanguage::Ja => "日本語 (Japanese)",
            AppLanguage::Ka => "ქართული (Georgian)",
            AppLanguage::Kk => "Қазақша (Kazakh)",
            AppLanguage::Ko => "Korean (한국어)",
            AppLanguage::Lt => "Lietuvių (Lithuanian)",
            AppLanguage::Lv => "Latviešu (Latvian)",
            AppLanguage::Nl => "Nederlands (Dutch)",
            AppLanguage::No => "Norsk (Bokmål)",
            AppLanguage::Pl => "Polski (Polish)",
            AppLanguage::Pt => "Português (Portuguese)",
            AppLanguage::Ro => "Română (Romanian)",
            AppLanguage::Ru => "Russian (Русский)",
            AppLanguage::Sk => "Slovenský (Slovak)",
            AppLanguage::Sl => "Slovenski (Slovenian)",
            AppLanguage::Sr => "Српски (Serbian)",
            AppLanguage::Sv => "Svenska (Swedish)",
            AppLanguage::Tr => "Türkçe (Turkish)",
            AppLanguage::Uk => "Українська (Ukrainian)",
            AppLanguage::Uz => "Oʻzbekcha (Uzbek)",
            AppLanguage::ZhCn => "简体中文 (Chinese Simpl.)",
            AppLanguage::ZhTw => "繁體中文 (Chinese Trad.)",
        }
    }

    /// Label on language switcher button next to theme choice.
    pub fn switcher_button_text(self) -> &'static str {
        match self {
            AppLanguage::Az => "🌐 AZ",
            AppLanguage::Be => "🌐 BE",
            AppLanguage::Bg => "🌐 BG",
            AppLanguage::Cs => "🌐 CS",
            AppLanguage::Da => "🌐 DA",
            AppLanguage::De => "🌐 DE",
            AppLanguage::El => "🌐 EL",
            AppLanguage::En => "🌐 EN",
            AppLanguage::Es => "🌐 ES",
            AppLanguage::Et => "🌐 ET",
            AppLanguage::Fi => "🌐 FI",
            AppLanguage::Fr => "🌐 FR",
            AppLanguage::Hr => "🌐 HR",
            AppLanguage::Hu => "🌐 HU",
            AppLanguage::Hy => "🌐 HY",
            AppLanguage::It => "🌐 IT",
            AppLanguage::Ja => "🌐 JA",
            AppLanguage::Ka => "🌐 KA",
            AppLanguage::Kk => "🌐 KK",
            AppLanguage::Ko => "🌐 KO",
            AppLanguage::Lt => "🌐 LT",
            AppLanguage::Lv => "🌐 LV",
            AppLanguage::Nl => "🌐 NL",
            AppLanguage::No => "🌐 NO",
            AppLanguage::Pl => "🌐 PL",
            AppLanguage::Pt => "🌐 PT",
            AppLanguage::Ro => "🌐 RO",
            AppLanguage::Ru => "🌐 RU",
            AppLanguage::Sk => "🌐 SK",
            AppLanguage::Sl => "🌐 SL",
            AppLanguage::Sr => "🌐 SR",
            AppLanguage::Sv => "🌐 SV",
            AppLanguage::Tr => "🌐 TR",
            AppLanguage::Uk => "🌐 UK",
            AppLanguage::Uz => "🌐 UZ",
            AppLanguage::ZhCn => "🌐 ZH",
            AppLanguage::ZhTw => "🌐 ZH-TW",
        }
    }

    /// Hover tooltip text for language switcher button.
    pub fn switcher_hover_tooltip(self) -> &'static str {
        self.t("l10n.switcher_hover_tooltip")
    }

    /// Looks up a translation key in the embedded JSON dictionary (`assets/locales/<lang>.json`),
    /// falling back to `en.json` or the key itself.
    #[inline]
    pub fn t<'a>(self, key: &'a str) -> &'a str {
        if let Some(bundle) = LOCALES.get(&self) {
            if let Some(&val) = bundle.map.get(key) {
                return val;
            }
        }
        if let Some(en_bundle) = LOCALES.get(&AppLanguage::En) {
            if let Some(&val) = en_bundle.map.get(key) {
                return val;
            }
        }
        key
    }

    /// Formats a parameterized translation template from `assets/locales/<lang>.json`
    /// by substituting `{0}`, `{1}`, `{2}`, etc.
    pub fn tf(self, key: &str, args: &[&dyn std::fmt::Display]) -> String {
        let mut out = self.t(key).to_string();
        for (i, arg) in args.iter().enumerate() {
            let placeholder = format!("{{{}}}", i);
            out = out.replace(&placeholder, &arg.to_string());
        }
        out
    }

    /// Looks up a phrase in the embedded JSON dictionary by its English key.
    #[inline]
    pub fn tr<'a>(self, ru: &'a str, en: &'a str) -> &'a str {
        if let Some(bundle) = LOCALES.get(&self) {
            if let Some(&val) = bundle.map.get(en) {
                return val;
            }
        }
        match self {
            AppLanguage::Ru => ru,
            _ => en,
        }
    }

    /// Returns a reference to the static localization dictionary for this language.
    #[inline]
    pub fn dict(self) -> &'static L10n {
        &LOCALES
            .get(&self)
            .expect("Locale bundle must be initialized")
            .l10n
    }
}

/// Returns true if the application is running on macOS (either native or in browser on macOS).
pub fn is_macos_platform() -> bool {
    #[cfg(target_os = "macos")]
    {
        true
    }
    #[cfg(not(target_os = "macos"))]
    {
        crate::web::is_macos_web()
    }
}

/// Dynamically adapts shortcut text from Windows/Linux (Ctrl) to macOS (Command (⌘)) if running on macOS.
pub fn adapt_shortcuts(text: &str) -> String {
    if is_macos_platform() {
        text.replace("Ctrl+", "Command (⌘)+")
            .replace("Ctrl +", "Command (⌘) +")
            .replace("Shift/Ctrl", "Shift/Command (⌘)")
            .replace("Shift / Ctrl", "Shift / Command (⌘)")
            .replace("Cmd+", "Command (⌘)+")
            .replace("Cmd +", "Command (⌘) +")
            .replace("Shift/Cmd", "Shift/Command (⌘)")
            .replace("Shift / Cmd", "Shift / Command (⌘)")
            .replace("Ctrl", "Command (⌘)")
    } else {
        text.to_string()
    }
}

/// Static type-safe localization dictionary for UI and PDF generator (populated from JSON).
pub struct L10n {
    pub window_title: &'static str,
    pub open_in_new_tab: &'static str,
    pub save_current_product: &'static str,
    pub save_current_product_hover: &'static str,
    pub undo_btn: &'static str,
    pub tab_designer: &'static str,
    pub tab_specification: &'static str,
    pub tab_cutting: &'static str,
    pub tab_materials: &'static str,
    pub theme_label: &'static str,
    pub theme_dark: &'static str,
    pub theme_light: &'static str,
    pub theme_system: &'static str,
    pub products_bar_label: &'static str,
    pub empty_object_btn: &'static str,
    pub empty_object_hover: &'static str,
    pub cabinet_template_btn: &'static str,
    pub cabinet_template_hover: &'static str,
    pub start_page_subtitle: &'static str,
    pub start_create_section: &'static str,
    pub start_product_name_label: &'static str,
    pub start_product_name_hint: &'static str,
    pub start_create_empty_btn: &'static str,
    pub start_create_empty_hover: &'static str,
    pub start_create_template_btn: &'static str,
    pub start_create_template_hover: &'static str,
    pub start_open_section: &'static str,
    pub start_open_btn: &'static str,
    pub start_open_hover: &'static str,
    pub start_recent_empty: &'static str,
    pub start_footer_status: &'static str,
    // General UI and measurement units
    pub unit_mm: &'static str,
    pub unit_mm_spaced: &'static str,
    pub unit_pcs: &'static str,
    pub unit_m2: &'static str,
    pub unit_mp: &'static str,
    pub unit_sheets: &'static str,
    pub btn_cancel: &'static str,
    pub btn_delete: &'static str,
    pub btn_export_pdf: &'static str,
    pub btn_export_csv: &'static str,
    pub inspector_heading: &'static str,
    pub part_properties_heading: &'static str,
    pub spec_heading: &'static str,
    pub materials_heading: &'static str,
    // PDF document localization (Specification, 4 projections, cutting layout, materials and edges)
    pub pdf_sec1_title: &'static str,
    pub pdf_positions_label: &'static str,
    pub pdf_total_parts_label: &'static str,
    pub pdf_materials_in_project_label: &'static str,
    pub pdf_all_dims_in_mm: &'static str,
    pub pdf_col_group: &'static str,
    pub pdf_col_name: &'static str,
    pub pdf_col_material: &'static str,
    pub pdf_col_full_dim: &'static str,
    pub pdf_col_cut_dim: &'static str,
    pub pdf_col_qty: &'static str,
    pub pdf_col_grain: &'static str,
    pub pdf_col_edge_l1: &'static str,
    pub pdf_col_edge_l2: &'static str,
    pub pdf_col_edge_w1: &'static str,
    pub pdf_col_edge_w2: &'static str,
    pub pdf_no_parts: &'static str,
    pub pdf_grain_fixed: &'static str,
    pub pdf_grain_free: &'static str,
    pub pdf_sec2_title: &'static str,
    pub pdf_overall_bounds: &'static str,
    pub pdf_visible_parts: &'static str,
    pub pdf_view_top: &'static str,
    pub pdf_view_front: &'static str,
    pub pdf_view_side: &'static str,
    pub pdf_view_3d: &'static str,
    pub pdf_axis_x: &'static str,
    pub pdf_axis_y: &'static str,
    pub pdf_axis_z: &'static str,
    pub pdf_sec3_title: &'static str,
    pub pdf_sheet_word: &'static str,
    pub pdf_of_word: &'static str,
    pub pdf_no_placed_parts: &'static str,
    pub pdf_utilization: &'static str,
    pub pdf_parts_on_sheet: &'static str,
    pub pdf_kerf: &'static str,
    pub pdf_trim: &'static str,
    pub pdf_sheet_length: &'static str,
    pub pdf_remnant: &'static str,
    pub pdf_sec4_title: &'static str,
    pub pdf_total_sheets_to_cut: &'static str,
    pub pdf_net_parts_area: &'static str,
    pub pdf_avg_utilization: &'static str,
    pub pdf_total_cut_length: &'static str,
    pub pdf_materials_table_heading: &'static str,
    pub pdf_col_material_name: &'static str,
    pub pdf_col_thickness: &'static str,
    pub pdf_col_color_decor: &'static str,
    pub pdf_col_sheet_format: &'static str,
    pub pdf_col_default_grain: &'static str,
    pub pdf_col_parts_count_area: &'static str,
    pub pdf_col_sheets_count: &'static str,
    pub pdf_grain_locked: &'static str,
    pub pdf_grain_none: &'static str,
    pub pdf_edges_table_heading: &'static str,
    pub pdf_col_edge_name: &'static str,
    pub pdf_col_edge_thickness: &'static str,
    pub pdf_col_marker_color: &'static str,
    pub pdf_col_banded_edges: &'static str,
    pub pdf_col_net_length: &'static str,
    pub pdf_col_with_waste: &'static str,
    pub pdf_footer_passport: &'static str,
    pub pdf_page_word: &'static str,
}

impl L10n {
    pub fn undo_btn_label(&self) -> String {
        adapt_shortcuts(self.undo_btn)
    }

    pub fn save_hover_label(&self) -> String {
        adapt_shortcuts(self.save_current_product_hover)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_dictionaries_key_parity() {
        let en_bundle = LOCALES
            .get(&AppLanguage::En)
            .expect("English locale must exist");
        assert!(
            en_bundle.map.len() >= 450,
            "Expected at least 450 keys in en.json, got {}",
            en_bundle.map.len()
        );

        for lang in AppLanguage::ALL {
            let bundle = LOCALES
                .get(&lang)
                .unwrap_or_else(|| panic!("Missing locale bundle for {:?}", lang));
            assert_eq!(
                en_bundle.map.len(),
                bundle.map.len(),
                "Key count mismatch in {:?}",
                lang
            );
            for (k, _) in &en_bundle.map {
                let val = bundle
                    .map
                    .get(k)
                    .unwrap_or_else(|| panic!("Missing key '{}' in {:?}", k, lang));
                assert!(
                    !val.is_empty(),
                    "Empty translation for key '{}' in {:?}",
                    k,
                    lang
                );
            }
        }
    }

    #[test]
    fn test_asian_translations_dont_fallback_to_english() {
        assert_eq!(AppLanguage::Ja.t("Tool:"), "ツール:");
        assert_eq!(AppLanguage::ZhCn.t("Tool:"), "工具:");
        assert_eq!(AppLanguage::Ko.t("Tool:"), "도구:");

        assert_eq!(AppLanguage::Ja.t("⊞ Equal Views"), "⊞ ウィンドウ均等化");
        assert_eq!(AppLanguage::ZhCn.t("⊞ Equal Views"), "⊞ 均分视图");
        assert_eq!(AppLanguage::Ko.t("⊞ Equal Views"), "⊞ 창 크기 균등화");

        assert_eq!(AppLanguage::Ja.t("Cutting Layout"), "木取り図");
        assert_eq!(AppLanguage::ZhCn.t("Cutting Layout"), "排料裁切图");
        assert_eq!(AppLanguage::Ko.t("Cutting Layout"), "재단 배치도");

        assert_eq!(AppLanguage::Ja.t("Saw kerf width:"), "ノコ刃の切削幅:");
    }

    #[test]
    fn test_adapt_shortcuts_logic() {
        let orig = "Save (Ctrl+S) and Copy (Ctrl+C), Shift/Ctrl + click";
        let expected_mac = "Save (Command (⌘)+S) and Copy (Command (⌘)+C), Shift/Command (⌘) + click";
        if is_macos_platform() {
            let adapted = adapt_shortcuts(orig);
            assert_eq!(adapted, expected_mac);
        } else {
            let adapted = adapt_shortcuts(orig);
            assert_eq!(adapted, orig);
        }
    }

    #[test]
    fn test_adapt_shortcuts_mac_formatting() {
        let text = "Undo (Ctrl+Z), Copy (Ctrl+C), Paste (Ctrl+V), Duplicate (Ctrl+D), Save (Ctrl+S), Shift/Ctrl + click, hold Ctrl";
        let mac_adapted = text
            .replace("Ctrl+", "Command (⌘)+")
            .replace("Ctrl +", "Command (⌘) +")
            .replace("Shift/Ctrl", "Shift/Command (⌘)")
            .replace("Shift / Ctrl", "Shift / Command (⌘)")
            .replace("Cmd+", "Command (⌘)+")
            .replace("Cmd +", "Command (⌘) +")
            .replace("Shift/Cmd", "Shift/Command (⌘)")
            .replace("Shift / Cmd", "Shift / Command (⌘)")
            .replace("Ctrl", "Command (⌘)");
        assert_eq!(
            mac_adapted,
            "Undo (Command (⌘)+Z), Copy (Command (⌘)+C), Paste (Command (⌘)+V), Duplicate (Command (⌘)+D), Save (Command (⌘)+S), Shift/Command (⌘) + click, hold Command (⌘)"
        );
    }
}
