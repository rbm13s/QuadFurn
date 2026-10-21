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

static WEB_MOUSE_DELTA_X: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
static WEB_MOUSE_DELTA_Y: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
#[allow(dead_code)]
static WEB_POINTER_LOCKED: AtomicBool = AtomicBool::new(false);
#[allow(dead_code)]
static WEB_POINTER_LOCK_LOST: AtomicBool = AtomicBool::new(false);

#[allow(dead_code)]
pub fn is_web_pointer_locked() -> bool {
    WEB_POINTER_LOCKED.load(Ordering::SeqCst)
}

pub fn check_and_clear_pointer_lock_lost() -> bool {
    WEB_POINTER_LOCK_LOST.swap(false, Ordering::SeqCst)
}

pub fn get_and_clear_web_mouse_delta() -> glam::Vec2 {
    let dx = WEB_MOUSE_DELTA_X.swap(0, Ordering::SeqCst);
    let dy = WEB_MOUSE_DELTA_Y.swap(0, Ordering::SeqCst);
    glam::Vec2::new(dx as f32, dy as f32)
}

pub fn request_web_pointer_lock() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(w) = web_sys::window() {
            if let Some(d) = w.document() {
                if let Some(canvas) = d.get_element_by_id("the_canvas_id") {
                    let _ = canvas.request_pointer_lock();
                }
            }
        }
    }
}

pub fn exit_web_pointer_lock() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(w) = web_sys::window() {
            if let Some(d) = w.document() {
                d.exit_pointer_lock();
            }
        }
    }
}

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

