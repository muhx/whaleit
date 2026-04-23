#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::error;
use tauri::{Emitter, Manager};
use tauri_plugin_deep_link::DeepLinkExt;

#[cfg(desktop)]
mod menu;
#[cfg(desktop)]
mod updater;

fn emit_app_ready(handle: &tauri::AppHandle) {
    let _ = handle.emit("app-ready", ());
}

#[cfg(desktop)]
fn setup_menu(handle: &tauri::AppHandle) {
    if let Ok(menu) = menu::create_menu(handle) {
        if let Err(e) = handle.set_menu(menu) {
            error!("Failed to set menu: {}", e);
        }
    }
    handle.on_menu_event(|app, event| {
        menu::handle_menu_event(app, event.id().as_ref());
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();

    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.unminimize();
            let _ = window.set_focus();
        }
    }));

    let builder = builder
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    log::LevelFilter::Info
                })
                .build(),
        )
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_deep_link::init());

    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_window_state::Builder::default().build());

    builder
        .setup(|app| {
            let handle = app.handle().clone();

            #[cfg(desktop)]
            {
                let _ = handle.plugin(tauri_plugin_updater::Builder::new().build());
            }

            #[cfg(any(target_os = "android", target_os = "ios"))]
            {
                let _ = handle.plugin(tauri_plugin_haptics::init());
                let _ = handle.plugin(tauri_plugin_barcode_scanner::init());
            }

            #[cfg(target_os = "ios")]
            {
                let _ = handle.plugin(tauri_plugin_web_auth::init());
                let _ = handle.plugin(tauri_plugin_mobile_share::init());
            }

            #[cfg(desktop)]
            setup_menu(&handle);

            let deep_link_handle = handle.clone();
            app.deep_link().on_open_url(move |event| {
                for url in event.urls() {
                    let _ = deep_link_handle.emit("deep-link-received", url.to_string());
                }
            });

            emit_app_ready(&handle);

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Failed to build WhaleIt application")
        .run(|_handle, _event| {});
}
