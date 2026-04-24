use tauri::command;
use tauri::AppHandle;
use tauri::Manager;
use crate::input::{self, paste, hotkey};
use crate::errors::AppError;
use crate::{
    config, is_single_key_hotkey, register_pipeline_shortcut, unregister_pipeline_shortcut,
    CurrentShortcut,
};

#[command]
pub async fn paste_text(text: String) -> Result<(), AppError> {
    tokio::task::spawn_blocking(move || {
        paste::paste_text(&text)
    }).await.map_err(|e| AppError::Input(e.to_string()))?
}

#[command]
pub fn parse_hotkey_string(hotkey_str: String) -> Result<(Vec<String>, String), AppError> {
    hotkey::parse_hotkey(&hotkey_str)
}

#[command]
pub fn get_tauri_shortcut(hotkey_str: String) -> Result<String, AppError> {
    hotkey::to_tauri_shortcut(&hotkey_str)
}

#[command]
pub fn update_hotkey(app: AppHandle, hotkey_str: String) -> Result<(), AppError> {
    if !is_single_key_hotkey(&hotkey_str) {
        hotkey::to_tauri_shortcut(&hotkey_str)?;
    }

    let current_shortcut = app.state::<CurrentShortcut>();
    let old_hotkey = {
        let guard = current_shortcut.lock().map_err(|e| {
            AppError::Input(format!("CurrentShortcut mutex poisoned: {}", e))
        })?;
        guard.clone()
    };

    if old_hotkey.eq_ignore_ascii_case(&hotkey_str) {
        config::update_field("hotkey", &hotkey_str)?;
        return Ok(());
    }

    let _ = unregister_pipeline_shortcut(&app, &old_hotkey);

    if let Err(e) = register_pipeline_shortcut(&app, &hotkey_str) {
        let _ = register_pipeline_shortcut(&app, &old_hotkey);
        return Err(e);
    }

    config::update_field("hotkey", &hotkey_str)?;

    {
        let mut guard = current_shortcut.lock().map_err(|e| {
            AppError::Input(format!("CurrentShortcut mutex poisoned: {}", e))
        })?;
        *guard = hotkey_str;
    }

    Ok(())
}

#[command]
pub fn unregister_hotkey(app: AppHandle) -> Result<(), AppError> {
    let current_shortcut = app.state::<CurrentShortcut>();
    let shortcut = {
        let guard = current_shortcut.lock().map_err(|e| {
            AppError::Input(format!("CurrentShortcut mutex poisoned: {}", e))
        })?;
        guard.clone()
    };
    unregister_pipeline_shortcut(&app, &shortcut)
}

#[command]
pub fn reregister_hotkey(app: AppHandle) -> Result<(), AppError> {
    let current_shortcut = app.state::<CurrentShortcut>();
    let shortcut = {
        let guard = current_shortcut.lock().map_err(|e| {
            AppError::Input(format!("CurrentShortcut mutex poisoned: {}", e))
        })?;
        guard.clone()
    };
    register_pipeline_shortcut(&app, &shortcut)
}

#[command]
pub fn check_accessibility_permission() -> bool {
    input::check_accessibility()
}

#[command]
pub fn request_accessibility_permission() -> bool {
    input::request_accessibility()
}

#[command]
pub fn open_accessibility_settings() {
    input::open_accessibility_settings();
}

#[command]
pub fn check_microphone_permission() -> String {
    input::check_microphone_permission()
}

#[command]
pub fn request_microphone_permission() {
    input::request_microphone_permission()
}

#[command]
pub fn open_microphone_settings() {
    input::open_microphone_settings()
}
