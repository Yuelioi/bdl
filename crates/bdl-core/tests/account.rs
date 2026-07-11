use bdl_core::BdlResult;
use bdl_core::account::{
    AccountLibraryFolderKind, AccountLibraryPage, AccountSummary, ImportedCookie, QrLoginStatus,
    redact_sensitive,
};
use bpi_rs::fav::info::{
    CollectedFolderItem, CollectedFolderUpper, CreatedFolderItem, CreatedFolderListData,
};
use bpi_rs::ids::Mid;
use bpi_rs::login::{LoginNav, LoginWbiImg};

#[test]
fn imported_cookie_accepts_required_bilibili_login_cookies() -> BdlResult<()> {
    let cookie = ImportedCookie::parse(
        "DedeUserID=42; SESSDATA=session-value; bili_jct=csrf-value; buvid3=device",
    )?;

    assert_eq!(cookie.dede_user_id(), Some("42"));
    Ok(())
}

#[test]
fn imported_cookie_rejects_missing_session_cookie() {
    let error =
        ImportedCookie::parse("DedeUserID=42; bili_jct=csrf-value").expect_err("missing SESSDATA");

    assert!(error.to_string().contains("SESSDATA"));
}

#[test]
fn imported_cookie_rejects_missing_csrf_cookie() {
    let error = ImportedCookie::parse("DedeUserID=42; SESSDATA=session-value")
        .expect_err("missing bili_jct");

    assert!(error.to_string().contains("bili_jct"));
}

#[test]
fn account_summary_defaults_to_logged_out() {
    assert!(!AccountSummary::default().logged_in);
}

#[test]
fn account_summary_from_imported_cookie_marks_logged_in() -> BdlResult<()> {
    let cookie = ImportedCookie::parse("DedeUserID=42; SESSDATA=session; bili_jct=csrf")?;
    let account = AccountSummary::from_imported_cookie(&cookie);

    assert!(account.logged_in);
    Ok(())
}

#[test]
fn account_summary_from_login_nav_uses_verified_profile() -> BdlResult<()> {
    let account = AccountSummary::from_login_nav(&LoginNav {
        is_login: true,
        mid: Some(Mid::new(42).expect("fixture mid should be valid")),
        uname: Some("fixture user".to_owned()),
        face: Some("https://example.test/avatar.jpg".to_owned()),
        wbi_img: LoginWbiImg {
            img_url: "https://example.test/img.png".to_owned(),
            sub_url: "https://example.test/sub.png".to_owned(),
        },
    });

    assert_eq!(
        account,
        AccountSummary {
            logged_in: true,
            name: Some("fixture user".to_owned()),
            avatar_url: Some("https://example.test/avatar.jpg".to_owned()),
            mid: Some("42".to_owned()),
            vip_label: None,
        }
    );
    Ok(())
}

#[test]
fn account_summary_from_logged_out_nav_clears_profile() {
    let account = AccountSummary::from_login_nav(&LoginNav {
        is_login: false,
        mid: None,
        uname: None,
        face: None,
        wbi_img: LoginWbiImg {
            img_url: "https://example.test/img.png".to_owned(),
            sub_url: "https://example.test/sub.png".to_owned(),
        },
    });

    assert_eq!(account, AccountSummary::default());
}

#[test]
fn qr_login_status_maps_known_bilibili_codes() {
    assert_eq!(
        QrLoginStatus::from_bilibili_code(86101),
        QrLoginStatus::Waiting
    );
    assert_eq!(
        QrLoginStatus::from_bilibili_code(86090),
        QrLoginStatus::Scanned
    );
    assert_eq!(
        QrLoginStatus::from_bilibili_code(0),
        QrLoginStatus::Confirmed
    );
    assert_eq!(
        QrLoginStatus::from_bilibili_code(86038),
        QrLoginStatus::Expired
    );
}

#[test]
fn redact_sensitive_removes_cookie_values_and_signed_urls() {
    let redacted = redact_sensitive(
        "Cookie: SESSDATA=secret-session; bili_jct=csrf-secret; DedeUserID=42\nAuthorization: Bearer secret-token\nhttps://cdn.test/video.m4s?token=abc&deadline=123&bcdn_token=long",
    );

    assert!(!redacted.contains("secret-session"));
    assert!(!redacted.contains("csrf-secret"));
    assert!(!redacted.contains("DedeUserID=42"));
    assert!(!redacted.contains("Bearer secret-token"));
    assert!(!redacted.contains("token=abc"));
    assert!(redacted.contains("Authorization: <redacted>"));
    assert!(redacted.contains("https://cdn.test/video.m4s?<redacted>"));
}

#[test]
fn redact_sensitive_truncates_large_response_bodies() {
    let redacted = redact_sensitive(&format!("response body: {}", "x".repeat(5000)));

    assert!(redacted.len() < 4100);
    assert!(redacted.ends_with("...<truncated>"));
}

#[test]
fn account_library_created_folders_are_paginated_and_linkable() {
    let page = AccountLibraryPage::from_created(
        CreatedFolderListData {
            count: 3,
            list: vec![
                created_folder(11, "稍后整理"),
                created_folder(12, "课程"),
                created_folder(13, "音乐"),
            ],
        },
        2,
        2,
    );

    assert_eq!(page.total, 3);
    assert_eq!(page.items.len(), 1);
    assert_eq!(
        page.items[0].kind,
        AccountLibraryFolderKind::CreatedFavorite
    );
    assert_eq!(
        page.items[0].source_url,
        "https://space.bilibili.com/42/favlist?fid=13"
    );
    assert_eq!(page.items[0].owner_mid.as_deref(), Some("42"));
    assert!(!page.has_more);
}

#[test]
fn account_library_collected_folders_keep_cover_and_owner() {
    let page = AccountLibraryPage::from_collected(
        1,
        20,
        1,
        vec![CollectedFolderItem {
            id: 21,
            fid: 22,
            mid: 84,
            attr: 0,
            title: "动画短片集".to_owned(),
            cover: "http://example.test/cover.jpg".to_owned(),
            upper: CollectedFolderUpper {
                mid: 84,
                name: "创作者".to_owned(),
                face: "https://example.test/avatar.jpg".to_owned(),
            },
            cover_type: 0,
            intro: "精选短片".to_owned(),
            ctime: 0,
            mtime: 0,
            state: 0,
            fav_state: 1,
            media_count: 8,
        }],
    );

    assert_eq!(
        page.items[0].kind,
        AccountLibraryFolderKind::CollectedFavorite
    );
    assert_eq!(
        page.items[0].cover_url.as_deref(),
        Some("https://example.test/cover.jpg")
    );
    assert_eq!(page.items[0].owner_name.as_deref(), Some("创作者"));
    assert_eq!(page.items[0].owner_mid.as_deref(), Some("84"));
    assert_eq!(
        page.items[0].source_url,
        "https://space.bilibili.com/84/favlist?fid=21"
    );
}

fn created_folder(id: u64, title: &str) -> CreatedFolderItem {
    CreatedFolderItem {
        id,
        fid: id + 100,
        mid: 42,
        attr: 0,
        title: title.to_owned(),
        fav_state: 1,
        media_count: 5,
    }
}
