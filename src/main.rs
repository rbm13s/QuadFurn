#![cfg_attr(all(not(test), target_os = "windows"), windows_subsystem = "windows")]

mod cutter;
mod export;
mod i18n;
mod model;
mod snapping;
mod ui_panels;
mod viewport;
mod batch_screenshot;
#[cfg(target_arch = "wasm32")]
mod web;

use crate::i18n::AppLanguage;
use crate::model::{EdgeId, MaterialId, PartId, Project};
use crate::ui_panels::{
    copy_selected_parts_to_clipboard, load_texture_from_memory, paste_parts_into_project,
    resolve_and_load_material_texture, ui_cutting_layout_tab, ui_materials_manager_tab,
    ui_right_inspector, ui_specification_tab, ui_top_transform_toolbar, AppTab, ClipboardPart,
    CLIPBOARD_PREFIX,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::ui_panels::load_texture_from_path;
use crate::viewport::{EditorTool, QuadViewportController};
use egui::{Color32, Key, RichText, TextureHandle};
use glam::Vec3;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThemeChoice {
    Dark,
    Light,
    System,
}

impl ThemeChoice {
    pub fn label(&self) -> &'static str {
        self.label_i18n(AppLanguage::Ru)
    }

    pub fn label_i18n(&self, lang: AppLanguage) -> &'static str {
        let d = lang.dict();
        match self {
            ThemeChoice::Dark => d.theme_dark,
            ThemeChoice::Light => d.theme_light,
            ThemeChoice::System => d.theme_system,
        }
    }

    pub fn apply(&self, ctx: &egui::Context) {
        match self {
            ThemeChoice::Dark => ctx.set_theme(egui::ThemePreference::Dark),
            ThemeChoice::Light => ctx.set_theme(egui::ThemePreference::Light),
            ThemeChoice::System => ctx.set_theme(egui::ThemePreference::System),
        }
    }
}

/// Program settings (saved on disk in `%APPDATA%\QuadFurn\settings.json`)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    pub theme_choice: ThemeChoice,
    #[serde(default)]
    pub language: AppLanguage,
    #[serde(default)]
    pub recent_projects: Vec<std::path::PathBuf>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme_choice: ThemeChoice::Dark,
            language: AppLanguage::detect_system_or_en(),
            recent_projects: Vec::new(),
        }
    }
}

impl AppSettings {
    /// Returns cross-platform path to the program settings file:
    /// - Windows: `%APPDATA%\QuadFurn\settings.json`
    /// - macOS: `~/Library/Application Support/QuadFurn/settings.json`
    /// - Linux: `$XDG_CONFIG_HOME/QuadFurn/settings.json` or `~/.config/QuadFurn/settings.json`
    pub fn settings_file_path() -> std::path::PathBuf {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return std::path::PathBuf::from(appdata)
                .join("QuadFurn")
                .join("settings.json");
        }
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            return std::path::PathBuf::from(userprofile)
                .join("AppData")
                .join("Roaming")
                .join("QuadFurn")
                .join("settings.json");
        }
        #[cfg(target_os = "macos")]
        if let Ok(home) = std::env::var("HOME") {
            return std::path::PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("QuadFurn")
                .join("settings.json");
        }
        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            if !xdg_config.trim().is_empty() {
                return std::path::PathBuf::from(xdg_config)
                    .join("QuadFurn")
                    .join("settings.json");
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            return std::path::PathBuf::from(home)
                .join(".config")
                .join("QuadFurn")
                .join("settings.json");
        }
        std::path::PathBuf::from("quadfurn_settings.json")
    }

    pub fn load() -> Self {
        let path = Self::settings_file_path();
        if let Ok(txt) = std::fs::read_to_string(&path) {
            if let Ok(mut cfg) = serde_json::from_str::<Self>(&txt) {
                cfg.recent_projects.truncate(10);
                // If settings file was created before language field existed, detect system language and save
                if !txt.contains("\"language\"") {
                    cfg.language = AppLanguage::detect_system_or_en();
                    let _ = cfg.save();
                }
                return cfg;
            }
        }
        // First launch: detect current OS language (or fallback to English if unsupported)
        // and immediately save to settings.json alongside the color theme.
        let cfg = Self::default();
        let _ = cfg.save();
        cfg
    }

    pub fn save(&self) -> Result<std::path::PathBuf, String> {
        let path = Self::settings_file_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        std::fs::write(&path, json)
            .map_err(|e| format!("Failed to write settings ({}): {}", path.display(), e))?;
        Ok(path)
    }
}

/// Prompt for locating a missing texture file upon opening a project
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissingTexturePrompt {
    pub material_id: MaterialId,
    pub material_name: String,
    pub expected_filename: String,
}

/// Individual tab with a furniture product (allows working with multiple products and copying parts between them)
pub struct FurnitureDocument {
    pub project: Project,
    pub saved_json: String,
    pub file_path: Option<std::path::PathBuf>,
    pub undo_stack: Vec<Project>,
    pub selected_part: Option<PartId>,
    pub active_material_id: MaterialId,
    pub viewport_ctrl: QuadViewportController,
    pub loaded_textures: HashMap<MaterialId, TextureHandle>,
    pub texture_warnings: HashMap<MaterialId, String>,
    pub missing_textures_queue: Vec<MissingTexturePrompt>,
    pub new_group_name_buf: String,
    pub selected_sheet_idx: usize,
    pub pending_delete_material: Option<MaterialId>,
    pub pending_delete_edge: Option<EdgeId>,
    pub renaming_tab: bool,
}

impl FurnitureDocument {
    pub fn new(project: Project) -> Self {
        let first_mat = project.materials.first().map(|m| m.id).unwrap_or(1);
        let first_part = project.parts.first().map(|p| p.id);
        let mut viewport_ctrl = QuadViewportController::default();
        if let Some(fp) = first_part {
            viewport_ctrl.selected_parts = vec![fp];
        }
        viewport_ctrl.fit_all_to_window(&project);
        let saved_json = serde_json::to_string(&project).unwrap_or_default();
        Self {
            project,
            saved_json,
            file_path: None,
            undo_stack: Vec::new(),
            selected_part: first_part,
            active_material_id: first_mat,
            viewport_ctrl,
            loaded_textures: HashMap::new(),
            texture_warnings: HashMap::new(),
            missing_textures_queue: Vec::new(),
            new_group_name_buf: String::new(),
            selected_sheet_idx: 0,
            pending_delete_material: None,
            pending_delete_edge: None,
            renaming_tab: false,
        }
    }

    /// Loads textures for all materials in the project.
    /// Any texture that cannot be resolved (embedded base64 or relative/absolute path)
    /// is added to `self.missing_textures_queue`.
    pub fn load_project_textures(
        &mut self,
        ctx: &egui::Context,
        project_path: Option<&std::path::Path>,
        lang: AppLanguage,
    ) {
        self.missing_textures_queue.clear();
        for m in &mut self.project.materials {
            let has_texture_ref = m.texture_path.is_some()
                || m.texture_base64.is_some()
                || m.texture_relative_path.is_some();

            if !has_texture_ref {
                continue;
            }

            match resolve_and_load_material_texture(ctx, m, project_path, lang) {
                Ok(info) => {
                    self.loaded_textures.insert(m.id, info.handle);
                    if let Some(w) = info.warning {
                        self.texture_warnings.insert(m.id, w);
                    } else {
                        self.texture_warnings.remove(&m.id);
                    }
                }
                Err(_) => {
                    let expected_fname = m
                        .texture_relative_path
                        .as_ref()
                        .map(|s| {
                            std::path::Path::new(s)
                                .file_name()
                                .map(|f| f.to_string_lossy().to_string())
                                .unwrap_or_else(|| s.clone())
                        })
                        .or_else(|| {
                            m.texture_path.as_ref().and_then(|p| {
                                p.file_name().map(|f| f.to_string_lossy().to_string())
                            })
                        })
                        .unwrap_or_else(|| format!("{}.png", m.name));

                    self.missing_textures_queue.push(MissingTexturePrompt {
                        material_id: m.id,
                        material_name: m.name.clone(),
                        expected_filename: expected_fname,
                    });
                }
            }
        }
    }

    /// Checks if project has unsaved changes
    pub fn is_dirty(&self) -> bool {
        serde_json::to_string(&self.project).ok().as_deref() != Some(self.saved_json.as_str())
    }

    /// Marks current project state as saved
    pub fn mark_saved(&mut self, path: Option<std::path::PathBuf>) {
        self.saved_json = serde_json::to_string(&self.project).unwrap_or_default();
        if path.is_some() {
            self.file_path = path;
        }
    }

    /// Prepares texture paths and data before saving:
    /// - Option A: computes relative paths to texture files relative to project save directory
    /// - Option B: if embed_textures is enabled, packs texture bytes into Base64 for portability
    pub fn prepare_textures_for_saving(&mut self, target_save_path: &std::path::Path) {
        let save_dir = target_save_path
            .parent()
            .unwrap_or(std::path::Path::new("."));
        for m in &mut self.project.materials {
            // Option A: compute relative path
            if let Some(ref p) = m.texture_path {
                if let Ok(rel) = p.strip_prefix(save_dir) {
                    m.texture_relative_path = Some(rel.to_string_lossy().replace('\\', "/"));
                } else if let Some(file_name) = p.file_name() {
                    m.texture_relative_path = Some(file_name.to_string_lossy().to_string());
                }
            }

            // Option B: pack into Base64
            if self.project.embed_textures {
                if m.texture_base64.is_none() {
                    if let Some(ref p) = m.texture_path {
                        if let Ok(bytes) = std::fs::read(p) {
                            if bytes.len() <= 60 * 1024 * 1024 {
                                use base64::Engine;
                                m.texture_base64 =
                                    Some(base64::engine::general_purpose::STANDARD.encode(&bytes));
                            }
                        }
                    }
                }
            } else {
                m.texture_base64 = None;
            }
        }
    }

