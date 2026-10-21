use crate::cutter::{build_specification, compute_cutting_layout, CuttingReport, SpecRow};
use crate::i18n::{AppLanguage, L10n};
use crate::model::{EdgeId, Project};
use ab_glyph::{Font, FontArc, GlyphId};
use glam::Vec3;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Exports parts specification to CSV with UTF-8 BOM and ';' delimiter using default language.
#[allow(dead_code)]
pub fn export_spec_to_csv_file(project: &Project, path: &Path) -> Result<(), String> {
    export_spec_to_csv_file_i18n(project, path, AppLanguage::Ru)
}

/// Exports parts specification to CSV localized to `lang`.
pub fn export_spec_to_csv_file_i18n(
    project: &Project,
    path: &Path,
    lang: AppLanguage,
) -> Result<(), String> {
    let mut file = File::create(path).map_err(|e| {
        format!("{}: {}", lang.tr("Ошибка создания файла", "Error creating file"), e)
    })?;

    // UTF-8 BOM so Microsoft Excel correctly displays Unicode characters
    file.write_all(&[0xEF, 0xBB, 0xBF]).map_err(|e| {
        format!("{}: {}", lang.tr("Ошибка записи BOM", "Error writing BOM"), e)
    })?;

    let content = generate_csv_string_i18n(project, lang);
    file.write_all(content.as_bytes()).map_err(|e| {
        format!("{}: {}", lang.tr("Ошибка записи CSV", "Error writing CSV"), e)
    })?;

    Ok(())
}

/// Generates CSV string of parts specification (';' delimiter) with default language.
#[allow(dead_code)]
pub fn generate_csv_string(project: &Project) -> String {
    generate_csv_string_i18n(project, AppLanguage::Ru)
}

/// Generates CSV string of parts specification localized to `lang`.
pub fn generate_csv_string_i18n(project: &Project, lang: AppLanguage) -> String {
    let rows = build_specification(project);
    let mut out = String::new();

    if lang == AppLanguage::Ru {
        out.push_str("№;Группа;Наименование;Материал;Толщина (мм);Габарит Длина (мм);Габарит Ширина (мм);Пил Длина (мм);Пил Ширина (мм);Кол-во (шт);Текстура (Вращение);Кромка Д1;Кромка Д2;Кромка Ш1;Кромка Ш2\r\n");
    } else if lang == AppLanguage::Uk {
        out.push_str("№;Група;Найменування;Матеріал;Товщина (мм);Габарит Довжина (мм);Габарит Ширина (мм);Пил Довжина (мм);Пил Ширина (мм);Кіл-ть (шт);Текстура (Обертання);Крайка Д1;Крайка Д2;Крайка Ш1;Крайка Ш2\r\n");
    } else {
        out.push_str("#;Group;Name;Material;Thickness (mm);Full Length (mm);Full Width (mm);Cut Length (mm);Cut Width (mm);Qty (pcs);Grain (Rotation);Edge L1;Edge L2;Edge W1;Edge W2\r\n");
    }

    for r in rows {
        let grain_str = if r.grain_fixed {
            match lang {
                AppLanguage::Ru => "Фикс. (По текстуре)",
                AppLanguage::Uk => "Фікс. (За текстурою)",
                _ => "Locked (With grain)",
            }
        } else {
            match lang {
                AppLanguage::Ru => "Свободно (Вращ. Да)",
                AppLanguage::Uk => "Вільно (Оберт. Так)",
                _ => "Free (Can rotate)",
            }
        };
        let l1 = SpecRow::edge_label_i18n(r.edges.l1, &project.edge_bandings, lang);
        let l2 = SpecRow::edge_label_i18n(r.edges.l2, &project.edge_bandings, lang);
        let w1 = SpecRow::edge_label_i18n(r.edges.w1, &project.edge_bandings, lang);
        let w2 = SpecRow::edge_label_i18n(r.edges.w2, &project.edge_bandings, lang);

        out.push_str(&format!(
            "{};\"{}\";\"{}\";\"{}\";{:.1};{:.1};{:.1};{:.1};{:.1};{};{};\"{}\";\"{}\";\"{}\";\"{}\"\r\n",
            r.index,
            r.group_name.replace('"', "\"\""),
            r.name.replace('"', "\"\""),
            r.material_name.replace('"', "\"\""),
            r.thickness_mm,
            r.full_length_mm,
            r.full_width_mm,
            r.cut_length_mm,
            r.cut_width_mm,
            r.quantity,
            grain_str,
            l1,
            l2,
            w1,
            w2,
        ));
    }

    out
}

/// Generates tab-separated text table of cutting sizes for clipboard pasting into external optimization software (Length, Width, Qty, Grain, Edges, Note), localized to `lang`.
pub fn generate_cutting_clipboard_text_i18n(project: &Project, lang: AppLanguage) -> String {
    let rows = build_specification(project);
    let mut out = String::new();
    match lang {
        AppLanguage::Ru => {
            out.push_str("# Материал\tПил Длина (мм)\tПил Ширина (мм)\tКол-во\tТекстура\tД1\tД2\tШ1\tШ2\tНаименование (Габарит)\r\n");
        }
        AppLanguage::Uk => {
            out.push_str("# Матеріал\tПил Довжина (мм)\tПил Ширина (мм)\tКіл-ть\tТекстура\tД1\tД2\tШ1\tШ2\tНайменування (Габарит)\r\n");
        }
        _ => {
            out.push_str("# Material\tCut Length (mm)\tCut Width (mm)\tQty\tGrain\tL1\tL2\tW1\tW2\tName (Bounds)\r\n");
        }
    }
    for r in rows {
        let tex = if r.grain_fixed { "1" } else { "0" };
        let l1 = SpecRow::edge_short_mm(r.edges.l1, &project.edge_bandings);
        let l2 = SpecRow::edge_short_mm(r.edges.l2, &project.edge_bandings);
        let w1 = SpecRow::edge_short_mm(r.edges.w1, &project.edge_bandings);
        let w2 = SpecRow::edge_short_mm(r.edges.w2, &project.edge_bandings);
        out.push_str(&format!(
            "{}\t{:.1}\t{:.1}\t{}\t{}\t{}\t{}\t{}\t{}\t{} ({:.0}x{:.0})\r\n",
            r.material_name,
            r.cut_length_mm,
            r.cut_width_mm,
            r.quantity,
            tex,
            l1,
            l2,
            w1,
            w2,
            r.name,
            r.full_length_mm,
            r.full_width_mm
        ));
    }
    out
}

/// Generates tab-separated text table of cutting sizes for clipboard pasting into external optimization software (default Russian headers).
#[allow(dead_code)]
pub fn generate_cutting_clipboard_text(project: &Project) -> String {
    generate_cutting_clipboard_text_i18n(project, AppLanguage::Ru)
}

/// If given a TrueType Collection (`.ttc`, header `ttcf`), extracts the first face (face 0)
/// into a standalone TrueType (`.ttf`) buffer suitable for `ab_glyph::FontArc`, `egui::FontData`, and `/FontFile2` in PDF.
pub fn ensure_ttf_bytes(bytes: Vec<u8>) -> Vec<u8> {
    if bytes.len() < 16 || &bytes[0..4] != b"ttcf" {
        return bytes;
    }
    let num_fonts = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
    if num_fonts == 0 {
        return bytes;
    }
    let face0_offset = u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
    if face0_offset + 12 > bytes.len() {
        return bytes;
    }
    let num_tables =
        u16::from_be_bytes([bytes[face0_offset + 4], bytes[face0_offset + 5]]) as usize;
    let dir_end = face0_offset + 12 + num_tables * 16;
    if dir_end > bytes.len() {
        return bytes;
    }

    let mut out = Vec::with_capacity(bytes.len() / 2);
    out.extend_from_slice(&bytes[face0_offset..face0_offset + 12]);
    let records_pos = out.len();
    out.resize(records_pos + num_tables * 16, 0);

    for i in 0..num_tables {
        let rec_src = face0_offset + 12 + i * 16;
        let tag = &bytes[rec_src..rec_src + 4];
        let checksum = &bytes[rec_src + 4..rec_src + 8];
        let orig_offset = u32::from_be_bytes([
            bytes[rec_src + 8],
            bytes[rec_src + 9],
            bytes[rec_src + 10],
            bytes[rec_src + 11],
        ]) as usize;
        let length = u32::from_be_bytes([
            bytes[rec_src + 12],
            bytes[rec_src + 13],
            bytes[rec_src + 14],
            bytes[rec_src + 15],
        ]) as usize;

        if orig_offset + length > bytes.len() {
            return bytes;
        }
        let new_offset = out.len() as u32;
        let rec_dst = records_pos + i * 16;
        out[rec_dst..rec_dst + 4].copy_from_slice(tag);
        out[rec_dst + 4..rec_dst + 8].copy_from_slice(checksum);
        out[rec_dst + 8..rec_dst + 12].copy_from_slice(&new_offset.to_be_bytes());
        out[rec_dst + 12..rec_dst + 16].copy_from_slice(&(length as u32).to_be_bytes());

        out.extend_from_slice(&bytes[orig_offset..orig_offset + length]);
        while out.len() % 4 != 0 {
            out.push(0);
        }
    }
    out
}

/// Returns the Windows Fonts directory, checking `%WINDIR%` or `%SystemRoot%` before falling back to `C:\Windows\Fonts`.
#[allow(dead_code)]
pub fn get_windows_fonts_dir() -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(windir) = std::env::var("WINDIR") {
            let trimmed = windir.trim();
            if !trimmed.is_empty() {
                return std::path::PathBuf::from(trimmed).join("Fonts");
            }
        }
        if let Ok(sysroot) = std::env::var("SystemRoot") {
            let trimmed = sysroot.trim();
            if !trimmed.is_empty() {
                return std::path::PathBuf::from(trimmed).join("Fonts");
            }
        }
        std::path::PathBuf::from("C:\\Windows\\Fonts")
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::path::PathBuf::from("C:/Windows/Fonts")
    }
}

/// Reads a system font file bytes, resolving Windows font paths dynamically via `%WINDIR%` / `%SystemRoot%` with fallback.
pub fn read_system_font_bytes(path: &str) -> Option<Vec<u8>> {
    #[cfg(target_os = "windows")]
    if let Some(stripped) = path.strip_prefix("C:\\Windows\\Fonts\\") {
        let win_fonts = get_windows_fonts_dir();
        let dynamic_path = win_fonts.join(stripped);
        if let Ok(bytes) = std::fs::read(&dynamic_path) {
            return Some(bytes);
        }
    }
    std::fs::read(path).ok()
}

/// Helper structure for TrueType font handling in vector PDF generation with full Latin, Cyrillic, and CJK support.
struct PdfTrueTypeFont {
    raw_bytes: Vec<u8>,
    font: FontArc,
    upm: f32,
    used_glyphs: BTreeMap<u16, (char, i32)>,
}

