use bdl_core::naming::DuplicateNamingStrategy;
use bdl_core::planner::ArchiveAssetSelection;
use bdl_core::settings::AppSettings;

#[test]
fn settings_missing_fields_recursively_use_current_defaults() {
    fn check(defaults: &serde_json::Value, path: &mut Vec<String>, object: &serde_json::Value) {
        for (key, value) in object.as_object().unwrap() {
            path.push(key.clone());
            let mut partial = defaults.clone();
            let mut parent = &mut partial;
            for segment in &path[..path.len() - 1] {
                parent = parent.get_mut(segment).unwrap();
            }
            parent.as_object_mut().unwrap().remove(key);
            let restored: AppSettings = serde_json::from_value(partial).unwrap();
            assert_eq!(
                serde_json::to_value(restored.normalized()).unwrap(),
                *defaults,
                "missing {}",
                path.join(".")
            );
            if value.is_object() {
                check(defaults, path, value);
            }
            path.pop();
        }
    }
    let defaults = serde_json::to_value(AppSettings::default()).unwrap();
    check(&defaults, &mut Vec::new(), &defaults);
}

#[test]
fn settings_partial_config_preserves_choices_through_save_and_reload() {
    let saved = serde_json::json!({
        "download_dir": "D:/Videos",
        "naming_template": "{title}.{ext}",
        "quality": "80", "codec": "hevc", "audio_quality": "30280",
        "retry_count": 0, "segment_count": 1,
        "auto_refresh_expired_urls": false,
        "global_speed_limit_bytes_per_second": null,
        "archive_assets": { "cover": false },
        "parse_rules": { "interval_seconds": 2 },
        "media_preferences": { "video": [], "fallback": "error" },
        "future_unknown_option": true
    });
    let settings: AppSettings = serde_json::from_value(saved.clone()).unwrap();
    let settings = settings.normalized();
    settings.validate().unwrap();
    let complete = serde_json::to_value(&settings).unwrap();
    for (key, value) in saved.as_object().unwrap() {
        if key == "future_unknown_option" {
            continue;
        }
        if let Some(fields) = value.as_object() {
            for (field, value) in fields {
                assert_eq!(&complete[key][field], value);
            }
        } else {
            assert_eq!(&complete[key], value);
        }
    }
    assert!(settings.archive_assets.subtitles);
    assert_eq!(
        settings.parse_rules.rest_seconds,
        AppSettings::default().parse_rules.rest_seconds
    );
    let reloaded: AppSettings = serde_json::from_value(complete).unwrap();
    assert_eq!(reloaded.normalized(), settings);
}

#[test]
fn settings_normalization_preserves_existing_valid_title_template() {
    let settings = AppSettings {
        naming_template: "{title}/{title} - P{part_index} - {part_title}.{ext}".to_owned(),
        ..AppSettings::default()
    };

    assert_eq!(
        settings.normalized().naming_template,
        "{title}/{title} - P{part_index} - {part_title}.{ext}"
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
        DuplicateNamingStrategy::SkipExisting
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
    assert_eq!(settings.segment_count, 4);
    assert_eq!(settings.global_speed_limit_bytes_per_second, None);
    assert!(!settings.startup_auto_recovery);
}

#[test]
fn settings_normalization_preserves_explicit_single_segment_without_version() {
    let legacy: AppSettings =
        serde_json::from_str(r#"{"segment_count":1,"naming_template":"{title}.{ext}"}"#)
            .expect("legacy settings should deserialize");
    let migrated = legacy.normalized();

    assert_eq!(migrated.settings_schema_version, 1);
    assert_eq!(migrated.segment_count, 1);

    let explicit_single_segment = AppSettings {
        segment_count: 1,
        ..AppSettings::default()
    }
    .normalized();
    assert_eq!(explicit_single_segment.segment_count, 1);
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

#[test]
fn settings_validate_accepts_a_global_speed_limit() {
    let settings = AppSettings {
        global_speed_limit_bytes_per_second: Some(8 * 1024 * 1024),
        ..AppSettings::default()
    };

    settings
        .validate()
        .expect("a practical global speed limit should be valid");
}

#[test]
fn settings_validate_rejects_an_excessive_global_speed_limit() {
    let settings = AppSettings {
        global_speed_limit_bytes_per_second: Some(10 * 1024 * 1024 * 1024 + 1),
        ..AppSettings::default()
    };

    let error = settings
        .validate()
        .expect_err("an excessive global speed limit should fail");

    assert!(error.to_string().contains("全局下载限速"));
}

#[test]
fn settings_validate_rejects_cover_embedding_for_mp4() {
    let settings = AppSettings {
        output_extension: "mp4".to_owned(),
        embed_cover: true,
        ..AppSettings::default()
    };

    let error = settings
        .validate()
        .expect_err("MP4 cover embedding should fail");

    assert!(error.to_string().contains("仅支持 MKV"));
}

#[test]
fn settings_validate_accepts_subtitle_embedding_for_mkv() {
    let settings = AppSettings {
        output_extension: "mkv".to_owned(),
        embed_subtitles: true,
        ..AppSettings::default()
    };

    settings
        .validate()
        .expect("MKV subtitle embedding should be valid");
}
