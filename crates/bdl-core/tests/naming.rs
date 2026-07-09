use std::collections::HashSet;
use std::path::PathBuf;

use bdl_core::error::BdlError;
use bdl_core::naming::{NamingContext, render_output_path, sanitize_path_component, unique_path};

#[test]
fn render_output_path_replaces_known_variables() -> Result<(), BdlError> {
    let context = NamingContext {
        title: "Fixture",
        part_title: "Opening",
        part_index: 2,
        bvid: Some("BV1xx411c7mD"),
        cid: Some(9001),
        ext: "mp4",
        ..NamingContext::default()
    };

    let path = render_output_path(
        "{title}/P{part_index}-{part_title}-{bvid}-{cid}.{ext}",
        &context,
    )?;

    assert_eq!(
        path,
        PathBuf::from("Fixture").join("P2-Opening-BV1xx411c7mD-9001.mp4")
    );
    Ok(())
}

#[test]
fn sanitize_path_component_replaces_windows_invalid_characters_and_trims() {
    assert_eq!(sanitize_path_component(" bad:name*?. "), "bad_name__");
    assert_eq!(sanitize_path_component("..."), "untitled");
}

#[test]
fn render_output_path_rejects_unknown_variables() {
    let context = NamingContext {
        title: "Fixture",
        part_title: "Opening",
        part_index: 1,
        ext: "mp4",
        ..NamingContext::default()
    };

    let error =
        render_output_path("{unknown}.{ext}", &context).expect_err("unknown variable should fail");

    assert!(error.to_string().contains("未知变量"));
}

#[test]
fn unique_path_adds_counter_for_reserved_paths() {
    let mut reserved = HashSet::new();
    let first = unique_path(PathBuf::from("downloads/video.mp4"), &mut reserved);
    let second = unique_path(PathBuf::from("downloads/video.mp4"), &mut reserved);

    assert_eq!(first, PathBuf::from("downloads/video.mp4"));
    assert_eq!(second, PathBuf::from("downloads/video (1).mp4"));
}