impl PdfTrueTypeFont {
    fn load(candidates: &[&str], lang: AppLanguage, bold: bool) -> Result<Self, String> {
        for path in candidates {
            if let Some(raw) = read_system_font_bytes(path) {
                let bytes = ensure_ttf_bytes(raw);
                if let Ok(font) = FontArc::try_from_vec(bytes.clone()) {
                    let upm = font.units_per_em().unwrap_or(2048.0).max(1.0);
                    return Ok(Self {
                        raw_bytes: bytes,
                        font,
                        upm,
                        used_glyphs: BTreeMap::new(),
                    });
                }
            }
        }

        // Embedded fallback: When no system fonts can be read from disk
        // (critical for WebAssembly in browsers where local filesystem is absent,
        // as well as containerized / minimal Linux environments),
        // fallback to the embedded TrueType fonts provided by egui (Ubuntu-Light / Hack).
        let def = egui::FontDefinitions::default();
        let fallback_key = if bold { "Hack" } else { "Ubuntu-Light" };
        if let Some(font_data) = def
            .font_data
            .get(fallback_key)
            .or_else(|| def.font_data.get("Ubuntu-Light"))
            .or_else(|| def.font_data.get("Hack"))
        {
            let bytes = font_data.font.to_vec();
            if let Ok(font) = FontArc::try_from_vec(bytes.clone()) {
                let upm = font.units_per_em().unwrap_or(2048.0).max(1.0);
                return Ok(Self {
                    raw_bytes: bytes,
                    font,
                    upm,
                    used_glyphs: BTreeMap::new(),
                });
            }
        }

        Err(match lang {
            AppLanguage::Ru => {
                "Не удалось найти шрифт TrueType для экспорта в PDF".to_string()
            }
            AppLanguage::Uk => {
                "Не вдалося знайти шрифт TrueType для експорту в PDF".to_string()
            }
            _ => {
                "Failed to find a TrueType font for PDF export".to_string()
            }
        })
    }

    fn load_for_lang(lang: AppLanguage, bold: bool) -> Result<Self, String> {
        let candidates: &[&str] = match (lang, bold) {
            (AppLanguage::ZhCn | AppLanguage::ZhTw, false) => &[
                "C:\\Windows\\Fonts\\msjh.ttc",
                "C:\\Windows\\Fonts\\msyh.ttc",
                "C:\\Windows\\Fonts\\simhei.ttf",
                "C:\\Windows\\Fonts\\malgun.ttf",
                "/System/Library/Fonts/PingFang.ttc",
                "/System/Library/Fonts/STHeiti Light.ttc",
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
                "C:\\Windows\\Fonts\\segoeui.ttf",
                "/System/Library/Fonts/Supplemental/Arial.ttf",
                "/System/Library/Fonts/Helvetica.ttc",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            ],
            (AppLanguage::ZhCn | AppLanguage::ZhTw, true) => &[
                "C:\\Windows\\Fonts\\msjhbd.ttc",
                "C:\\Windows\\Fonts\\msjh.ttc",
                "C:\\Windows\\Fonts\\msyhbd.ttc",
                "C:\\Windows\\Fonts\\msyh.ttc",
                "C:\\Windows\\Fonts\\malgunbd.ttf",
                "/System/Library/Fonts/PingFang.ttc",
                "/System/Library/Fonts/STHeiti Medium.ttc",
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc",
                "/usr/share/fonts/truetype/noto/NotoSansCJK-Bold.ttc",
                "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
                "C:\\Windows\\Fonts\\seguisb.ttf",
                "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
                "/System/Library/Fonts/Helvetica.ttc",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
                "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
            ],
            (AppLanguage::Ka | AppLanguage::Hy, false) => &[
                "C:\\Windows\\Fonts\\sylfaen.ttf",
                "C:\\Windows\\Fonts\\segoeui.ttf",
                "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "C:\\Windows\\Fonts\\arial.ttf",
            ],
            (AppLanguage::Ka | AppLanguage::Hy, true) => &[
                "C:\\Windows\\Fonts\\sylfaen.ttf",
                "C:\\Windows\\Fonts\\seguisb.ttf",
                "C:\\Windows\\Fonts\\segoeuib.ttf",
                "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "C:\\Windows\\Fonts\\arialbd.ttf",
            ],
            (AppLanguage::Ja, false) => &[
                "C:\\Windows\\Fonts\\YuGothM.ttc",
                "C:\\Windows\\Fonts\\msgothic.ttc",
                "C:\\Windows\\Fonts\\msyh.ttc",
                "C:\\Windows\\Fonts\\malgun.ttf",
                "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
                "/System/Library/Fonts/Hiragino Sans GB.ttc",
                "/System/Library/Fonts/PingFang.ttc",
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
                "C:\\Windows\\Fonts\\segoeui.ttf",
                "/System/Library/Fonts/Supplemental/Arial.ttf",
                "/System/Library/Fonts/Helvetica.ttc",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            ],
            (AppLanguage::Ja, true) => &[
                "C:\\Windows\\Fonts\\YuGothB.ttc",
                "C:\\Windows\\Fonts\\YuGothM.ttc",
                "C:\\Windows\\Fonts\\msyhbd.ttc",
                "C:\\Windows\\Fonts\\malgunbd.ttf",
                "/System/Library/Fonts/ヒラギノ角ゴシック W6.ttc",
                "/System/Library/Fonts/Hiragino Sans GB.ttc",
                "/System/Library/Fonts/PingFang.ttc",
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc",
                "/usr/share/fonts/truetype/noto/NotoSansCJK-Bold.ttc",
                "C:\\Windows\\Fonts\\seguisb.ttf",
                "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
                "/System/Library/Fonts/Helvetica.ttc",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
                "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
            ],
            (AppLanguage::Ko, false) => &[
                "C:\\Windows\\Fonts\\malgun.ttf",
                "C:\\Windows\\Fonts\\msyh.ttc",
                "/System/Library/Fonts/AppleSDGothicNeo.ttc",
                "/System/Library/Fonts/PingFang.ttc",
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
                "C:\\Windows\\Fonts\\segoeui.ttf",
                "/System/Library/Fonts/Supplemental/Arial.ttf",
                "/System/Library/Fonts/Helvetica.ttc",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            ],
            (AppLanguage::Ko, true) => &[
                "C:\\Windows\\Fonts\\malgunbd.ttf",
                "C:\\Windows\\Fonts\\malgun.ttf",
                "C:\\Windows\\Fonts\\msyhbd.ttc",
                "/System/Library/Fonts/AppleSDGothicNeo.ttc",
                "/System/Library/Fonts/PingFang.ttc",
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc",
                "/usr/share/fonts/truetype/noto/NotoSansCJK-Bold.ttc",
                "C:\\Windows\\Fonts\\seguisb.ttf",
                "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
                "/System/Library/Fonts/Helvetica.ttc",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
                "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
            ],
            (_, false) => &[
                "C:\\Windows\\Fonts\\segoeui.ttf",
                "C:\\Windows\\Fonts\\arial.ttf",
                "C:\\Windows\\Fonts\\tahoma.ttf",
                "/System/Library/Fonts/Supplemental/Arial.ttf",
                "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
                "/System/Library/Fonts/Helvetica.ttc",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
                "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
                "/usr/share/fonts/TTF/DejaVuSans.ttf",
            ],
            (_, true) => &[
                "C:\\Windows\\Fonts\\seguisb.ttf",
                "C:\\Windows\\Fonts\\segoeuib.ttf",
                "C:\\Windows\\Fonts\\arialbd.ttf",
                "C:\\Windows\\Fonts\\segoeui.ttf",
                "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
                "/System/Library/Fonts/Supplemental/Arial.ttf",
                "/System/Library/Fonts/Helvetica.ttc",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
                "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
                "/usr/share/fonts/truetype/noto/NotoSans-Bold.ttf",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
            ],
        };
        Self::load(candidates, lang, bold)
    }

    fn glyph_width_1000(&self, gid: GlyphId) -> i32 {
        ((self.font.h_advance_unscaled(gid) * 1000.0) / self.upm).round() as i32
    }

    /// Measures string width in points (pt) for a given font size.
    fn measure_text_pt(&self, text: &str, font_size_pt: f32) -> f32 {
        let mut total_1000 = 0i32;
        for ch in text.chars() {
            let gid = self.font.glyph_id(ch);
            total_1000 += self.glyph_width_1000(gid);
        }
        (total_1000 as f32) * (font_size_pt / 1000.0)
    }

    /// Truncates string with ellipsis (`…`) if it exceeds max allowed column width in pt.
    fn fit_text_to_width(&self, text: &str, font_size_pt: f32, max_width_pt: f32) -> String {
        if self.measure_text_pt(text, font_size_pt) <= max_width_pt {
            return text.to_string();
        }
        let ellipsis = "…";
        let ell_w = self.measure_text_pt(ellipsis, font_size_pt);
        let target_w = (max_width_pt - ell_w).max(0.0);
        let mut acc = String::new();
        for ch in text.chars() {
            let mut next = acc.clone();
            next.push(ch);
            if self.measure_text_pt(&next, font_size_pt) > target_w {
                break;
            }
            acc.push(ch);
        }
        acc.push_str(ellipsis);
        acc
    }

    /// Encodes string into 16-bit hexadecimal Glyph IDs (`<001A002B...>`) for the `Tj` PDF operator
    /// and registers used glyphs for the `/W` widths and `/ToUnicode` mapping tables.
    fn encode_hex_gids(&mut self, text: &str) -> String {
        let mut hex = String::with_capacity(text.len() * 4 + 2);
        hex.push('<');
        for ch in text.chars() {
            let gid = self.font.glyph_id(ch);
            let w1000 = self.glyph_width_1000(gid);
            self.used_glyphs.entry(gid.0).or_insert((ch, w1000));
            hex.push_str(&format!("{:04X}", gid.0));
        }
        hex.push('>');
        hex
    }

    /// Builds glyph widths array `/W [ gid [w] ... ]` for `/CIDFontType2` dictionary.
    fn build_widths_array(&self) -> String {
        let mut out = String::from("[");
        for (&gid, &(_, w)) in &self.used_glyphs {
            out.push_str(&format!(" {} [{}]", gid, w));
        }
        out.push_str(" ]");
        out
    }

