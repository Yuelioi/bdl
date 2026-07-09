use bdl_core::BdlResult;
use bdl_core::account::{AccountSummary, ImportedCookie, QrLoginStatus, redact_sensitive};
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
    assert_eq!(AccountSummary::default().logged_in, false);
}

#[test]
fn account_summary_from_imported_cookie_marks_logged_in() -> BdlResult<()> {
    let cookie = ImportedCookie::parse("DedeUserID=42; SESSDATA=session; bili_jct=csrf")?;
    let account = AccountSummary::from_imported_cookie(&cookie);

    assert_eq!(account.logged_in, true);
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