    /// Saves project via file dialog (or to current file). Returns path on successful save.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_interactive(
        &mut self,
        status_msg: &mut String,
        lang: AppLanguage,
    ) -> Option<std::path::PathBuf> {
        let default_name = self
            .file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}.qfurn", self.project.name.replace(' ', "_")));

        let mut dialog = rfd::FileDialog::new()
            .set_file_name(&default_name)
            .add_filter("QuadFurn Project", &["qfurn", "json"]);

        if let Some(parent) = self.file_path.as_ref().and_then(|p| p.parent()) {
            dialog = dialog.set_directory(parent);
        }

        if let Some(path) = dialog.save_file() {
            self.prepare_textures_for_saving(&path);
            if let Ok(json) = serde_json::to_string_pretty(&self.project) {
                match std::fs::write(&path, json) {
                    Ok(()) => {
                        let is_embedded = self.project.embed_textures;
                        *status_msg = if is_embedded {
                            match lang {
                                AppLanguage::Ru => format!(
                                    "Изделие сохранено (с упакованными текстурами): {}",
                                    path.display()
                                ),
                                AppLanguage::Uk => format!(
                                    "Виріб збережено (із запакованими текстурами): {}",
                                    path.display()
                                ),
                                _ => format!(
                                    "Saved product (with embedded textures): {}",
                                    path.display()
                                ),
                            }
                        } else {
                            match lang {
                                AppLanguage::Ru => {
                                    format!("Изделие сохранено: {}", path.display())
                                }
                                AppLanguage::Uk => {
                                    format!("Виріб збережено: {}", path.display())
                                }
                                _ => format!("Saved product: {}", path.display()),
                            }
                        };
                        self.mark_saved(Some(path.clone()));
                        return Some(path);
                    }
                    Err(e) => {
                        *status_msg = match lang {
                            AppLanguage::Ru => format!("Ошибка сохранения: {}", e),
                            AppLanguage::Uk => format!("Помилка збереження: {}", e),
                            _ => format!("Save error: {}", e),
                        };
                    }
                }
            }
        }
        None
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save_interactive(
        &mut self,
        status_msg: &mut String,
        lang: AppLanguage,
    ) -> Option<std::path::PathBuf> {
        let clean_name = self.project.name.replace(['/', '\\', '?', '%', '*', ':', '|', '"', '<', '>'], "_");
        let filename = format!("{}.qfurn", clean_name);
        if let Ok(json) = serde_json::to_string_pretty(&self.project) {
            crate::web::download_file(&filename, "application/json", json.as_bytes());
            self.mark_saved(None);
            *status_msg = match lang {
                AppLanguage::Ru => format!("Проект скачан: {}", filename),
                AppLanguage::Uk => format!("Проєкт завантажено: {}", filename),
                _ => format!("Project downloaded: {}", filename),
            };
        }
        None
    }
}

pub struct QuadFurnApp {
    /// Open tabs of furniture products (if empty, Start Page is shown)
    pub documents: Vec<FurnitureDocument>,
    /// Index of active furniture tab
    pub active_doc_idx: usize,
    /// Input field for new product name on start page
    pub new_product_name_buf: String,
    /// Recent projects list (up to 10 entries), persisted in settings
    pub recent_projects: Vec<std::path::PathBuf>,
    /// Modal confirmation for closing furniture tab: Option<usize>
    pub pending_close_doc_idx: Option<usize>,
    /// Whether to show save confirmation modal before exiting application
    pub show_exit_confirm_dialog: bool,
    /// Flag allowing window close after user choice in exit dialog
    pub allow_exit: bool,
    /// Shared clipboard for copying one or more selected parts (Ctrl+C / Ctrl+V) between different furniture tabs
    pub clipboard_parts: Vec<ClipboardPart>,
    /// App icon texture (from `assets/icon.jpeg`)
    pub app_icon_tex: Option<TextureHandle>,
    /// View mode (Designer / Specification / Cutting layout / Materials)
    pub active_tab: AppTab,
    pub theme_choice: ThemeChoice,
    pub language: AppLanguage,
    pub status_msg: String,
    /// Startup frame counter to guarantee window maximization in Windows
    pub startup_maximize_frames: u8,
    pub screenshot_target: Option<std::path::PathBuf>,
    pub screenshot_frame_delay: u8,
    pub batch_screenshots_dir: Option<std::path::PathBuf>,
    pub batch_lang_idx: usize,
    pub batch_shot_idx: usize,
    pub batch_frame_delay: usize,
    pub batch_startup_delay: usize,
    pub force_open_lang_menu: bool,
}

impl Default for QuadFurnApp {
    fn default() -> Self {
        let settings = AppSettings::load();
        let lang = settings.language;
        let default_prod_name = lang
            .tr("Новое изделие 1", "New Product 1")
            .to_string();
        let default_status = lang
            .tr(
                "Выберите действие на стартовой странице: создать новое изделие или открыть сохранённый проект.",
                "Select an action on the start page: create a new product or open a saved project.",
            )
            .to_string();
        Self {
            documents: Vec::new(),
            active_doc_idx: 0,
            new_product_name_buf: default_prod_name,
            recent_projects: settings.recent_projects,
            pending_close_doc_idx: None,
            show_exit_confirm_dialog: false,
            allow_exit: false,
            clipboard_parts: Vec::new(),
            app_icon_tex: None,
            active_tab: AppTab::Designer4View,
            theme_choice: settings.theme_choice,
            language: lang,
            status_msg: default_status,
            startup_maximize_frames: 0,
            screenshot_target: None,
            screenshot_frame_delay: 0,
            batch_screenshots_dir: None,
            batch_lang_idx: 0,
            batch_shot_idx: 0,
            batch_frame_delay: 0,
            batch_startup_delay: 0,
            force_open_lang_menu: false,
        }
    }
}

impl QuadFurnApp {
    pub fn push_undo(undo_stack: &mut Vec<Project>, current: &Project) {
        if undo_stack.len() >= 40 {
            undo_stack.remove(0);
        }
        undo_stack.push(current.clone());
    }

    /// Saves current settings (color theme, language, and recent 10 projects) to `settings.json`
    pub fn save_settings(&self) -> Result<std::path::PathBuf, String> {
        let cfg = AppSettings {
            theme_choice: self.theme_choice,
            language: self.language,
            recent_projects: self.recent_projects.clone(),
        };
        cfg.save()
    }

    /// Adds project path to top of recent projects list (max 10 items) and saves settings
    pub fn record_recent_project(&mut self, path: std::path::PathBuf) {
        self.recent_projects.retain(|p| p != &path);
        self.recent_projects.insert(0, path);
        self.recent_projects.truncate(10);
        let _ = self.save_settings();
    }

    /// Opens project at specified path and adds it to recent projects list (up to 10)
    pub fn open_project_from_path(&mut self, ctx: &egui::Context, path: &std::path::Path) {
        match std::fs::read_to_string(path) {
            Ok(txt) => match serde_json::from_str::<Project>(&txt) {
                Ok(proj) => {
                    let mut new_doc = FurnitureDocument::new(proj);
                    new_doc.mark_saved(Some(path.to_path_buf()));
                    new_doc.load_project_textures(ctx, Some(path), self.language);
                    self.documents.push(new_doc);
                    self.active_doc_idx = self.documents.len() - 1;
                    self.active_tab = AppTab::Designer4View;
                    self.record_recent_project(path.to_path_buf());
                    self.status_msg = format!(
                        "{}: {}",
                        self.language.tr("Изделие открыто", "Product opened"),
                        path.display()
                    );
                }
                Err(e) => {
                    self.status_msg = format!(
                        "{}: {}",
                        self.language
                            .tr("Ошибка чтения проекта", "Error reading project"),
                        e
                    );
                }
            },
            Err(e) => {
                self.status_msg = format!(
                    "{} ({}): {}",
                    self.language
                        .tr("Не удалось открыть файл", "Failed to open file"),
                    path.display(),
                    e
                );
            }
        }
    }

