use serde::{Deserialize, Serialize};
use tauri::{plugin::PluginHandle, Manager, Runtime};

/// Holds the Android plugin handle for UnifiedPush operations.
/// Stored as app state after plugin initialization.
pub struct UnifiedPushHandle<R: Runtime>(pub PluginHandle<R>);

impl<R: Runtime> UnifiedPushHandle<R> {
    pub fn new(handle: PluginHandle<R>) -> Self {
        Self(handle)
    }

    pub fn register(&self) -> Result<(), String> {
        #[derive(Serialize)]
        struct Empty {}
        self.0
            .run_mobile_plugin::<serde_json::Value>("register", Empty {})
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    pub fn unregister(&self) -> Result<(), String> {
        #[derive(Serialize)]
        struct Empty {}
        self.0
            .run_mobile_plugin::<serde_json::Value>("unregister", Empty {})
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    pub fn get_endpoint(&self) -> Result<Option<String>, String> {
        #[derive(Serialize)]
        struct Empty {}
        #[derive(Deserialize)]
        struct EndpointResult {
            endpoint: Option<String>,
        }
        self.0
            .run_mobile_plugin::<EndpointResult>("getEndpoint", Empty {})
            .map(|r| r.endpoint)
            .map_err(|e| e.to_string())
    }

    pub fn get_distributors(&self) -> Result<Vec<String>, String> {
        #[derive(Serialize)]
        struct Empty {}
        self.0
            .run_mobile_plugin::<Vec<String>>("getDistributors", Empty {})
            .map_err(|e| e.to_string())
    }

    pub fn save_distributor(&self, distributor: &str) -> Result<(), String> {
        #[derive(Serialize)]
        struct Args<'a> {
            distributor: &'a str,
        }
        self.0
            .run_mobile_plugin::<serde_json::Value>("saveDistributor", Args { distributor })
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn register_unifiedpush<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        app.state::<UnifiedPushHandle<R>>().register()
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        Ok(())
    }
}

#[tauri::command]
pub async fn unregister_unifiedpush<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        app.state::<UnifiedPushHandle<R>>().unregister()
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        Ok(())
    }
}

#[tauri::command]
pub async fn get_push_endpoint<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Option<String>, String> {
    #[cfg(target_os = "android")]
    {
        app.state::<UnifiedPushHandle<R>>().get_endpoint()
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        Ok(None)
    }
}

#[tauri::command]
pub async fn get_push_distributors<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Vec<String>, String> {
    #[cfg(target_os = "android")]
    {
        app.state::<UnifiedPushHandle<R>>().get_distributors()
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        Ok(vec![])
    }
}

#[tauri::command]
pub async fn save_push_distributor<R: Runtime>(
    app: tauri::AppHandle<R>,
    distributor: String,
) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        app.state::<UnifiedPushHandle<R>>().save_distributor(&distributor)
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (app, distributor);
        Ok(())
    }
}
