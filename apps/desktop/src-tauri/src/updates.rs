use std::time::Duration;
use url::Url;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
const MANIFEST: &str = "https://github.com/Yuelioi/bdl/releases/latest/download/latest.json";

// The URL template is transport configuration; the embedded updater public key
// remains the authority for installation, including with a user-supplied mirror.
fn mirrored(template: &str, original: &str) -> Result<Url, String> {
    let upstream = Url::parse(original).map_err(|e| e.to_string())?;
    if upstream.host_str() != Some("github.com")
        || !upstream.path().starts_with("/Yuelioi/bdl/releases/")
        || upstream.scheme() != "https"
        || !upstream.username().is_empty()
        || upstream.password().is_some()
        || upstream.port().is_some()
        || upstream.fragment().is_some()
    {
        return Err("更新清单包含非 BDL 官方下载地址。".into());
    }
    let value = if template.contains("{url}") {
        template.replace("{url}", original)
    } else if template.contains("{path}") {
        template.replace("{path}", original.trim_start_matches("https://"))
    } else {
        format!("{}/{original}", template.trim_end_matches('/'))
    };
    let url = Url::parse(&value).map_err(|_| "加速地址无效。")?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err("加速地址须为不含账号和片段的 HTTPS 地址。".into());
    }
    Ok(url)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadMirror {
    template: String,
    official_url: String,
}

fn download_url(value: &str, mirror: Option<&DownloadMirror>) -> Result<Url, String> {
    let url = Url::parse(value).map_err(|_| "更新下载地址无效。")?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.fragment().is_some()
    {
        return Err("更新下载地址须为不含账号和片段的 HTTPS 地址。".into());
    }
    if let Some(mirror) = mirror {
        if mirrored(mirror.template.trim(), &mirror.official_url)? == url {
            return Ok(url);
        }
    } else if (url.host_str() == Some("github.com")
        && url.path().starts_with("/Yuelioi/bdl/releases/"))
        || (url.host_str() == Some("pan.quark.cn")
            && url.path().starts_with("/s/")
            && url.path().len() > 3)
    {
        return Ok(url);
    }
    Err("此地址不是已验证的 BDL 更新下载来源。".into())
}

#[tauri::command]
pub fn update_open_download(
    app: tauri::AppHandle,
    url: String,
    mirror: Option<DownloadMirror>,
) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let url = download_url(&url, mirror.as_ref())?;
    app.opener()
        .open_url(url.as_str(), None::<&str>)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn update_distribution(
    state: tauri::State<'_, bdl_tauri::state::AppState>,
) -> Result<serde_json::Value, String> {
    let settings = state.settings().map_err(|e| e.to_string())?;
    let platform = std::env::consts::OS;
    let arch = match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        other => other,
    };
    settings
        .proxy_mode
        .configure_client(reqwest::Client::builder(), settings.proxy_url.as_deref())
        .map_err(|e| e.to_string())?
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?
        .get("https://apps.yuelili.com/api/v1/software/bdl/download-config")
        .query(&[
            ("platform", platform),
            ("architecture", arch),
            ("channel", "stable"),
        ])
        .send()
        .await
        .map_err(|e| format!("软件站连接失败：{e}"))?
        .error_for_status()
        .map_err(|e| format!("软件站检测失败：{e}"))?
        .json()
        .await
        .map_err(|e| format!("软件站响应无效：{e}"))
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[tauri::command]
pub async fn update_check(
    webview: tauri::Webview,
    state: tauri::State<'_, bdl_tauri::state::AppState>,
    template: Option<String>,
) -> Result<Option<serde_json::Value>, String> {
    use tauri::Manager;
    use tauri_plugin_updater::UpdaterExt;
    let settings = state.settings().map_err(|e| e.to_string())?;
    let endpoint = match template.as_deref() {
        Some(template) => mirrored(template.trim(), MANIFEST)?,
        None => Url::parse(MANIFEST).map_err(|e| e.to_string())?,
    };
    let mut builder = webview
        .updater_builder()
        .timeout(Duration::from_secs(20))
        .endpoints(vec![endpoint])
        .map_err(|e| e.to_string())?;
    builder = match settings.proxy_mode {
        bdl_core::settings::ProxyMode::System => builder,
        bdl_core::settings::ProxyMode::Direct => builder.no_proxy(),
        bdl_core::settings::ProxyMode::Custom => {
            let proxy = settings
                .proxy_url
                .as_deref()
                .filter(|url| !url.trim().is_empty())
                .ok_or("手动设置代理时，请填写代理地址。")?;
            builder.proxy(Url::parse(proxy).map_err(|_| "代理地址无效。")?)
        }
    };
    let result = builder
        .build()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| e.to_string())?;
    let Some(mut update) = result else {
        return Ok(None);
    };
    if let Some(template) = template {
        update.download_url = mirrored(template.trim(), update.download_url.as_str())?;
    }
    let mut metadata = serde_json::json!({
        "currentVersion": update.current_version, "version": update.version,
        "body": update.body, "rawJson": update.raw_json,
    });
    metadata["rid"] = serde_json::json!(webview.resources_table().add(update));
    Ok(Some(metadata))
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[tauri::command]
pub async fn update_check() -> Result<Option<serde_json::Value>, String> {
    Err("移动端请下载安装包后手动安装。".into())
}

#[cfg(all(test, not(any(target_os = "android", target_os = "ios"))))]
mod tests {
    use super::*;
    #[test]
    fn mirror_templates_cover_manifest_and_asset() {
        for original in [
            MANIFEST,
            "https://github.com/Yuelioi/bdl/releases/download/v1.0.0/BDL.tar.gz",
        ] {
            assert_eq!(
                mirrored("https://mirror.test/", original).unwrap().as_str(),
                format!("https://mirror.test/{original}")
            );
            assert_eq!(
                mirrored("https://mirror.test/{path}", original)
                    .unwrap()
                    .as_str(),
                format!(
                    "https://mirror.test/{}",
                    original.trim_start_matches("https://")
                )
            );
        }
        assert!(mirrored("http://mirror.test", MANIFEST).is_err());
        // The same transport that builds a download URL must be accepted by
        // the native update opener, without expanding generic link permissions.
        let official = "https://github.com/Yuelioi/bdl/releases/download/v1.0.0/BDL.exe";
        let context: DownloadMirror = serde_json::from_value(
            serde_json::json!({ "template": "https://custom.test/{url}", "officialUrl": official }),
        )
        .unwrap();
        let generated = mirrored(&context.template, official).unwrap();
        assert_eq!(
            download_url(generated.as_str(), Some(&context)).unwrap(),
            generated
        );
        assert!(download_url(official, None).is_ok());
        assert!(download_url("https://pan.quark.cn/s/test", None).is_ok());
        for rejected in [
            "https://arbitrary.test/file.exe",
            "https://pan.quark.cn.evil.test/s/test",
            "https://pan.quark.cn/other",
            "https://user@pan.quark.cn/s/test",
            "http://pan.quark.cn/s/test",
        ] {
            assert!(download_url(rejected, None).is_err());
        }
        assert!(download_url("https://custom.test/unrelated.exe", Some(&context)).is_err());
        assert!(
            mirrored(
                "https://mirror.test",
                "https://github.com/other/app/releases/download/v1/a"
            )
            .is_err()
        );
    }
}
