//! XML 弹幕
//!
//! [文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/danmaku)

use crate::ids::Cid;
use crate::{BpiError, BpiResult};
use flate2::read::DeflateDecoder;
use quick_xml::de::from_str;
use std::io::Read;

use serde::{Deserialize, Serialize};

// 用于解析 <d> 标签的 p 属性的元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DanmakuMeta {
    pub time: f32,         // 视频内弹幕出现时间（秒）
    pub danmaku_type: i32, // 弹幕类型
    pub font_size: i32,    // 字号
    pub color: i32,        // 颜色（十进制RGB888值）
    pub send_time: i64,    // 发送时间戳
    pub pool_type: i32,    // 弹幕池类型
    pub user_hash: String, // 发送者mid的HASH
    pub dmid: i64,         // 弹幕dmid（唯一标识）
    pub block_level: i32,  // 屏蔽等级
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "d")]
pub struct Danmaku {
    #[serde(rename = "$value")]
    pub content: String, // 弹幕内容

    #[serde(rename = "@p")]
    pub p_value: String, // 原始的 p 属性字符串

    #[serde(skip_serializing)]
    pub meta: Option<DanmakuMeta>,
}

impl Danmaku {
    /// 解析 p 属性并返回 DanmakuMeta
    pub fn parse_p(&mut self) -> Result<(), BpiError> {
        let parts: Vec<&str> = self.p_value.split(',').collect();
        if parts.len() < 9 {
            return Err(BpiError::parse("解析xml失败 弹幕参数不足9"));
        }

        let time: f32 = parts[0].parse().unwrap_or(0.0);
        let danmaku_type: i32 = parts[1].parse().unwrap_or(1);
        let font_size: i32 = parts[2].parse().unwrap_or(25);
        let color: i32 = parts[3].parse().unwrap_or(16777215); // 默认白色
        let send_time: i64 = parts[4].parse().unwrap_or(0);
        let pool_type: i32 = parts[5].parse().unwrap_or(0);
        let user_hash = parts[6].to_string();
        let dmid: i64 = parts[7].parse().unwrap_or(0);

        let block_level = parts[8].parse().unwrap_or(0);
        self.meta = Some(DanmakuMeta {
            time,
            danmaku_type,
            font_size,
            color,
            send_time,
            pool_type,
            user_hash,
            dmid,
            block_level,
        });
        Ok(())
    }
}

// 根标签 i
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "i")]
pub struct DanmakuXml {
    pub chatserver: String,
    pub chatid: String,
    pub mission: i32,
    pub maxlimit: i32,
    pub state: i32, // 0: 正常, 1: 弹幕已关闭
    pub real_name: i32,
    pub source: String,
    #[serde(rename = "d", default)]
    pub danmakus: Vec<Danmaku>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DanmakuXmlListParams {
    cid: Cid,
}

impl DanmakuXmlListParams {
    pub fn new(cid: Cid) -> Self {
        Self { cid }
    }

    pub fn query_pairs(&self) -> [(&'static str, String); 1] {
        [("oid", self.cid.to_string())]
    }

    pub fn comment_xml_url(&self) -> String {
        format!("https://comment.bilibili.com/{}.xml", self.cid)
    }
}

pub(crate) fn parse_deflate_danmaku_xml(bytes: &[u8]) -> BpiResult<DanmakuXml> {
    let mut decoder = DeflateDecoder::new(bytes);
    let mut xml = String::new();
    decoder
        .read_to_string(&mut xml)
        .map_err(|_| BpiError::parse("读取xml失败"))?;

    parse_danmaku_xml(&xml)
}

fn parse_danmaku_xml(xml: &str) -> BpiResult<DanmakuXml> {
    let mut parsed: DanmakuXml = from_str(xml).map_err(|_| BpiError::parse("解析xml失败"))?;
    parsed.danmakus.iter_mut().try_for_each(Danmaku::parse_p)?;
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::{HttpMethod, ResponseDecoding};
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{BpiClient, BpiError, BpiResult};
    use base64::{Engine as _, engine::general_purpose};
    use serde::Deserialize;
    use std::collections::BTreeMap;
    use tokio::time::Instant;
    use tracing::info;

    const TEST_CID: u64 = 16546;

    #[derive(Debug, Deserialize)]
    struct BinaryFixture {
        body_base64: String,
        content_type: Option<String>,
        encoding: String,
        kind: String,
        length: usize,
    }

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "list-so" => {
                include_bytes!("../../tests/contracts/danmaku/xml-read/list-so/contract.json")
                    .as_slice()
            }
            "comment-xml" => {
                include_bytes!("../../tests/contracts/danmaku/xml-read/comment-xml/contract.json")
                    .as_slice()
            }
            _ => unreachable!("unknown danmaku xml contract endpoint"),
        };

