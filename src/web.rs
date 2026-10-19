#![cfg(target_arch = "wasm32")]

use std::sync::{LazyLock, Mutex};
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};
use base64::Engine;
use crate::model::MaterialId;

pub static PENDING_PROJECT_UPLOADS: LazyLock<Mutex<Vec<(String, Vec<u8>)>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

pub static PENDING_TEXTURE_UPLOADS: LazyLock<Mutex<Vec<(MaterialId, String)>>> =
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
            let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
            if let Ok(mut lock) = PENDING_TEXTURE_UPLOADS.lock() {
                lock.push((mat_id, b64));
            }
        }
    });
}
