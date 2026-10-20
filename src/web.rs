#[cfg(target_arch = "wasm32")]
use std::sync::{LazyLock, Mutex};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};
use crate::model::MaterialId;

#[cfg(target_arch = "wasm32")]
pub static PENDING_PROJECT_UPLOADS: LazyLock<Mutex<Vec<(String, Vec<u8>)>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct PendingTextureUpload {
    pub material_id: MaterialId,
    pub filename: String,
    pub bytes: Vec<u8>,
}

#[cfg(target_arch = "wasm32")]
pub static PENDING_TEXTURE_UPLOADS: LazyLock<Mutex<Vec<PendingTextureUpload>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

/// Triggers a browser file download in WebAssembly.
#[cfg(target_arch = "wasm32")]
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

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn download_file(_filename: &str, _mime_type: &str, _data: &[u8]) {}

/// Opens an asynchronous file picker in the browser for `.qfurn` projects.
#[cfg(target_arch = "wasm32")]
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

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn pick_project_file_web() {}

/// Opens an asynchronous file picker in the browser for texture images.
#[cfg(target_arch = "wasm32")]
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

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn pick_texture_file_web(_mat_id: MaterialId) {}

/// Detects language from URL query string `?lang=...` or from `navigator.languages`/`navigator.language`.
#[cfg(target_arch = "wasm32")]
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

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn detect_browser_language() -> Option<crate::i18n::AppLanguage> {
    None
}

use std::sync::atomic::{AtomicBool, Ordering};

pub static WEB_UNDO_TRIGGERED: AtomicBool = AtomicBool::new(false);
pub static WEB_NOCLIP_TOGGLE_TRIGGERED: AtomicBool = AtomicBool::new(false);
pub static WEB_SAVE_TRIGGERED: AtomicBool = AtomicBool::new(false);
pub static WEB_DUPLICATE_TRIGGERED: AtomicBool = AtomicBool::new(false);
pub static WEB_TOOL_V_TRIGGERED: AtomicBool = AtomicBool::new(false);
pub static WEB_TOOL_B_TRIGGERED: AtomicBool = AtomicBool::new(false);
pub static WEB_TOOL_M_TRIGGERED: AtomicBool = AtomicBool::new(false);

pub static WEB_KEY_W_DOWN: AtomicBool = AtomicBool::new(false);
pub static WEB_KEY_A_DOWN: AtomicBool = AtomicBool::new(false);
pub static WEB_KEY_S_DOWN: AtomicBool = AtomicBool::new(false);
pub static WEB_KEY_D_DOWN: AtomicBool = AtomicBool::new(false);
pub static WEB_KEY_Q_DOWN: AtomicBool = AtomicBool::new(false);
pub static WEB_KEY_E_DOWN: AtomicBool = AtomicBool::new(false);
pub static WEB_KEY_SPACE_DOWN: AtomicBool = AtomicBool::new(false);

pub fn check_and_clear_web_undo() -> bool {
    WEB_UNDO_TRIGGERED.swap(false, Ordering::SeqCst)
}

pub fn check_and_clear_web_noclip() -> bool {
    WEB_NOCLIP_TOGGLE_TRIGGERED.swap(false, Ordering::SeqCst)
}

pub fn check_and_clear_web_save() -> bool {
    WEB_SAVE_TRIGGERED.swap(false, Ordering::SeqCst)
}

pub fn check_and_clear_web_duplicate() -> bool {
    WEB_DUPLICATE_TRIGGERED.swap(false, Ordering::SeqCst)
}

pub fn check_and_clear_web_tool_v() -> bool {
    WEB_TOOL_V_TRIGGERED.swap(false, Ordering::SeqCst)
}

pub fn check_and_clear_web_tool_b() -> bool {
    WEB_TOOL_B_TRIGGERED.swap(false, Ordering::SeqCst)
}

pub fn check_and_clear_web_tool_m() -> bool {
    WEB_TOOL_M_TRIGGERED.swap(false, Ordering::SeqCst)
}

pub fn is_web_key_w_down() -> bool {
    WEB_KEY_W_DOWN.load(Ordering::SeqCst)
}

pub fn is_web_key_a_down() -> bool {
    WEB_KEY_A_DOWN.load(Ordering::SeqCst)
}

pub fn is_web_key_s_down() -> bool {
    WEB_KEY_S_DOWN.load(Ordering::SeqCst)
}

pub fn is_web_key_d_down() -> bool {
    WEB_KEY_D_DOWN.load(Ordering::SeqCst)
}

pub fn is_web_key_q_down() -> bool {
    WEB_KEY_Q_DOWN.load(Ordering::SeqCst)
}