        EndpointContract::from_slice(bytes)
    }

    fn query_map<I>(params: I) -> BTreeMap<String, String>
    where
        I: IntoIterator<Item = (&'static str, String)>,
    {
        params
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect()
    }

    fn fixture_xml(bytes: &[u8]) -> BpiResult<DanmakuXml> {
        let fixture: BinaryFixture = serde_json::from_slice(bytes)?;
        assert_eq!(fixture.kind, "binary");
        assert_eq!(fixture.encoding, "base64");
        assert_eq!(fixture.content_type.as_deref(), Some("text/xml"));

        let body = general_purpose::STANDARD
            .decode(fixture.body_base64)
            .map_err(|err| BpiError::parse(format!("base64 decode failed: {err}")))?;
        assert_eq!(body.len(), fixture.length);

        parse_deflate_danmaku_xml(&body)
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_danmaku_xml_api() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let start = Instant::now();

        let data = bpi
            .danmaku()
            .xml_list_so(DanmakuXmlListParams::new(Cid::new(TEST_CID)?))
            .await?;
        let duration = start.elapsed();

        info!(
            "耗时1 {:?} 弹幕装填个数: {:?} ",
            duration,
            data.danmakus.len()
        );
        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_danmaku_xml_cid() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let start = Instant::now();

        let data = bpi
            .danmaku()
            .xml_list(DanmakuXmlListParams::new(Cid::new(TEST_CID)?))
            .await?;
        let duration = start.elapsed();

        info!(
            "耗时2 {:?} 弹幕装填个数: {:?} ",
            duration,
            data.danmakus.len()
        );

        Ok(())
    }

    #[test]
    fn danmaku_xml_list_rejects_incomplete_metadata() {
        let mut danmaku = Danmaku {
            content: "bad".to_string(),
            p_value: "1,1,25,16777215,0,0,hash,1".to_string(),
            meta: None,
        };

        let err = danmaku.parse_p().unwrap_err();
        assert!(matches!(
            err,
            BpiError::Decode { .. } | BpiError::Parse { .. }
        ));
    }

    #[test]
    fn danmaku_xml_contracts_match_endpoint_requests() -> BpiResult<()> {
        let params = DanmakuXmlListParams::new(Cid::new(TEST_CID)?);

        let list_so = contract("list-so")?;
        assert_eq!(list_so.name, "danmaku.xml.list_so");
        assert_eq!(list_so.request.method, HttpMethod::Get);
        assert_eq!(
            list_so.request.url.as_str(),
            "https://api.bilibili.com/x/v1/dm/list.so"
        );
        assert_eq!(query_map(params.query_pairs()), list_so.request.query);
        assert_eq!(
            list_so.request.response_decoding,
            ResponseDecoding::Disabled
        );

        let comment_xml = contract("comment-xml")?;
        assert_eq!(comment_xml.name, "danmaku.xml.comment_xml");
        assert_eq!(comment_xml.request.method, HttpMethod::Get);
        assert_eq!(
            comment_xml.request.url.as_str(),
            params.comment_xml_url().as_str()
        );
        assert!(comment_xml.request.query.is_empty());
        assert_eq!(
            comment_xml.request.response_decoding,
            ResponseDecoding::Disabled
        );
        Ok(())
    }

    #[test]
    fn danmaku_xml_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!("../../tests/contracts/danmaku/xml-read/list-so/responses/success.json")
                .as_slice(),
            include_bytes!(
                "../../tests/contracts/danmaku/xml-read/comment-xml/responses/success.json"
            )
            .as_slice(),
        ] {
            let payload = fixture_xml(bytes)?;
            assert_eq!(payload.chatid, TEST_CID.to_string());
            assert_eq!(payload.danmakus.len(), 307);
            assert!(payload.danmakus[0].meta.is_some());
        }
        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/danmaku/xml-read/{endpoint}/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn danmaku_xml_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for endpoint in ["list-so", "comment-xml"] {
            for profile in ["anonymous", "normal", "vip"] {
                let Some(body) = local_probe_body(endpoint, profile) else {
                    continue;
                };
                let bytes = serde_json::to_vec(&body)?;
                let payload = fixture_xml(&bytes)?;
                assert_eq!(payload.chatid, TEST_CID.to_string());
                assert!(!payload.danmakus.is_empty());
            }
        }
        Ok(())
    }
}
