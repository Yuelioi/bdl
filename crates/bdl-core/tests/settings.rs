use bdl_core::naming::{DEFAULT_NAMING_TEMPLATE, DuplicateNamingStrategy};
use bdl_core::planner::ArchiveAssetSelection;
use bdl_core::settings::AppSettings;

#[test]
fn settings_normalization_replaces_legacy_duplicate_title_template() {
    let settings = AppSettings {
        naming_template: "{title}/{title} - P{part_index} - {part_title}.{ext}".to_owned(),
        ..AppSettings::default()
    };

    assert_eq!(
        settings.normalized().naming_template,
        DEFAULT_NAMING_TEMPLATE
    );
}

#[test]
fn settings_normalization_preserves_custom_naming_template() {
    let settings = AppSettings {
        naming_template: "{title}.{ext}".to_owned(),
        ..AppSettings::default()
    };

    assert_eq!(settings.normalized().naming_template, "{title}.{ext}");
}

#[test]
fn settings_validate_rejects_unknown_naming_variables() {
    let settings = AppSettings {
        naming_template: "{unknown}.{ext}".to_owned(),
        ..AppSettings::default()
    };

    let error = settings
        .validate()
        .expect_err("unknown variable should fail");

    assert!(error.to_string().contains("未知变量"));
}

#[test]
fn settings_deserialize_old_config_defaults_duplicate_naming_strategy() {
    let settings: AppSettings = serde_json::from_str(r#"{"naming_template":"{title}.{ext}"}"#)
        .expect("old settings should deserialize");

    assert_eq!(
        settings.duplicate_naming_strategy,
        DuplicateNamingStrategy::AppendSuffix
    );
    assert_eq!(settings.audio_quality, "best");
    assert_eq!(settings.codec, "auto");
    assert_eq!(settings.missing_quality_policy, "lower");
    assert_eq!(settings.archive_assets, ArchiveAssetSelection::all());
    assert_eq!(settings.ffmpeg_path, None);
    assert!(!settings.retain_raw_streams);
    assert!(!settings.embed_cover);
    assert!(!settings.embed_subtitles);
    assert_eq!(settings.proxy_url, None);
    assert_eq!(settings.log_level, "info");
    assert_eq!(settings.data_dir, None);
    assert_eq!(settings.segment_count, 1);
    assert!(!settings.startup_auto_recovery);
}

#[test]
fn settings_validate_accepts_custom_archive_mode() {
    let settings = AppSettings {
        archive_mode: "custom".to_owned(),
        archive_assets: ArchiveAssetSelection {
            cover: true,
            subtitles: false,
            danmaku: true,
            nfo: false,
        },
        ..AppSettings::default()
    };

    settings
        .validate()
        .expect("custom archive mode should be valid");
}

#[test]
fn settings_validate_rejects_invalid_media_defaults() {
    let settings = AppSettings {
        quality: "not-a-quality".to_owned(),
        ..AppSettings::default()
    };

    let error = settings
        .validate()
        .expect_err("invalid video quality should fail");

    assert!(error.to_string().contains("视频清晰度"));
}

#[test]
fn settings_validate_rejects_invalid_proxy_url() {
    let settings = AppSettings {
        proxy_url: Some("file:///not-a-proxy".to_owned()),
        ..AppSettings::default()
    };

    let error = settings
        .validate()
        .expect_err("unsupported proxy scheme should fail");

    assert!(error.to_string().contains("代理地址协议不支持"));
}

#[test]
fn settings_validate_rejects_invalid_segment_count() {
    let settings = AppSettings {
        segment_count: 6,
        ..AppSettings::default()
    };

    let error = settings
        .validate()
        .expect_err("unsupported segment count should fail");

    assert!(error.to_string().contains("单任务分段数"));
}