    /// Builds `/ToUnicode` CMap stream for correct Unicode copy-paste and text searching in PDF.
    fn build_tounicode_cmap(&self) -> Vec<u8> {
        let mut cmap = String::new();
        cmap.push_str("/CIDInit /ProcSet findresource begin\n");
        cmap.push_str("12 dict begin\n");
        cmap.push_str("begincmap\n");
        cmap.push_str("/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n");
        cmap.push_str("/CMapName /Adobe-Identity-UCS def\n");
        cmap.push_str("/CMapType 2 def\n");
        cmap.push_str("1 begincodespacerange\n");
        cmap.push_str("<0000> <FFFF>\n");
        cmap.push_str("endcodespacerange\n");

        let entries: Vec<(u16, char)> = self
            .used_glyphs
            .iter()
            .map(|(&gid, &(ch, _))| (gid, ch))
            .collect();

        for chunk in entries.chunks(100) {
            cmap.push_str(&format!("{} beginbfchar\n", chunk.len()));
            for &(gid, ch) in chunk {
                let code = ch as u32;
                if code <= 0xFFFF {
                    cmap.push_str(&format!("<{:04X}> <{:04X}>\n", gid, code));
                } else {
                    let mut utf16 = [0u16; 2];
                    let encoded = ch.encode_utf16(&mut utf16);
                    if encoded.len() == 2 {
                        cmap.push_str(&format!(
                            "<{:04X}> <{:04X}{:04X}>\n",
                            gid, encoded[0], encoded[1]
                        ));
                    }
                }
            }
            cmap.push_str("endbfchar\n");
        }

        cmap.push_str("endcmap\n");
        cmap.push_str("CMapName currentdict /CMap defineresource pop\n");
        cmap.push_str("end\nend\n");
        cmap.into_bytes()
    }
}

/// Exports complete manufacturing passport to a PDF document (A4 Landscape)
/// localized to `lang`:
/// 1. Parts specification table
/// 2. Designer 4-view projection preview (Top XY, 3D View, Front XZ, Side YZ)
/// 3. Cutting layout sheets
/// 4. Materials and edge banding summary
#[allow(dead_code)]
pub fn export_spec_to_pdf_file(
    project: &Project,
    path: &Path,
    lang: AppLanguage,
) -> Result<(), String> {
    let pdf_bytes = generate_spec_pdf_bytes(project, lang)?;
    let mut file = File::create(path).map_err(|e| {
        format!("{}: {}", lang.tr("Ошибка создания PDF-файла", "Error creating PDF file"), e)
    })?;
    file.write_all(&pdf_bytes).map_err(|e| {
        format!("{}: {}", lang.tr("Ошибка записи PDF-файла", "Error writing PDF file"), e)
    })?;
    Ok(())
}

/// Renders parts specification table page(s) in language `d`.
fn render_spec_table_pages(
    project: &Project,
    rows: &[SpecRow],
    d: &L10n,
    font_reg: &mut PdfTrueTypeFont,
    font_bold: &mut PdfTrueTypeFont,
    page_streams: &mut Vec<String>,
) {
    let total_parts_qty: usize = rows.iter().map(|r| r.quantity).sum();
    let page_w: f32 = 841.89;
    let page_h: f32 = 595.28;
    let margin_x: f32 = 24.0;
    let table_w: f32 = page_w - margin_x * 2.0;

    let columns: [(&str, f32); 12] = [
        ("№", 24.0),
        (d.pdf_col_group, 76.0),
        (d.pdf_col_name, 136.0),
        (d.pdf_col_material, 116.0),
        (d.pdf_col_full_dim, 74.0),
        (d.pdf_col_cut_dim, 78.0),
        (d.pdf_col_qty, 40.0),
        (d.pdf_col_grain, 65.89),
        (d.pdf_col_edge_l1, 46.0),
        (d.pdf_col_edge_l2, 46.0),
        (d.pdf_col_edge_w1, 46.0),
        (d.pdf_col_edge_w2, 46.0),
    ];

    let row_h: f32 = 18.5;
    let header_h: f32 = 22.0;
    let rows_per_page: usize = 24;

    let page_chunks: Vec<&[SpecRow]> = if rows.is_empty() {
        vec![&[]]
    } else {
        rows.chunks(rows_per_page).collect()
    };

    for chunk in page_chunks {
        let mut s = String::with_capacity(8192);

        let title_y = page_h - 34.0;
        let title_str = format!("{} — {}", d.pdf_sec1_title, project.name);
        let title_hex = font_bold.encode_hex_gids(&title_str);
        s.push_str(&format!(
            "BT /F2 15 Tf 0.10 0.12 0.16 rg {:.2} {:.2} Td {} Tj ET\n",
            margin_x, title_y, title_hex
        ));

        let sub_y = title_y - 16.0;
        let sub_str = format!(
            "{}: {}  |  {}: {} {}  |  {}: {}  |  {}",
            d.pdf_positions_label,
            rows.len(),
            d.pdf_total_parts_label,
            total_parts_qty,
            d.unit_pcs,
            d.pdf_materials_in_project_label,
            project.materials.len(),
            d.pdf_all_dims_in_mm
        );
        let sub_hex = font_reg.encode_hex_gids(&sub_str);
        s.push_str(&format!(
            "BT /F1 9.5 Tf 0.35 0.38 0.43 rg {:.2} {:.2} Td {} Tj ET\n",
            margin_x, sub_y, sub_hex
        ));

        let table_top_y = sub_y - 12.0;
        let header_bottom_y = table_top_y - header_h;

        s.push_str(&format!(
            "0.16 0.21 0.29 rg {:.2} {:.2} {:.2} {:.2} re f\n",
            margin_x, header_bottom_y, table_w, header_h
        ));

        let mut col_x = margin_x;
        for (col_title, col_w) in &columns {
            let fitted = font_bold.fit_text_to_width(col_title, 8.5, *col_w - 6.0);
            let hex = font_bold.encode_hex_gids(&fitted);
            s.push_str(&format!(
                "BT /F2 8.5 Tf 1 1 1 rg {:.2} {:.2} Td {} Tj ET\n",
                col_x + 3.5,
                header_bottom_y + 6.5,
                hex
            ));
            col_x += *col_w;
        }

        let mut cur_top_y = header_bottom_y;
        if chunk.is_empty() {
            let empty_bottom_y = cur_top_y - row_h;
            let msg = d.pdf_no_parts;
            let hex = font_reg.encode_hex_gids(msg);
            s.push_str(&format!(
                "BT /F1 9.5 Tf 0.45 0.45 0.45 rg {:.2} {:.2} Td {} Tj ET\n",
                margin_x + 8.0,
                empty_bottom_y + 5.5,
                hex
            ));
            cur_top_y = empty_bottom_y;
        } else {
            for (r_idx, r) in chunk.iter().enumerate() {
                let row_bottom_y = cur_top_y - row_h;

                if r_idx % 2 == 1 {
                    s.push_str(&format!(
                        "0.95 0.96 0.98 rg {:.2} {:.2} {:.2} {:.2} re f\n",
                        margin_x, row_bottom_y, table_w, row_h
                    ));
                }

                let cut_col_x: f32 =
                    margin_x + columns[0..5].iter().map(|(_, w)| *w).sum::<f32>();
                let cut_col_w: f32 = columns[5].1;
                s.push_str(&format!(
                    "0.91 0.98 0.93 rg {:.2} {:.2} {:.2} {:.2} re f\n",
                    cut_col_x, row_bottom_y, cut_col_w, row_h
                ));

                let grain_label = if r.grain_fixed {
                    d.pdf_grain_fixed
                } else {
                    d.pdf_grain_free
                };
                let e_l1 = SpecRow::edge_short_mm(r.edges.l1, &project.edge_bandings);
                let e_l2 = SpecRow::edge_short_mm(r.edges.l2, &project.edge_bandings);
                let e_w1 = SpecRow::edge_short_mm(r.edges.w1, &project.edge_bandings);
                let e_w2 = SpecRow::edge_short_mm(r.edges.w2, &project.edge_bandings);

                let cells: [(String, bool, [f32; 3]); 12] = [
                    (format!("{}", r.index), false, [0.15, 0.17, 0.20]),
                    (r.group_name.clone(), false, [0.22, 0.25, 0.30]),
                    (r.name.clone(), true, [0.08, 0.10, 0.14]),
                    (
                        format!("{} ({:.0}{})", r.material_name, r.thickness_mm, d.unit_mm),
                        false,
                        [0.18, 0.20, 0.25],
                    ),
                    (
                        format!("{:.1} × {:.1}", r.full_length_mm, r.full_width_mm),
                        false,
                        [0.18, 0.20, 0.25],
                    ),
                    (
                        format!("{:.1} × {:.1}", r.cut_length_mm, r.cut_width_mm),
                        true,
                        [0.06, 0.45, 0.18],
                    ),
                    (
                        format!("{} {}", r.quantity, d.unit_pcs),
                        true,
                        [0.10, 0.12, 0.16],
                    ),
                    (grain_label.to_string(), false, [0.22, 0.25, 0.30]),
                    (e_l1, false, [0.20, 0.22, 0.26]),
                    (e_l2, false, [0.20, 0.22, 0.26]),
                    (e_w1, false, [0.20, 0.22, 0.26]),
                    (e_w2, false, [0.20, 0.22, 0.26]),
                ];

                let mut cx = margin_x;
                for (c_idx, (cell_text, is_bold, rgb)) in cells.iter().enumerate() {
                    let cw = columns[c_idx].1;
                    let font_size = 8.5;
                    let hex = if *is_bold {
                        let fitted = font_bold.fit_text_to_width(cell_text, font_size, cw - 6.0);
                        font_bold.encode_hex_gids(&fitted)
                    } else {
                        let fitted = font_reg.fit_text_to_width(cell_text, font_size, cw - 6.0);
                        font_reg.encode_hex_gids(&fitted)
                    }
                    ;
                    let font_tag = if *is_bold { "/F2" } else { "/F1" };
                    s.push_str(&format!(
                        "BT {} {:.1} Tf {:.2} {:.2} {:.2} rg {:.2} {:.2} Td {} Tj ET\n",
                        font_tag,
                        font_size,
                        rgb[0],
                        rgb[1],
                        rgb[2],
                        cx + 3.5,
                        row_bottom_y + 5.2,
                        hex
                    ));
                    cx += cw;
                }

                s.push_str(&format!(
                    "0.78 0.81 0.85 RG 0.45 w {:.2} {:.2} m {:.2} {:.2} l S\n",
                    margin_x,
                    row_bottom_y,
                    margin_x + table_w,
                    row_bottom_y
                ));

                cur_top_y = row_bottom_y;
            }
        }

        let table_bottom_y = cur_top_y;
        s.push_str("0.72 0.76 0.82 RG 0.5 w\n");
        let mut vx = margin_x;
        for (_, cw) in &columns {
            vx += *cw;
            s.push_str(&format!(
                "{:.2} {:.2} m {:.2} {:.2} l S\n",
                vx, table_top_y, vx, table_bottom_y
            ));
        }
        s.push_str(&format!(
            "0.25 0.30 0.38 RG 0.8 w {:.2} {:.2} {:.2} {:.2} re S\n",
            margin_x,
            table_bottom_y,
            table_w,
            table_top_y - table_bottom_y
        ));

        page_streams.push(s);
    }
}

