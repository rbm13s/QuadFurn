use image::imageops::FilterType;
use image::GenericImageView;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Generates a 32-bit BGRA DIB block for a single icon entry of size `sz × sz` inside the `.ico` format
fn encode_ico_dib_entry(img: &image::DynamicImage, sz: u32) -> Vec<u8> {
    let resized = img.resize_exact(sz, sz, FilterType::Lanczos3);
    let rgba = resized.to_rgba8();

    let xor_bytes = (sz * sz * 4) as usize;
    let and_row_bytes = (((sz + 31) / 32) * 4) as usize;
    let and_bytes = and_row_bytes * (sz as usize);
    let mut buf = Vec::with_capacity(40 + xor_bytes + and_bytes);

    // BITMAPINFOHEADER (40 bytes)
    buf.extend_from_slice(&40u32.to_le_bytes()); // biSize
    buf.extend_from_slice(&(sz as i32).to_le_bytes()); // biWidth
    buf.extend_from_slice(&((sz * 2) as i32).to_le_bytes()); // biHeight (doubled height for XOR + AND masks)
    buf.extend_from_slice(&1u16.to_le_bytes()); // biPlanes
    buf.extend_from_slice(&32u16.to_le_bytes()); // biBitCount
    buf.extend_from_slice(&0u32.to_le_bytes()); // biCompression (BI_RGB)
    buf.extend_from_slice(&(xor_bytes as u32).to_le_bytes()); // biSizeImage
    buf.extend_from_slice(&0i32.to_le_bytes()); // biXPelsPerMeter
    buf.extend_from_slice(&0i32.to_le_bytes()); // biYPelsPerMeter
    buf.extend_from_slice(&0u32.to_le_bytes()); // biClrUsed
    buf.extend_from_slice(&0u32.to_le_bytes()); // biClrImportant

    // BGRA pixels bottom-up
    for y in (0..sz).rev() {
        for x in 0..sz {
            let p = rgba.get_pixel(x, y).0;
            buf.push(p[2]); // B
            buf.push(p[1]); // G
            buf.push(p[0]); // R
            buf.push(p[3]); // A
        }
    }

    // 1-bit transparency AND mask (zero-filled since alpha channel is in 32-bit BGRA)
    buf.resize(40 + xor_bytes + and_bytes, 0u8);
    buf
}

/// Writes a single resource entry in 32-bit Windows binary `.res` format (aligned to 4 bytes)
fn write_res_entry(
    out: &mut Vec<u8>,
    type_ordinal: u16,
    name_ordinal: Option<u16>,
    name_str: Option<&str>,
    data: &[u8],
) {
    let mut header = Vec::with_capacity(64);
    // DataSize (4 bytes) and HeaderSize (4 bytes — will populate later)
    header.extend_from_slice(&(data.len() as u32).to_le_bytes());
    header.extend_from_slice(&0u32.to_le_bytes());

    // Type (Ordinal: 0xFFFF, then u16)
    header.extend_from_slice(&0xFFFFu16.to_le_bytes());
    header.extend_from_slice(&type_ordinal.to_le_bytes());

    // Name (Ordinal or null-terminated UTF-16LE string)
    if let Some(ord) = name_ordinal {
        header.extend_from_slice(&0xFFFFu16.to_le_bytes());
        header.extend_from_slice(&ord.to_le_bytes());
    } else if let Some(s) = name_str {
        for u in s.encode_utf16() {
            header.extend_from_slice(&u.to_le_bytes());
        }
        header.extend_from_slice(&0u16.to_le_bytes());
    }

    // Align header to DWORD boundary (4 bytes) before DataVersion
    while header.len() % 4 != 0 {
        header.push(0);
    }

    header.extend_from_slice(&0u32.to_le_bytes()); // DataVersion
    header.extend_from_slice(&0x0030u16.to_le_bytes()); // MemoryFlags (MOVEABLE | PURE)
    header.extend_from_slice(&0x0409u16.to_le_bytes()); // LanguageId (en-US)
    header.extend_from_slice(&0u32.to_le_bytes()); // Version
    header.extend_from_slice(&0u32.to_le_bytes()); // Characteristics

    let header_size = header.len() as u32;
    header[4..8].copy_from_slice(&header_size.to_le_bytes());

    out.extend_from_slice(&header);
    out.extend_from_slice(data);
    while out.len() % 4 != 0 {
        out.push(0);
    }
}

