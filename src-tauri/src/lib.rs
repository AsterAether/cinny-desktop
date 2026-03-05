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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let port: u16 = 44548;
    let context = tauri::generate_context!();
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_localhost::Builder::new(port).build())
            .plugin(tauri_plugin_window_state::Builder::default().build());
    }

    #[cfg(target_os = "android")]
    {
        builder = builder
            .plugin(tauri_plugin_notification::init())
            .plugin(
                tauri::plugin::Builder::new("unified-push")
                    .setup(|app, api: tauri::plugin::PluginApi<_, ()>| {
                        let handle = api.register_android_plugin("in.cinny.app", "UnifiedPushPlugin")?;
                        app.manage(unifiedpush::UnifiedPushHandle::new(handle));
                        Ok(())
                    })
                    .build()
            )
            .manage(notifications::NotificationManager::new())
            .invoke_handler(tauri::generate_handler![
                notifications::request_notification_permission,
                notifications::get_notification_permission,
                notifications::show_notification,
                notifications::close_notification,
                unifiedpush::register_unifiedpush,
                unifiedpush::unregister_unifiedpush,
                unifiedpush::get_push_endpoint,
                unifiedpush::get_push_distributors,
                unifiedpush::save_push_distributor,
                unifiedpush::get_launch_notification,
            ]);
    }

    builder
        .setup(|app| {
            use tauri::webview::WebviewWindowBuilder;
            use tauri::WebviewUrl;

            // On Android, create window with polyfill
            #[cfg(target_os = "android")]
            {
                let polyfill_script = include_str!("../gen/android/notification-polyfill.js");

                let _window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                    .initialization_script(polyfill_script)
                    .build()?;

                println!("Android window created with polyfill");
            }

            // On desktop, create window with localhost plugin
            #[cfg(not(target_os = "android"))]
            {
                let url = format!("http://localhost:{}", port).parse().unwrap();
                let window_url = WebviewUrl::External(url);
                WebviewWindowBuilder::new(app, "main".to_string(), window_url)
                    .title("Cinny")
                    .build()?;
            }

            Ok(())
        })
        .run(context)
        .expect("error while building tauri application");
}