/// Computes overall bounding box (min_mm, max_mm) of all visible project parts.
fn compute_project_bounds(project: &Project) -> (Vec3, Vec3) {
    let mut min_v = Vec3::splat(f32::INFINITY);
    let mut max_v = Vec3::splat(f32::NEG_INFINITY);
    let mut found = false;

    for p in &project.parts {
        if !p.visible {
            continue;
        }
        let thick = project
            .get_material(p.material_id)
            .map(|m| m.thickness_mm)
            .unwrap_or(18.0);
        let (pmin, pmax) = p.world_aabb(thick);
        min_v = min_v.min(pmin);
        max_v = max_v.max(pmax);
        found = true;
    }

    if !found {
        (Vec3::ZERO, Vec3::new(800.0, 500.0, 720.0))
    } else {
        (min_v, max_v)
    }
}

/// Generates a dedicated PDF page with previews of all 4 views from designer:
/// 1) Top view (Top XY — Plan)
/// 2) 3D View (Isometric projection)
/// 3) Front view (Front XZ — Elevation)
/// 4) Side view (Side YZ — Section)
fn render_designer_views_page(
    project: &Project,
    d: &L10n,
    font_reg: &mut PdfTrueTypeFont,
    font_bold: &mut PdfTrueTypeFont,
) -> String {
    let mut s = String::with_capacity(16384);
    let page_w: f32 = 841.89;
    let page_h: f32 = 595.28;
    let margin_x: f32 = 24.0;

    let (bmin, bmax) = compute_project_bounds(project);
    let bsize = (bmax - bmin).max(Vec3::splat(10.0));

    // Page header
    let title_y = page_h - 34.0;
    let title_str = format!("{} — {}", d.pdf_sec2_title, project.name);
    let title_hex = font_bold.encode_hex_gids(&title_str);
    s.push_str(&format!(
        "BT /F2 15 Tf 0.10 0.12 0.16 rg {:.2} {:.2} Td {} Tj ET\n",
        margin_x, title_y, title_hex
    ));

    let sub_y = title_y - 16.0;
    let sub_str = format!(
        "{}: {:.0} × {:.0} × {:.0} {}  |  {}: {} {}  |  Top (XY), 3D, Front (XZ), Side (YZ)",
        d.pdf_overall_bounds,
        bsize.x,
        bsize.y,
        bsize.z,
        d.unit_mm,
        d.pdf_visible_parts,
        project.parts.iter().filter(|p| p.visible).count(),
        d.unit_pcs
    );
    let sub_hex = font_reg.encode_hex_gids(&sub_str);
    s.push_str(&format!(
        "BT /F1 9.5 Tf 0.35 0.38 0.43 rg {:.2} {:.2} Td {} Tj ET\n",
        margin_x, sub_y, sub_hex
    ));

    // Quadrants area: from y = 34 to y = sub_y - 10
    let area_top = sub_y - 10.0;
    let area_bottom = 36.0;
    let gap = 10.0;
    let pane_w = (page_w - margin_x * 2.0 - gap) * 0.5;
    let pane_h = (area_top - area_bottom - gap) * 0.5;

    let left_x = margin_x;
    let right_x = margin_x + pane_w + gap;
    let top_row_y = area_bottom + pane_h + gap;
    let bot_row_y = area_bottom;

    // Helper closure for drawing orthogonal 2D projection in given PDF rectangle (pane_x, pane_y, pane_w, pane_h)
    // mode: 0 = Top XY (u=X, v=Y, depth=Z), 1 = Front XZ (u=X, v=Z, depth=-Y), 2 = Side YZ (u=Y, v=Z, depth=X)
    let mut draw_ortho_quadrant =
        |s: &mut String,
         px: f32,
         py: f32,
         pw: f32,
         ph: f32,
         header_title: &str,
         axis_u_label: &str,
         axis_v_label: &str,
         mode: u8| {
            let hdr_h = 18.0;
            // Background and frame of viewport
            s.push_str(&format!(
                "0.97 0.98 0.99 rg 0.30 0.36 0.45 RG 0.7 w {:.2} {:.2} {:.2} {:.2} re B\n",
                px, py, pw, ph
            ));
            // Header badge of quadrant
            s.push_str(&format!(
                "0.18 0.23 0.31 rg {:.2} {:.2} {:.2} {:.2} re f\n",
                px,
                py + ph - hdr_h,
                pw,
                hdr_h
            ));
            let h_hex = font_bold.encode_hex_gids(header_title);
            s.push_str(&format!(
                "BT /F2 8.5 Tf 1 1 1 rg {:.2} {:.2} Td {} Tj ET\n",
                px + 6.0,
                py + ph - hdr_h + 5.2,
                h_hex
            ));

            let pad = 22.0;
            let draw_x = px + pad;
            let draw_y = py + pad;
            let draw_w = (pw - pad * 2.0).max(20.0);
            let draw_h = (ph - hdr_h - pad * 2.0).max(20.0);

            let (u_min, u_max, v_min, v_max) = match mode {
                0 => (bmin.x, bmax.x, bmin.y, bmax.y),
                1 => (bmin.x, bmax.x, bmin.z, bmax.z),
                _ => (bmin.y, bmax.y, bmin.z, bmax.z),
            };
            let span_u = (u_max - u_min).max(50.0);
            let span_v = (v_max - v_min).max(50.0);
            let scale = (draw_w / span_u).min(draw_h / span_v) * 0.92;

            let used_w = span_u * scale;
            let used_h = span_v * scale;
            let orig_x = draw_x + (draw_w - used_w) * 0.5;
            let orig_y = draw_y + (draw_h - used_h) * 0.5;

            // Background scale grid (100 mm step)
            s.push_str("0.88 0.90 0.93 RG 0.3 w\n");
            let step_mm = if span_u.max(span_v) > 1500.0 {
                200.0
            } else {
                100.0
            };
            let first_u = (u_min / step_mm).floor() * step_mm;
            let mut gu = first_u;
            while gu <= u_max + 0.1 {
                if gu >= u_min - 0.1 {
                    let gx = orig_x + (gu - u_min) * scale;
                    s.push_str(&format!(
                        "{:.2} {:.2} m {:.2} {:.2} l S\n",
                        gx,
                        orig_y,
                        gx,
                        orig_y + used_h
                    ));
                }
                gu += step_mm;
            }
            let first_v = (v_min / step_mm).floor() * step_mm;
            let mut gv = first_v;
            while gv <= v_max + 0.1 {
                if gv >= v_min - 0.1 {
                    let gy = if mode == 0 {
                        orig_y + used_h - (gv - v_min) * scale
                    } else {
                        orig_y + (gv - v_min) * scale
                    };
                    s.push_str(&format!(
                        "{:.2} {:.2} m {:.2} {:.2} l S\n",
                        orig_x,
                        gy,
                        orig_x + used_w,
                        gy
                    ));
                }
                gv += step_mm;
            }

            // Sort parts by depth so front parts occlude rear parts
            let mut sorted_parts: Vec<&crate::model::PartBlock> =
                project.parts.iter().filter(|p| p.visible).collect();
            sorted_parts.sort_by(|a, b| {
                let ta = project
                    .get_material(a.material_id)
                    .map(|m| m.thickness_mm)
                    .unwrap_or(18.0);
                let tb = project
                    .get_material(b.material_id)
                    .map(|m| m.thickness_mm)
                    .unwrap_or(18.0);
                let ca = a.world_center(ta);
                let cb = b.world_center(tb);
                let da = match mode {
                    0 => ca.z,
                    1 => -ca.y,
                    _ => ca.x,
                };
                let db = match mode {
                    0 => cb.z,
                    1 => -cb.y,
                    _ => cb.x,
                };
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            });

            for part in sorted_parts {
                let mat = project.get_material(part.material_id);
                let thick = mat.map(|m| m.thickness_mm).unwrap_or(18.0);
                let rgb = mat.map(|m| m.color_rgb).unwrap_or([200, 185, 160]);
                // Lighten material color slightly for drawing clarity
                let fr = (rgb[0] as f32 / 255.0) * 0.65 + 0.32;
                let fg = (rgb[1] as f32 / 255.0) * 0.65 + 0.32;
                let fb = (rgb[2] as f32 / 255.0) * 0.65 + 0.32;

                if mode == 0 && part.yaw_deg.abs() > 1e-2 {
                    // In Top XY view, part may be rotated by yaw_deg: draw exact 4-vertex polygon of top face
                    let verts = part.world_vertices(thick);
                    let pts: [(f32, f32); 4] = [0, 1, 2, 3].map(|vi| {
                        let v = verts[vi];
                        let x_pt = orig_x + (v.x - u_min) * scale;
                        let y_pt = orig_y + used_h - (v.y - v_min) * scale;
                        (x_pt, y_pt)
                    });
                    s.push_str(&format!(
                        "{:.2} {:.2} {:.2} rg 0.18 0.22 0.28 RG 0.65 w {:.2} {:.2} m {:.2} {:.2} l {:.2} {:.2} l {:.2} {:.2} l h B\n",
                        fr, fg, fb, pts[0].0, pts[0].1, pts[1].0, pts[1].1, pts[2].0, pts[2].1, pts[3].0, pts[3].1
                    ));
                } else {
                    let (pmin, pmax) = part.world_aabb(thick);
                    let (pu0, pu1, pv0, pv1) = match mode {
                        0 => (pmin.x, pmax.x, pmin.y, pmax.y),
                        1 => (pmin.x, pmax.x, pmin.z, pmax.z),
                        _ => (pmin.y, pmax.y, pmin.z, pmax.z),
                    };
                    let rx = orig_x + (pu0 - u_min) * scale;
                    let rw = ((pu1 - pu0) * scale).max(1.2);
                    let rh = ((pv1 - pv0) * scale).max(1.2);
                    let ry = if mode == 0 {
                        orig_y + used_h - (pv1 - v_min) * scale
                    } else {
                        orig_y + (pv0 - v_min) * scale
                    };

                    s.push_str(&format!(
                        "{:.2} {:.2} {:.2} rg 0.18 0.22 0.28 RG 0.65 w {:.2} {:.2} {:.2} {:.2} re B\n",
                        fr, fg, fb, rx, ry, rw, rh
                    ));

                    // Part name and size label inside large parts
                    if rw > 42.0 && rh > 13.0 {
                        let lbl = format!("{} ({:.0}×{:.0})", part.name, part.length_mm, part.width_mm);
                        let fitted = font_reg.fit_text_to_width(&lbl, 6.8, rw - 4.0);
                        let tw = font_reg.measure_text_pt(&fitted, 6.8);
                        let hex = font_reg.encode_hex_gids(&fitted);
                        s.push_str(&format!(
                            "BT /F1 6.8 Tf 0.10 0.12 0.16 rg {:.2} {:.2} Td {} Tj ET\n",
                            rx + (rw - tw) * 0.5,
                            ry + rh * 0.5 - 2.3,
                            hex
                        ));
                    }
                }
            }

            // Overall dimension labels along projection axes
            let u_dim_str = format!("{}: {:.0} {}", axis_u_label, u_max - u_min, d.unit_mm);
            let u_dim_w = font_bold.measure_text_pt(&u_dim_str, 7.5);
            let u_dim_hex = font_bold.encode_hex_gids(&u_dim_str);
            s.push_str(&format!(
                "BT /F2 7.5 Tf 0.20 0.26 0.36 rg {:.2} {:.2} Td {} Tj ET\n",
                orig_x + (used_w - u_dim_w) * 0.5,
                py + 5.5,
                u_dim_hex
            ));

            let v_dim_str = format!("{}: {:.0} {}", axis_v_label, v_max - v_min, d.unit_mm);
            let v_dim_hex = font_bold.encode_hex_gids(&v_dim_str);
            s.push_str(&format!(
                "BT /F2 7.5 Tf 0.20 0.26 0.36 rg {:.2} {:.2} Td {} Tj ET\n",
                px + 6.0,
                py + ph - hdr_h - 11.0,
                v_dim_hex
            ));
        };

    // 1. Top left: Top View (Top XY)
    draw_ortho_quadrant(
        &mut s,
        left_x,
        top_row_y,
        pane_w,
        pane_h,
        d.pdf_view_top,
        d.pdf_axis_x,
        d.pdf_axis_y,
        0,
    );

    // 2. Bottom left: Front View (Front XZ)
    draw_ortho_quadrant(
        &mut s,
        left_x,
        bot_row_y,
        pane_w,
        pane_h,
        d.pdf_view_front,
        d.pdf_axis_x,
        d.pdf_axis_z,
        1,
    );

    // 3. Bottom right: Side View (Side YZ)
    draw_ortho_quadrant(
        &mut s,
        right_x,
        bot_row_y,
        pane_w,
        pane_h,
        d.pdf_view_side,
        d.pdf_axis_y,
        d.pdf_axis_z,
        2,
    );

    // 4. Top right: 3D View (Isometric projection with shaded faces)
    {
        let px = right_x;
        let py = top_row_y;
        let pw = pane_w;
        let ph = pane_h;
        let hdr_h = 18.0;

        s.push_str(&format!(
            "0.96 0.97 0.99 rg 0.30 0.36 0.45 RG 0.7 w {:.2} {:.2} {:.2} {:.2} re B\n",
            px, py, pw, ph
        ));
        s.push_str(&format!(
            "0.18 0.23 0.31 rg {:.2} {:.2} {:.2} {:.2} re f\n",
            px,
            py + ph - hdr_h,
            pw,
            hdr_h
        ));
        let h_hex = font_bold.encode_hex_gids(d.pdf_view_3d);
        s.push_str(&format!(
            "BT /F2 8.5 Tf 1 1 1 rg {:.2} {:.2} Td {} Tj ET\n",
            px + 6.0,
            py + ph - hdr_h + 5.2,
            h_hex
        ));

        let center3 = (bmin + bmax) * 0.5;
        // Camera looks from front-right-above (yaw = -38°, pitch = 24°)
        let yaw = (-38.0_f32).to_radians();
        let pitch = (24.0_f32).to_radians();
        let cam_right = Vec3::new(yaw.cos(), yaw.sin(), 0.0);
        let cam_fwd = Vec3::new(-yaw.sin() * pitch.cos(), yaw.cos() * pitch.cos(), -pitch.sin());
        let cam_up = cam_right.cross(cam_fwd).normalize();

        let project_pt = |p: Vec3| -> (f32, f32, f32) {
            let rel = p - center3;
            let u = rel.dot(cam_right);
            let v = rel.dot(cam_up);
            let depth = rel.dot(cam_fwd);
            (u, v, depth)
        };

        // Determine 2D bounds of isometric projection of all vertices
        let mut min_u = f32::INFINITY;
        let mut max_u = f32::NEG_INFINITY;
        let mut min_v = f32::INFINITY;
        let mut max_v = f32::NEG_INFINITY;

        let mut faces_to_draw: Vec<(f32, [(f32, f32); 4], [f32; 3])> = Vec::new();
        let face_indices: [[usize; 4]; 6] = [
            [0, 1, 2, 3], // Bottom (-Z)
            [4, 5, 6, 7], // Top (+Z)
            [0, 1, 5, 4], // Front (-Y)
            [3, 2, 6, 7], // Back (+Y)
            [0, 3, 7, 4], // Left (-X)
            [1, 2, 6, 5], // Right (+X)
        ];
        let face_light_factors: [f32; 6] = [0.68, 1.02, 0.92, 0.74, 0.80, 0.86];

        for part in project.parts.iter().filter(|p| p.visible) {
            let mat = project.get_material(part.material_id);
            let thick = mat.map(|m| m.thickness_mm).unwrap_or(18.0);
            let rgb = mat.map(|m| m.color_rgb).unwrap_or([205, 185, 155]);
            let base_r = rgb[0] as f32 / 255.0;
            let base_g = rgb[1] as f32 / 255.0;
            let base_b = rgb[2] as f32 / 255.0;

            let verts = part.world_vertices(thick);
            let proj_verts: [(f32, f32, f32); 8] = verts.map(project_pt);
            for &(u, v, _) in &proj_verts {
                min_u = min_u.min(u);
                max_u = max_u.max(u);
                min_v = min_v.min(v);
                max_v = max_v.max(v);
            }

            for (f_idx, f_vis) in face_indices.iter().enumerate() {
                let quad_2d = [
                    (proj_verts[f_vis[0]].0, proj_verts[f_vis[0]].1),
                    (proj_verts[f_vis[1]].0, proj_verts[f_vis[1]].1),
                    (proj_verts[f_vis[2]].0, proj_verts[f_vis[2]].1),
                    (proj_verts[f_vis[3]].0, proj_verts[f_vis[3]].1),
                ];
                let avg_depth = (proj_verts[f_vis[0]].2
                    + proj_verts[f_vis[1]].2
                    + proj_verts[f_vis[2]].2
                    + proj_verts[f_vis[3]].2)
                    * 0.25;
                let lf = face_light_factors[f_idx];
                let col = [
                    (base_r * lf).clamp(0.08, 0.98),
                    (base_g * lf).clamp(0.08, 0.98),
                    (base_b * lf).clamp(0.08, 0.98),
                ];
                faces_to_draw.push((avg_depth, quad_2d, col));
            }
        }

        if !faces_to_draw.is_empty() {
            // Sort from back faces to front faces (larger depth = farther from camera)
            faces_to_draw.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

            let pad = 16.0;
            let draw_w = (pw - pad * 2.0).max(20.0);
            let draw_h = (ph - hdr_h - pad * 2.0).max(20.0);
            let span_u = (max_u - min_u).max(50.0);
            let span_v = (max_v - min_v).max(50.0);
            let scale = (draw_w / span_u).min(draw_h / span_v) * 0.90;
            let cx = px + pw * 0.5;
            let cy = py + (ph - hdr_h) * 0.5;
            let mid_u = (min_u + max_u) * 0.5;
            let mid_v = (min_v + max_v) * 0.5;

            for (_, q, col) in faces_to_draw {
                let p0 = (cx + (q[0].0 - mid_u) * scale, cy + (q[0].1 - mid_v) * scale);
                let p1 = (cx + (q[1].0 - mid_u) * scale, cy + (q[1].1 - mid_v) * scale);
                let p2 = (cx + (q[2].0 - mid_u) * scale, cy + (q[2].1 - mid_v) * scale);
                let p3 = (cx + (q[3].0 - mid_u) * scale, cy + (q[3].1 - mid_v) * scale);
                s.push_str(&format!(
                    "{:.2} {:.2} {:.2} rg 0.16 0.20 0.26 RG 0.45 w {:.2} {:.2} m {:.2} {:.2} l {:.2} {:.2} l {:.2} {:.2} l h B\n",
                    col[0], col[1], col[2], p0.0, p0.1, p1.0, p1.1, p2.0, p2.1, p3.0, p3.1
                ));
            }
        }
    }

    s
}

