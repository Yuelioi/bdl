use bdl_core::naming::DEFAULT_NAMING_TEMPLATE;
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
