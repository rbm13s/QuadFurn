#![cfg(target_arch = "wasm32")]

use std::sync::{LazyLock, Mutex};
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};
use crate::model::MaterialId;

pub static PENDING_PROJECT_UPLOADS: LazyLock<Mutex<Vec<(String, Vec<u8>)>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

#[derive(Clone, Debug)]
pub struct PendingTextureUpload {
    pub material_id: MaterialId,
    pub filename: String,
    pub bytes: Vec<u8>,
}

pub static PENDING_TEXTURE_UPLOADS: LazyLock<Mutex<Vec<PendingTextureUpload>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

/// Triggers a browser file download in WebAssembly.
pub fn download_file(filename: &str, mime_type: &str, data: &[u8]) {
    let array = js_sys::Array::new();
    let uint8_array = js_sys::Uint8Array::from(data);
    array.push(&uint8_array);

    let blob_props = BlobPropertyBag::new();
    blob_props.set_type(mime_type);
    if let Ok(blob) = Blob::new_with_u8_array_sequence_and_options(&array, &blob_props) {
        if let Ok(url) = Url::create_object_url_with_blob(&blob) {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Ok(element) = document.create_element("a") {
                        if let Ok(anchor) = element.dyn_into::<HtmlAnchorElement>() {
                            anchor.set_href(&url);
                            anchor.set_download(filename);
                            anchor.click();
                            let _ = Url::revoke_object_url(&url);
                        }
                    }
                }
            }
        }
    }
}

/// Opens an asynchronous file picker in the browser for `.qfurn` projects.
pub fn pick_project_file_web() {
    wasm_bindgen_futures::spawn_local(async move {
        if let Some(handle) = rfd::AsyncFileDialog::new()
            .add_filter("QuadFurn Project", &["qfurn"])
            .pick_file()
            .await
        {
            let bytes = handle.read().await;
            let name = handle.file_name();
            if let Ok(mut lock) = PENDING_PROJECT_UPLOADS.lock() {
                lock.push((name, bytes));
            }
        }
    });
}

/// Opens an asynchronous file picker in the browser for texture images.
pub fn pick_texture_file_web(mat_id: MaterialId) {
    wasm_bindgen_futures::spawn_local(async move {
        if let Some(handle) = rfd::AsyncFileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
            .pick_file()
            .await
        {
            let bytes = handle.read().await;
            let filename = handle.file_name();
            if let Ok(mut lock) = PENDING_TEXTURE_UPLOADS.lock() {
                lock.push(PendingTextureUpload {
                    material_id: mat_id,
                    filename,
                    bytes,
                });
            }
        }
    });
}

/// Detects language from URL query string `?lang=...` or from `navigator.languages`/`navigator.language`.
pub fn detect_browser_language() -> Option<crate::i18n::AppLanguage> {
    let window = web_sys::window()?;

    // 1. Check URL query parameters, e.g. "?lang=ru" or "?lang=de"
    if let Ok(search) = window.location().search() {
        let search = search.trim_start_matches('?');
        for pair in search.split('&') {
            let mut parts = pair.split('=');
            if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                if k.eq_ignore_ascii_case("lang") {
                    if let Some(lang) = crate::i18n::AppLanguage::from_locale_tag(v) {
                        return Some(lang);
                    }
                }
            }
        }
    }

    // 2. Check navigator.languages (Array of strings, in order of user preference)
    let nav = window.navigator();
    let languages = nav.languages();
    for i in 0..languages.length() {
        if let Some(tag) = languages.get(i).as_string() {
            if let Some(lang) = crate::i18n::AppLanguage::from_locale_tag(&tag) {
                return Some(lang);
            }
        }
    }

    // 3. Check navigator.language
    if let Some(tag) = nav.language() {
        if let Some(lang) = crate::i18n::AppLanguage::from_locale_tag(&tag) {
            return Some(lang);
        }
    }

    None
}
