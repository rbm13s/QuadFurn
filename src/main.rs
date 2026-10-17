#![cfg_attr(not(test), windows_subsystem = "windows")]

mod cutter;
mod export;
mod i18n;
mod model;
mod snapping;
mod ui_panels;
mod viewport;

use crate::i18n::AppLanguage;
use crate::model::{EdgeId, MaterialId, PartId, Project};
use crate::ui_panels::{
    copy_selected_parts_to_clipboard, paste_parts_into_project, resolve_and_load_material_texture,
    ui_cutting_layout_tab, ui_materials_manager_tab, ui_right_inspector, ui_specification_tab,
    ui_top_coreldraw_toolbar, AppTab, ClipboardPart, CLIPBOARD_PREFIX,
};
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

/// Настройки программы (сохраняются на диске в `%APPDATA%\QuadFurn\settings.json`)
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
    /// Возвращает кроссплатформенный путь к файлу настроек программы:
    /// - Windows: `%APPDATA%\QuadFurn\settings.json`
    /// - macOS: `~/Library/Application Support/QuadFurn/settings.json`
    /// - Linux: `$XDG_CONFIG_HOME/QuadFurn/settings.json` или `~/.config/QuadFurn/settings.json`
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
                // Если файл настроек был создан до появления поля `language`, определяем системный язык и сохраняем
                if !txt.contains("\"language\"") {
                    cfg.language = AppLanguage::detect_system_or_en();
                    let _ = cfg.save();
                }
                return cfg;
            }
        }
        // Первый запуск приложения: определяем текущий язык ОС (или English, если язык не поддерживается)
        // и сразу сохраняем в тот же файл settings.json, где хранится цветовая схема.
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
            .map_err(|e| format!("Ошибка сериализации настроек: {}", e))?;
        std::fs::write(&path, json)
            .map_err(|e| format!("Ошибка записи настроек ({}): {}", path.display(), e))?;
        Ok(path)
    }
}

/// Отдельная вкладка с объектом мебели (позволяет работать сразу с несколькими изделиями и копировать детали между ними)
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
            new_group_name_buf: String::new(),
            selected_sheet_idx: 0,
            pending_delete_material: None,
            pending_delete_edge: None,
            renaming_tab: false,
        }
    }

    /// Проверяет, есть ли несохранённые изменения в проекте
    pub fn is_dirty(&self) -> bool {
        serde_json::to_string(&self.project).ok().as_deref() != Some(self.saved_json.as_str())
    }

    /// Помечает текущее состояние проекта как сохранённое
    pub fn mark_saved(&mut self, path: Option<std::path::PathBuf>) {
        self.saved_json = serde_json::to_string(&self.project).unwrap_or_default();
        if path.is_some() {
            self.file_path = path;
        }
    }

    /// Подготавливает пути и данные текстур перед сохранением:
    /// - Вариант A: вычисляет относительные пути к файлам текстур относительно каталога сохранения проекта
    /// - Вариант B: если включен embed_textures, упаковывает байты текстур в Base64 для портативности
    pub fn prepare_textures_for_saving(&mut self, target_save_path: &std::path::Path) {
        let save_dir = target_save_path
            .parent()
            .unwrap_or(std::path::Path::new("."));
        for m in &mut self.project.materials {
            // Вариант A: вычисляем относительный путь
            if let Some(ref p) = m.texture_path {
                if let Ok(rel) = p.strip_prefix(save_dir) {
                    m.texture_relative_path = Some(rel.to_string_lossy().replace('\\', "/"));
                } else if let Some(file_name) = p.file_name() {
                    m.texture_relative_path = Some(file_name.to_string_lossy().to_string());
                }
            }

            // Вариант B: упаковка в Base64
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

    /// Сохраняет проект через диалог выбора файла (или в текущий файл). Возвращает путь при успешном сохранении.
    pub fn save_interactive(&mut self, status_msg: &mut String) -> Option<std::path::PathBuf> {
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
                            format!("Изделие сохранено (с упакованными текстурами) / Saved (embedded textures): {}", path.display())
                        } else {
                            format!("Изделие сохранено / Saved: {}", path.display())
                        };
                        self.mark_saved(Some(path.clone()));
                        return Some(path);
                    }
                    Err(e) => {
                        *status_msg = format!("Ошибка сохранения / Save error: {}", e);
                    }
                }
            }
        }
        None
    }
}