/// Installs low-level DOM `keydown`, `keyup`, and `mousemove` listeners on `window` to intercept physical keys
/// and mouse look deltas, ensuring shortcuts (Ctrl+Z, Z Noclip, WASD flight, Tools) work regardless of the active keyboard layout or browser language.
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
                let code = event.code();
                let key = event.key();
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
                        _ => {
                            match key.as_str() {
                                "я" | "Я" => {
                                    WEB_UNDO_TRIGGERED.store(true, Ordering::SeqCst);
                                    event.prevent_default();
                                }
                                "ы" | "Ы" => {
                                    WEB_SAVE_TRIGGERED.store(true, Ordering::SeqCst);
                                    event.prevent_default();
                                }
                                "в" | "В" => {
                                    WEB_DUPLICATE_TRIGGERED.store(true, Ordering::SeqCst);
                                    event.prevent_default();
                                }
                                _ => {}
                            }
                        }
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
                            _ => {
                                match key.as_str() {
                                    "я" | "Я" | "z" | "Z" => WEB_NOCLIP_TOGGLE_TRIGGERED.store(true, Ordering::SeqCst),
                                    "м" | "М" | "v" | "V" => WEB_TOOL_V_TRIGGERED.store(true, Ordering::SeqCst),
                                    "и" | "И" | "b" | "B" => WEB_TOOL_B_TRIGGERED.store(true, Ordering::SeqCst),
                                    "ь" | "Ь" | "m" | "M" => WEB_TOOL_M_TRIGGERED.store(true, Ordering::SeqCst),
                                    _ => {}
                                }
                            }
                        }
                    }

                    // WASD / Flight key tracking:
                    // Checks both physical scancode (KeyW/A/S/D/Q/E) and character key (e.g. Russian ЦФЫВЙУ, etc.)
                    if code == "KeyW" || key == "w" || key == "W" || key == "ц" || key == "Ц" {
                        WEB_KEY_W_DOWN.store(true, Ordering::SeqCst);
                    }
                    if code == "KeyA" || key == "a" || key == "A" || key == "ф" || key == "Ф" {
                        WEB_KEY_A_DOWN.store(true, Ordering::SeqCst);
                    }
                    if code == "KeyS" || key == "s" || key == "S" || key == "ы" || key == "Ы" {
                        WEB_KEY_S_DOWN.store(true, Ordering::SeqCst);
                    }
                    if code == "KeyD" || key == "d" || key == "D" || key == "в" || key == "В" {
                        WEB_KEY_D_DOWN.store(true, Ordering::SeqCst);
                    }
                    if code == "KeyQ" || key == "q" || key == "Q" || key == "й" || key == "Й" {
                        WEB_KEY_Q_DOWN.store(true, Ordering::SeqCst);
                    }
                    if code == "KeyE" || key == "e" || key == "E" || key == "у" || key == "У" {
                        WEB_KEY_E_DOWN.store(true, Ordering::SeqCst);
                    }
                    if code == "Space" || key == " " {
                        WEB_KEY_SPACE_DOWN.store(true, Ordering::SeqCst);
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
                let key = event.key();

                if code == "KeyW" || key == "w" || key == "W" || key == "ц" || key == "Ц" {
                    WEB_KEY_W_DOWN.store(false, Ordering::SeqCst);
                }
                if code == "KeyA" || key == "a" || key == "A" || key == "ф" || key == "Ф" {
                    WEB_KEY_A_DOWN.store(false, Ordering::SeqCst);
                }
                if code == "KeyS" || key == "s" || key == "S" || key == "ы" || key == "Ы" {
                    WEB_KEY_S_DOWN.store(false, Ordering::SeqCst);
                }
                if code == "KeyD" || key == "d" || key == "D" || key == "в" || key == "В" {
                    WEB_KEY_D_DOWN.store(false, Ordering::SeqCst);
                }
                if code == "KeyQ" || key == "q" || key == "Q" || key == "й" || key == "Й" {
                    WEB_KEY_Q_DOWN.store(false, Ordering::SeqCst);
                }
                if code == "KeyE" || key == "e" || key == "E" || key == "у" || key == "У" {
                    WEB_KEY_E_DOWN.store(false, Ordering::SeqCst);
                }
                if code == "Space" || key == " " {
                    WEB_KEY_SPACE_DOWN.store(false, Ordering::SeqCst);
                }
            },
        );

        let _ = window.add_event_listener_with_callback("keyup", keyup_closure.as_ref().unchecked_ref());
        keyup_closure.forget();

        // 3. Blur listener (clears all held keys if window loses focus)
        let blur_closure = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
            WEB_KEY_W_DOWN.store(false, Ordering::SeqCst);
            WEB_KEY_A_DOWN.store(false, Ordering::SeqCst);
            WEB_KEY_S_DOWN.store(false, Ordering::SeqCst);
            WEB_KEY_D_DOWN.store(false, Ordering::SeqCst);
            WEB_KEY_Q_DOWN.store(false, Ordering::SeqCst);
            WEB_KEY_E_DOWN.store(false, Ordering::SeqCst);
            WEB_KEY_SPACE_DOWN.store(false, Ordering::SeqCst);
        });
        let _ = window.add_event_listener_with_callback("blur", blur_closure.as_ref().unchecked_ref());
        blur_closure.forget();

        // 4. Mousemove listener for Pointer Lock camera deltas
        let mousemove_closure = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |event: web_sys::MouseEvent| {
                if WEB_POINTER_LOCKED.load(Ordering::SeqCst) {
                    let dx = event.movement_x();
                    let dy = event.movement_y();
                    if dx != 0 || dy != 0 {
                        WEB_MOUSE_DELTA_X.fetch_add(dx, Ordering::SeqCst);
                        WEB_MOUSE_DELTA_Y.fetch_add(dy, Ordering::SeqCst);
                    }
                }
            },
        );
        let _ = window.add_event_listener_with_callback("mousemove", mousemove_closure.as_ref().unchecked_ref());
        mousemove_closure.forget();

        // 5. Pointerlockchange listener on document
        if let Some(doc) = window.document() {
            let pointerlock_closure = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::new(
                move |_event: web_sys::Event| {
                    if let Some(w) = web_sys::window() {
                        if let Some(d) = w.document() {
                            let locked = d.pointer_lock_element().is_some();
                            WEB_POINTER_LOCKED.store(locked, Ordering::SeqCst);
                            if !locked {
                                WEB_POINTER_LOCK_LOST.store(true, Ordering::SeqCst);
                            }
                        }
                    }
                },
            );
            let _ = doc.add_event_listener_with_callback("pointerlockchange", pointerlock_closure.as_ref().unchecked_ref());
            pointerlock_closure.forget();
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub static PENDING_FONT_LOADS: LazyLock<Mutex<Vec<(String, Vec<u8>)>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

#[cfg(target_arch = "wasm32")]
static INSTALLED_FONTS: LazyLock<Mutex<std::collections::BTreeMap<String, Vec<u8>>>> =
    LazyLock::new(|| Mutex::new(std::collections::BTreeMap::new()));

#[allow(dead_code)]
pub fn get_installed_font(name: &str) -> Option<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    {
        INSTALLED_FONTS.lock().ok().and_then(|guard| guard.get(name).cloned())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = name;
        None
    }
}