    /// Opens saved product (.qfurn / .json) via system file picker dialog
    pub fn open_project_interactive(&mut self, ctx: &egui::Context) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter(
                    self.language.tr("Проект QuadFurn", "QuadFurn Project"),
                    &["qfurn", "json"],
                )
                .pick_file()
            {
                self.open_project_from_path(ctx, &path);
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = ctx;
            crate::web::pick_project_file_web();
        }
    }

    fn render_start_page(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let lang = self.language;
        let d = lang.dict();
        egui::ScrollArea::vertical()
            .id_salt("start_page_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    let avail_h = ui.available_height();
                    let top_pad = if self.recent_projects.is_empty() {
                        (avail_h * 0.10).max(16.0)
                    } else {
                        (avail_h * 0.04).max(12.0)
                    };
                    ui.add_space(top_pad);

                    egui::Frame::group(ui.style())
                        .inner_margin(egui::Margin::symmetric(36.0, 26.0))
                        .rounding(8.0)
                        .show(ui, |ui| {
                            ui.set_max_width(560.0);
                            ui.vertical_centered(|ui| {
                                if let Some(tex) = &self.app_icon_tex {
                                    ui.add(
                                        egui::Image::new((tex.id(), egui::vec2(72.0, 72.0)))
                                            .rounding(10.0),
                                    );
                                    ui.add_space(6.0);
                                }
                                ui.label(
                                    RichText::new("🪚 QuadFurn CAD")
                                        .strong()
                                        .size(28.0)
                                        .color(Color32::from_rgb(235, 165, 40)),
                                );
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new(d.start_page_subtitle)
                                        .size(14.5)
                                        .color(Color32::GRAY),
                                );

                                ui.add_space(18.0);
                                ui.separator();
                                ui.add_space(14.0);

                                ui.label(
                                    RichText::new(d.start_create_section)
                                        .strong()
                                        .size(16.0),
                                );
                                ui.add_space(8.0);

                                ui.horizontal(|ui| {
                                    ui.label(d.start_product_name_label);
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.new_product_name_buf)
                                            .hint_text(d.start_product_name_hint)
                                            .desired_width(310.0),
                                    );
                                });

                                ui.add_space(12.0);

                                let prod_name = if self.new_product_name_buf.trim().is_empty() {
                                    format!(
                                        "{} {}",
                                        lang.tr("Изделие", "Product"),
                                        self.documents.len() + 1
                                    )
                                } else {
                                    self.new_product_name_buf.trim().to_string()
                                };

                                if ui
                                    .add_sized(
                                        [460.0, 42.0],
                                        egui::Button::new(
                                            RichText::new(d.start_create_empty_btn)
                                                .strong()
                                                .size(15.5),
                                        ),
                                    )
                                    .on_hover_text(d.start_create_empty_hover)
                                    .clicked()
                                {
                                    let proj = Project::new_empty_i18n(prod_name.clone(), lang);
                                    self.documents.push(FurnitureDocument::new(proj));
                                    self.active_doc_idx = self.documents.len() - 1;
                                    self.active_tab = AppTab::Designer4View;
                                    self.new_product_name_buf = format!(
                                        "{} {}",
                                        lang.tr("Новое изделие", "New Product"),
                                        self.documents.len() + 1
                                    );
                                    self.status_msg = match lang {
                                        AppLanguage::Ru => {
                                            format!("Создано новое изделие «{}»", prod_name)
                                        }
                                        AppLanguage::Uk => {
                                            format!("Створено новий виріб «{}»", prod_name)
                                        }
                                        _ => {
                                            format!("Created new product \"{}\"", prod_name)
                                        }
                                    };
                                }

                                ui.add_space(8.0);

                                if ui
                                    .add_sized(
                                        [460.0, 38.0],
                                        egui::Button::new(
                                            RichText::new(d.start_create_template_btn).size(14.5),
                                        ),
                                    )
                                    .on_hover_text(d.start_create_template_hover)
                                    .clicked()
                                {
                                    let mut proj = Project::new_demo_cabinet_i18n(lang);
                                    let trimmed = self.new_product_name_buf.trim();
                                    let is_default_placeholder = trimmed.is_empty()
                                        || trimmed == "Новое изделие 1"
                                        || trimmed == "Новий виріб 1"
                                        || trimmed == "New Product 1"
                                        || trimmed.starts_with("Новое изделие")
                                        || trimmed.starts_with("Новий виріб")
                                        || trimmed.starts_with("New Product");
                                    if !is_default_placeholder {
                                        proj.name = prod_name;
                                    } else {
                                        proj.name = match lang {
                                            AppLanguage::Ru => "Тумба 800х500х720".to_string(),
                                            AppLanguage::Uk => "Тумба 800х500х720".to_string(),
                                            AppLanguage::De => "Schrank 800x500x720".to_string(),
                                            AppLanguage::Pl => "Szafka 800x500x720".to_string(),
                                            _ => "Cabinet 800x500x720".to_string(),
                                        };
                                    }
                                    let created_name = proj.name.clone();
                                    self.documents.push(FurnitureDocument::new(proj));
                                    self.active_doc_idx = self.documents.len() - 1;
                                    self.active_tab = AppTab::Designer4View;
                                    self.status_msg = match lang {
                                        AppLanguage::Ru => format!(
                                            "Создано изделие по шаблону «{}»",
                                            created_name
                                        ),
                                        AppLanguage::Uk => format!(
                                            "Створено виріб за шаблоном «{}»",
                                            created_name
                                        ),
                                        _ => format!(
                                            "Created product from template \"{}\"",
                                            created_name
                                        ),
                                    };
                                }

                                ui.add_space(16.0);
                                ui.separator();
                                ui.add_space(12.0);

                                ui.label(
                                    RichText::new(d.start_open_section)
                                        .strong()
                                        .size(16.0),
                                );
                                ui.add_space(8.0);

                                if ui
                                    .add_sized(
                                        [460.0, 42.0],
                                        egui::Button::new(
                                            RichText::new(d.start_open_btn).strong().size(15.5),
                                        ),
                                    )
                                    .on_hover_text(d.start_open_hover)
                                    .clicked()
                                {
                                    self.open_project_interactive(ctx);
                                }

                                ui.add_space(16.0);
                                ui.separator();
                                ui.add_space(10.0);

                                ui.label(
                                    RichText::new(format!(
                                        "{} ({}/10):",
                                        lang.tr(
                                            "Последние открытые проекты",
                                            "Recent projects"
                                        ),
                                        self.recent_projects.len().min(10)
                                    ))
                                    .strong()
                                    .size(14.5),
                                );
                                ui.add_space(6.0);

                                if self.recent_projects.is_empty() {
                                    ui.label(
                                        RichText::new(d.start_recent_empty)
                                            .size(12.5)
                                            .color(Color32::GRAY),
                                    );
                                } else {
                                    let mut open_recent_path: Option<std::path::PathBuf> = None;
                                    let mut remove_recent_idx: Option<usize> = None;

                                    for (idx, rpath) in
                                        self.recent_projects.iter().take(10).enumerate()
                                    {
                                        let file_title = rpath
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or_else(|| lang.tr("Проект", "Project"));
                                        let exists = rpath.exists();

                                        ui.horizontal(|ui| {
                                            let btn_label = if exists {
                                                format!(
                                                    "{}. 🗄 {}   —   {}",
                                                    idx + 1,
                                                    file_title,
                                                    rpath.display()
                                                )
                                            } else {
                                                format!(
                                                    "{}. ⚠ {} ({})   —   {}",
                                                    idx + 1,
                                                    file_title,
                                                    lang.tr("файл не найден", "file not found"),
                                                    rpath.display()
                                                )
                                            };

                                            let resp = ui
                                                .add_sized(
                                                    [424.0, 28.0],
                                                    egui::Button::new(
                                                        RichText::new(btn_label).size(12.5),
                                                    ),
                                                )
                                                .on_hover_text(format!(
                                                    "{}: {}",
                                                    lang.tr("Открыть проект", "Open project"),
                                                    rpath.display()
                                                ));
                                            if resp.clicked() {
                                                open_recent_path = Some(rpath.clone());
                                            }

                                            if ui
                                                .add_sized([28.0, 28.0], egui::Button::new("✖"))
                                                .on_hover_text(lang.tr(
                                                    "Убрать из списка последних проектов",
                                                    "Remove from recent projects list",
                                                ))
                                                .clicked()
                                            {
                                                remove_recent_idx = Some(idx);
                                            }
                                        });
                                    }

                                    if let Some(rem_idx) = remove_recent_idx {
                                        if rem_idx < self.recent_projects.len() {
                                            self.recent_projects.remove(rem_idx);
                                            let _ = self.save_settings();
                                        }
                                    }
                                    if let Some(p) = open_recent_path {
                                        self.open_project_from_path(ctx, &p);
                                    }
                                }

                                ui.add_space(14.0);
                                ui.label(
                                    RichText::new(format!(
                                        "{}: {}",
                                        lang.tr(
                                            "Настройки и цветовая схема сохраняются в",
                                            "Settings & theme are saved in"
                                        ),
                                        AppSettings::settings_file_path().display()
                                    ))
                                    .size(11.5)
                                    .color(Color32::GRAY),
                                );
                            });
                        });
                    ui.add_space(20.0);
                });
            });
    }

    fn handle_global_shortcuts(&mut self, ctx: &egui::Context) {
        if self.documents.is_empty() {
            return;
        }
        if self.active_doc_idx >= self.documents.len() {
            self.active_doc_idx = 0;
        }

        let (wants_kb, noclip_on) = {
            let doc = &self.documents[self.active_doc_idx];
            (
                ctx.wants_keyboard_input(),
                doc.viewport_ctrl.cam_3d.noclip_active,
            )
        };
        if wants_kb || noclip_on {
            return;
        }

        // Ctrl+S / Cmd+S -> Quick save current project
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(Key::S)) {
            let saved_path =
                self.documents[self.active_doc_idx].save_interactive(&mut self.status_msg, self.language);
            if let Some(path) = saved_path {
                self.record_recent_project(path);
            }
            return;
        }

        let lang = self.language;
        let doc = &mut self.documents[self.active_doc_idx];

        // Ctrl+Z / Cmd+Z -> Undo in current furniture tab
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(Key::Z)) {
            if let Some(prev) = doc.undo_stack.pop() {
                doc.project = prev;
                self.status_msg = lang
                    .tr(
                        "Отменено последнее действие (Ctrl+Z)",
                        "Last action undone (Ctrl+Z)",
                    )
                    .to_string();
            }
            return;
        }

        // Ctrl+C / Cmd+C -> Copy selected part or parts to shared clipboard (to paste into any furniture tab or window)
        if ctx.input(|i| {
            ((i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(Key::C))
                || i.events.iter().any(|e| matches!(e, egui::Event::Copy))
        }) {
            let copied = copy_selected_parts_to_clipboard(
                &doc.project,
                doc.selected_part,
                &doc.viewport_ctrl.selected_parts,
            );
            if !copied.is_empty() {
                if let Ok(json) = serde_json::to_string(&copied) {
                    ctx.copy_text(format!("{}{}", CLIPBOARD_PREFIX, json));
                }
                if copied.len() == 1 {
                    self.status_msg = match lang {
                        AppLanguage::Ru => format!(
                            "Деталь «{}» скопирована (Ctrl+C). Переключитесь на нужную вкладку мебели и нажмите Ctrl+V.",
                            copied[0].part.name
                        ),
                        _ => format!(
                            "Part \"{}\" copied (Ctrl+C). Switch to the target furniture tab and press Ctrl+V.",
                            copied[0].part.name
                        ),
                    };
                } else {
                    self.status_msg = match lang {
                        AppLanguage::Ru => format!(
                            "Скопировано деталей: {} шт. (Ctrl+C). Переключитесь на нужную вкладку мебели и нажмите Ctrl+V.",
                            copied.len()
                        ),
                        _ => format!(
                            "Copied {} parts (Ctrl+C). Switch to the target furniture tab and press Ctrl+V.",
                            copied.len()
                        ),
                    };
                }
                self.clipboard_parts = copied;
            }
            return;
        }

        // Ctrl+V / Cmd+V -> Paste part or parts from clipboard into current furniture tab
        let mut paste_requested = false;
        let mut pasted_from_os: Option<Vec<ClipboardPart>> = None;
        ctx.input(|i| {
            if (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(Key::V) {
                paste_requested = true;
            }
            for ev in &i.events {
                if let egui::Event::Paste(text) = ev {
                    paste_requested = true;
                    if let Some(json_str) = text.strip_prefix(CLIPBOARD_PREFIX) {
                        if let Ok(parsed) = serde_json::from_str::<Vec<ClipboardPart>>(json_str) {
                            if !parsed.is_empty() {
                                pasted_from_os = Some(parsed);
                            }
                        }
                    }
                }
            }
        });

        if paste_requested {
            if let Some(ext_clips) = pasted_from_os {
                self.clipboard_parts = ext_clips;
            }
            if !self.clipboard_parts.is_empty() {
                let clips = self.clipboard_parts.clone();
                Self::push_undo(&mut doc.undo_stack, &doc.project);
                let new_ids = paste_parts_into_project(&mut doc.project, &clips);
                doc.selected_part = new_ids.last().copied();
                doc.viewport_ctrl.selected_parts = new_ids.clone();
                if new_ids.len() == 1 {
                    self.status_msg = match lang {
                        AppLanguage::Ru => format!(
                            "Деталь «{}» вставлена во вкладку «{}» (Ctrl+V)",
                            clips[0].part.name, doc.project.name
                        ),
                        _ => format!(
                            "Part \"{}\" pasted into tab \"{}\" (Ctrl+V)",
                            clips[0].part.name, doc.project.name
                        ),
                    };
                } else {
                    self.status_msg = match lang {
                        AppLanguage::Ru => format!(
                            "Вставлено деталей: {} шт. во вкладку «{}» (Ctrl+V)",
                            new_ids.len(),
                            doc.project.name
                        ),
                        _ => format!(
                            "Pasted {} parts into tab \"{}\" (Ctrl+V)",
                            new_ids.len(),
                            doc.project.name
                        ),
                    };
                }
            }
            return;
        }

        // Ctrl+D / Cmd+D -> Duplicate selected part or parts within current tab
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(Key::D)) {
            let ids_to_dup: Vec<PartId> = if !doc.viewport_ctrl.selected_parts.is_empty() {
                doc.viewport_ctrl.selected_parts.clone()
            } else {
                doc.selected_part.into_iter().collect()
            };
            if !ids_to_dup.is_empty() {
                Self::push_undo(&mut doc.undo_stack, &doc.project);
                let mut new_ids = Vec::new();
                for sid in ids_to_dup {
                    if let Some(orig) = doc.project.parts.iter().find(|p| p.id == sid).cloned() {
                        let new_id = doc.project.alloc_id();
                        let mut copy = orig;
                        copy.id = new_id;
                        copy.name = format!("{} ({})", copy.name, lang.tr("копия", "copy"));
                        copy.position_mm += Vec3::new(36.0, 0.0, 36.0);
                        doc.project.parts.push(copy);
                        new_ids.push(new_id);
                    }
                }
                doc.selected_part = new_ids.last().copied();
                doc.viewport_ctrl.selected_parts = new_ids;
                self.status_msg = lang
                    .tr(
                        "Выделенные детали продублированы (Ctrl+D)",
                        "Selected parts duplicated (Ctrl+D)",
                    )
                    .to_string();
            }
        }

        // Delete / Backspace -> Delete selected part or parts
        if ctx.input(|i| i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace)) {
            let ids_to_del: Vec<PartId> = if !doc.viewport_ctrl.selected_parts.is_empty() {
                doc.viewport_ctrl.selected_parts.clone()
            } else {
                doc.selected_part.into_iter().collect()
            };
            if !ids_to_del.is_empty() {
                Self::push_undo(&mut doc.undo_stack, &doc.project);
                doc.project.parts.retain(|p| !ids_to_del.contains(&p.id));
                doc.selected_part = None;
                doc.viewport_ctrl.selected_parts.clear();
                self.status_msg = lang
                    .tr("Выделенные детали удалены", "Selected parts deleted")
                    .to_string();
            }
        }

        // Tool switching hotkeys (V, B, M — only without Ctrl, Cmd, or Alt modifiers!)
        if ctx.input(|i| !i.modifiers.ctrl && !i.modifiers.command && !i.modifiers.alt && i.key_pressed(Key::V)) {
            doc.viewport_ctrl.tool = EditorTool::SelectAndMove;
        }
        if ctx.input(|i| !i.modifiers.ctrl && !i.modifiers.command && !i.modifiers.alt && i.key_pressed(Key::B)) {
            doc.viewport_ctrl.tool = EditorTool::CreateBlock;
        }
        if ctx.input(|i| !i.modifiers.ctrl && !i.modifiers.command && !i.modifiers.alt && i.key_pressed(Key::M)) {
            doc.viewport_ctrl.tool = EditorTool::MeasureTape;
        }
    }
}

