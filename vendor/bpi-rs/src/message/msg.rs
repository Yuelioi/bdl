use serde::{Deserialize, Serialize};

// --- API 结构体 ---

/// 未读消息数
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UnreadCountData {
    pub coin: u32, // 未读投币数
    #[serde(default)]
    pub danmu: u32, // 未读弹幕数
    pub favorite: u32, // 未读收藏数
    pub recv_like: u32, // 未读收到喜欢数
    pub recv_reply: u32, // 未读回复
    pub sys_msg: u32, // 未读系统通知数
    pub up: u32,   // 未读UP主助手信息数
}

/// "回复我的"信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReplyFeedData {
    pub cursor: ReplyCursor,
    pub items: Vec<ReplyItem>,
    pub last_view_at: u64,
}

/// 分页游标
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReplyCursor {
    pub is_end: bool,
    pub id: Option<u64>,
    pub time: Option<u64>,
}

/// 单条回复通知
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReplyItem {
    pub id: u64,
    pub user: ReplyUser,
    pub item: ReplyDetail,
    pub counts: u32,
    pub is_multi: u32,
    pub reply_time: u64,
}

/// 回复者用户信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReplyUser {
    pub mid: u64,
    pub nickname: String,
    pub avatar: String,
    pub follow: bool,
    // 以下字段文档表示固定或不返回，但为了完整性保留
    pub fans: Option<u32>,
    pub mid_link: Option<String>,
}

/// 回复通知详情
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReplyDetail {
    pub subject_id: u64,
    pub root_id: u64,
    pub source_id: u64,
    pub target_id: u64,
    #[serde(rename = "type")]
    pub reply_type: String,
    pub business_id: u32,
    pub business: String,
    pub title: String,
    pub desc: String,
    pub uri: String,
    pub native_uri: String,
    pub root_reply_content: String,
    pub source_content: String,
    pub target_reply_content: String,
    pub at_details: Vec<AtUserDetail>,
    pub hide_reply_button: bool,
    pub hide_like_button: bool,
    pub like_state: u32,
}

/// @的用户详情
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AtUserDetail {
    pub mid: u64,
    pub nickname: String,
    pub avatar: String,
    pub follow: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::params::{MessageReplyFeedParams, MessageUnreadCountParams};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "unread-count" => {
                include_bytes!("../../tests/contracts/message/read/unread-count/contract.json")
                    .as_slice()
            }
            "reply-feed" => {
                include_bytes!("../../tests/contracts/message/read/reply-feed/contract.json")
                    .as_slice()
            }
            _ => {
                return Err(BpiError::invalid_parameter(
                    "endpoint",
                    "unknown message contract",
                ));
            }
        };

        EndpointContract::from_slice(bytes)
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_unread_count() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");

        let new_data = bpi
            .message()
            .unread_count(MessageUnreadCountParams::new())
            .await?;
        println!("未读消息数 (新接口): {:?}", new_data);
        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_reply_feed() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");

        let data = bpi
            .message()
            .reply_feed(MessageReplyFeedParams::new())
            .await?;

        println!("最近回复我的信息:");
        println!("  上次查看时间: {}", data.last_view_at);
        println!("  游标信息: {:?}", data.cursor);

        for item in data.items {
            println!("---");
            println!("  回复者: {}", item.user.nickname);
            println!("  回复内容: {}", item.item.source_content);
            println!("  回复时间: {}", item.reply_time);
            println!("  关联视频/动态: {}", item.item.title);
            println!("  根评论: {}", item.item.root_reply_content);
            println!("  跳转链接: {}", item.item.uri);
        }

        if !data.cursor.is_end {
            println!("---");
            println!(
                "还有更多数据，下次请求可使用 id: {:?}, time: {:?}",
                data.cursor.id, data.cursor.time
            );
        }

        Ok(())
    }

    #[test]
    fn message_unread_count_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("unread-count")?;

        assert_eq!(contract.name, "message.unread_count");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.vc.bilibili.com/x/im/web/msgfeed/unread"
        );
        assert_eq!(
            contract.request.query.get("build").map(String::as_str),
            Some("0")
        );
        assert_eq!(
            contract.request.query.get("mobi_app").map(String::as_str),
            Some("web")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(contract.cases[0].response.api_code, Some(-101));
        assert_eq!(
            contract.cases[0].response.error.as_deref(),
            Some("requires_login")
        );
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("UnreadCountData")
        );
        Ok(())
    }

    #[test]
    fn message_reply_feed_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("reply-feed")?;

        assert_eq!(contract.name, "message.reply_feed");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/msgfeed/reply"
        );
        assert_eq!(
            contract.request.query.get("platform").map(String::as_str),
            Some("web")
        );
        assert_eq!(
            contract
                .request
                .query
                .get("web_location")
                .map(String::as_str),
            Some("")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(contract.cases[0].response.api_code, Some(-101));
        assert_eq!(
            contract.cases[0].response.error.as_deref(),
            Some("requires_login")
        );
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("ReplyFeedData")
        );
        Ok(())
    }

    #[test]
    fn message_unread_count_response_fixtures_parse_declared_model() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/message/read/unread-count/responses/anonymous.requires_login.json"
        ))
        .and_then(ApiEnvelope::ensure_success)
        .unwrap_err();
        assert!(err.requires_login());

        let payload = ApiEnvelope::<UnreadCountData>::from_slice(include_bytes!(
            "../../tests/contracts/message/read/unread-count/responses/authenticated.success.json"
        ))?
        .into_payload()?;

        assert_eq!(payload.danmu, 0);
        assert_eq!(payload.sys_msg, 1);
        Ok(())
    }

    #[test]
    fn message_reply_feed_response_fixtures_parse_declared_model() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/message/read/reply-feed/responses/anonymous.requires_login.json"
        ))
        .and_then(ApiEnvelope::ensure_success)
        .unwrap_err();
        assert!(err.requires_login());

        let payload = ApiEnvelope::<ReplyFeedData>::from_slice(include_bytes!(
            "../../tests/contracts/message/read/reply-feed/responses/authenticated.success.json"
        ))?
        .into_payload()?;

        assert_eq!(payload.items.len(), 1);
        assert_eq!(payload.items[0].user.nickname, "sanitized user");
        assert_eq!(payload.items[0].item.reply_type, "video");
        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/message/read/{endpoint}/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn message_unread_count_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body("unread-count", profile) else {
                continue;
            };

            if profile == "anonymous" {
                let err = serde_json::from_value::<ApiEnvelope<serde_json::Value>>(body)?
                    .ensure_success()
                    .unwrap_err();
                assert!(err.requires_login());
                continue;
            }

            let payload =
                serde_json::from_value::<ApiEnvelope<UnreadCountData>>(body)?.into_payload()?;
            let _total_unread = payload.coin
                + payload.danmu
                + payload.favorite
                + payload.recv_like
                + payload.recv_reply
                + payload.sys_msg
                + payload.up;
        }
        Ok(())
    }

    #[test]
    fn message_reply_feed_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body("reply-feed", profile) else {
                continue;
            };

            if profile == "anonymous" {
                let err = serde_json::from_value::<ApiEnvelope<serde_json::Value>>(body)?
                    .ensure_success()
                    .unwrap_err();
                assert!(err.requires_login());
                continue;
            }

            let payload =
                serde_json::from_value::<ApiEnvelope<ReplyFeedData>>(body)?.into_payload()?;
            assert!(
                payload.cursor.is_end || payload.cursor.id.is_some() || !payload.items.is_empty()
            );
        }
        Ok(())
    }
}
