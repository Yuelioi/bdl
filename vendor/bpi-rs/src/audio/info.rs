//! 歌曲基本信息
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/audio/info.md)

use serde::{Deserialize, Serialize};

/// 歌曲基本信息数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioInfoData {
    /// 音频auid
    pub id: i64,
    /// UP主mid
    pub uid: i64,
    /// UP主昵称
    pub uname: String,
    /// 作者名
    pub author: String,
    /// 歌曲标题
    pub title: String,
    /// 封面图片url
    pub cover: String,
    /// 歌曲简介
    pub intro: String,
    /// lrc歌词url
    pub lyric: String,
    /// 1 作用尚不明确
    pub crtype: i32,
    /// 歌曲时间长度 单位为秒
    pub duration: i64,
    /// 歌曲发布时间 时间戳
    pub passtime: i64,
    /// 当前请求时间 时间戳
    pub curtime: i64,
    /// 关联稿件avid 无为0
    pub aid: i64,
    /// 关联稿件bvid 无为空
    pub bvid: String,
    /// 关联视频cid 无为0
    pub cid: i64,
    /// 0 作用尚不明确
    pub msid: i64,
    /// 0 作用尚不明确
    pub attr: i64,
    /// 0 作用尚不明确
    pub limit: i64,
    /// 0 作用尚不明确
    #[serde(rename = "activityId")]
    pub activity_id: i64,
    pub limitdesc: String,
    /// null 作用尚不明确
    pub ctime: Option<serde_json::Value>,
    /// 状态数
    pub statistic: AudioStatistic,
    /// UP主会员状态
    #[serde(rename = "vipInfo")]
    pub vip_info: AudioVipInfo,
    /// 歌曲所在的收藏夹mlid 需要登录(SESSDATA)
    #[serde(rename = "collectIds")]
    pub collect_ids: Vec<i64>,
    /// 投币数
    pub coin_num: i64,
}

/// 音频状态数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStatistic {
    /// 音频auid
    pub sid: i64,
    /// 播放次数
    pub play: i64,
    /// 收藏数
    pub collect: i64,
    /// 评论数
    pub comment: i64,
    /// 分享数
    pub share: i64,
}

/// UP主会员状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioVipInfo {
    /// 会员类型 0：无 1：月会员 2：年会员
    pub r#type: i32,
    /// 会员状态 0：无 1：有
    pub status: i32,
    /// 会员到期时间 时间戳 毫秒
    pub due_date: i64,
    /// 会员开通状态 0：无 1：有
    pub vip_pay_type: i32,
}

/// 歌曲TAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTag {
    /// song 作用尚不明确
    pub r#type: String,
    /// ？？？ 作用尚不明确
    pub subtype: i32,
    /// TAG id？？ 作用尚不明确
    pub key: i32,
    /// TAG名
    pub info: String,
}

/// 歌曲创作成员类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMemberType {
    /// 成员列表
    pub list: Vec<AudioMember>,
    /// 成员类型代码 1：歌手 2：作词 3：作曲 4：编曲 5：后期/混音 7：封面制作 8：音源 9：调音 10：演奏 11：乐器 127：UP主
    pub r#type: i32,
}