impl eframe::App for QuadFurnApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.startup_maximize_frames < 2 {
            self.startup_maximize_frames += 1;
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
            #[cfg(target_os = "windows")]
            unsafe {
                #[link(name = "user32")]
                extern "system" {
                    fn GetActiveWindow() -> isize;
                    fn ShowWindow(hwnd: isize, n_cmd_show: i32) -> i32;
                }
                let hwnd = GetActiveWindow();
                if hwnd != 0 {
                    const SW_MAXIMIZE: i32 = 3;
                    ShowWindow(hwnd, SW_MAXIMIZE);
                }
            }
            ctx.request_repaint();
        }

        if !self.documents.is_empty() && self.active_doc_idx >= self.documents.len() {
            self.active_doc_idx = self.documents.len() - 1;
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(mut uploads) = crate::web::PENDING_PROJECT_UPLOADS.lock() {
                for (name, bytes) in uploads.drain(..) {
                    if let Ok(mut proj) = serde_json::from_slice::<Project>(&bytes) {
                        if proj.name.is_empty() {
                            proj.name = name.trim_end_matches(".qfurn").trim_end_matches(".json").to_string();
                        }
                        let mut doc = FurnitureDocument::new(proj);
                        doc.mark_saved(None);
                        doc.load_project_textures(ctx, None, self.language);
                        self.documents.push(doc);
                        self.active_doc_idx = self.documents.len() - 1;
                        self.active_tab = AppTab::Designer4View;
                        ctx.request_repaint();
                    }
                }
            }

            if let Ok(mut tex_uploads) = crate::web::PENDING_TEXTURE_UPLOADS.lock() {
                if let Some(doc) = self.documents.get_mut(self.active_doc_idx) {
                    for upload in tex_uploads.drain(..) {
                        let mat_id = upload.material_id;
                        use base64::Engine;
                        let b64 = base64::engine::general_purpose::STANDARD.encode(&upload.bytes);
                        if let Some(m) = doc.project.materials.iter_mut().find(|m| m.id == mat_id) {
                            m.texture_base64 = Some(b64);
                            m.texture_relative_path = Some(upload.filename.clone());
                            m.texture_path = None;
                            match load_texture_from_memory(
                                ctx,
                                &upload.bytes,
                                &m.name,
                                self.language,
                            ) {
                                Ok(tex_info) => {
                                    doc.loaded_textures.insert(mat_id, tex_info.handle);
                                    if let Some(w) = tex_info.warning {
                                        doc.texture_warnings.insert(mat_id, w);
                                    } else {
                                        doc.texture_warnings.remove(&mat_id);
                                    }
                                    self.status_msg = match self.language {
                                        AppLanguage::Ru => format!("Загружена текстура для «{}»: {}", m.name, upload.filename),
                                        AppLanguage::Uk => format!("Завантажена текстура для «{}»: {}", m.name, upload.filename),
                                        _ => format!("Loaded texture for \"{}\": {}", m.name, upload.filename),
                                    };
                                }
                                Err(e) => {
                                    self.status_msg = e;
                                }
                            }
                        }
                        doc.missing_textures_queue.retain(|q| q.material_id != mat_id);
                        ctx.request_repaint();
                    }
                }
            }
        }

        // Drag & Drop files handler (supported on both Desktop and Web)
        for file in ctx.input(|i| i.raw.dropped_files.clone()) {
            let lower_name = file.name.to_lowercase();
            // 1. Project file (.qfurn / .json)
            if lower_name.ends_with(".qfurn") || lower_name.ends_with(".json") {
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(ref p) = file.path {
                    self.open_project_from_path(ctx, p);
                    continue;
                }

                if let Some(ref bytes) = file.bytes {
                    if let Ok(mut proj) = serde_json::from_slice::<Project>(bytes) {
                        if proj.name.is_empty() {
                            proj.name = file.name.trim_end_matches(".qfurn").trim_end_matches(".json").to_string();
                        }
                        let mut doc = FurnitureDocument::new(proj);
                        doc.mark_saved(None);
                        doc.load_project_textures(ctx, None, self.language);
                        self.documents.push(doc);
                        self.active_doc_idx = self.documents.len() - 1;
                        self.active_tab = AppTab::Designer4View;
                        ctx.request_repaint();
                        continue;
                    }
                }
            }

            // 2. Texture image file (.png / .jpg / .jpeg / .webp)
            let is_img = lower_name.ends_with(".png")
                || lower_name.ends_with(".jpg")
                || lower_name.ends_with(".jpeg")
                || lower_name.ends_with(".webp");

            if is_img {
                let img_bytes: Option<Vec<u8>> = if let Some(ref b) = file.bytes {
                    Some(b.to_vec())
                } else {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        file.path.as_ref().and_then(|p| std::fs::read(p).ok())
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        None
                    }
                };

                if let Some(bytes) = img_bytes {
                    if let Some(doc) = self.documents.get_mut(self.active_doc_idx) {
                        let target_mat_id = if !doc.missing_textures_queue.is_empty() {
                            let matched_idx = doc.missing_textures_queue.iter().position(|prompt| {
                                let exp_lower = prompt.expected_filename.to_lowercase();
                                exp_lower == lower_name
                                    || std::path::Path::new(&exp_lower)
                                        .file_name()
                                        .map(|f| f.to_string_lossy().to_string())
                                        .as_deref()
                                        == Some(&lower_name)
                            });
                            let idx = matched_idx.or_else(|| {
                                if doc.missing_textures_queue.len() == 1 {
                                    Some(0)
                                } else {
                                    None
                                }
                            });
                            if let Some(i) = idx {
                                let prompt = doc.missing_textures_queue.remove(i);
                                Some(prompt.material_id)
                            } else {
                                None
                            }
                        } else if self.active_tab == AppTab::MaterialsManager {
                            Some(doc.active_material_id)
                        } else {
                            None
                        };

                        if let Some(mat_id) = target_mat_id {
                            use base64::Engine;
                            let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                            if let Some(m) = doc.project.materials.iter_mut().find(|m| m.id == mat_id) {
                                #[cfg(not(target_arch = "wasm32"))]
                                {
                                    if let Some(ref p) = file.path {
                                        m.texture_path = Some(p.clone());
                                    }
                                    m.texture_relative_path = Some(file.name.clone());
                                    if doc.project.embed_textures {
                                        m.texture_base64 = Some(b64);
                                    }
                                }
                                #[cfg(target_arch = "wasm32")]
                                {
                                    m.texture_base64 = Some(b64);
                                    m.texture_relative_path = Some(file.name.clone());
                                    m.texture_path = None;
                                }

                                match load_texture_from_memory(ctx, &bytes, &m.name, self.language) {
                                    Ok(tex_info) => {
                                        doc.loaded_textures.insert(mat_id, tex_info.handle);
                                        if let Some(w) = tex_info.warning {
                                            doc.texture_warnings.insert(mat_id, w);
                                        } else {
                                            doc.texture_warnings.remove(&mat_id);
                                        }
                                        self.status_msg = match self.language {
                                            AppLanguage::Ru => format!("Загружена текстура для «{}»: {}", m.name, file.name),
                                            AppLanguage::Uk => format!("Завантажена текстура для «{}»: {}", m.name, file.name),
                                            _ => format!("Loaded texture for \"{}\": {}", m.name, file.name),
                                        };
                                    }
                                    Err(e) => {
                                        self.status_msg = e;
                                    }
                                }
                            }
                            ctx.request_repaint();
                        }
                    }
                }
            }
        }

        // Automated window screenshot via egui::ViewportCommand::Screenshot
        for event in ctx.input(|i| i.raw.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event {
                if let Some(ref batch_dir) = self.batch_screenshots_dir {
                    let (lang_code, _) = crate::batch_screenshot::BATCH_LANGS[self.batch_lang_idx];
                    let shot_name = match self.batch_shot_idx {
                        0 => "1designer.png",
                        1 => "2specs.png",
                        2 => "3cutting.png",
                        3 => "4materials.png",
                        4 => "5lighttheme.png",
                        _ => "6languages.png",
                    };
                    let out_dir = batch_dir.join(lang_code);
                    let _ = std::fs::create_dir_all(&out_dir);
                    let out_path = out_dir.join(shot_name);
                    let (w, h) = (image.width() as u32, image.height() as u32);
                    let _ = image::save_buffer(
                        &out_path,
                        image.as_raw(),
                        w,
                        h,
                        image::ExtendedColorType::Rgba8,
                    );
                    let log_line = format!("Saved [{}/{}]: {}\n", lang_code, shot_name, out_path.display());
                    eprint!("{}", log_line);
                    let _ = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(batch_dir.join("progress.log"))
                        .and_then(|mut f| {
                            use std::io::Write;
                            f.write_all(log_line.as_bytes())
                        });

                    self.batch_shot_idx += 1;
                    self.batch_frame_delay = 0;
                    if self.batch_shot_idx >= 6 {
                        self.batch_shot_idx = 0;
                        self.batch_lang_idx += 1;
                    }
                    if self.batch_lang_idx >= crate::batch_screenshot::BATCH_LANGS.len() {
                        self.batch_screenshots_dir = None;
                        self.allow_exit = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    } else {
                        ctx.request_repaint();
                    }
                } else if let Some(path) = self.screenshot_target.take() {
                    let (w, h) = (image.width() as u32, image.height() as u32);
                    let _ = image::save_buffer(
                        &path,
                        image.as_raw(),
                        w,
                        h,
                        image::ExtendedColorType::Rgba8,
                    );
                    self.allow_exit = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }

        if self.batch_screenshots_dir.is_some() {
            crate::batch_screenshot::handle_batch_screenshot_step(self, ctx);
        } else if self.screenshot_target.is_some() {
            if self.screenshot_frame_delay < 6 {
                self.screenshot_frame_delay += 1;
                ctx.request_repaint();
            } else if self.screenshot_frame_delay == 6 {
                self.screenshot_frame_delay += 1;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
                ctx.request_repaint();
            } else {
                ctx.request_repaint();
            }
        }

        // Intercept window close: check for unsaved changes
        if ctx.input(|i| i.viewport().close_requested()) {
            // Always release mouse cursor if captured in 3D Noclip
            for d in &mut self.documents {
                d.viewport_ctrl.cam_3d.noclip_active = false;
            }
            QuadViewportController::set_noclip_cursor_grab(ctx, false, egui::Pos2::ZERO);

            if !self.allow_exit && self.documents.iter().any(|d| d.is_dirty()) {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.show_exit_confirm_dialog = true;
            }
        }

        self.handle_global_shortcuts(ctx);

        let has_open_docs = !self.documents.is_empty();
        let lang = self.language;
        let d = lang.dict();

        // 1. Top menu bar and main mode switcher
        egui::TopBottomPanel::top("main_menu_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                if let Some(tex) = &self.app_icon_tex {
                    ui.add(egui::Image::new((tex.id(), egui::vec2(20.0, 20.0))).rounding(4.0));
                }
                ui.label(
                    RichText::new("QuadFurn CAD")
                        .strong()
                        .size(16.0)
                        .color(Color32::from_rgb(235, 165, 40)),
                );
                ui.separator();

                if ui.button(d.open_in_new_tab).clicked() {
                    self.open_project_interactive(ctx);
                }

                if ui
                    .add_enabled(
                        has_open_docs,
                        egui::Button::new(d.save_current_product),
                    )
                    .on_hover_text(d.save_current_product_hover)
                    .clicked()
                {
                    let saved_path = if let Some(doc) = self.documents.get_mut(self.active_doc_idx)
                    {
                        doc.save_interactive(&mut self.status_msg, self.language)
                    } else {
                        None
                    };
                    if let Some(path) = saved_path {
                        self.record_recent_project(path);
                    }
                }

                if has_open_docs {
                    if let Some(doc) = self.documents.get_mut(self.active_doc_idx) {
                        ui.checkbox(
                            &mut doc.project.embed_textures,
                            lang.tr("📦 Упаковать текстуры", "📦 Embed textures"),
                        )
                        .on_hover_text(lang.tr(
                            "Упаковывать текстуры прямо в файл проекта (.qfurn) в формате Base64 при сохранении для бесшовного переноса на любой ПК",
                            "Embed textures directly into the project file (.qfurn) as Base64 when saving for seamless portability across PCs",
                        ));
                    }
                }

                let has_undo = self
                    .documents
                    .get(self.active_doc_idx)
                    .map(|doc_item| !doc_item.undo_stack.is_empty())
                    .unwrap_or(false);
                if ui
                    .add_enabled(has_undo, egui::Button::new(d.undo_btn))
                    .clicked()
                {
                    if let Some(doc) = self.documents.get_mut(self.active_doc_idx) {
                        if let Some(prev) = doc.undo_stack.pop() {
                            doc.project = prev;
                        }
                    }
                }

                if has_open_docs {
                    ui.separator();

                    // Mode tab switcher (clean labels without redundant parentheses)
                    ui.selectable_value(
                        &mut self.active_tab,
                        AppTab::Designer4View,
                        RichText::new(d.tab_designer).strong(),
                    );
                    ui.selectable_value(
                        &mut self.active_tab,
                        AppTab::Specification,
                        RichText::new(d.tab_specification).strong(),
                    );
                    ui.selectable_value(
                        &mut self.active_tab,
                        AppTab::CuttingLayout,
                        RichText::new(d.tab_cutting).strong(),
                    );
                    ui.selectable_value(
                        &mut self.active_tab,
                        AppTab::MaterialsManager,
                        RichText::new(d.tab_materials).strong(),
                    );
                }

                ui.separator();

                // Color theme switcher: Dark, Light, System (with autosave to AppSettings)
                ui.label(d.theme_label);
                egui::ComboBox::from_id_salt("theme_selector_combo")
                    .selected_text(self.theme_choice.label_i18n(lang))
                    .width(135.0)
                    .show_ui(ui, |ui| {
                        for choice in [ThemeChoice::Dark, ThemeChoice::Light, ThemeChoice::System] {
                            if ui
                                .selectable_value(
                                    &mut self.theme_choice,
                                    choice,
                                    choice.label_i18n(lang),
                                )
                                .clicked()
                            {
                                self.theme_choice.apply(ctx);
                                if let Ok(saved_path) = self.save_settings() {
                                    self.status_msg = match lang {
                                        AppLanguage::Ru => format!(
                                            "Цветовая схема «{}» сохранена в {}",
                                            self.theme_choice.label_i18n(lang),
                                            saved_path.display()
                                        ),
                                        AppLanguage::Uk => format!(
                                            "Колірна схема «{}» збережена в {}",
                                            self.theme_choice.label_i18n(lang),
                                            saved_path.display()
                                        ),
                                        _ => format!(
                                            "Theme \"{}\" saved to {}",
                                            self.theme_choice.label_i18n(lang),
                                            saved_path.display()
                                        ),
                                    };
                                }
                            }
                        }
                    })
                    .response
                    .on_hover_text(format!(
                        "{}: {}",
                        lang.tr(
                            "Цветовая схема сохраняется в",
                            "Theme preference is saved in"
                        ),
                        AppSettings::settings_file_path().display()
                    ));

                // Language switcher icon next to theme selector
                let btn_resp = ui.button(RichText::new(self.language.switcher_button_text()).strong());
                let pop_id = ui.make_persistent_id("lang_switcher_menu_popup");
                if btn_resp.clicked() {
                    ui.memory_mut(|m| m.toggle_popup(pop_id));
                }
                if self.force_open_lang_menu {
                    ui.memory_mut(|m| m.open_popup(pop_id));
                } else if self.batch_screenshots_dir.is_some() {
                    ui.memory_mut(|m| m.close_popup());
                }
                egui::popup_below_widget(
                    ui,
                    pop_id,
                    &btn_resp,
                    egui::PopupCloseBehavior::CloseOnClickOutside,
                    |ui| {
                        ui.set_min_width(210.0);
                        ui.label(
                            RichText::new(self.language.tr(
                                "🌐 Выберите язык интерфейса:",
                                "🌐 Select interface language:",
                            ))
                            .strong()
                            .size(12.5),
                        );
                        ui.separator();
                        egui::ScrollArea::vertical()
                            .id_salt("language_selector_scroll")
                            .max_height(380.0)
                            .show(ui, |ui| {
                                for candidate in AppLanguage::ALL {
                                    if ui
                                        .selectable_value(
                                            &mut self.language,
                                            candidate,
                                            format!("🌐 {}", candidate.label()),
                                        )
                                        .clicked()
                                    {
                                        ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                                            self.language.dict().window_title.to_string(),
                                        ));
                                        if self.new_product_name_buf == "Новое изделие 1"
                                            || self.new_product_name_buf == "New Product 1"
                                        {
                                            self.new_product_name_buf = self
                                                .language
                                                .tr("Новое изделие 1", "New Product 1")
                                                .to_string();
                                        }
                                        if let Ok(saved_path) = self.save_settings() {
                                            self.status_msg = match self.language {
                                                AppLanguage::Ru => format!(
                                                    "Язык интерфейса изменён на «{}» (сохранено в {})",
                                                    self.language.label(),
                                                    saved_path.display()
                                                ),
                                                AppLanguage::Uk => format!(
                                                    "Мову інтерфейсу змінено на «{}» (збережено в {})",
                                                    self.language.label(),
                                                    saved_path.display()
                                                ),
                                                _ => format!(
                                                    "Interface language changed to \"{}\" (saved in {})",
                                                    self.language.label(),
                                                    saved_path.display()
                                                ),
                                            };
                                        }
                                        ui.close_menu();
                                    }
                                }
                            });
                    },
                );
                btn_resp.on_hover_text(self.language.switcher_hover_tooltip());
            });
        });

        let lang = self.language;
        let d = lang.dict();

        // 2. Furniture product tabs bar (displayed when at least one product is open)
        if has_open_docs {
            egui::TopBottomPanel::top("furniture_objects_tab_bar").show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(d.products_bar_label).strong());

                    let mut switch_to: Option<usize> = None;

                    for idx in 0..self.documents.len() {
                        let is_active = idx == self.active_doc_idx;
                        let is_dirty = self.documents[idx].is_dirty();
                        ui.group(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            if self.documents[idx].renaming_tab {
                                let resp = ui.add(
                                    egui::TextEdit::singleline(
                                        &mut self.documents[idx].project.name,
                                    )
                                    .desired_width(145.0),
                                );
                                if resp.lost_focus()
                                    || ui.input(|i| i.key_pressed(Key::Enter))
                                {
                                    self.documents[idx].renaming_tab = false;
                                }
                            } else {
                                let dirty_mark = if is_dirty { " *" } else { "" };
                                let tab_title = format!(
                                    "🗄 {}{} ({})",
                                    self.documents[idx].project.name,
                                    dirty_mark,
                                    self.documents[idx].project.parts.len()
                                );
                                let resp = ui.selectable_label(
                                    is_active,
                                    if is_active {
                                        RichText::new(tab_title).strong()
                                    } else {
                                        RichText::new(tab_title)
                                    },
                                );
                                if resp.clicked() {
                                    switch_to = Some(idx);
                                }
                                if resp
                                    .on_hover_text(lang.tr(
                                        "Клик — переключиться на этот объект мебели. Двойной клик — переименовать.",
                                        "Click to switch to this furniture product. Double-click to rename.",
                                    ))
                                    .double_clicked()
                                {
                                    self.documents[idx].renaming_tab = true;
                                }
                            }

                            if ui
                                .small_button("✖")
                                .on_hover_text(lang.tr(
                                    "Закрыть вкладку изделия",
                                    "Close product tab",
                                ))
                                .clicked()
                            {
                                self.pending_close_doc_idx = Some(idx);
                            }
                        });
                    }

                    if let Some(target_idx) = switch_to {
                        if target_idx != self.active_doc_idx {
                            // When switching tabs, disable Noclip mouse capture in old tab
                            self.documents[self.active_doc_idx]
                                .viewport_ctrl
                                .cam_3d
                                .noclip_active = false;
                            QuadViewportController::set_noclip_cursor_grab(
                                ctx,
                                false,
                                egui::Pos2::ZERO,
                            );
                            self.active_doc_idx = target_idx;
                        }
                    }

                    ui.separator();

                    if ui
                        .button(d.empty_object_btn)
                        .on_hover_text(d.empty_object_hover)
                        .clicked()
                    {
                        let p = Project::new_empty_i18n(
                            format!(
                                "{} {}",
                                lang.tr("Изделие", "Product"),
                                self.documents.len() + 1
                            ),
                            lang,
                        );
                        self.documents.push(FurnitureDocument::new(p));
                        self.active_doc_idx = self.documents.len() - 1;
                        self.status_msg = lang
                            .tr("Создано новое пустое изделие", "Created new empty product")
                            .to_string();
                    }

                    if ui
                        .button(d.cabinet_template_btn)
                        .on_hover_text(d.cabinet_template_hover)
                        .clicked()
                    {
                        let mut p = Project::new_demo_cabinet_i18n(lang);
                        p.name = format!(
                            "{} {}",
                            lang.tr("Шкаф", "Cabinet"),
                            self.documents.len() + 1
                        );
                        self.documents.push(FurnitureDocument::new(p));
                        self.active_doc_idx = self.documents.len() - 1;
                        self.status_msg = lang
                            .tr(
                                "Создана новая вкладка с шаблоном шкафа",
                                "Created new tab with cabinet template",
                            )
                            .to_string();
                    }
                });
            });
        }

        // Dialog confirming closure of furniture tab
        if let Some(close_idx) = self.pending_close_doc_idx {
            if close_idx < self.documents.len() {
                let doc_name = self.documents[close_idx].project.name.clone();
                let doc_dirty = self.documents[close_idx].is_dirty();
                let mut save_and_close = false;
                let mut close_without_saving = false;
                let mut cancel = false;

                egui::Window::new(lang.tr("Закрытие вкладки изделия", "Close Product Tab"))
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                    .show(ctx, |ui| {
                        if doc_dirty {
                            ui.label(
                                RichText::new(match lang {
                                    AppLanguage::Ru => format!(
                                        "Сохранить изменения в изделии «{}» перед закрытием вкладки?",
                                        doc_name
                                    ),
                                    AppLanguage::Uk => format!(
                                        "Зберегти зміни у виробі «{}» перед закриттям вкладки?",
                                        doc_name
                                    ),
                                    _ => format!(
                                        "Save changes to product \"{}\" before closing the tab?",
                                        doc_name
                                    ),
                                })
                                .strong()
                                .size(15.0),
                            );
                            ui.label(lang.tr(
                                "В этой вкладке есть несохранённые изменения.",
                                "This tab has unsaved changes.",
                            ));
                        } else {
                            ui.label(
                                RichText::new(match lang {
                                    AppLanguage::Ru => format!("Закрыть вкладку «{}»?", doc_name),
                                    AppLanguage::Uk => format!("Закрити вкладку «{}»?", doc_name),
                                    _ => format!("Close tab \"{}\"?", doc_name),
                                })
                                .strong()
                                .size(15.0),
                            );
                        }
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if doc_dirty {
                                if ui
                                    .button(
                                        RichText::new(lang.tr(
                                            "💾 Сохранить и закрыть",
                                            "💾 Save and close",
                                        ))
                                        .strong()
                                        .color(Color32::from_rgb(90, 210, 120)),
                                    )
                                    .clicked()
                                {
                                    save_and_close = true;
                                }
                                ui.add_space(6.0);
                            }
                            let discard_label = if doc_dirty {
                                lang.tr("Закрыть без сохранения", "Close without saving")
                            } else {
                                lang.tr("✔ Да, закрыть", "✔ Yes, close")
                            };
                            if ui
                                .button(
                                    RichText::new(discard_label)
                                        .strong()
                                        .color(Color32::from_rgb(245, 95, 95)),
                                )
                                .clicked()
                            {
                                close_without_saving = true;
                            }
                            ui.add_space(6.0);
                            if ui.button(lang.tr("Отмена", "Cancel")).clicked() {
                                cancel = true;
                            }
                        });
                    });

                if save_and_close {
                    let saved_path =
                        self.documents[close_idx].save_interactive(&mut self.status_msg, self.language);
                    if let Some(path) = saved_path {
                        self.record_recent_project(path);
                        self.documents.remove(close_idx);
                        if !self.documents.is_empty() && self.active_doc_idx >= self.documents.len()
                        {
                            self.active_doc_idx = self.documents.len() - 1;
                        } else if self.documents.is_empty() {
                            self.active_doc_idx = 0;
                        }
                        self.pending_close_doc_idx = None;
                    }
                } else if close_without_saving {
                    self.documents.remove(close_idx);
                    if !self.documents.is_empty() && self.active_doc_idx >= self.documents.len() {
                        self.active_doc_idx = self.documents.len() - 1;
                    } else if self.documents.is_empty() {
                        self.active_doc_idx = 0;
                    }
                    self.pending_close_doc_idx = None;
                } else if cancel {
                    self.pending_close_doc_idx = None;
                }
            } else {
                self.pending_close_doc_idx = None;
            }
        }

        // Save project confirmation dialog before exiting program
        if self.show_exit_confirm_dialog {
            let dirty_names: Vec<String> = self
                .documents
                .iter()
                .filter(|d| d.is_dirty())
                .map(|d| format!("«{}»", d.project.name))
                .collect();

            if dirty_names.is_empty() {
                self.show_exit_confirm_dialog = false;
                self.allow_exit = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else {
                let mut save_and_exit = false;
                let mut exit_without_saving = false;
                let mut cancel_exit = false;

                egui::Window::new(lang.tr(
                    "Сохранение проекта перед выходом",
                    "Save Project Before Exit",
                ))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label(
                        RichText::new(lang.tr(
                            "Сохранить открытый проект перед выходом?",
                            "Save open project before exiting?",
                        ))
                        .strong()
                        .size(16.0),
                    );
                    ui.add_space(4.0);
                    ui.label(format!(
                        "{}: {}",
                        lang.tr("Несохранённые изменения в", "Unsaved changes in"),
                        dirty_names.join(", ")
                    ));
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                RichText::new(lang.tr(
                                    "💾 Сохранить и выйти",
                                    "💾 Save and exit",
                                ))
                                .strong()
                                .color(Color32::from_rgb(90, 210, 120)),
                            )
                            .clicked()
                        {
                            save_and_exit = true;
                        }
                        ui.add_space(8.0);
                        if ui
                            .button(
                                RichText::new(lang.tr(
                                    "Выйти без сохранения",
                                    "Exit without saving",
                                ))
                                .strong()
                                .color(Color32::from_rgb(245, 95, 95)),
                            )
                            .clicked()
                        {
                            exit_without_saving = true;
                        }
                        ui.add_space(8.0);
                        if ui.button(lang.tr("Отмена", "Cancel")).clicked() {
                            cancel_exit = true;
                        }
                    });
                });

                if save_and_exit {
                    let mut all_saved = true;
                    let mut saved_paths = Vec::new();
                    for d in &mut self.documents {
                        if d.is_dirty() {
                            if let Some(p) = d.save_interactive(&mut self.status_msg, self.language) {
                                saved_paths.push(p);
                            } else {
                                all_saved = false;
                                break;
                            }
                        }
                    }
                    for p in saved_paths {
                        self.record_recent_project(p);
                    }
                    if all_saved {
                        self.show_exit_confirm_dialog = false;
                        self.allow_exit = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                } else if exit_without_saving {
                    self.show_exit_confirm_dialog = false;
                    self.allow_exit = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                } else if cancel_exit {
                    self.show_exit_confirm_dialog = false;
                }
            }
        }

        // Missing textures sequential prompt modal dialog
        #[allow(unused_mut)]
        let mut missing_status_update = None;
        if let Some(doc) = self.documents.get_mut(self.active_doc_idx) {
            if !doc.missing_textures_queue.is_empty() {
                let current = doc.missing_textures_queue[0].clone();
                let queue_len = doc.missing_textures_queue.len();
                let mut action_browse = false;
                let mut action_skip = false;
                let mut action_skip_all = false;

                egui::Window::new(lang.tr("🔍 Поиск отсутствующих текстур", "🔍 Locate Missing Textures"))
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                    .show(ctx, |ui| {
                        ui.set_min_width(380.0);
                        ui.label(
                            RichText::new(lang.tr(
                                "Не найден файл текстуры для материала:",
                                "Texture file not found for material:",
                            ))
                            .strong()
                            .size(15.0),
                        );

                        ui.add_space(4.0);

                        let progress_text = match lang {
                            AppLanguage::Ru => format!("Осталось найти текстур: {}", queue_len),
                            AppLanguage::Uk => format!("Залишилося знайти текстур: {}", queue_len),
                            _ => format!("Missing textures remaining: {}", queue_len),
                        };
                        ui.label(RichText::new(progress_text).color(Color32::from_rgb(240, 180, 50)).strong());

                        ui.add_space(8.0);

                        egui::Frame::group(ui.style())
                            .fill(ui.visuals().faint_bg_color)
                            .show(ui, |ui| {
                                egui::Grid::new("missing_tex_grid")
                                    .num_columns(2)
                                    .spacing([12.0, 6.0])
                                    .show(ui, |ui| {
                                        ui.label(RichText::new(lang.tr("Материал:", "Material:")).strong());
                                        ui.label(RichText::new(&current.material_name).strong());
                                        ui.end_row();

                                        ui.label(RichText::new(lang.tr("Ожидаемый файл:", "Expected file:")).strong());
                                        ui.label(
                                            RichText::new(&current.expected_filename)
                                                .monospace()
                                                .color(Color32::from_rgb(100, 180, 240)),
                                        );
                                        ui.end_row();
                                    });
                            });

                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(lang.tr(
                                "💡 Выберите файл на диске или перетащите его мышью в окно программы.",
                                "💡 Choose file from disk or drag & drop it into the application window.",
                            ))
                            .small()
                            .color(Color32::GRAY),
                        );

                        ui.add_space(12.0);

                        ui.horizontal(|ui| {
                            if ui
                                .button(
                                    RichText::new(lang.tr("📂 Указать файл...", "📂 Browse File..."))
                                        .strong()
                                        .color(Color32::from_rgb(90, 210, 120)),
                                )
                                .clicked()
                            {
                                action_browse = true;
                            }

                            ui.add_space(8.0);

                            if ui
                                .button(lang.tr("⏭ Пропустить", "⏭ Skip"))
                                .on_hover_text(lang.tr(
                                    "Пропустить эту текстуру и перейти к следующей",
                                    "Skip this texture and proceed to the next",
                                ))
                                .clicked()
                            {
                                action_skip = true;
                            }

                            ui.add_space(8.0);

                            if ui
                                .button(lang.tr("✖ Пропустить все", "✖ Skip All"))
                                .on_hover_text(lang.tr(
                                    "Пропустить все оставшиеся отсутствующие текстуры",
                                    "Skip all remaining missing textures",
                                ))
                                .clicked()
                            {
                                action_skip_all = true;
                            }
                        });
                    });

                if action_skip_all {
                    doc.missing_textures_queue.clear();
                } else if action_skip {
                    doc.missing_textures_queue.remove(0);
                } else if action_browse {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let mut dialog = rfd::FileDialog::new()
                            .add_filter(
                                lang.tr(
                                    "Изображения текстур (PNG, JPG, WEBP)",
                                    "Texture Images (PNG, JPG, WEBP)",
                                ),
                                &["png", "jpg", "jpeg", "webp"],
                            )
                            .set_file_name(&current.expected_filename);

                        if let Some(ref proj_path) = doc.file_path {
                            if let Some(parent) = proj_path.parent() {
                                dialog = dialog.set_directory(parent);
                            }
                        }

                        if let Some(path) = dialog.pick_file() {
                            match load_texture_from_path(ctx, &path, &current.material_name, lang) {
                                Ok(tex_info) => {
                                    let mat_id = current.material_id;
                                    doc.loaded_textures.insert(mat_id, tex_info.handle);
                                    if let Some(m) = doc.project.materials.iter_mut().find(|m| m.id == mat_id) {
                                        m.texture_path = Some(path.clone());
                                        if let Some(fname) = path.file_name() {
                                            m.texture_relative_path = Some(fname.to_string_lossy().to_string());
                                        }
                                        if doc.project.embed_textures {
                                            if let Ok(bytes) = std::fs::read(&path) {
                                                if bytes.len() <= 60 * 1024 * 1024 {
                                                    use base64::Engine;
                                                    m.texture_base64 = Some(
                                                        base64::engine::general_purpose::STANDARD.encode(&bytes),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    if let Some(w) = tex_info.warning {
                                        doc.texture_warnings.insert(mat_id, w);
                                    } else {
                                        doc.texture_warnings.remove(&mat_id);
                                    }
                                    doc.missing_textures_queue.remove(0);
                                    missing_status_update = Some(match lang {
                                        AppLanguage::Ru => format!(
                                            "Загружена текстура для «{}»: {}",
                                            current.material_name,
                                            path.display()
                                        ),
                                        AppLanguage::Uk => format!(
                                            "Завантажена текстура для «{}»: {}",
                                            current.material_name,
                                            path.display()
                                        ),
                                        _ => format!(
                                            "Loaded texture for \"{}\": {}",
                                            current.material_name,
                                            path.display()
                                        ),
                                    });
                                    ctx.request_repaint();
                                }
                                Err(e) => {
                                    missing_status_update = Some(e);
                                }
                            }
                        }
                    }

                    #[cfg(target_arch = "wasm32")]
                    {
                        crate::web::pick_texture_file_web(current.material_id);
                    }
                }
            }
        }
        if let Some(msg) = missing_status_update {
            self.status_msg = msg;
        }

        // If no products are open yet — show bottom status bar and Start Page
        if self.documents.is_empty() {
            egui::TopBottomPanel::bottom("status_footer").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(&self.status_msg);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(d.start_footer_status)
                                .color(Color32::GRAY),
                        );
                    });
                });
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                self.render_start_page(ctx, ui);
            });
            return;
        }

        let doc = &mut self.documents[self.active_doc_idx];
        doc.viewport_ctrl.lang = lang;

        // If we leave Designer tab, reliably disable Noclip mouse capture
        if self.active_tab != AppTab::Designer4View && doc.viewport_ctrl.cam_3d.noclip_active {
            doc.viewport_ctrl.cam_3d.noclip_active = false;
            QuadViewportController::set_noclip_cursor_grab(ctx, false, egui::Pos2::ZERO);
        }

        // Bottom status bar
        egui::TopBottomPanel::bottom("status_footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_msg);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let right_footer = match lang {
                        AppLanguage::Ru => format!(
                            "Изделие: «{}» | Деталей: {} шт. | Все размеры в мм",
                            doc.project.name,
                            doc.project.parts.len()
                        ),
                        AppLanguage::Uk => format!(
                            "Виріб: «{}» | Деталей: {} шт. | Усі розміри в мм",
                            doc.project.name,
                            doc.project.parts.len()
                        ),
                        _ => format!(
                            "Product: \"{}\" | Parts: {} pcs | All dimensions in mm",
                            doc.project.name,
                            doc.project.parts.len()
                        ),
                    };
                    ui.label(RichText::new(right_footer).color(Color32::GRAY));
                });
            });
        });

        match self.active_tab {
            AppTab::Designer4View => {
                // Top transform toolbar for precise dimensions and coordinates
                egui::TopBottomPanel::top("top_transform_bar").show(ctx, |ui| {
                    let undo_ref = &mut doc.undo_stack;
                    ui_top_transform_toolbar(
                        ui,
                        &mut doc.project,
                        &mut doc.selected_part,
                        &mut doc.active_material_id,
                        &mut doc.viewport_ctrl,
                        |proj| Self::push_undo(undo_ref, proj),
                    );
                });

                // Right inspector for blocks, groups, materials, and edge banding
                egui::SidePanel::right("right_block_inspector")
                    .default_width(330.0)
                    .min_width(275.0)
                    .max_width(460.0)
                    .resizable(true)
                    .show(ctx, |ui| {
                        let undo_ref = &mut doc.undo_stack;
                        ui_right_inspector(
                            ui,
                            &mut doc.project,
                            &mut doc.selected_part,
                            &mut doc.viewport_ctrl.selected_parts,
                            &mut doc.active_material_id,
                            &mut doc.new_group_name_buf,
                            &mut self.clipboard_parts,
                            &mut self.status_msg,
                            lang,
                            |proj| Self::push_undo(undo_ref, proj),
                        );
                    });

                // Central viewports: 4 views (Top XY, 3D Orbit/Z-Noclip, Front XZ, Side YZ)
                egui::CentralPanel::default()
                    .frame(egui::Frame::none().fill(Color32::from_rgb(14, 17, 21)))
                    .show(ctx, |ui| {
                        let undo_ref = &mut doc.undo_stack;
                        doc.viewport_ctrl.ui_quad_viewports(
                            ui,
                            &mut doc.project,
                            &mut doc.selected_part,
                            doc.active_material_id,
                            &doc.loaded_textures,
                            |proj| Self::push_undo(undo_ref, proj),
                        );
                    });
            }
            AppTab::Specification => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui_specification_tab(ui, &mut doc.project, &mut self.status_msg, lang);
                });
            }
            AppTab::CuttingLayout => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui_cutting_layout_tab(ui, &mut doc.project, &mut doc.selected_sheet_idx, lang);
                });
            }
            AppTab::MaterialsManager => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui_materials_manager_tab(
                        ui,
                        &mut doc.project,
                        &mut doc.active_material_id,
                        &mut doc.loaded_textures,
                        &mut doc.texture_warnings,
                        &mut doc.pending_delete_material,
                        &mut doc.pending_delete_edge,
                        &mut self.status_msg,
                        lang,
                    );
                });
            }
        }
    }
}