#[allow(dead_code)]
pub fn get_all_installed_fonts() -> Vec<(String, Vec<u8>)> {
    #[cfg(target_arch = "wasm32")]
    {
        INSTALLED_FONTS
            .lock()
            .ok()
            .map(|guard| guard.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Vec::new()
    }
}

#[allow(dead_code)]
pub fn set_installed_font(name: String, bytes: Vec<u8>) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Ok(mut guard) = INSTALLED_FONTS.lock() {
            guard.insert(name, bytes);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (name, bytes);
    }
}

#[allow(dead_code)]
pub fn get_installed_cjk_font() -> Option<Vec<u8>> {
    get_installed_font("cjk_noto_web")
}

#[allow(dead_code)]
pub fn set_installed_cjk_font(bytes: Vec<u8>) {
    set_installed_font("cjk_noto_web".to_string(), bytes);
}

#[allow(dead_code)]
pub fn take_pending_fonts() -> Vec<(String, Vec<u8>)> {
    #[cfg(target_arch = "wasm32")]
    {
        PENDING_FONT_LOADS
            .lock()
            .ok()
            .map(|mut guard| guard.drain(..).collect())
            .unwrap_or_default()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Vec::new()
    }
}

#[allow(dead_code)]
pub fn take_pending_font() -> Option<(String, Vec<u8>)> {
    take_pending_fonts().into_iter().next()
}

#[cfg(target_arch = "wasm32")]
fn queue_pending_font(name: String, bytes: Vec<u8>) {
    if let Ok(mut lock) = PENDING_FONT_LOADS.lock() {
        lock.push((name, bytes));
    }
}

