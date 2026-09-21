use bdl_core::{BdlError, BdlResult};
use bdl_tauri::secure_store::{CredentialBackend, SecureStore};
use serde::{Deserialize, Serialize};
use tauri::{
    Manager, Runtime,
    plugin::{Builder, PluginHandle, TauriPlugin},
};

const PLUGIN_IDENTIFIER: &str = "com.yueli.bdl.credentials";

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::<R>::new("bdl-mobile-credentials")
        .setup(|app, api| {
            let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "CredentialPlugin")?;
            app.manage(SecureStore::from_backend(AndroidCredentialBackend {
                handle,
            }));
            Ok(())
        })
        .build()
}

struct AndroidCredentialBackend<R: Runtime> {
    handle: PluginHandle<R>,
}

#[derive(Debug, Deserialize)]
struct LoadCookieResponse {
    cookie: Option<String>,
}

#[derive(Serialize)]
struct SaveCookiePayload<'a> {
    cookie: &'a str,
}

impl<R: Runtime> CredentialBackend for AndroidCredentialBackend<R> {
    fn load_cookie(&self) -> BdlResult<Option<String>> {
        self.handle
            .run_mobile_plugin::<LoadCookieResponse>("loadCookie", ())
            .map(|response| response.cookie)
            .map_err(|_| credential_error("读取账户凭据"))
    }

    fn save_cookie(&self, cookie: &str) -> BdlResult<()> {
        self.handle
            .run_mobile_plugin::<serde_json::Value>("saveCookie", SaveCookiePayload { cookie })
            .map(|_| ())
            .map_err(|_| credential_error("保存账户凭据"))
    }

    fn clear_cookie(&self) -> BdlResult<()> {
        self.handle
            .run_mobile_plugin::<serde_json::Value>("clearCookie", ())
            .map(|_| ())
            .map_err(|_| credential_error("删除账户凭据"))
    }
}

fn credential_error(action: &str) -> BdlError {
    BdlError::Account {
        message: format!("{action}失败，请重试。"),
    }
}