/// Configures UI fonts: attaches system fonts of Windows, Linux and macOS
/// to display icons, technical symbols (Δ, ×, ∠, ↺), Cyrillic and CJK.
pub fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 1. Windows emoji font (Segoe UI Emoji — monochromatic outlines for modern icons, including 🪚 and 🌐)
    if let Some(bytes) = crate::export::read_system_font_bytes("C:\\Windows\\Fonts\\seguiemj.ttf") {
        fonts.font_data.insert(
            "seguiemj".to_owned(),
            egui::FontData::from_owned(bytes).into(),
        );
        if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            vec.insert(0, "seguiemj".to_owned());
        }
        if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            vec.insert(0, "seguiemj".to_owned());
        }
    }

    // 2. Windows technical symbols font (Segoe UI Symbol — vector icons 🗄, 🗑, 👁, 🖥, 📁, 💾, 🔍, 🔒 and arrows)
    if let Some(bytes) = crate::export::read_system_font_bytes("C:\\Windows\\Fonts\\seguisym.ttf") {
        fonts.font_data.insert(
            "seguisym".to_owned(),
            egui::FontData::from_owned(bytes).into(),
        );
        if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            vec.insert(0, "seguisym".to_owned());
        }
        if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            vec.insert(0, "seguisym".to_owned());
        }
    }

    // 3. Main proportional UI font (Windows: Segoe UI, macOS: SF / Helvetica / Arial, Linux: DejaVu Sans / Liberation Sans / Noto Sans)
    for (font_key, path) in [
        ("sys_sans_win", "C:\\Windows\\Fonts\\segoeui.ttf"),
        (
            "sys_sans_mac",
            "/System/Library/Fonts/Supplemental/Arial.ttf",
        ),
        (
            "sys_sans_linux_dejavu",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ),
        (
            "sys_sans_linux_liberation",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        ),
        (
            "sys_sans_linux_noto",
            "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
        ),
    ] {
        if let Some(raw) = crate::export::read_system_font_bytes(path) {
            let ttf = crate::export::ensure_ttf_bytes(raw);
            fonts
                .font_data
                .insert(font_key.to_owned(), egui::FontData::from_owned(ttf).into());
            if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                vec.insert(0, font_key.to_owned());
            }
            if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                vec.insert(0, font_key.to_owned());
            }
            break;
        }
    }

    // 4. Monospace font (Windows: Consolas, macOS: Menlo / Courier New, Linux: DejaVu Sans Mono)
    for (font_key, path) in [
        ("sys_mono_win", "C:\\Windows\\Fonts\\consola.ttf"),
        (
            "sys_mono_mac",
            "/System/Library/Fonts/Supplemental/Courier New.ttf",
        ),
        (
            "sys_mono_linux",
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
        ),
    ] {
        if let Some(raw) = crate::export::read_system_font_bytes(path) {
            let ttf = crate::export::ensure_ttf_bytes(raw);
            fonts
                .font_data
                .insert(font_key.to_owned(), egui::FontData::from_owned(ttf).into());
            if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                vec.insert(0, font_key.to_owned());
            }
            break;
        }
    }

    // 5. Symbol fallback fonts for Linux / macOS (Apple Symbols, Noto Sans Symbols2, Symbola)
    for (font_key, path) in [
        ("sym_mac", "/System/Library/Fonts/Apple Symbols.ttf"),
        (
            "sym_mac_unicode",
            "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        ),
        (
            "sym_linux_noto2",
            "/usr/share/fonts/truetype/noto/NotoSansSymbols2-Regular.ttf",
        ),
        (
            "sym_linux_noto",
            "/usr/share/fonts/truetype/noto/NotoSansSymbols-Regular.ttf",
        ),
    ] {
        if let Some(raw) = crate::export::read_system_font_bytes(path) {
            let ttf = crate::export::ensure_ttf_bytes(raw);
            fonts
                .font_data
                .insert(font_key.to_owned(), egui::FontData::from_owned(ttf).into());
            if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                vec.push(font_key.to_owned());
            }
            if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                vec.push(font_key.to_owned());
            }
        }
    }

    // 6. CJK support (Korean 한국어, Simplified Chinese 简体中文, Japanese 日本語) on Windows, macOS, and Linux
    for (font_key, path) in [
        ("cjk_msyh", "C:\\Windows\\Fonts\\msyh.ttc"),
        ("cjk_malgun", "C:\\Windows\\Fonts\\malgun.ttf"),
        ("cjk_mac_pingfang", "/System/Library/Fonts/PingFang.ttc"),
        (
            "cjk_mac_sdgothic",
            "/System/Library/Fonts/AppleSDGothicNeo.ttc",
        ),
        (
            "cjk_linux_noto_ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        ),
        (
            "cjk_linux_wqy",
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        ),
    ] {
        if let Some(raw) = crate::export::read_system_font_bytes(path) {
            let ttf = crate::export::ensure_ttf_bytes(raw);
            fonts
                .font_data
                .insert(font_key.to_owned(), egui::FontData::from_owned(ttf).into());
            if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                vec.push(font_key.to_owned());
            }
            if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                vec.push(font_key.to_owned());
            }
        }
    }

    ctx.set_fonts(fonts);
}