pub fn is_web_key_e_down() -> bool {
    WEB_KEY_E_DOWN.load(Ordering::SeqCst)
}

pub fn is_web_key_space_down() -> bool {
    WEB_KEY_SPACE_DOWN.load(Ordering::SeqCst)
}

/// Checks if browser is running on macOS
pub fn is_macos_web() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(w) = web_sys::window() {
            if let Ok(ua) = w.navigator().user_agent() {
                return ua.contains("Macintosh") || ua.contains("Mac OS X");
            }
        }
    }
    false
}

/// Installs low-level DOM `keydown` and `keyup` listeners on `window` to intercept physical keys (like physical `KeyZ` on Russian keyboard layout)
/// ensuring shortcuts (Ctrl+Z, Z Noclip, WASD flight, Tools) work regardless of the active keyboard layout or browser language.
#[allow(dead_code)]
pub fn init_web_keyboard_listener() {
    #[cfg(target_arch = "wasm32")]
    {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };

        // 1. Keydown listener
        let keydown_closure = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
            move |event: web_sys::KeyboardEvent| {
                // Ignore if active element is an input or textarea
                if let Some(target) = event.target() {
                    if let Ok(el) = target.dyn_into::<web_sys::HtmlElement>() {
                        let tag = el.tag_name().to_uppercase();
                        if tag == "INPUT" || tag == "TEXTAREA" {
                            return;
                        }
                    }
                }

                let code = event.code();
                let ctrl_or_cmd = event.ctrl_key() || event.meta_key();

                if ctrl_or_cmd {
                    match code.as_str() {
                        "KeyZ" => {
                            WEB_UNDO_TRIGGERED.store(true, Ordering::SeqCst);
                            event.prevent_default();
                        }
                        "KeyS" => {
                            WEB_SAVE_TRIGGERED.store(true, Ordering::SeqCst);
                            event.prevent_default();
                        }
                        "KeyD" => {
                            WEB_DUPLICATE_TRIGGERED.store(true, Ordering::SeqCst);
                            event.prevent_default();
                        }
                        _ => {}
                    }
                } else if !event.alt_key() {
                    if !event.shift_key() {
                        match code.as_str() {
                            "KeyZ" => {
                                WEB_NOCLIP_TOGGLE_TRIGGERED.store(true, Ordering::SeqCst);
                            }
                            "KeyV" => {
                                WEB_TOOL_V_TRIGGERED.store(true, Ordering::SeqCst);
                            }
                            "KeyB" => {
                                WEB_TOOL_B_TRIGGERED.store(true, Ordering::SeqCst);
                            }
                            "KeyM" => {
                                WEB_TOOL_M_TRIGGERED.store(true, Ordering::SeqCst);
                            }
                            _ => {}
                        }
                    }

                    // WASD / Flight key tracking
                    match code.as_str() {
                        "KeyW" => WEB_KEY_W_DOWN.store(true, Ordering::SeqCst),
                        "KeyA" => WEB_KEY_A_DOWN.store(true, Ordering::SeqCst),
                        "KeyS" => WEB_KEY_S_DOWN.store(true, Ordering::SeqCst),
                        "KeyD" => WEB_KEY_D_DOWN.store(true, Ordering::SeqCst),
                        "KeyQ" => WEB_KEY_Q_DOWN.store(true, Ordering::SeqCst),
                        "KeyE" => WEB_KEY_E_DOWN.store(true, Ordering::SeqCst),
                        "Space" => WEB_KEY_SPACE_DOWN.store(true, Ordering::SeqCst),
                        _ => {}
                    }
                }
            },
        );

        let _ = window.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref());
        keydown_closure.forget();

        // 2. Keyup listener
        let keyup_closure = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
            move |event: web_sys::KeyboardEvent| {
                let code = event.code();
                match code.as_str() {
                    "KeyW" => WEB_KEY_W_DOWN.store(false, Ordering::SeqCst),
                    "KeyA" => WEB_KEY_A_DOWN.store(false, Ordering::SeqCst),
                    "KeyS" => WEB_KEY_S_DOWN.store(false, Ordering::SeqCst),
                    "KeyD" => WEB_KEY_D_DOWN.store(false, Ordering::SeqCst),
                    "KeyQ" => WEB_KEY_Q_DOWN.store(false, Ordering::SeqCst),
                    "KeyE" => WEB_KEY_E_DOWN.store(false, Ordering::SeqCst),
                    "Space" => WEB_KEY_SPACE_DOWN.store(false, Ordering::SeqCst),
                    _ => {}
                }
            },
        );

        let _ = window.add_event_listener_with_callback("keyup", keyup_closure.as_ref().unchecked_ref());
        keyup_closure.forget();
    }
}