/// Generates separate PDF pages for each cut sheet of material (Cutting Layout) in language `d`.
fn render_cutting_layout_pages(
    project: &Project,
    report: &CuttingReport,
    d: &L10n,
    font_reg: &mut PdfTrueTypeFont,
    font_bold: &mut PdfTrueTypeFont,
    page_streams: &mut Vec<String>,
) {
    let page_w: f32 = 841.89;
    let page_h: f32 = 595.28;
    let margin_x: f32 = 24.0;
    let cs = &project.cutting_settings;

    if report.sheets.is_empty() {
        let mut s = String::with_capacity(2048);
        let title_y = page_h - 34.0;
        let title_hex = font_bold.encode_hex_gids(&format!(
            "{} — {}",
            d.pdf_sec3_title, project.name
        ));
        s.push_str(&format!(
            "BT /F2 15 Tf 0.10 0.12 0.16 rg {:.2} {:.2} Td {} Tj ET\n",
            margin_x, title_y, title_hex
        ));
        let msg_hex = font_reg.encode_hex_gids(d.pdf_no_placed_parts);
        s.push_str(&format!(
            "BT /F1 10.5 Tf 0.40 0.42 0.46 rg {:.2} {:.2} Td {} Tj ET\n",
            margin_x,
            title_y - 28.0,
            msg_hex
        ));
        page_streams.push(s);
        return;
    }

    let total_sheets = report.sheets.len();

    for (s_idx, sheet) in report.sheets.iter().enumerate() {
        let mut s = String::with_capacity(12288);

        let title_y = page_h - 34.0;
        let title_str = format!(
            "{} — {} #{} {} {}: {} ({:.0} × {:.0} {})",
            d.pdf_sec3_title,
            d.pdf_sheet_word,
            s_idx + 1,
            d.pdf_of_word,
            total_sheets,
            sheet.material_name,
            sheet.sheet_length_mm,
            sheet.sheet_width_mm,
            d.unit_mm
        );
        let title_hex = font_bold.encode_hex_gids(&title_str);
        s.push_str(&format!(
            "BT /F2 14.5 Tf 0.10 0.12 0.16 rg {:.2} {:.2} Td {} Tj ET\n",
            margin_x, title_y, title_hex
        ));

        let sub_y = title_y - 16.0;
        let sub_str = format!(
            "{}: {:.1}%  |  {}: {} {}  |  {}: {:.1} {}  |  {}: {:.0}/{:.0}/{:.0}/{:.0} {}",
            d.pdf_utilization,
            sheet.utilization_percent,
            d.pdf_parts_on_sheet,
            sheet.placed_parts.len(),
            d.unit_pcs,
            d.pdf_kerf,
            cs.kerf_mm,
            d.unit_mm,
            d.pdf_trim,
            cs.trim_left_mm,
            cs.trim_right_mm,
            cs.trim_top_mm,
            cs.trim_bottom_mm,
            d.unit_mm
        );
        let sub_hex = font_reg.encode_hex_gids(&sub_str);
        s.push_str(&format!(
            "BT /F1 9.2 Tf 0.35 0.38 0.43 rg {:.2} {:.2} Td {} Tj ET\n",
            margin_x, sub_y, sub_hex
        ));

        // Working area for sheet cutting layout
        let box_top = sub_y - 14.0;
        let box_bottom = 38.0;
        let box_w = page_w - margin_x * 2.0;
        let box_h = box_top - box_bottom;

        let sheet_l = sheet.sheet_length_mm.max(100.0);
        let sheet_w = sheet.sheet_width_mm.max(100.0);
        let scale = ((box_w - 36.0) / sheet_l).min((box_h - 36.0) / sheet_w);
        let draw_w = sheet_l * scale;
        let draw_h = sheet_w * scale;
        let orig_x = margin_x + (box_w - draw_w) * 0.5;
        let orig_top_y = box_bottom + (box_h + draw_h) * 0.5;
        let orig_bot_y = orig_top_y - draw_h;

        // Helper conversion from sheet Y coordinate (mm top-down) to PDF Y coordinate (bottom-up)
        let map_rect = |x_mm: f32, y_mm: f32, w_mm: f32, h_mm: f32| -> (f32, f32, f32, f32) {
            let rx = orig_x + x_mm * scale;
            let rw = w_mm * scale;
            let rh = h_mm * scale;
            let ry = orig_top_y - (y_mm + h_mm) * scale;
            (rx, ry, rw, rh)
        };

        // 1. Sheet outline
        s.push_str(&format!(
            "0.93 0.94 0.96 rg 0.22 0.26 0.32 RG 1.1 w {:.2} {:.2} {:.2} {:.2} re B\n",
            orig_x, orig_bot_y, draw_w, draw_h
        ));

        // Sheet overall dimension label on top
        let top_dim = format!("{}: {:.0} {}", d.pdf_sheet_length, sheet.sheet_length_mm, d.unit_mm);
        let top_dim_w = font_bold.measure_text_pt(&top_dim, 8.5);
        let top_dim_hex = font_bold.encode_hex_gids(&top_dim);
        s.push_str(&format!(
            "BT /F2 8.5 Tf 0.18 0.22 0.30 rg {:.2} {:.2} Td {} Tj ET\n",
            orig_x + (draw_w - top_dim_w) * 0.5,
            orig_top_y + 4.5,
            top_dim_hex
        ));

        // 2. Offcuts / remnants
        for rem in &sheet.remnants {
            let (rx, ry, rw, rh) = map_rect(rem.x_mm, rem.y_mm, rem.w_mm, rem.h_mm);
            s.push_str(&format!(
                "0.97 0.97 0.98 rg 0.72 0.75 0.80 RG 0.45 w {:.2} {:.2} {:.2} {:.2} re B\n",
                rx, ry, rw, rh
            ));
            if rw > 42.0 && rh > 16.0 {
                let r_lbl = format!("{} {:.0}×{:.0}", d.pdf_remnant, rem.w_mm, rem.h_mm);
                let fitted = font_reg.fit_text_to_width(&r_lbl, 6.8, rw - 4.0);
                let tw = font_reg.measure_text_pt(&fitted, 6.8);
                let hex = font_reg.encode_hex_gids(&fitted);
                s.push_str(&format!(
                    "BT /F1 6.8 Tf 0.50 0.53 0.58 rg {:.2} {:.2} Td {} Tj ET\n",
                    rx + (rw - tw) * 0.5,
                    ry + rh * 0.5 - 2.3,
                    hex
                ));
            }
        }

        // 3. Placed parts on sheet
        let mat_rgb = project
            .get_material(sheet.material_id)
            .map(|m| m.color_rgb)
            .unwrap_or([215, 190, 155]);
        let pr = (mat_rgb[0] as f32 / 255.0) * 0.55 + 0.42;
        let pg = (mat_rgb[1] as f32 / 255.0) * 0.55 + 0.42;
        let pb = (mat_rgb[2] as f32 / 255.0) * 0.55 + 0.42;

        for p in &sheet.placed_parts {
            let (rx, ry, rw, rh) = map_rect(p.x_mm, p.y_mm, p.w_on_sheet_mm, p.h_on_sheet_mm);
            s.push_str(&format!(
                "{:.2} {:.2} {:.2} rg 0.12 0.15 0.20 RG 0.8 w {:.2} {:.2} {:.2} {:.2} re B\n",
                pr, pg, pb, rx, ry, rw, rh
            ));

            // Edge banding markings on 4 sides of part
            let (top_e, bot_e, left_e, right_e) = if !p.rotated_90 {
                (p.edges.l1, p.edges.l2, p.edges.w1, p.edges.w2)
            } else {
                (p.edges.w1, p.edges.w2, p.edges.l1, p.edges.l2)
            };

            let draw_edge_line =
                |s: &mut String, eid: Option<EdgeId>, x1: f32, y1: f32, x2: f32, y2: f32| {
                    if let Some(id) = eid {
                        if let Some(ed) = project.get_edge(id) {
                            let er = ed.color_rgb[0] as f32 / 255.0;
                            let eg = ed.color_rgb[1] as f32 / 255.0;
                            let eb = ed.color_rgb[2] as f32 / 255.0;
                            s.push_str(&format!(
                                "{:.2} {:.2} {:.2} RG 1.8 w {:.2} {:.2} m {:.2} {:.2} l S\n",
                                er, eg, eb, x1, y1, x2, y2
                            ));
                        }
                    }
                };
            let inset = 1.6;
            draw_edge_line(
                &mut s,
                top_e,
                rx + inset,
                ry + rh - inset,
                rx + rw - inset,
                ry + rh - inset,
            );
            draw_edge_line(
                &mut s,
                bot_e,
                rx + inset,
                ry + inset,
                rx + rw - inset,
                ry + inset,
            );
            draw_edge_line(
                &mut s,
                left_e,
                rx + inset,
                ry + inset,
                rx + inset,
                ry + rh - inset,
            );
            draw_edge_line(
                &mut s,
                right_e,
                rx + rw - inset,
                ry + inset,
                rx + rw - inset,
                ry + rh - inset,
            );

            // Part index, name, and saw cut dimensions label
            if rw > 26.0 && rh > 11.0 {
                let line1 = format!("#{} {}", p.spec_index, p.name);
                let line2 = format!("{:.1}×{:.1}", p.w_on_sheet_mm, p.h_on_sheet_mm);
                if rh > 24.0 && rw > 48.0 {
                    let fit1 = font_bold.fit_text_to_width(&line1, 7.2, rw - 5.0);
                    let w1 = font_bold.measure_text_pt(&fit1, 7.2);
                    let hex1 = font_bold.encode_hex_gids(&fit1);
                    s.push_str(&format!(
                        "BT /F2 7.2 Tf 0.08 0.10 0.14 rg {:.2} {:.2} Td {} Tj ET\n",
                        rx + (rw - w1) * 0.5,
                        ry + rh * 0.5 + 1.5,
                        hex1
                    ));

                    let fit2 = font_reg.fit_text_to_width(&line2, 6.8, rw - 5.0);
                    let w2 = font_reg.measure_text_pt(&fit2, 6.8);
                    let hex2 = font_reg.encode_hex_gids(&fit2);
                    s.push_str(&format!(
                        "BT /F1 6.8 Tf 0.15 0.18 0.24 rg {:.2} {:.2} Td {} Tj ET\n",
                        rx + (rw - w2) * 0.5,
                        ry + rh * 0.5 - 6.5,
                        hex2
                    ));
                } else {
                    let compact = format!("#{} {:.0}×{:.0}", p.spec_index, p.w_on_sheet_mm, p.h_on_sheet_mm);
                    let fit = font_bold.fit_text_to_width(&compact, 6.5, rw - 3.0);
                    let w = font_bold.measure_text_pt(&fit, 6.5);
                    let hex = font_bold.encode_hex_gids(&fit);
                    s.push_str(&format!(
                        "BT /F2 6.5 Tf 0.08 0.10 0.14 rg {:.2} {:.2} Td {} Tj ET\n",
                        rx + (rw - w) * 0.5,
                        ry + rh * 0.5 - 2.2,
                        hex
                    ));
                }
            }
        }

        // 4. Red guillotine through-cut blade lines
        s.push_str("0.92 0.22 0.22 RG 0.65 w\n");
        for cl in &sheet.cut_lines {
            let x1 = orig_x + cl.x1 * scale;
            let y1 = orig_top_y - cl.y1 * scale;
            let x2 = orig_x + cl.x2 * scale;
            let y2 = orig_top_y - cl.y2 * scale;
            s.push_str(&format!("{:.2} {:.2} m {:.2} {:.2} l S\n", x1, y1, x2, y2));
        }

        page_streams.push(s);
    }
}