const APP_ICON_JPEG: &[u8] = include_bytes!("../assets/icon.jpeg");

/// Loads application icon from `assets/icon.jpeg` for window titlebar and taskbar
pub fn load_app_icon_data() -> Option<std::sync::Arc<egui::IconData>> {
    let dyn_img = image::load_from_memory(APP_ICON_JPEG).ok()?;
    let resized = dyn_img.resize_exact(256, 256, image::imageops::FilterType::Lanczos3);
    let rgba = resized.to_rgba8();
    let (width, height) = rgba.dimensions();
    Some(std::sync::Arc::new(egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    }))
}

/// Loads application icon texture for start page and top bar display
pub fn load_app_icon_texture(ctx: &egui::Context) -> Option<TextureHandle> {
    let dyn_img = image::load_from_memory(APP_ICON_JPEG).ok()?;
    let resized = dyn_img.resize_exact(256, 256, image::imageops::FilterType::Lanczos3);
    let rgba = resized.to_rgba8();
    let dims = [rgba.width() as usize, rgba.height() as usize];
    let color_img =
        egui::ColorImage::from_rgba_unmultiplied(dims, rgba.as_flat_samples().as_slice());
    Some(ctx.load_texture("quadfurn_app_icon", color_img, egui::TextureOptions::LINEAR))
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let initial_settings = AppSettings::load();
    // Set base (restored) window size 1520×920 without .with_maximized(true),
    // because winit on Windows with simultaneous with_maximized(true) + with_inner_size(...)
    // issues SetWindowPos over WS_MAXIMIZE (leaving window 1520×920 with restore icon ❐).
    // Actual SW_MAXIMIZE maximization is executed on first frame render.
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(initial_settings.language.dict().window_title)
        .with_inner_size([1520.0, 920.0])
        .with_min_inner_size([1024.0, 680.0]);

    if let Some(icon_data) = load_app_icon_data() {
        viewport = viewport.with_icon(icon_data);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        vsync: true,
        ..Default::default()
    };

    eframe::run_native(
        "QuadFurn CAD",
        native_options,
        Box::new(|cc| {
            configure_fonts(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            let mut app = QuadFurnApp::default();
            app.app_icon_tex = load_app_icon_texture(&cc.egui_ctx);
            app.theme_choice.apply(&cc.egui_ctx);

            let raw_args: Vec<String> = std::env::args().skip(1).collect();

            // First pass: extract --lang to set app language before creating documents
            let mut i = 0;
            while i < raw_args.len() {
                if raw_args[i] == "--lang" && i + 1 < raw_args.len() {
                    let l = &raw_args[i + 1];
                    if l.eq_ignore_ascii_case("ru") {
                        app.language = AppLanguage::Ru;
                    } else if l.eq_ignore_ascii_case("en") {
                        app.language = AppLanguage::En;
                    } else if let Some(lang) = AppLanguage::from_locale_tag(l) {
                        app.language = lang;
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }

            // Second pass: process document creation and execution flags
            let mut iter = raw_args.into_iter();
            while let Some(arg) = iter.next() {
                if arg == "--demo" {
                    app.documents.push(FurnitureDocument::new(Project::new_demo_cabinet_i18n(app.language)));
                    app.active_doc_idx = app.documents.len() - 1;
                    app.active_tab = AppTab::Designer4View;
                } else if arg == "--materials" {
                    app.documents.push(FurnitureDocument::new(Project::new_demo_cabinet_i18n(app.language)));
                    app.active_doc_idx = app.documents.len() - 1;
                    app.active_tab = AppTab::MaterialsManager;
                } else if arg == "--embed" {
                    if let Some(doc) = app.documents.last_mut() {
                        doc.project.embed_textures = true;
                    }
                } else if arg == "--lang" {
                    let _ = iter.next();
                } else if arg == "--batch-screenshots" {
                    if let Some(target) = iter.next() {
                        app.batch_screenshots_dir = Some(std::path::PathBuf::from(target));
                    }
                } else if arg == "--screenshot" {
                    if let Some(target) = iter.next() {
                        app.screenshot_target = Some(std::path::PathBuf::from(target));
                    }
                } else {
                    let p = std::path::PathBuf::from(&arg);
                    if p.exists() {
                        app.open_project_from_path(&cc.egui_ctx, &p);
                    }
                }
            }

            Ok(Box::new(app))
        }),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    configure_fonts(&cc.egui_ctx);
                    egui_extras::install_image_loaders(&cc.egui_ctx);
                    let mut app = QuadFurnApp::default();
                    app.app_icon_tex = load_app_icon_texture(&cc.egui_ctx);
                    app.theme_choice.apply(&cc.egui_ctx);

                    // Preload a demo cabinet so online user immediately has a 3D model to inspect
                    app.documents.push(FurnitureDocument::new(Project::new_demo_cabinet_i18n(app.language)));
                    app.active_doc_idx = 0;
                    app.active_tab = AppTab::Designer4View;

                    Ok(Box::new(app))
                }),
            )
            .await;

        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p style=\"color:#f87171;font-family:sans-serif;text-align:center;padding:40px;\">⚠️ Failed to start WebAssembly graphics context.</p>",
                    );
                    log::error!("Failed to start eframe: {:?}", e);
                }
            }
        }
    });
}