#[cfg(target_arch = "wasm32")]
pub static PENDING_FONT_LOAD: LazyLock<Mutex<Option<(String, Vec<u8>)>>> =
    LazyLock::new(|| Mutex::new(None));

#[cfg(target_arch = "wasm32")]
static INSTALLED_CJK_FONT: LazyLock<Mutex<Option<Vec<u8>>>> =
    LazyLock::new(|| Mutex::new(None));

#[allow(dead_code)]
pub fn get_installed_cjk_font() -> Option<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    {
        INSTALLED_CJK_FONT.lock().ok().and_then(|guard| guard.clone())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

#[allow(dead_code)]
pub fn set_installed_cjk_font(bytes: Vec<u8>) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Ok(mut guard) = INSTALLED_CJK_FONT.lock() {
            *guard = Some(bytes);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = bytes;
    }
}

#[allow(dead_code)]
pub fn take_pending_font() -> Option<(String, Vec<u8>)> {
    #[cfg(target_arch = "wasm32")]
    {
        PENDING_FONT_LOAD.lock().ok().and_then(|mut guard| guard.take())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

#[cfg(target_arch = "wasm32")]
static CJK_FETCH_STATE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0); // 0 = not started, 1 = in progress, 2 = done

/// Returns true if CJK font is currently being fetched over network in WebAssembly.
#[cfg(target_arch = "wasm32")]
pub fn is_cjk_font_fetching_web() -> bool {
    CJK_FETCH_STATE.load(std::sync::atomic::Ordering::SeqCst) == 1
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn is_cjk_font_fetching_web() -> bool {
    false
}

/// Returns true if CJK font has been downloaded and installed in WebAssembly.
#[cfg(target_arch = "wasm32")]
pub fn is_cjk_font_loaded_web() -> bool {
    CJK_FETCH_STATE.load(std::sync::atomic::Ordering::SeqCst) == 2
}

#[cfg(not(target_arch = "wasm32"))]
pub fn is_cjk_font_loaded_web() -> bool {
    false
}

/// Asynchronously fetches Noto Sans CJK font bytes from CDN when CJK language is selected.
#[cfg(target_arch = "wasm32")]
pub fn fetch_cjk_font_web() {
    if CJK_FETCH_STATE.compare_exchange(0, 1, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst).is_err() {
        return; // Already started or completed
    }

    wasm_bindgen_futures::spawn_local(async move {
        // High-availability CDN mirrors for Noto Sans CJK OTF/TTF
        let urls = [
            "https://cdn.jsdelivr.net/gh/googlefonts/noto-cjk@main/Sans/OTF/SimplifiedChinese/NotoSansCJKsc-Regular.otf",
            "https://raw.githubusercontent.com/googlefonts/noto-cjk/main/Sans/OTF/SimplifiedChinese/NotoSansCJKsc-Regular.otf",
        ];

        let mut downloaded_bytes: Option<Vec<u8>> = None;
        for url in urls {
            if let Ok(bytes) = fetch_url_bytes(url).await {
                if !bytes.is_empty() {
                    downloaded_bytes = Some(bytes);
                    break;
                }
            }
        }

        if let Some(bytes) = downloaded_bytes {
            CJK_FETCH_STATE.store(2, std::sync::atomic::Ordering::SeqCst);
            if let Ok(mut lock) = PENDING_FONT_LOAD.lock() {
                *lock = Some(("cjk_noto_web".to_string(), bytes));
            }
        } else {
            // Reset to 0 on failure so user can retry by switching language again
            CJK_FETCH_STATE.store(0, std::sync::atomic::Ordering::SeqCst);
        }
    });
}

#[cfg(target_arch = "wasm32")]
async fn fetch_url_bytes(url: &str) -> Result<Vec<u8>, wasm_bindgen::JsValue> {
    use wasm_bindgen::JsCast;
    let window = web_sys::window().ok_or_else(|| wasm_bindgen::JsValue::from_str("no window"))?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(url)).await?;
    let resp: web_sys::Response = resp_value.dyn_into()?;
    if !resp.ok() {
        return Err(wasm_bindgen::JsValue::from_str(&format!("HTTP {}", resp.status())));
    }
    let array_buffer_val = wasm_bindgen_futures::JsFuture::from(resp.array_buffer()?).await?;
    let uint8_array = js_sys::Uint8Array::new(&array_buffer_val);
    Ok(uint8_array.to_vec())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn fetch_cjk_font_web() {}