/// Generates dedicated PDF page with detailed description of sheet materials, edge bandings, and consumption summary in language `d`.
fn render_materials_and_edges_page(
    project: &Project,
    rows: &[SpecRow],
    report: &CuttingReport,
    d: &L10n,
    font_reg: &mut PdfTrueTypeFont,
    font_bold: &mut PdfTrueTypeFont,
) -> String {
    let mut s = String::with_capacity(12288);
    let page_w: f32 = 841.89;
    let page_h: f32 = 595.28;
    let margin_x: f32 = 24.0;
    let table_w: f32 = page_w - margin_x * 2.0;

    let title_y = page_h - 34.0;
    let title_str = format!("{} — {}", d.pdf_sec4_title, project.name);
    let title_hex = font_bold.encode_hex_gids(&title_str);
    s.push_str(&format!(
        "BT /F2 15 Tf 0.10 0.12 0.16 rg {:.2} {:.2} Td {} Tj ET\n",
        margin_x, title_y, title_hex
    ));

    let sub_y = title_y - 16.0;
    let sub_str = format!(
        "{}: {} {} ({:.2} {})  |  {}: {:.2} {}  |  {}: {:.1}%  |  {}: {:.1} {}",
        d.pdf_total_sheets_to_cut,
        report.total_sheets_count,
        d.unit_pcs,
        report.total_sheets_area_m2,
        d.unit_m2,
        d.pdf_net_parts_area,
        report.total_parts_area_m2,
        d.unit_m2,
        d.pdf_avg_utilization,
        report.overall_utilization_percent,
        d.pdf_total_cut_length,
        report.total_cut_length_m,
        d.unit_mp
    );
    let sub_hex = font_reg.encode_hex_gids(&sub_str);
    s.push_str(&format!(
        "BT /F1 9.5 Tf 0.35 0.38 0.43 rg {:.2} {:.2} Td {} Tj ET\n",
        margin_x, sub_y, sub_hex
    ));

    // --- Section 1: Sheet materials table ---
    let sec1_y = sub_y - 24.0;
    let sec1_hex = font_bold.encode_hex_gids(d.pdf_materials_table_heading);
    s.push_str(&format!(
        "BT /F2 11.5 Tf 0.14 0.18 0.24 rg {:.2} {:.2} Td {} Tj ET\n",
        margin_x, sec1_y, sec1_hex
    ));

    let mat_cols: [(&str, f32); 8] = [
        ("№", 28.0),
        (d.pdf_col_material_name, 210.0),
        (d.pdf_col_thickness, 68.0),
        (d.pdf_col_color_decor, 98.0),
        (d.pdf_col_sheet_format, 115.0),
        (d.pdf_col_default_grain, 95.0),
        (d.pdf_col_parts_count_area, 95.0),
        (d.pdf_col_sheets_count, 84.89),
    ];

    let header_h = 21.0;
    let row_h = 19.0;
    let mat_top_y = sec1_y - 8.0;
    let mat_hdr_bot_y = mat_top_y - header_h;

    s.push_str(&format!(
        "0.16 0.21 0.29 rg {:.2} {:.2} {:.2} {:.2} re f\n",
        margin_x, mat_hdr_bot_y, table_w, header_h
    ));

    let mut cx = margin_x;
    for (ctitle, cw) in &mat_cols {
        let fitted = font_bold.fit_text_to_width(ctitle, 8.5, *cw - 6.0);
        let hex = font_bold.encode_hex_gids(&fitted);
        s.push_str(&format!(
            "BT /F2 8.5 Tf 1 1 1 rg {:.2} {:.2} Td {} Tj ET\n",
            cx + 4.0,
            mat_hdr_bot_y + 6.2,
            hex
        ));
        cx += *cw;
    }

    let mut cur_y = mat_hdr_bot_y;
    for (m_idx, m) in project.materials.iter().enumerate() {
        let rbot_y = cur_y - row_h;
        if m_idx % 2 == 1 {
            s.push_str(&format!(
                "0.95 0.96 0.98 rg {:.2} {:.2} {:.2} {:.2} re f\n",
                margin_x, rbot_y, table_w, row_h
            ));
        }

        let parts_count: usize = rows
            .iter()
            .filter(|r| r.material_id == m.id)
            .map(|r| r.quantity)
            .sum();
        let parts_area_m2: f32 = rows
            .iter()
            .filter(|r| r.material_id == m.id)
            .map(|r| (r.cut_length_mm * r.cut_width_mm * r.quantity as f32) / 1_000_000.0)
            .sum();
        let sheets_count = report
            .sheets
            .iter()
            .filter(|sh| sh.material_id == m.id)
            .count();

        // Colored swatch rectangle in "Color / Decor" column
        let swatch_col_x = margin_x + mat_cols[0..3].iter().map(|(_, w)| *w).sum::<f32>();
        let sr = m.color_rgb[0] as f32 / 255.0;
        let sg = m.color_rgb[1] as f32 / 255.0;
        let sb = m.color_rgb[2] as f32 / 255.0;
        s.push_str(&format!(
            "{:.2} {:.2} {:.2} rg 0.25 0.28 0.32 RG 0.5 w {:.2} {:.2} 14.0 11.0 re B\n",
            sr,
            sg,
            sb,
            swatch_col_x + 4.0,
            rbot_y + 4.0
        ));

        let color_str = format!("RGB({},{},{})", m.color_rgb[0], m.color_rgb[1], m.color_rgb[2]);
        let grain_str = if m.default_grain_fixed {
            d.pdf_grain_locked
        } else {
            d.pdf_grain_none
        };

        let cells: [String; 8] = [
            format!("{}", m_idx + 1),
            m.name.clone(),
            format!("{:.1} {}", m.thickness_mm, d.unit_mm),
            color_str,
            format!("{:.0} × {:.0} {}", m.sheet_length_mm, m.sheet_width_mm, d.unit_mm),
            grain_str.to_string(),
            format!(
                "{} {} ({:.2} {})",
                parts_count, d.unit_pcs, parts_area_m2, d.unit_m2
            ),
            format!("{} {}", sheets_count, d.unit_sheets),
        ];

        let mut col_x = margin_x;
        for (c_idx, cell_text) in cells.iter().enumerate() {
            let cw = mat_cols[c_idx].1;
            let text_offset_x = if c_idx == 3 { 22.0 } else { 4.0 };
            let is_bold = c_idx == 1 || c_idx == 7;
            let hex = if is_bold {
                let fit = font_bold.fit_text_to_width(cell_text, 8.5, cw - text_offset_x - 3.0);
                font_bold.encode_hex_gids(&fit)
            } else {
                let fit = font_reg.fit_text_to_width(cell_text, 8.5, cw - text_offset_x - 3.0);
                font_reg.encode_hex_gids(&fit)
            };
            let ftag = if is_bold { "/F2" } else { "/F1" };
            s.push_str(&format!(
                "BT {} 8.5 Tf 0.12 0.15 0.20 rg {:.2} {:.2} Td {} Tj ET\n",
                ftag,
                col_x + text_offset_x,
                rbot_y + 5.5,
                hex
            ));
            col_x += cw;
        }

        s.push_str(&format!(
            "0.78 0.81 0.85 RG 0.45 w {:.2} {:.2} m {:.2} {:.2} l S\n",
            margin_x,
            rbot_y,
            margin_x + table_w,
            rbot_y
        ));
        cur_y = rbot_y;
    }

    let mat_bot_y = cur_y;
    s.push_str("0.72 0.76 0.82 RG 0.5 w\n");
    let mut vx = margin_x;
    for (_, cw) in &mat_cols {
        vx += *cw;
        s.push_str(&format!(
            "{:.2} {:.2} m {:.2} {:.2} l S\n",
            vx, mat_top_y, vx, mat_bot_y
        ));
    }
    s.push_str(&format!(
        "0.25 0.30 0.38 RG 0.8 w {:.2} {:.2} {:.2} {:.2} re S\n",
        margin_x,
        mat_bot_y,
        table_w,
        mat_top_y - mat_bot_y
    ));

    // --- Section 2: Edge bandings and linear run table ---
    let sec2_y = mat_bot_y - 26.0;
    let sec2_hex = font_bold.encode_hex_gids(d.pdf_edges_table_heading);
    s.push_str(&format!(
        "BT /F2 11.5 Tf 0.14 0.18 0.24 rg {:.2} {:.2} Td {} Tj ET\n",
        margin_x, sec2_y, sec2_hex
    ));

    let edge_cols: [(&str, f32); 7] = [
        ("№", 28.0),
        (d.pdf_col_edge_name, 220.0),
        (d.pdf_col_edge_thickness, 90.0),
        (d.pdf_col_marker_color, 110.0),
        (d.pdf_col_banded_edges, 110.0),
        (d.pdf_col_net_length, 118.0),
        (d.pdf_col_with_waste, 117.89),
    ];

    let edge_top_y = sec2_y - 8.0;
    let edge_hdr_bot_y = edge_top_y - header_h;

    s.push_str(&format!(
        "0.16 0.21 0.29 rg {:.2} {:.2} {:.2} {:.2} re f\n",
        margin_x, edge_hdr_bot_y, table_w, header_h
    ));

    let mut cx = margin_x;
    for (ctitle, cw) in &edge_cols {
        let fitted = font_bold.fit_text_to_width(ctitle, 8.5, *cw - 6.0);
        let hex = font_bold.encode_hex_gids(&fitted);
        s.push_str(&format!(
            "BT /F2 8.5 Tf 1 1 1 rg {:.2} {:.2} Td {} Tj ET\n",
            cx + 4.0,
            edge_hdr_bot_y + 6.2,
            hex
        ));
        cx += *cw;
    }

    let mut cur_edge_y = edge_hdr_bot_y;
    for (e_idx, e) in project.edge_bandings.iter().enumerate() {
        let rbot_y = cur_edge_y - row_h;
        if e_idx % 2 == 1 {
            s.push_str(&format!(
                "0.95 0.96 0.98 rg {:.2} {:.2} {:.2} {:.2} re f\n",
                margin_x, rbot_y, table_w, row_h
            ));
        }

        let mut sides_count = 0usize;
        let mut total_mm = 0.0_f32;
        for r in rows {
            let q = r.quantity;
            if r.edges.l1 == Some(e.id) {
                sides_count += q;
                total_mm += r.full_length_mm * q as f32;
            }
            if r.edges.l2 == Some(e.id) {
                sides_count += q;
                total_mm += r.full_length_mm * q as f32;
            }
            if r.edges.w1 == Some(e.id) {
                sides_count += q;
                total_mm += r.full_width_mm * q as f32;
            }
            if r.edges.w2 == Some(e.id) {
                sides_count += q;
                total_mm += r.full_width_mm * q as f32;
            }
        }
        let clean_m = total_mm / 1000.0;
        let reserve_m = clean_m * 1.15;

        let swatch_col_x = margin_x + edge_cols[0..3].iter().map(|(_, w)| *w).sum::<f32>();
        let er = e.color_rgb[0] as f32 / 255.0;
        let eg = e.color_rgb[1] as f32 / 255.0;
        let eb = e.color_rgb[2] as f32 / 255.0;
        s.push_str(&format!(
            "{:.2} {:.2} {:.2} rg 0.25 0.28 0.32 RG 0.5 w {:.2} {:.2} 14.0 11.0 re B\n",
            er,
            eg,
            eb,
            swatch_col_x + 4.0,
            rbot_y + 4.0
        ));

        let cells: [String; 7] = [
            format!("{}", e_idx + 1),
            e.name.clone(),
            format!("{:.1} {}", e.thickness_mm, d.unit_mm),
            format!("RGB({},{},{})", e.color_rgb[0], e.color_rgb[1], e.color_rgb[2]),
            format!("{} {}", sides_count, d.unit_pcs),
            format!("{:.2} {}", clean_m, d.unit_mp),
            format!("{:.2} {}", reserve_m, d.unit_mp),
        ];

        let mut col_x = margin_x;
        for (c_idx, cell_text) in cells.iter().enumerate() {
            let cw = edge_cols[c_idx].1;
            let text_offset_x = if c_idx == 3 { 22.0 } else { 4.0 };
            let is_bold = c_idx == 1 || c_idx == 6;
            let hex = if is_bold {
                let fit = font_bold.fit_text_to_width(cell_text, 8.5, cw - text_offset_x - 3.0);
                font_bold.encode_hex_gids(&fit)
            } else {
                let fit = font_reg.fit_text_to_width(cell_text, 8.5, cw - text_offset_x - 3.0);
                font_reg.encode_hex_gids(&fit)
            };
            let ftag = if is_bold { "/F2" } else { "/F1" };
            s.push_str(&format!(
                "BT {} 8.5 Tf 0.12 0.15 0.20 rg {:.2} {:.2} Td {} Tj ET\n",
                ftag,
                col_x + text_offset_x,
                rbot_y + 5.5,
                hex
            ));
            col_x += cw;
        }

        s.push_str(&format!(
            "0.78 0.81 0.85 RG 0.45 w {:.2} {:.2} m {:.2} {:.2} l S\n",
            margin_x,
            rbot_y,
            margin_x + table_w,
            rbot_y
        ));
        cur_edge_y = rbot_y;
    }

    let edge_bot_y = cur_edge_y;
    s.push_str("0.72 0.76 0.82 RG 0.5 w\n");
    let mut vx = margin_x;
    for (_, cw) in &edge_cols {
        vx += *cw;
        s.push_str(&format!(
            "{:.2} {:.2} m {:.2} {:.2} l S\n",
            vx, edge_top_y, vx, edge_bot_y
        ));
    }
    s.push_str(&format!(
        "0.25 0.30 0.38 RG 0.8 w {:.2} {:.2} {:.2} {:.2} re S\n",
        margin_x,
        edge_bot_y,
        table_w,
        edge_top_y - edge_bot_y
    ));

    s
}