/// Creates multi-size `.ico` file and binary resource file `.res` (16, 32, 48, 64, 128, 256 px) from `assets/icon.jpeg`
fn build_multi_size_ico_and_res(
    jpeg_path: &Path,
    ico_path: &Path,
    res_path: &Path,
) -> Result<(), String> {
    let bytes = fs::read(jpeg_path)
        .map_err(|e| format!("Failed to read {}: {}", jpeg_path.display(), e))?;
    let dyn_img = image::load_from_memory(&bytes)
        .map_err(|e| format!("Failed to decode {}: {}", jpeg_path.display(), e))?;
    let _ = dyn_img.dimensions();

    let sizes: [u32; 6] = [256, 128, 64, 48, 32, 16];
    let mut payloads: Vec<(u32, Vec<u8>)> = Vec::with_capacity(sizes.len());
    for &sz in &sizes {
        payloads.push((sz, encode_ico_dib_entry(&dyn_img, sz)));
    }

    let count = payloads.len() as u16;
    let header_and_dir_size = 6 + (count as usize) * 16;
    let total_payload_size: usize = payloads.iter().map(|(_, b)| b.len()).sum();
    let mut ico = Vec::with_capacity(header_and_dir_size + total_payload_size);

    // ICONDIR (6 bytes)
    ico.extend_from_slice(&0u16.to_le_bytes()); // Reserved = 0
    ico.extend_from_slice(&1u16.to_le_bytes()); // Type = 1 (ICO)
    ico.extend_from_slice(&count.to_le_bytes()); // Count

    let mut current_offset = header_and_dir_size as u32;
    for (sz, data) in &payloads {
        let dim_byte = if *sz >= 256 { 0u8 } else { *sz as u8 };
        ico.push(dim_byte); // bWidth
        ico.push(dim_byte); // bHeight
        ico.push(0u8); // bColorCount
        ico.push(0u8); // bReserved
        ico.extend_from_slice(&1u16.to_le_bytes()); // wPlanes
        ico.extend_from_slice(&32u16.to_le_bytes()); // wBitCount
        ico.extend_from_slice(&(data.len() as u32).to_le_bytes()); // dwBytesInRes
        ico.extend_from_slice(&current_offset.to_le_bytes()); // dwImageOffset
        current_offset += data.len() as u32;
    }

    for (_, data) in &payloads {
        ico.extend_from_slice(data);
    }

    fs::write(ico_path, ico)
        .map_err(|e| format!("Failed to write {}: {}", ico_path.display(), e))?;

    // Generate standard 32-bit Windows Resource (.res) directly in pure Rust (does not require rc.exe in PATH)
    let mut res = Vec::with_capacity(64 + total_payload_size);
    // 1. Empty 32-bit .res header signature (32 bytes)
    write_res_entry(&mut res, 0, Some(0), None, &[]);
    // In header signature, MemoryFlags and LanguageId must be zeros
    res[20..24].copy_from_slice(&0u32.to_le_bytes());

    // 2. RT_ICON entries (Type = 3) with ID 1..=count
    for (idx, (_, data)) in payloads.iter().enumerate() {
        let icon_id = (idx + 1) as u16;
        write_res_entry(&mut res, 3, Some(icon_id), None, data);
    }

    // 3. GRPICONDIR structure for RT_GROUP_ICON (Type = 14)
    let mut grp = Vec::with_capacity(6 + (count as usize) * 14);
    grp.extend_from_slice(&0u16.to_le_bytes()); // idReserved = 0
    grp.extend_from_slice(&1u16.to_le_bytes()); // idType = 1 (ICON)
    grp.extend_from_slice(&count.to_le_bytes()); // idCount
    for (idx, (sz, data)) in payloads.iter().enumerate() {
        let dim_byte = if *sz >= 256 { 0u8 } else { *sz as u8 };
        let icon_id = (idx + 1) as u16;
        grp.push(dim_byte); // bWidth
        grp.push(dim_byte); // bHeight
        grp.push(0u8); // bColorCount
        grp.push(0u8); // bReserved
        grp.extend_from_slice(&1u16.to_le_bytes()); // wPlanes
        grp.extend_from_slice(&32u16.to_le_bytes()); // wBitCount
        grp.extend_from_slice(&(data.len() as u32).to_le_bytes()); // dwBytesInRes
        grp.extend_from_slice(&icon_id.to_le_bytes()); // nID (u16)
    }

    // Write RT_GROUP_ICON with ordinal 1 (main EXE icon in Windows Explorer) and name GLFW_ICON
    write_res_entry(&mut res, 14, Some(1), None, &grp);
    write_res_entry(&mut res, 14, None, Some("GLFW_ICON"), &grp);

    fs::write(res_path, res).map_err(|e| format!("Failed to write {}: {}", res_path.display(), e))
}