#[cfg(all(test, target_os = "windows"))]
mod icon_tests {
    use super::*;

    #[test]
    fn test_all_ui_icons_rasterize() {
        let ctx = egui::Context::default();
        configure_fonts(&ctx);
        let icons = [
            "🌙", "☀", "🖥", "🪚", "📂", "💾", "↩", "📐", "📋", "🎨", "🗄", "✖", "➕", "✔",
            "↖", "📏", "🔍", "↺", "📁", "📄", "📥", "🗑", "▼", "▶", "👁", "🚫", "🔒", "🔓",
            "⇄", "🌲", "🔄", "⚠", "►", "◄", "▲", "∠", "Δ", "×", "°", "🌐", "📦",
        ];
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            ctx.fonts(|f| {
                let notdef_galley = f.layout_no_wrap(
                    "\u{FFFD}".to_string(),
                    egui::FontId::proportional(16.0),
                    Color32::WHITE,
                );
                let notdef_uv = notdef_galley.rows[0].glyphs[0].uv_rect;
                println!("notdef uv_rect: min={:?}, max={:?}, size={:?}", notdef_uv.min, notdef_uv.max, notdef_uv.size);

                for ic in icons {
                    let galley = f.layout_no_wrap(
                        ic.to_string(),
                        egui::FontId::proportional(16.0),
                        Color32::WHITE,
                    );
                    for row in &galley.rows {
                        for g in &row.glyphs {
                            let is_tofu = g.uv_rect.min == notdef_uv.min && g.uv_rect.max == notdef_uv.max;
                            println!(
                                "Icon '{}' (U+{:04X}) -> uv_min={:?}, size={:?}, is_tofu={}",
                                ic,
                                ic.chars().next().unwrap() as u32,
                                g.uv_rect.min,
                                g.uv_rect.size,
                                is_tofu
                            );
                            assert!(!is_tofu, "Icon '{}' rendered as .notdef tofu!", ic);
                        }
                    }
                }
            });
        });
    }
}

