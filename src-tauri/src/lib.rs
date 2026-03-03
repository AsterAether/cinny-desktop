#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// mod menu;

#[cfg(target_os = "android")]
mod notification_channels;

#[cfg(target_os = "android")]
mod notification_state;

#[cfg(target_os = "android")]
mod notifications;

#[cfg(target_os = "android")]
mod unifiedpush;

use tauri::Manager;

#[cfg(target_os = "android")]
fn inject_notification_polyfill(window: &tauri::WebviewWindow) -> Result<(), String> {
    let polyfill_script = include_str!("../gen/android/notification-polyfill.js");

    window.eval(polyfill_script)
        .map_err(|e| format!("Failed to inject notification polyfill: {}", e))?;

    println!("Notification polyfill injected successfully");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let port: u16 = 44548;
    let context = tauri::generate_context!();
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_window_state::Builder::default().build());
    }

    #[cfg(target_os = "android")]
    {
        builder = builder
            .plugin(tauri_plugin_notification::init())
            .manage(notifications::NotificationManager::new())
            .invoke_handler(tauri::generate_handler![
                notifications::request_notification_permission,
                notifications::get_notification_permission,
                notifications::show_notification,
                notifications::close_notification,
                unifiedpush::register_unifiedpush,
                unifiedpush::unregister_unifiedpush,
            ]);
    }

    builder
        .setup(|app| {
            use tauri::webview::WebviewWindowBuilder;

            // On Android, create window with polyfill
            #[cfg(target_os = "android")]
            {
                let polyfill_script = include_str!("../gen/android/notification-polyfill.js");

                let _window = WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("index.html".into()))
                    .initialization_script(polyfill_script)
                    .build()?;

                println!("Android window created with polyfill");
            }

            // On desktop, create window without polyfill
            #[cfg(not(target_os = "android"))]
            {
                let _window = WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("index.html".into()))
                    .build()?;
            }

            Ok(())
        })
        .run(context)
        .expect("error while building tauri application");
}
