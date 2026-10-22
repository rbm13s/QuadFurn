pub mod baltic_nordic;
pub mod caucasus;
pub mod central_asia;
pub mod cjk;
pub mod defaults;
pub mod formatters;
pub mod slavic;
pub mod west_eu;

pub use defaults::default_project_l10n;
pub use formatters::*;

use crate::i18n::AppLanguage;

/// Looks up the translation for an English key across all 37 supported languages.
/// Returns `Some(&'static str)` if a translation exists in the glossary, or `None`.
pub fn lookup(lang: AppLanguage, en: &str) -> Option<&'static str> {
    match lang {
        AppLanguage::Ru => None,
        AppLanguage::En => None,

        // Caucasus
        AppLanguage::Ka => caucasus::lookup_ka(en),
        AppLanguage::Hy => caucasus::lookup_hy(en),
        AppLanguage::Az => caucasus::lookup_az(en),

        // Central Asia
        AppLanguage::Uz => central_asia::lookup_uz(en),
        AppLanguage::Kk => central_asia::lookup_kk(en),

        // Slavic
        AppLanguage::Uk => slavic::lookup_uk(en),
        AppLanguage::Be => slavic::lookup_be(en),
        AppLanguage::Pl => slavic::lookup_pl(en),
        AppLanguage::Cs => slavic::lookup_cs(en),
        AppLanguage::Sk => slavic::lookup_sk(en),
        AppLanguage::Sl => slavic::lookup_sl(en),
        AppLanguage::Bg => slavic::lookup_bg(en),
        AppLanguage::Sr => slavic::lookup_sr(en),
        AppLanguage::Hr => slavic::lookup_hr(en),

        // Baltic & Nordic
        AppLanguage::Lt => baltic_nordic::lookup_lt(en),
        AppLanguage::Lv => baltic_nordic::lookup_lv(en),
        AppLanguage::Et => baltic_nordic::lookup_et(en),
        AppLanguage::Fi => baltic_nordic::lookup_fi(en),
        AppLanguage::Da => baltic_nordic::lookup_da(en),
        AppLanguage::No => baltic_nordic::lookup_no(en),
        AppLanguage::Sv => baltic_nordic::lookup_sv(en),

        // Western & Southern European
        AppLanguage::De => west_eu::lookup_de(en),
        AppLanguage::Fr => west_eu::lookup_fr(en),
        AppLanguage::Es => west_eu::lookup_es(en),
        AppLanguage::It => west_eu::lookup_it(en),
        AppLanguage::Pt => west_eu::lookup_pt(en),
        AppLanguage::Nl => west_eu::lookup_nl(en),
        AppLanguage::Ro => west_eu::lookup_ro(en),
        AppLanguage::El => west_eu::lookup_el(en),
        AppLanguage::Hu => west_eu::lookup_hu(en),
        AppLanguage::Tr => west_eu::lookup_tr(en),

        // CJK
        AppLanguage::Ja => cjk::lookup_ja(en),
        AppLanguage::ZhCn => cjk::lookup_zh_cn(en),
        AppLanguage::ZhTw => cjk::lookup_zh_tw(en),
        AppLanguage::Ko => cjk::lookup_ko(en),
    }
}
