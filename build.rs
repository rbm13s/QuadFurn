use image::imageops::FilterType;
use image::GenericImageView;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Генерирует 32-битный BGRA DIB блок для одной иконки заданного размера `sz × sz` внутри формата `.ico`
fn encode_ico_dib_entry(img: &image::DynamicImage, sz: u32) -> Vec<u8> {
    let resized = img.resize_exact(sz, sz, FilterType::Lanczos3);
    let rgba = resized.to_rgba8();

    let xor_bytes = (sz * sz * 4) as usize;
    let and_row_bytes = (((sz + 31) / 32) * 4) as usize;
    let and_bytes = and_row_bytes * (sz as usize);
    let mut buf = Vec::with_capacity(40 + xor_bytes + and_bytes);

    // BITMAPINFOHEADER (40 байт)
    buf.extend_from_slice(&40u32.to_le_bytes()); // biSize
    buf.extend_from_slice(&(sz as i32).to_le_bytes()); // biWidth
    buf.extend_from_slice(&((sz * 2) as i32).to_le_bytes()); // biHeight (удвоенная высота для XOR + AND масок)
    buf.extend_from_slice(&1u16.to_le_bytes()); // biPlanes
    buf.extend_from_slice(&32u16.to_le_bytes()); // biBitCount
    buf.extend_from_slice(&0u32.to_le_bytes()); // biCompression (BI_RGB)
    buf.extend_from_slice(&(xor_bytes as u32).to_le_bytes()); // biSizeImage
    buf.extend_from_slice(&0i32.to_le_bytes()); // biXPelsPerMeter
    buf.extend_from_slice(&0i32.to_le_bytes()); // biYPelsPerMeter
    buf.extend_from_slice(&0u32.to_le_bytes()); // biClrUsed
    buf.extend_from_slice(&0u32.to_le_bytes()); // biClrImportant

    // Пиксели BGRA снизу вверх (bottom-up)
    for y in (0..sz).rev() {
        for x in 0..sz {
            let p = rgba.get_pixel(x, y).0;
            buf.push(p[2]); // B
            buf.push(p[1]); // G
            buf.push(p[0]); // R
            buf.push(p[3]); // A
        }
    }

    // 1-битная маска прозрачности AND (заполняется нулями, так как альфа-канал в 32-битном BGRA)
    buf.resize(40 + xor_bytes + and_bytes, 0u8);
    buf
}

/// Создаёт мультиразмерный файл `.ico` (16, 32, 48, 64, 128, 256 px) из `assets/icon.jpeg`
fn build_multi_size_ico(jpeg_path: &Path, ico_path: &Path) -> Result<(), String> {
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

    // ICONDIR (6 байт)
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

    for (_, data) in payloads {
        ico.extend_from_slice(&data);
    }

    fs::write(ico_path, ico).map_err(|e| format!("Failed to write {}: {}", ico_path.display(), e))
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

    build_multi_size_ico(&jpeg_path, &out_ico_path)
        .expect("Failed to generate icon.ico from assets/icon.jpeg");
    let _ = fs::copy(&out_ico_path, &assets_ico_path);

    let rc_path = out_dir.join("app_icon.rc");
    // Используем прямое имя файла icon.ico в рабочей папке OUT_DIR
    let rc_content = "1 ICON \"icon.ico\"\r\nGLFW_ICON ICON \"icon.ico\"\r\n";
    fs::write(&rc_path, rc_content).expect("Failed to write app_icon.rc");

    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_env == "gnu" {
        let obj_path = out_dir.join("app_icon.res.o");
        let status = Command::new("windres")
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
            .expect("Failed to execute windres to compile Windows icon resource");
        assert!(status.success(), "windres failed to compile app_icon.rc");
        println!("cargo:rustc-link-arg-bins={}", obj_path.display());
    } else {
        let res_path = out_dir.join("app_icon.res");
        let status = Command::new("rc")
            .current_dir(&out_dir)
            .args(["/fo", res_path.to_str().unwrap(), "app_icon.rc"])
            .status()
            .expect("Failed to execute rc.exe to compile Windows icon resource");
        assert!(status.success(), "rc.exe failed to compile app_icon.rc");
        println!("cargo:rustc-link-arg-bins={}", res_path.display());
    }
}