#[cfg(target_arch = "wasm32")]
static CJK_FETCH_STATE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0); // 0 = not started, 1 = in progress, 2 = done
#[cfg(target_arch = "wasm32")]
static GEORGIAN_FETCH_STATE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
#[cfg(target_arch = "wasm32")]
static ARMENIAN_FETCH_STATE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
#[cfg(target_arch = "wasm32")]
static EXTENDED_FETCH_STATE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

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
            queue_pending_font("cjk_noto_web".to_string(), bytes);
        } else {
            // Reset to 0 on failure so user can retry by switching language again
            CJK_FETCH_STATE.store(0, std::sync::atomic::Ordering::SeqCst);
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn fetch_cjk_font_web() {}

/// Returns true if Georgian font is currently being fetched over network in WebAssembly.
#[cfg(target_arch = "wasm32")]
pub fn is_georgian_font_fetching_web() -> bool {
    GEORGIAN_FETCH_STATE.load(std::sync::atomic::Ordering::SeqCst) == 1
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn is_georgian_font_fetching_web() -> bool {
    false
}

/// Returns true if Georgian font has been downloaded and installed in WebAssembly.
#[cfg(target_arch = "wasm32")]
pub fn is_georgian_font_loaded_web() -> bool {
    GEORGIAN_FETCH_STATE.load(std::sync::atomic::Ordering::SeqCst) == 2
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn is_georgian_font_loaded_web() -> bool {
    false
}

/// Asynchronously fetches Noto Sans Georgian font bytes when Georgian (ka) language is selected.
#[cfg(target_arch = "wasm32")]
pub fn fetch_georgian_font_web() {
    if GEORGIAN_FETCH_STATE.compare_exchange(0, 1, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst).is_err() {
        return;
    }

    wasm_bindgen_futures::spawn_local(async move {
        let urls = [
            "fonts/NotoSansGeorgian-Regular.ttf",
            "https://raw.githubusercontent.com/googlefonts/noto-fonts/main/hinted/ttf/NotoSansGeorgian/NotoSansGeorgian-Regular.ttf",
            "https://cdn.jsdelivr.net/gh/googlefonts/noto-fonts@main/hinted/ttf/NotoSansGeorgian/NotoSansGeorgian-Regular.ttf",
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
            GEORGIAN_FETCH_STATE.store(2, std::sync::atomic::Ordering::SeqCst);
            queue_pending_font("noto_sans_georgian".to_string(), bytes);
        } else {
            GEORGIAN_FETCH_STATE.store(0, std::sync::atomic::Ordering::SeqCst);
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn fetch_georgian_font_web() {}

/// Returns true if Armenian font is currently being fetched over network in WebAssembly.
#[cfg(target_arch = "wasm32")]
pub fn is_armenian_font_fetching_web() -> bool {
    ARMENIAN_FETCH_STATE.load(std::sync::atomic::Ordering::SeqCst) == 1
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn is_armenian_font_fetching_web() -> bool {
    false
}

/// Returns true if Armenian font has been downloaded and installed in WebAssembly.
#[cfg(target_arch = "wasm32")]
pub fn is_armenian_font_loaded_web() -> bool {
    ARMENIAN_FETCH_STATE.load(std::sync::atomic::Ordering::SeqCst) == 2
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn is_armenian_font_loaded_web() -> bool {
    false
}

/// Asynchronously fetches Noto Sans Armenian font bytes when Armenian (hy) language is selected.
#[cfg(target_arch = "wasm32")]
pub fn fetch_armenian_font_web() {
    if ARMENIAN_FETCH_STATE.compare_exchange(0, 1, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst).is_err() {
        return;
    }

    wasm_bindgen_futures::spawn_local(async move {
        let urls = [
            "fonts/NotoSansArmenian-Regular.ttf",
            "https://raw.githubusercontent.com/googlefonts/noto-fonts/main/hinted/ttf/NotoSansArmenian/NotoSansArmenian-Regular.ttf",
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
            ARMENIAN_FETCH_STATE.store(2, std::sync::atomic::Ordering::SeqCst);
            queue_pending_font("noto_sans_armenian".to_string(), bytes);
        } else {
            ARMENIAN_FETCH_STATE.store(0, std::sync::atomic::Ordering::SeqCst);
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn fetch_armenian_font_web() {}

/// Returns true if Extended Latin/Cyrillic font (Azerbaijani & Uzbek) is fetching in WebAssembly.
#[cfg(target_arch = "wasm32")]
pub fn is_extended_font_fetching_web() -> bool {
    EXTENDED_FETCH_STATE.load(std::sync::atomic::Ordering::SeqCst) == 1
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn is_extended_font_fetching_web() -> bool {
    false
}

/// Returns true if Extended font (Azerbaijani & Uzbek) is loaded in WebAssembly.
#[cfg(target_arch = "wasm32")]
pub fn is_extended_font_loaded_web() -> bool {
    EXTENDED_FETCH_STATE.load(std::sync::atomic::Ordering::SeqCst) == 2
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn is_extended_font_loaded_web() -> bool {
    false
}

/// Asynchronously fetches Noto Sans font bytes when Azerbaijani (az) or Uzbek (uz) language is selected.
#[cfg(target_arch = "wasm32")]
pub fn fetch_extended_font_web() {
    if EXTENDED_FETCH_STATE.compare_exchange(0, 1, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst).is_err() {
        return;
    }

    wasm_bindgen_futures::spawn_local(async move {
        let urls = [
            "fonts/NotoSans-Regular.ttf",
            "https://raw.githubusercontent.com/googlefonts/noto-fonts/main/hinted/ttf/NotoSans/NotoSans-Regular.ttf",
            "https://cdn.jsdelivr.net/gh/googlefonts/noto-fonts@main/hinted/ttf/NotoSans/NotoSans-Regular.ttf",
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
            EXTENDED_FETCH_STATE.store(2, std::sync::atomic::Ordering::SeqCst);
            queue_pending_font("noto_sans_ext".to_string(), bytes);
        } else {
            EXTENDED_FETCH_STATE.store(0, std::sync::atomic::Ordering::SeqCst);
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn fetch_extended_font_web() {}

/// Checks if the current language requires a dynamic font on WebAssembly,
/// and starts fetching it if not yet loaded.
/// Returns true if any font is currently being fetched (useful for requesting repaint).
#[allow(dead_code)]
pub fn check_and_fetch_web_fonts(lang: crate::i18n::AppLanguage) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        use crate::i18n::AppLanguage;
        if matches!(lang, AppLanguage::Ja | AppLanguage::ZhCn | AppLanguage::ZhTw | AppLanguage::Ko) {
            if is_cjk_font_fetching_web() {
                return true;
            } else if !is_cjk_font_loaded_web() {
                fetch_cjk_font_web();
                return true;
            }
        } else if lang == AppLanguage::Ka {
            if is_georgian_font_fetching_web() {
                return true;
            } else if !is_georgian_font_loaded_web() {
                fetch_georgian_font_web();
                return true;
            }
        } else if lang == AppLanguage::Hy {
            if is_armenian_font_fetching_web() {
                return true;
            } else if !is_armenian_font_loaded_web() {
                fetch_armenian_font_web();
                return true;
            }
        } else if matches!(lang, AppLanguage::Az | AppLanguage::Uz) {
            if is_extended_font_fetching_web() {
                return true;
            } else if !is_extended_font_loaded_web() {
                fetch_extended_font_web();
                return true;
            }
        }
        false
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = lang;
        false
    }
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_web_fonts_files_and_glyphs() {
        use ab_glyph::{Font, FontArc};

        // Georgian font
        let ka_bytes = std::fs::read("web/fonts/NotoSansGeorgian-Regular.ttf")
            .expect("web/fonts/NotoSansGeorgian-Regular.ttf must exist");
        let ka_font = FontArc::try_from_vec(ka_bytes).expect("NotoSansGeorgian must parse");
        assert!(ka_font.glyph_id('ქ').0 > 0, "Must contain Georgian letter q");
        assert!(ka_font.glyph_id('ა').0 > 0, "Must contain Georgian letter a");
        assert!(ka_font.glyph_id('რ').0 > 0, "Must contain Georgian letter r");

        // Armenian font
        let hy_bytes = std::fs::read("web/fonts/NotoSansArmenian-Regular.ttf")
            .expect("web/fonts/NotoSansArmenian-Regular.ttf must exist");
        let hy_font = FontArc::try_from_vec(hy_bytes).expect("NotoSansArmenian must parse");
        assert!(hy_font.glyph_id('Հ').0 > 0, "Must contain Armenian capital H");
        assert!(hy_font.glyph_id('ա').0 > 0, "Must contain Armenian small a");
        assert!(hy_font.glyph_id('յ').0 > 0, "Must contain Armenian small y");

        // Extended Latin font (Azerbaijani & Uzbek)
        let ext_bytes = std::fs::read("web/fonts/NotoSans-Regular.ttf")
            .expect("web/fonts/NotoSans-Regular.ttf must exist");
        let ext_font = FontArc::try_from_vec(ext_bytes).expect("NotoSans must parse");
        // Azerbaijani
        assert!(ext_font.glyph_id('ə').0 > 0, "Must contain schwa ə");
        assert!(ext_font.glyph_id('Ə').0 > 0, "Must contain schwa Ə");
        assert!(ext_font.glyph_id('ğ').0 > 0, "Must contain g-breve ğ");
        assert!(ext_font.glyph_id('ı').0 > 0, "Must contain dotless i ı");
        assert!(ext_font.glyph_id('ş').0 > 0, "Must contain s-cedilla ş");
        // Uzbek
        assert!(ext_font.glyph_id('ʻ').0 > 0, "Must contain modifier letter turned comma ʻ");
        assert!(ext_font.glyph_id('ʼ').0 > 0, "Must contain modifier letter apostrophe ʼ");
    }
}