/// Generates binary content of multi-page PDF product passport (A4 Landscape)
/// in the selected language `lang`:
/// - Page(s) 1: Parts specification table
/// - Page 2: Designer 4-view projection preview (Top XY, 3D, Front XZ, Side YZ)
/// - Page(s) 3+: Cutting layout per sheet
/// - Final Page: Materials and edge banding summary
pub fn generate_spec_pdf_bytes(project: &Project, lang: AppLanguage) -> Result<Vec<u8>, String> {
    let d = lang.dict();
    let mut font_reg = PdfTrueTypeFont::load_for_lang(lang, false)?;
    let mut font_bold = PdfTrueTypeFont::load_for_lang(lang, true)?;

    let rows = build_specification(project);
    let report = compute_cutting_layout(project);

    let page_w: f32 = 841.89;
    let page_h: f32 = 595.28;
    let margin_x: f32 = 24.0;

    let mut page_strings: Vec<String> = Vec::new();

    // 1. Specification table pages
    render_spec_table_pages(
        project,
        &rows,
        d,
        &mut font_reg,
        &mut font_bold,
        &mut page_strings,
    );

    // 2. Separate page: 4 Designer projections (Top XY, 3D, Front XZ, Side YZ)
    let designer_page = render_designer_views_page(project, d, &mut font_reg, &mut font_bold);
    page_strings.push(designer_page);

    // 3. Separate page(s): Cutting layout
    render_cutting_layout_pages(
        project,
        &report,
        d,
        &mut font_reg,
        &mut font_bold,
        &mut page_strings,
    );

    // 4. Separate page: Materials and edge bandings summary
    let materials_page = render_materials_and_edges_page(
        project,
        &rows,
        &report,
        d,
        &mut font_reg,
        &mut font_bold,
    );
    page_strings.push(materials_page);

    let total_pages = page_strings.len();
    let mut page_streams: Vec<Vec<u8>> = Vec::with_capacity(total_pages);

    for (page_idx, mut s) in page_strings.into_iter().enumerate() {
        let footer_left = format!("{} «{}»", d.pdf_footer_passport, project.name);
        let footer_left_hex = font_reg.encode_hex_gids(&footer_left);
        s.push_str(&format!(
            "BT /F1 8.5 Tf 0.45 0.48 0.52 rg {:.2} 16.00 Td {} Tj ET\n",
            margin_x, footer_left_hex
        ));

        let footer_right = format!(
            "{} {} {} {}",
            d.pdf_page_word,
            page_idx + 1,
            d.pdf_of_word,
            total_pages
        );
        let footer_right_w = font_reg.measure_text_pt(&footer_right, 8.5);
        let footer_right_hex = font_reg.encode_hex_gids(&footer_right);
        s.push_str(&format!(
            "BT /F1 8.5 Tf 0.45 0.48 0.52 rg {:.2} 16.00 Td {} Tj ET\n",
            page_w - margin_x - footer_right_w,
            footer_right_hex
        ));

        page_streams.push(s.into_bytes());
    }

    let first_page_obj_id = 13usize;
    let total_objects = first_page_obj_id + total_pages * 2 - 1;

    let mut pdf: Vec<u8> = Vec::with_capacity(
        font_reg.raw_bytes.len() + font_bold.raw_bytes.len() + 98304,
    );
    pdf.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");

    let mut offsets: Vec<usize> = vec![0; total_objects + 1];

    let write_obj = |pdf: &mut Vec<u8>, offsets: &mut [usize], id: usize, body: &[u8]| {
        offsets[id] = pdf.len();
        pdf.extend_from_slice(format!("{} 0 obj\n", id).as_bytes());
        pdf.extend_from_slice(body);
        pdf.extend_from_slice(b"\nendobj\n");
    };

    let write_stream_obj =
        |pdf: &mut Vec<u8>, offsets: &mut [usize], id: usize, extra_dict: &str, data: &[u8]| {
            offsets[id] = pdf.len();
            let header = if extra_dict.is_empty() {
                format!("{} 0 obj\n<< /Length {} >>\nstream\n", id, data.len())
            } else {
                format!(
                    "{} 0 obj\n<< /Length {} {} >>\nstream\n",
                    id,
                    data.len(),
                    extra_dict
                )
            };
            pdf.extend_from_slice(header.as_bytes());
            pdf.extend_from_slice(data);
            pdf.extend_from_slice(b"\nendstream\nendobj\n");
        };

    // 1: Catalog
    write_obj(
        &mut pdf,
        &mut offsets,
        1,
        b"<< /Type /Catalog /Pages 2 0 R >>",
    );

    // 2: Pages
    let kids_str: String = (0..total_pages)
        .map(|i| format!("{} 0 R", first_page_obj_id + i * 2))
        .collect::<Vec<_>>()
        .join(" ");
    let pages_dict = format!(
        "<< /Type /Pages /Kids [ {} ] /Count {} >>",
        kids_str, total_pages
    );
    write_obj(&mut pdf, &mut offsets, 2, pages_dict.as_bytes());

    let mut write_font_family =
        |base_id: usize, font_name: &str, ttf: &PdfTrueTypeFont| {
            let cid_id = base_id + 1;
            let desc_id = base_id + 2;
            let touni_id = base_id + 3;
            let file_id = base_id + 4;

            let type0_dict = format!(
                "<< /Type /Font /Subtype /Type0 /BaseFont /{} /Encoding /Identity-H /DescendantFonts [ {} 0 R ] /ToUnicode {} 0 R >>",
                font_name, cid_id, touni_id
            );
            write_obj(&mut pdf, &mut offsets, base_id, type0_dict.as_bytes());

            let widths_arr = ttf.build_widths_array();
            let cid_dict = format!(
                "<< /Type /Font /Subtype /CIDFontType2 /BaseFont /{} /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /FontDescriptor {} 0 R /CIDToGIDMap /Identity /DW 600 /W {} >>",
                font_name, desc_id, widths_arr
            );
            write_obj(&mut pdf, &mut offsets, cid_id, cid_dict.as_bytes());

            let desc_dict = format!(
                "<< /Type /FontDescriptor /FontName /{} /Flags 32 /FontBBox [-500 -300 1500 1000] /ItalicAngle 0 /Ascent 900 /Descent -250 /CapHeight 700 /StemV 80 /FontFile2 {} 0 R >>",
                font_name, file_id
            );
            write_obj(&mut pdf, &mut offsets, desc_id, desc_dict.as_bytes());

            let cmap_bytes = ttf.build_tounicode_cmap();
            write_stream_obj(&mut pdf, &mut offsets, touni_id, "", &cmap_bytes);

            let extra_len1 = format!("/Length1 {}", ttf.raw_bytes.len());
            write_stream_obj(
                &mut pdf,
                &mut offsets,
                file_id,
                &extra_len1,
                &ttf.raw_bytes,
            );
        };

    write_font_family(3, "QuadFurnFontReg", &font_reg);
    write_font_family(8, "QuadFurnFontBold", &font_bold);

    for (i, stream_bytes) in page_streams.iter().enumerate() {
        let page_obj_id = first_page_obj_id + i * 2;
        let content_obj_id = page_obj_id + 1;
        let page_dict = format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.2} {:.2}] /Resources << /Font << /F1 3 0 R /F2 8 0 R >> >> /Contents {} 0 R >>",
            page_w, page_h, content_obj_id
        );
        write_obj(&mut pdf, &mut offsets, page_obj_id, page_dict.as_bytes());
        write_stream_obj(&mut pdf, &mut offsets, content_obj_id, "", stream_bytes);
    }

    let xref_offset = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n", total_objects + 1).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \r\n");
    for id in 1..=total_objects {
        pdf.extend_from_slice(format!("{:010} 00000 n \r\n", offsets[id]).as_bytes());
    }

    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            total_objects + 1,
            xref_offset
        )
        .as_bytes(),
    );

    Ok(pdf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_font_fallback_for_wasm() {
        // Test loading font with NO system fonts (empty candidate list), exactly as in WebAssembly
        let reg = PdfTrueTypeFont::load(&[], AppLanguage::Ru, false).expect("Regular font fallback must succeed");
        let bold = PdfTrueTypeFont::load(&[], AppLanguage::Ru, true).expect("Bold font fallback must succeed");

        assert!(reg.measure_text_pt("Шкаф 800×720 мм", 10.0) > 0.0);
        assert!(bold.measure_text_pt("Спецификация деталей", 12.0) > 0.0);
    }

    #[test]
    fn test_generate_spec_pdf_bytes_multilang() {
        let proj = Project::new_demo_cabinet();
        for &lang in &[
            AppLanguage::Ru,
            AppLanguage::En,
            AppLanguage::Uk,
            AppLanguage::De,
            AppLanguage::ZhCn,
            AppLanguage::ZhTw,
            AppLanguage::Ja,
            AppLanguage::Ko,
            AppLanguage::Ka,
            AppLanguage::Hy,
            AppLanguage::El,
            AppLanguage::Kk,
        ] {
            let bytes = generate_spec_pdf_bytes(&proj, lang)
                .unwrap_or_else(|e| panic!("PDF generation failed for {:?}: {}", lang, e));
            assert!(bytes.starts_with(b"%PDF-1.4"));
            assert!(bytes.ends_with(b"%%EOF\n"));
            let pdf_str = String::from_utf8_lossy(&bytes);
            assert!(pdf_str.contains("/Count 6"));
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_fonts_dir_resolution() {
        let dir = get_windows_fonts_dir();
        assert!(!dir.as_os_str().is_empty());
        let dir_str = dir.to_string_lossy();
        assert!(dir_str.ends_with("Fonts") || dir_str.ends_with("fonts"));
        assert!(dir.ends_with("Fonts"));
    }
}