/// Searches for `rc.exe` in `PATH` or standard Windows Kits directories
fn find_rc_exe() -> Option<PathBuf> {
    if Command::new("rc")
        .arg("/?")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some(PathBuf::from("rc"));
    }

    let kits_roots = [
        r"C:\Program Files (x86)\Windows Kits\10\bin",
        r"C:\Program Files\Windows Kits\10\bin",
    ];
    for root in kits_roots {
        let root_path = Path::new(root);
        if let Ok(entries) = fs::read_dir(root_path) {
            let mut sdk_dirs: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect();
            sdk_dirs.sort();
            sdk_dirs.reverse();
            for dir in sdk_dirs {
                for arch in ["x64", "x86", "arm64"] {
                    let candidate = dir.join(arch).join("rc.exe");
                    if candidate.is_file() {
                        return Some(candidate);
                    }
                }
            }
        }
    }
    None
}

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.jpeg");
    println!("cargo:rerun-if-changed=build.rs");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let jpeg_path = manifest_dir.join("assets").join("icon.jpeg");

    let out_ico_path = out_dir.join("icon.ico");
    let assets_ico_path = manifest_dir.join("assets").join("icon.ico");
    let res_path = out_dir.join("app_icon.res");

    build_multi_size_ico_and_res(&jpeg_path, &out_ico_path, &res_path)
        .expect("Failed to generate icon.ico and app_icon.res from assets/icon.jpeg");
    let _ = fs::copy(&out_ico_path, &assets_ico_path);

    let rc_path = out_dir.join("app_icon.rc");
    let rc_content = "1 ICON \"icon.ico\"\r\nGLFW_ICON ICON \"icon.ico\"\r\n";
    fs::write(&rc_path, rc_content).expect("Failed to write app_icon.rc");

    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_env == "gnu" {
        let obj_path = out_dir.join("app_icon.res.o");
        if let Ok(status) = Command::new("windres")
            .current_dir(&out_dir)
            .args([
                "-i",
                "app_icon.rc",
                "-O",
                "coff",
                "-o",
                obj_path.to_str().unwrap(),
            ])
            .status()
        {
            if status.success() {
                println!("cargo:rustc-link-arg-bins={}", obj_path.display());
            }
        }
    } else {
        // If rc.exe is available (in PATH or in Windows Kits), compile through it;
        // otherwise use the pure-Rust generated binary app_icon.res
        if let Some(rc_bin) = find_rc_exe() {
            let _ = Command::new(rc_bin)
                .current_dir(&out_dir)
                .args(["/nologo", "/fo", res_path.to_str().unwrap(), "app_icon.rc"])
                .status();
        }
        println!("cargo:rustc-link-arg-bins={}", res_path.display());
    }
}