pub struct QuadFurnApp {
    /// Список открытых вкладок объектов мебели (если пуст — отображается Стартовая страница)
    pub documents: Vec<FurnitureDocument>,
    /// Индекс активной вкладки мебели
    pub active_doc_idx: usize,
    /// Поле ввода имени нового изделия на стартовой странице
    pub new_product_name_buf: String,
    /// Список последних открытых проектов (до 10 штук), сохраняемый в настройках
    pub recent_projects: Vec<std::path::PathBuf>,
    /// Модальное подтверждение закрытия вкладки мебели: Option<usize>
    pub pending_close_doc_idx: Option<usize>,
    /// Показывать ли модальное окно сохранения перед выходом из программы
    pub show_exit_confirm_dialog: bool,
    /// Флаг разрешения закрытия окна после выбора пользователя в диалоге выхода
    pub allow_exit: bool,
    /// Общий буфер обмена для копирования одной или нескольких выделенных деталей (Ctrl+C / Ctrl+V) между разными вкладками мебели
    pub clipboard_parts: Vec<ClipboardPart>,
    /// Текстура иконки приложения (из `assets/icon.jpeg`)
    pub app_icon_tex: Option<TextureHandle>,
    /// Режим просмотра (Конструктор / Спецификация / Карта раскроя / Материалы)
    pub active_tab: AppTab,
    pub theme_choice: ThemeChoice,
    pub language: AppLanguage,
    pub status_msg: String,
    /// Счётчик стартовых кадров для гарантированного разворачивания окна на весь экран в Windows
    pub startup_maximize_frames: u8,
    pub screenshot_target: Option<std::path::PathBuf>,
    pub screenshot_frame_delay: u8,
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

    /// Сохраняет текущие настройки (тему оформления, язык и список последних 10 проектов) в `settings.json`
    pub fn save_settings(&self) -> Result<std::path::PathBuf, String> {
        let cfg = AppSettings {
            theme_choice: self.theme_choice,
            language: self.language,
            recent_projects: self.recent_projects.clone(),
        };
        cfg.save()
    }

    /// Добавляет путь проекта в начало списка последних открытых проектов (максимум 10 штук) и сохраняет настройки
    pub fn record_recent_project(&mut self, path: std::path::PathBuf) {
        self.recent_projects.retain(|p| p != &path);
        self.recent_projects.insert(0, path);
        self.recent_projects.truncate(10);
        let _ = self.save_settings();
    }