/// 歌曲创作成员
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMember {
    /// 0 作用尚不明确
    pub mid: i64,
    /// 成员名
    pub name: String,
    /// 成员id？？ 作用尚不明确
    pub member_id: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::params::AudioSongParams;
    use crate::ids::AudioId;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    const TEST_SID: u64 = 13603;

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "info" => include_bytes!("../../tests/contracts/audio/info/contract.json").as_slice(),
            "tags" => include_bytes!("../../tests/contracts/audio/tags/contract.json").as_slice(),
            "members" => {
                include_bytes!("../../tests/contracts/audio/members/contract.json").as_slice()
            }
            "lyric" => include_bytes!("../../tests/contracts/audio/lyric/contract.json").as_slice(),
            _ => unreachable!("unknown audio info contract"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_audio_info() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .audio()
            .info(AudioSongParams::new(AudioId::new(TEST_SID)?))
            .await?;
        assert!(!data.title.is_empty());
        assert!(!data.author.is_empty());
        assert!(data.duration > 0);

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_audio_tags() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .audio()
            .tags(AudioSongParams::new(AudioId::new(TEST_SID)?))
            .await?;

        tracing::info!("{:#?}", data);

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_audio_members() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .audio()
            .members(AudioSongParams::new(AudioId::new(TEST_SID)?))
            .await?;

        tracing::info!("{:#?}", data);

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_audio_lyric() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");

        let data = bpi
            .audio()
            .lyric(AudioSongParams::new(AudioId::new(TEST_SID)?))
            .await?;

        tracing::info!("{:#?}", data);

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_audio_info_fields() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");

        let data = bpi
            .audio()
            .info(AudioSongParams::new(AudioId::new(13598)?))
            .await?;

        assert!(data.id > 0);
        assert!(data.uid > 0);
        assert!(!data.uname.is_empty());
        assert!(!data.title.is_empty());
        assert!(data.duration > 0);
        assert!(data.passtime > 0);

        let stats = &data.statistic;
        assert!(stats.sid > 0);
        assert!(stats.play >= 0);
        assert!(stats.collect >= 0);

        Ok(())
    }

    #[test]
    fn audio_info_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("info")?;
        let params = AudioSongParams::new(AudioId::new(TEST_SID)?);

        assert_eq!(contract.name, "audio.info");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://www.bilibili.com/audio/music-service-c/web/song/info"
        );
        assert_eq!(
            contract.request.query.get("sid").map(String::as_str),
            Some("13603")
        );
        assert_eq!(params.query_pairs(), vec![("sid", "13603".to_string())]);
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("AudioInfoData")
        );
        Ok(())
    }

    #[test]
    fn audio_info_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!("../../tests/contracts/audio/info/responses/anonymous.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/audio/info/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/audio/info/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<AudioInfoData>::from_slice(bytes)?.into_payload()?;

            assert_eq!(payload.id, TEST_SID as i64);
            assert!(!payload.title.is_empty());
        }
        Ok(())
    }

    #[test]
    fn audio_tags_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("tags")?;

        assert_eq!(contract.name, "audio.tags");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://www.bilibili.com/audio/music-service-c/web/tag/song"
        );
        assert_eq!(
            contract.request.query.get("sid").map(String::as_str),
            Some("13603")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("Vec<AudioTag>")
        );
        Ok(())
    }

    #[test]
    fn audio_tags_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!("../../tests/contracts/audio/tags/responses/anonymous.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/audio/tags/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/audio/tags/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<Vec<AudioTag>>::from_slice(bytes)?.into_payload()?;

            assert!(!payload.is_empty());
        }
        Ok(())
    }

    #[test]
    fn audio_members_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("members")?;

        assert_eq!(contract.name, "audio.members");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://www.bilibili.com/audio/music-service-c/web/member/song"
        );
        assert_eq!(
            contract.request.query.get("sid").map(String::as_str),
            Some("13603")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("Vec<AudioMemberType>")
        );
        Ok(())
    }

    #[test]
    fn audio_members_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!("../../tests/contracts/audio/members/responses/anonymous.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/audio/members/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/audio/members/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<Vec<AudioMemberType>>::from_slice(bytes)?.into_payload()?;

            assert!(!payload.is_empty());
        }
        Ok(())
    }

    #[test]
    fn audio_lyric_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("lyric")?;

        assert_eq!(contract.name, "audio.lyric");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://www.bilibili.com/audio/music-service-c/web/song/lyric"
        );
        assert_eq!(
            contract.request.query.get("sid").map(String::as_str),
            Some("13603")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.fixture_kind.as_deref(),
            Some("sanitized_probe_body")
        );
        Ok(())
    }

    #[test]
    fn audio_lyric_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!("../../tests/contracts/audio/lyric/responses/anonymous.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/audio/lyric/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/audio/lyric/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<String>::from_slice(bytes)?.into_payload()?;

            assert_eq!(payload, "<lyrics redacted from probe body>");
        }
        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/audio/public-read/{endpoint}/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn audio_info_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body("info", profile) {
                let payload =
                    serde_json::from_value::<ApiEnvelope<AudioInfoData>>(body)?.into_payload()?;

                assert_eq!(payload.id, TEST_SID as i64);
            }

            if let Some(body) = local_probe_body("tags", profile) {
                let payload =
                    serde_json::from_value::<ApiEnvelope<Vec<AudioTag>>>(body)?.into_payload()?;

                assert!(!payload.is_empty());
            }

            if let Some(body) = local_probe_body("members", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<Vec<AudioMemberType>>>(body)?
                    .into_payload()?;

                assert!(!payload.is_empty());
            }

            if let Some(body) = local_probe_body("lyric", profile) {
                let payload =
                    serde_json::from_value::<ApiEnvelope<String>>(body)?.into_payload()?;

                assert!(!payload.is_empty());
            }
        }
        Ok(())
    }
}