#[cfg(test)]
mod texture_missing_tests {
    use super::*;
    use crate::model::Material;

    #[test]
    fn test_missing_textures_queue_and_sequential_prompt() {
        let ctx = egui::Context::default();
        let mut proj = Project::default();
        proj.materials.clear();

        // Material 1: Missing texture with relative path
        let mut m1 = Material::default();
        m1.id = 1;
        m1.name = "Oak Sonoma".to_string();
        m1.texture_relative_path = Some("oak_sonoma.png".to_string());
        proj.materials.push(m1);

        // Material 2: Solid color (no texture)
        let mut m2 = Material::default();
        m2.id = 2;
        m2.name = "White Matte".to_string();
        proj.materials.push(m2);

        // Material 3: Missing texture with absolute path
        let mut m3 = Material::default();
        m3.id = 3;
        m3.name = "Dark Walnut".to_string();
        m3.texture_path = Some(std::path::PathBuf::from("C:\\NonExistent\\dark_walnut.jpg"));
        proj.materials.push(m3);

        let mut doc = FurnitureDocument::new(proj);
        doc.load_project_textures(&ctx, None, AppLanguage::En);

        // Both missing materials must be in the queue
        assert_eq!(doc.missing_textures_queue.len(), 2);
        assert_eq!(doc.missing_textures_queue[0].material_id, 1);
        assert_eq!(doc.missing_textures_queue[0].material_name, "Oak Sonoma");
        assert_eq!(doc.missing_textures_queue[0].expected_filename, "oak_sonoma.png");

        assert_eq!(doc.missing_textures_queue[1].material_id, 3);
        assert_eq!(doc.missing_textures_queue[1].material_name, "Dark Walnut");
        assert_eq!(doc.missing_textures_queue[1].expected_filename, "dark_walnut.jpg");

        // Simulate user skipping first texture:
        let skipped = doc.missing_textures_queue.remove(0);
        assert_eq!(skipped.material_id, 1);
        assert_eq!(doc.missing_textures_queue.len(), 1);
        assert_eq!(doc.missing_textures_queue[0].material_id, 3);

        // Simulate user clicking "Skip All":
        doc.missing_textures_queue.clear();
        assert!(doc.missing_textures_queue.is_empty());
    }

    #[test]
    fn test_embedded_texture_loads_without_queueing_missing() {
        let ctx = egui::Context::default();
        let mut proj = Project::default();
        proj.materials.clear();

        // Create a 2x2 solid PNG image
        let img = image::RgbaImage::new(2, 2);
        let mut png_bytes = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_bytes);
        img.write_to(&mut cursor, image::ImageFormat::Png).unwrap();

        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

        let mut m = Material::default();
        m.id = 10;
        m.name = "Embedded Wood".to_string();
        m.texture_base64 = Some(b64);
        proj.materials.push(m);

        let mut doc = FurnitureDocument::new(proj);
        doc.load_project_textures(&ctx, None, AppLanguage::En);

        // Should NOT be queued as missing because embedded base64 succeeded
        assert!(doc.missing_textures_queue.is_empty());
        // Should be loaded in loaded_textures
        assert!(doc.loaded_textures.contains_key(&10));
    }
}