    /// Открывает проект по указанному пути и добавляет его в список последних открытых проектов (до 10 шт.)
    pub fn open_project_from_path(&mut self, ctx: &egui::Context, path: &std::path::Path) {
        match std::fs::read_to_string(path) {
            Ok(txt) => match serde_json::from_str::<Project>(&txt) {
                Ok(proj) => {
                    let mut new_doc = FurnitureDocument::new(proj);
                    new_doc.mark_saved(Some(path.to_path_buf()));
                    for m in &mut new_doc.project.materials {
                        if m.texture_path.is_some()
                            || m.texture_base64.is_some()
                            || m.texture_relative_path.is_some()
                        {
                            if let Ok(info) = resolve_and_load_material_texture(
                                ctx,
                                m,
                                Some(path),
                                self.language,
                            ) {
                                new_doc.loaded_textures.insert(m.id, info.handle);
                                if let Some(w) = info.warning {
                                    new_doc.texture_warnings.insert(m.id, w);
                                }
                            }
                        }
                    }
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

    /// Открывает сохранённое изделие (.qfurn / .json) через системный диалог выбора файла
    pub fn open_project_interactive(&mut self, ctx: &egui::Context) {
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
                                    let proj = Project::new_empty(prod_name.clone());
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
                                    let mut proj = Project::new_demo_cabinet();
                                    let trimmed = self.new_product_name_buf.trim();
                                    if !trimmed.is_empty()
                                        && trimmed != "Новое изделие 1"
                                        && trimmed != "New Product 1"
                                    {
                                        proj.name = prod_name;
                                    } else {
                                        proj.name = lang
                                            .tr("Тумба 800х500х720", "Cabinet 800x500x720")
                                            .to_string();
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

        // Ctrl+S / Cmd+S -> Быстрое сохранение текущего проекта
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(Key::S)) {
            let saved_path =
                self.documents[self.active_doc_idx].save_interactive(&mut self.status_msg);
            if let Some(path) = saved_path {
                self.record_recent_project(path);
            }
            return;
        }

        let lang = self.language;
        let doc = &mut self.documents[self.active_doc_idx];

        // Ctrl+Z / Cmd+Z -> Отмена (Undo) в текущей вкладке мебели
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

        // Ctrl+C / Cmd+C -> Скопировать выделенную деталь или группу выделенных деталей в общий буфер обмена (для вставки в любую вкладку или окно мебели)
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

        // Ctrl+V / Cmd+V -> Вставить деталь или группу деталей из буфера обмена в текущую вкладку мебели
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

        // Ctrl+D / Cmd+D -> Дублировать выбранную деталь или группу деталей внутри текущей вкладки
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

        // Delete / Backspace -> Удалить выбранную деталь или группу деталей
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

        // Горячие клавиши переключения инструментов (V, B, M — только без Ctrl!)
        if ctx.input(|i| !i.modifiers.ctrl && i.key_pressed(Key::V)) {
            doc.viewport_ctrl.tool = EditorTool::SelectAndMove;
        }
        if ctx.input(|i| !i.modifiers.ctrl && i.key_pressed(Key::B)) {
            doc.viewport_ctrl.tool = EditorTool::CreateBlock;
        }
        if ctx.input(|i| !i.modifiers.ctrl && i.key_pressed(Key::M)) {
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

        // Автоматическое снятие скриншота окна через egui::ViewportCommand::Screenshot
        for event in ctx.input(|i| i.raw.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event {
                if let Some(path) = self.screenshot_target.take() {
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

        if self.screenshot_target.is_some() {
            if self.screenshot_frame_delay < 6 {
                self.screenshot_frame_delay += 1;
                ctx.request_repaint();
            } else if self.screenshot_frame_delay == 6 {
                self.screenshot_frame_delay += 1;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
            }
        }

        // Перехват закрытия окна приложения: проверяем, есть ли несохранённые изменения
        if ctx.input(|i| i.viewport().close_requested()) {
            // Всегда освобождаем курсор мыши, если он был захвачен в 3D Noclip
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

        // 1. Верхняя полоса меню и переключения главных режимов
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
                        doc.save_interactive(&mut self.status_msg)
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

                    // Переключатель вкладок режимов (без лишних подписей в скобках)
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

                // Переключатель темы оформления: Тёмная, Светлая, Как в системе (с автосохранением в AppSettings)
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

                // Иконка смены языка рядом с кнопкой выбора темы
                let lang_menu = ui.menu_button(
                    RichText::new(self.language.switcher_button_text()).strong(),
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
                lang_menu
                    .response
                    .on_hover_text(self.language.switcher_hover_tooltip());
            });
        });

        let lang = self.language;
        let d = lang.dict();

        // 2. Панель вкладок объектов мебели (отображается, когда открыто хотя бы одно изделие)
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
                            // При переключении вкладки отключаем захват мыши Noclip в старой вкладке
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
                        let p = Project::new_empty(format!(
                            "{} {}",
                            lang.tr("Изделие", "Product"),
                            self.documents.len() + 1
                        ));
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
                        let mut p = Project::new_demo_cabinet();
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

        // Диалог подтверждения закрытия вкладки мебели
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
                        self.documents[close_idx].save_interactive(&mut self.status_msg);
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

        // Диалог подтверждения сохранения несохранённого проекта при выходе из программы
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
                            if let Some(p) = d.save_interactive(&mut self.status_msg) {
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

        // Если ни одно изделие ещё не открыто — показываем нижнюю строку состояния и Стартовую страницу
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

        // Если мы ушли с вкладки Конструктора, гарантированно выключаем захват мыши Noclip
        if self.active_tab != AppTab::Designer4View && doc.viewport_ctrl.cam_3d.noclip_active {
            doc.viewport_ctrl.cam_3d.noclip_active = false;
            QuadViewportController::set_noclip_cursor_grab(ctx, false, egui::Pos2::ZERO);
        }

        // Нижняя строка состояния
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
                // Верхняя панель точного ввода размеров и координат CorelDRAW-style
                egui::TopBottomPanel::top("coreldraw_transform_bar").show(ctx, |ui| {
                    let undo_ref = &mut doc.undo_stack;
                    ui_top_coreldraw_toolbar(
                        ui,
                        &mut doc.project,
                        &mut doc.selected_part,
                        &mut doc.active_material_id,
                        &mut doc.viewport_ctrl,
                        |proj| Self::push_undo(undo_ref, proj),
                    );
                });

                // Правый инспектор блоков, групп, материалов и кромок
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

                // Центральная область: 4 экрана (Top XY, 3D Orbit/Z-Noclip, Front XZ, Side YZ)
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

/// Настраивает шрифты интерфейса: подключает системные шрифты Windows, Linux и macOS
/// для отображения иконок, технических символов (Δ, ×, ∠, ↺), кириллицы и CJK (简体中文, 日本語, 한국어).
pub fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 1. Шрифт эмодзи Windows (Segoe UI Emoji — содержит монохромные glyf-контуры для всех современных иконок, включая 🪚 и 🌐)
    if let Ok(bytes) = std::fs::read("C:\\Windows\\Fonts\\seguiemj.ttf") {
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

    // 2. Шрифт технических символов Windows (Segoe UI Symbol — векторные значки 🗄, 🗑, 👁, 🖥, 📁, 💾, 🔍, 🔒 и стрелки в едином стиле)
    if let Ok(bytes) = std::fs::read("C:\\Windows\\Fonts\\seguisym.ttf") {
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

    // 3. Основной пропорциональный шрифт интерфейса (Windows: Segoe UI, macOS: SF / Helvetica / Arial, Linux: DejaVu Sans / Liberation Sans / Noto Sans)
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
        if let Ok(raw) = std::fs::read(path) {
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

    // 4. Моноширинный шрифт (Windows: Consolas, macOS: Menlo / Courier New, Linux: DejaVu Sans Mono)
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
        if let Ok(raw) = std::fs::read(path) {
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

    // 5. Fallback-шрифты символов для Linux / macOS (Apple Symbols, Noto Sans Symbols2, Symbola)
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
        if let Ok(raw) = std::fs::read(path) {
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

    // 6. Поддержка CJK (корейский 한국어, китайский упрощённый 简体中文, японский 日本語) на Windows, macOS и Linux
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
        if let Ok(raw) = std::fs::read(path) {
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

/// Загружает иконку приложения из `assets/icon.jpeg` для заголовка окна и панели задач
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

/// Загружает текстуру иконки приложения для отображения на стартовой странице и в верхней панели
pub fn load_app_icon_texture(ctx: &egui::Context) -> Option<TextureHandle> {
    let dyn_img = image::load_from_memory(APP_ICON_JPEG).ok()?;
    let resized = dyn_img.resize_exact(256, 256, image::imageops::FilterType::Lanczos3);
    let rgba = resized.to_rgba8();
    let dims = [rgba.width() as usize, rgba.height() as usize];
    let color_img =
        egui::ColorImage::from_rgba_unmultiplied(dims, rgba.as_flat_samples().as_slice());
    Some(ctx.load_texture("quadfurn_app_icon", color_img, egui::TextureOptions::LINEAR))
}

fn main() -> eframe::Result<()> {
    let initial_settings = AppSettings::load();
    // Задаём базовый (restored) размер окна 1520×920 без одновременного .with_maximized(true),
    // так как в winit на Windows одновременная передача with_maximized(true) + with_inner_size(...)
    // вызывает SetWindowPos поверх уже установленного WS_MAXIMIZE (окно остаётся 1520×920 со значком ❐).
    // Настоящее разворачивание SW_MAXIMIZE на весь экран выполняется сразу при старте первого кадра.
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

            let mut args = std::env::args().skip(1);
            while let Some(arg) = args.next() {
                if arg == "--demo" {
                    app.documents.push(FurnitureDocument::new(Project::new_demo_cabinet()));
                    app.active_doc_idx = app.documents.len() - 1;
                    app.active_tab = AppTab::Designer4View;
                } else if arg == "--materials" {
                    app.documents.push(FurnitureDocument::new(Project::new_demo_cabinet()));
                    app.active_doc_idx = app.documents.len() - 1;
                    app.active_tab = AppTab::MaterialsManager;
                } else if arg == "--embed" {
                    if let Some(doc) = app.documents.last_mut() {
                        doc.project.embed_textures = true;
                    }
                } else if arg == "--lang" {
                    if let Some(l) = args.next() {
                        if l.eq_ignore_ascii_case("ru") {
                            app.language = AppLanguage::Ru;
                        } else if l.eq_ignore_ascii_case("en") {
                            app.language = AppLanguage::En;
                        }
                    }
                } else if arg == "--screenshot" {
                    if let Some(target) = args.next() {
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



