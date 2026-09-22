//! 创作中心统计数据 API
//!
//! [参考文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/creativecenter/statistics&data.md)

use serde::{Deserialize, Serialize};

/// UP主视频状态数据
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct UpStatData {
    /// 新增投币数
    #[serde(rename = "inc_coin")]
    pub inc_coin: i64,

    /// 新增充电数
    #[serde(rename = "inc_elec")]
    pub inc_elec: i64,

    /// 新增收藏数
    #[serde(rename = "inc_fav")]
    pub inc_fav: i64,

    /// 新增点赞数
    #[serde(rename = "inc_like")]
    pub inc_like: i64,

    /// 新增分享数
    #[serde(rename = "inc_share")]
    pub inc_share: i64,

    /// 新增播放数
    #[serde(rename = "incr_click")]
    pub incr_click: i64,

    /// 新增弹幕数
    #[serde(rename = "incr_dm")]
    pub incr_dm: i64,

    /// 新增粉丝数
    #[serde(rename = "incr_fans")]
    pub incr_fans: i64,

    /// 新增评论数
    #[serde(rename = "incr_reply")]
    pub incr_reply: i64,

    /// 总计播放数
    #[serde(rename = "total_click")]
    pub total_click: i64,

    /// 总计投币数
    #[serde(rename = "total_coin")]
    pub total_coin: i64,

    /// 总计弹幕数
    #[serde(rename = "total_dm")]
    pub total_dm: i64,

    /// 总计充电数
    #[serde(rename = "total_elec")]
    pub total_elec: i64,

    /// 总计粉丝数
    #[serde(rename = "total_fans")]
    pub total_fans: i64,

    /// 总计收藏数
    #[serde(rename = "total_fav")]
    pub total_fav: i64,

    /// 总计点赞数
    #[serde(rename = "total_like")]
    pub total_like: i64,

    /// 总计评论数
    #[serde(rename = "total_reply")]
    pub total_reply: i64,

    /// 总计分享数
    #[serde(rename = "total_share")]
    pub total_share: i64,
}

/// 单个视频对比数据
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveCompareItem {
    /// av号
    pub aid: i64,
    /// bv号
    pub bvid: String,
    /// 封面 url
    pub cover: String,
    /// 标题
    pub title: String,
    /// 发布时间（秒级时间戳）
    pub pubtime: i64,
    /// 视频长度（秒）
    pub duration: i64,
    pub stat: Stat,
    #[serde(rename = "is_only_self")]
    pub is_only_self: bool,
    #[serde(rename = "hour_stat")]
    pub hour_stat: Option<HourStat>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stat {
    #[serde(rename = "not_ready_field")]
    pub not_ready_field: serde_json::Value,
    /// 播放数
    pub play: i64,
    pub vt: i64,
    // ===== 百分比类指标，B站返回一般是整数，100 表示 1% =====
    /// 完播比
    #[serde(rename = "full_play_ratio")]
    pub full_play_ratio: i64,
    /// 游客播放数占比
    #[serde(rename = "play_viewer_rate")]
    pub play_viewer_rate: i64,
    #[serde(rename = "play_viewer_rate_med")]
    pub play_viewer_rate_med: i64,
    /// 粉丝观看率
    #[serde(rename = "play_fan_rate")]
    pub play_fan_rate: i64,
    #[serde(rename = "play_fan_rate_med")]
    pub play_fan_rate_med: i64,
    #[serde(rename = "active_fans_rate")]
    pub active_fans_rate: i64,
    #[serde(rename = "active_fans_med")]
    pub active_fans_med: i64,
    /// 封标点击率
    #[serde(rename = "tm_rate")]
    pub tm_rate: i64,
    /// 自己平均封标点击率
    #[serde(rename = "tm_rate_med")]
    pub tm_rate_med: i64,
    /// 同类up粉丝封标点击率
    #[serde(rename = "tm_fan_simi_rate_med")]
    pub tm_fan_simi_rate_med: i64,
    /// 同类up游客封标点击率
    #[serde(rename = "tm_viewer_simi_rate_med")]
    pub tm_viewer_simi_rate_med: i64,
    /// 粉丝封标点击率
    #[serde(rename = "tm_fan_rate")]
    pub tm_fan_rate: i64,
    /// 游客封标点击率
    #[serde(rename = "tm_viewer_rate")]
    pub tm_viewer_rate: i64,
    /// 封标点击率超过n%同类稿件
    #[serde(rename = "tm_pass_rate")]
    pub tm_pass_rate: i64,
    /// 粉丝封标点击率超过n%同类稿件
    #[serde(rename = "tm_fan_pass_rate")]
    pub tm_fan_pass_rate: i64,
    /// 游客封标点击率超过n%同类稿件
    #[serde(rename = "tm_viewer_pass_rate")]
    pub tm_viewer_pass_rate: i64,
    /// 3秒退出率
    #[serde(rename = "crash_rate")]
    pub crash_rate: i64,
    #[serde(rename = "crash_rate_med")]
    pub crash_rate_med: i64,
    /// 同类up粉丝3秒退出率
    #[serde(rename = "crash_fan_simi_rate_med")]
    pub crash_fan_simi_rate_med: i64,
    /// 同类up游客3秒退出率
    #[serde(rename = "crash_viewer_simi_rate_med")]
    pub crash_viewer_simi_rate_med: i64,
    /// 粉丝3秒退出率
    #[serde(rename = "crash_fan_rate")]
    pub crash_fan_rate: i64,
    /// 游客3秒退出率
    #[serde(rename = "crash_viewer_rate")]
    pub crash_viewer_rate: i64,
    /// 互动率
    #[serde(rename = "interact_rate")]
    pub interact_rate: i64,
    #[serde(rename = "interact_rate_med")]
    pub interact_rate_med: i64,
    /// 同类up粉丝互动率
    #[serde(rename = "interact_fan_simi_rate_med")]
    pub interact_fan_simi_rate_med: i64,
    /// 同类up游客互动率
    #[serde(rename = "interact_viewer_simi_rate_med")]
    pub interact_viewer_simi_rate_med: i64,
    /// 粉丝互动率
    #[serde(rename = "interact_fan_rate")]
    pub interact_fan_rate: i64,
    /// 游客互动率
    #[serde(rename = "interact_viewer_rate")]
    pub interact_viewer_rate: i64,
    /// 平均播放时间（目前总是0）
    #[serde(rename = "avg_play_time")]
    pub avg_play_time: i64,
    #[serde(rename = "avg_play_time_int")]
    pub avg_play_time_int: i64,
    /// 涨粉
    #[serde(rename = "total_new_attention_cnt")]
    pub total_new_attention_cnt: i64,
    /// 播转粉率
    #[serde(rename = "play_trans_fan_rate")]
    pub play_trans_fan_rate: i64,
    /// 其他up平均播转粉率
    #[serde(rename = "play_trans_fan_rate_med")]
    pub play_trans_fan_rate_med: i64,
    /// 点赞数
    pub like: i64,
    /// 评论数
    pub comment: i64,
    /// 弹幕数
    pub dm: i64,
    /// 收藏数
    pub fav: i64,
    /// 投币数
    pub coin: i64,
    /// 分享数
    pub share: i64,
    #[serde(rename = "unfollow")]
    pub unfollow: i64,
    #[serde(rename = "tm_star")]
    pub tm_star: i64,
    #[serde(rename = "tm_viewer_star")]
    pub tm_viewer_star: i64,
    #[serde(rename = "tm_fan_star")]
    pub tm_fan_star: i64,
    #[serde(rename = "crash_p50")]
    pub crash_p50: i64,
    #[serde(rename = "crash_viewer_p50")]
    pub crash_viewer_p50: i64,
    #[serde(rename = "crash_fan_p50")]
    pub crash_fan_p50: i64,
    #[serde(rename = "interact_p50")]
    pub interact_p50: i64,
    #[serde(rename = "interact_viewer_p50")]
    pub interact_viewer_p50: i64,
    #[serde(rename = "interact_fan_p50")]
    pub interact_fan_p50: i64,
    #[serde(rename = "play_trans_fan_p50")]
    pub play_trans_fan_p50: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HourStat {
    #[serde(rename = "not_ready_field")]
    pub not_ready_field: serde_json::Value,
    /// 播放数
    pub play: i64,
    pub vt: i64,
    /// 点赞数
    pub like: i64,
    /// 评论数
    pub comment: i64,
    /// 弹幕数
    pub dm: i64,
    /// 收藏数
    pub fav: i64,
    /// 投币数
    pub coin: i64,
    /// 分享数
    pub share: i64,
    /// 封标点击率超过n%同类稿件
    #[serde(rename = "tm_pass_rate")]
    pub tm_pass_rate: i64,
    /// 互动率
    #[serde(rename = "interact_rate")]
    pub interact_rate: i64,
    #[serde(rename = "tm_star")]
    pub tm_star: i64,
}

/// UP主视频数据比较
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ArchiveCompareData {
    pub list: Vec<ArchiveCompareItem>,
}

/// UP主专栏状态数据
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct UpArticleStatData {
    /// 总计阅读数
    pub view: i64,
    /// 总计评论数
    pub reply: i64,
    /// 总计点赞数
    pub like: i64,
    /// 总计投币数
    pub coin: i64,
    /// 总计收藏数
    pub fav: i64,
    /// 总计分享数
    pub share: i64,
    /// 新增阅读数
    #[serde(rename = "incr_view")]
    pub incr_view: i64,
    /// 新增评论数
    #[serde(rename = "incr_reply")]
    pub incr_reply: i64,
    /// 新增点赞数
    #[serde(rename = "incr_like")]
    pub incr_like: i64,
    /// 新增投币数
    #[serde(rename = "incr_coin")]
    pub incr_coin: i64,
    /// 新增收藏数
    #[serde(rename = "incr_fav")]
    pub incr_fav: i64,
    /// 新增分享数
    #[serde(rename = "incr_share")]
    pub incr_share: i64,
}

/// UP主视频数据增量趋势项
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct VideoTrendItem {
    /// 对应时间戳（前一天8:00）
    pub date_key: i64,
    /// 增加数量，数据类型决定
    pub total_inc: i64,
}

/// UP主专栏数据增量趋势项
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ArticleTrendItem {
    /// 对应时间戳（前一天8:00）
    pub date_key: i64,
    /// 增加数量，数据类型决定
    pub total_inc: i64,
}

/// 播放来源情况（播放方式）
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct PageSource {
    /// 通过动态
    pub dynamic: i64,
    /// 其他方式
    pub other: i64,
    /// 通过推荐列表
    #[serde(rename = "related_video")]
    pub related_video: i64,
    /// 通过搜索
    pub search: i64,
    /// 空间列表播放
    pub space: i64,
    /// 天马来源（APP推荐信息流）
    pub tenma: i64,
}

/// 播放平台占比
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct PlayProportion {
    /// 安卓端
    pub android: i64,
    /// 移动端H5
    pub h5: i64,
    /// iOS端
    pub ios: i64,
    /// 站外
    pub out: i64,
    /// PC网页版
    pub pc: i64,
}

/// 播放来源占比数据
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct PlaySourceData {
    pub page_source: PageSource,
    pub play_proportion: PlayProportion,
}

/// 播放地区提示信息
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Period {
    pub module_one: Option<String>,
    pub module_two: Option<String>,
    pub module_three: Option<String>,
    pub module_four: Option<String>,
}

/// 播放地区情况（粉丝或路人）
pub type ViewerAreaMap = std::collections::HashMap<String, i64>;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ViewerArea {
    pub fan: ViewerAreaMap,
    pub not_fan: ViewerAreaMap,
}

/// 播放数据情况（粉丝或路人）
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ViewerBaseDetail {
    pub male: i64,
    pub female: i64,
    #[serde(rename = "age_one")]
    pub age_one: i64,
    #[serde(rename = "age_two")]
    pub age_two: i64,
    #[serde(rename = "age_three")]
    pub age_three: i64,
    #[serde(rename = "age_four")]
    pub age_four: i64,
    #[serde(rename = "plat_pc")]
    pub plat_pc: i64,
    #[serde(rename = "plat_h5")]
    pub plat_h5: i64,
    #[serde(rename = "plat_out")]
    pub plat_out: i64,
    #[serde(rename = "plat_ios")]
    pub plat_ios: i64,
    #[serde(rename = "plat_android")]
    pub plat_android: i64,
    #[serde(rename = "plat_other_app")]
    pub plat_other_app: i64,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ViewerBase {
    pub fan: ViewerBaseDetail,
    pub not_fan: ViewerBaseDetail,
}

/// 播放分布情况
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ViewerData {
    pub period: Period,
    pub viewer_area: ViewerArea,
    pub viewer_base: ViewerBase,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creativecenter::{
        UpArchiveCompareParams, UpArticleTrendMetric, UpArticleTrendParams, UpVideoTrendMetric,
        UpVideoTrendParams,
    };
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError};
    use std::collections::BTreeMap;
    use tracing::info;

    fn contract(name: &str) -> Result<EndpointContract, BpiError> {
        let bytes = match name {
            "up-stat" => include_bytes!(
                "../../tests/contracts/creativecenter/statistics/up-stat/contract.json"
            )
            .as_slice(),
            "archive-compare" => include_bytes!(
                "../../tests/contracts/creativecenter/statistics/archive-compare/contract.json"
            )
            .as_slice(),
            "article-stat" => include_bytes!(
                "../../tests/contracts/creativecenter/statistics/article-stat/contract.json"
            )
            .as_slice(),
            "video-trend" => include_bytes!(
                "../../tests/contracts/creativecenter/statistics/video-trend/contract.json"
            )
            .as_slice(),
            "article-trend" => include_bytes!(
                "../../tests/contracts/creativecenter/statistics/article-trend/contract.json"
            )
            .as_slice(),
            "play-source" => include_bytes!(
                "../../tests/contracts/creativecenter/statistics/play-source/contract.json"
            )
            .as_slice(),
            "viewer-data" => include_bytes!(
                "../../tests/contracts/creativecenter/statistics/viewer-data/contract.json"
            )
            .as_slice(),
            _ => unreachable!("unknown creativecenter statistics contract"),
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

    fn live_creativecenter_tests_enabled() -> bool {
        std::env::var_os("BPI_LIVE_TEST").is_some() && std::env::var_os("BPI_COOKIE").is_some()
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_up_stat() -> Result<(), Box<BpiError>> {
        if !live_creativecenter_tests_enabled() {
            return Ok(());
        }

        let bpi = BpiClient::new().expect("client should build");
        let data = bpi.creativecenter().up_stat().await?;
        info!("UP主视频状态数据: {:?}", data);
        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_archive_compare() -> Result<(), Box<BpiError>> {
        if !live_creativecenter_tests_enabled() {
            return Ok(());
        }

        let bpi = BpiClient::new().expect("client should build");
        let params = UpArchiveCompareParams::new().with_size(3)?;
        let data = bpi.creativecenter().archive_compare(params).await?;
        info!("UP主视频数据比较: {:?}", data);
        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_up_article_stat() -> Result<(), Box<BpiError>> {
        if !live_creativecenter_tests_enabled() {
            return Ok(());
        }

        let bpi = BpiClient::new().expect("client should build");
        let data = bpi.creativecenter().article_stat().await?;
        info!("UP主专栏状态数据: {:?}", data);
        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_trend() -> Result<(), Box<BpiError>> {
        if !live_creativecenter_tests_enabled() {
            return Ok(());
        }

        let bpi = BpiClient::new().expect("client should build");
        let params = UpVideoTrendParams::new(UpVideoTrendMetric::Play);
        let data = bpi.creativecenter().video_trend(params).await?;
        info!("UP主视频数据增量趋势: {:?}", data);
        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_article_trend() -> Result<(), Box<BpiError>> {
        if !live_creativecenter_tests_enabled() {
            return Ok(());
        }

        let bpi = BpiClient::new().expect("client should build");
        let params = UpArticleTrendParams::new(UpArticleTrendMetric::Read);
        let data = bpi.creativecenter().article_trend(params).await?;
        info!("UP主专栏数据增量趋势: {:?}", data);
        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_viewer_data() -> Result<(), Box<BpiError>> {
        if !live_creativecenter_tests_enabled() {
            return Ok(());
        }

        let bpi = BpiClient::new().expect("client should build");
        let data = bpi.creativecenter().viewer_data().await?;
        info!("播放分布情况: {:?}", data);
        Ok(())
    }

    #[test]
    fn creativecenter_statistics_contracts_match_endpoint_requests() -> Result<(), BpiError> {
        let up_stat = contract("up-stat")?;
        assert_eq!(up_stat.name, "creativecenter.statistics.up_stat");
        assert_eq!(up_stat.request.method, HttpMethod::Get);
        assert_eq!(
            up_stat.request.url.as_str(),
            "https://member.bilibili.com/x/web/index/stat"
        );
        assert!(up_stat.request.query.is_empty());

        let archive_compare = contract("archive-compare")?;
        let archive_compare_params = UpArchiveCompareParams::new().with_size(3)?;
        assert_eq!(
            archive_compare.name,
            "creativecenter.statistics.archive_compare"
        );
        assert_eq!(
            archive_compare.request.url.as_str(),
            "https://member.bilibili.com/x/web/data/archive_diagnose/compare"
        );
        assert_eq!(
            query_map(archive_compare_params.query_pairs()),
            archive_compare.request.query
        );

        let article_stat = contract("article-stat")?;
        assert_eq!(
            article_stat.request.url.as_str(),
            "https://member.bilibili.com/x/web/data/article"
        );
        assert!(article_stat.request.query.is_empty());

        let video_trend = contract("video-trend")?;
        let video_trend_params = UpVideoTrendParams::new(UpVideoTrendMetric::Play);
        assert_eq!(
            video_trend.request.url.as_str(),
            "https://member.bilibili.com/x/web/data/pandect"
        );
        assert_eq!(
            query_map(video_trend_params.query_pairs()),
            video_trend.request.query
        );

        let article_trend = contract("article-trend")?;
        let article_trend_params = UpArticleTrendParams::new(UpArticleTrendMetric::Read);
        assert_eq!(
            article_trend.request.url.as_str(),
            "https://member.bilibili.com/x/web/data/article/thirty"
        );
        assert_eq!(
            query_map(article_trend_params.query_pairs()),
            article_trend.request.query
        );

        let play_source = contract("play-source")?;
        assert_eq!(
            play_source.request.url.as_str(),
            "https://member.bilibili.com/x/web/data/playsource"
        );
        assert_eq!(
            play_source
                .request
                .headers
                .get("Origin")
                .map(String::as_str),
            Some("https://www.bilibili.com")
        );

        let viewer_data = contract("viewer-data")?;
        assert_eq!(
            viewer_data.request.url.as_str(),
            "https://member.bilibili.com/x/web/data/base"
        );
        assert!(viewer_data.request.query.is_empty());

        Ok(())
    }

    #[test]
    fn creativecenter_statistics_response_fixtures_parse_declared_models() -> Result<(), BpiError> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/creativecenter/statistics/up-stat/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/creativecenter/statistics/up-stat/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<UpStatData>::from_slice(bytes)?.into_payload()?;
            assert_eq!(payload.total_click, 0);
        }

        for bytes in [
            include_bytes!(
                "../../tests/contracts/creativecenter/statistics/archive-compare/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/creativecenter/statistics/archive-compare/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<ArchiveCompareData>::from_slice(bytes)?.into_payload()?;
            assert_eq!(payload.list.len(), 1);
        }

        for bytes in [
            include_bytes!(
                "../../tests/contracts/creativecenter/statistics/article-stat/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/creativecenter/statistics/article-stat/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<UpArticleStatData>::from_slice(bytes)?.into_payload()?;
            assert_eq!(payload.view, 0);
        }

        for bytes in [
            include_bytes!(
                "../../tests/contracts/creativecenter/statistics/video-trend/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/creativecenter/statistics/video-trend/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload =
                ApiEnvelope::<Vec<VideoTrendItem>>::from_slice(bytes)?.into_payload()?;
            assert_eq!(payload.len(), 1);
        }

        let normal_article_trend = ApiEnvelope::<Vec<ArticleTrendItem>>::from_slice(include_bytes!(
            "../../tests/contracts/creativecenter/statistics/article-trend/responses/normal.success.json"
        ))?
        .into_optional_payload()?;
        assert!(normal_article_trend.is_none());

        let vip_article_trend = ApiEnvelope::<Vec<ArticleTrendItem>>::from_slice(include_bytes!(
            "../../tests/contracts/creativecenter/statistics/article-trend/responses/vip.success.json"
        ))?
        .into_payload()?;
        assert_eq!(vip_article_trend.len(), 1);

        for bytes in [
            include_bytes!(
                "../../tests/contracts/creativecenter/statistics/play-source/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/creativecenter/statistics/play-source/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<PlaySourceData>::from_slice(bytes)?.into_optional_payload()?;
            assert!(payload.is_none());
        }

        for bytes in [
            include_bytes!(
                "../../tests/contracts/creativecenter/statistics/viewer-data/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/creativecenter/statistics/viewer-data/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<ViewerData>::from_slice(bytes)?.into_payload()?;
            assert_eq!(payload.viewer_area.fan.get("<redacted>"), Some(&0));
        }

        Ok(())
    }

    #[test]
    fn creativecenter_statistics_error_fixtures_preserve_observed_api_errors()
    -> Result<(), BpiError> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/creativecenter/statistics/up-stat/responses/anonymous.requires_login.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/creativecenter/statistics/archive-compare/responses/anonymous.requires_login.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/creativecenter/statistics/article-stat/responses/anonymous.requires_login.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/creativecenter/statistics/video-trend/responses/anonymous.requires_login.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/creativecenter/statistics/article-trend/responses/anonymous.requires_login.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/creativecenter/statistics/play-source/responses/anonymous.requires_login.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/creativecenter/statistics/viewer-data/responses/anonymous.requires_login.json"
            )
            .as_slice(),
        ] {
            let err = ApiEnvelope::<serde_json::Value>::from_slice(bytes)
                .and_then(ApiEnvelope::ensure_success)
                .unwrap_err();
            assert!(err.requires_login());
        }
        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path = format!(
            "target/bpi-probe-runs/creativecenter/statistics-read/{endpoint}/{profile}.response.json"
        );
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn creativecenter_statistics_models_match_local_probe_outputs_when_available()
    -> Result<(), BpiError> {
        for profile in ["normal", "vip"] {
            if let Some(body) = local_probe_body("up-stat", profile) {
                let _payload =
                    serde_json::from_value::<ApiEnvelope<UpStatData>>(body)?.into_payload()?;
            }

            if let Some(body) = local_probe_body("archive-compare", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<ArchiveCompareData>>(body)?
                    .into_payload()?;
                assert!(!payload.list.is_empty());
            }

            if let Some(body) = local_probe_body("article-stat", profile) {
                let _payload = serde_json::from_value::<ApiEnvelope<UpArticleStatData>>(body)?
                    .into_payload()?;
            }

            if let Some(body) = local_probe_body("video-trend", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<Vec<VideoTrendItem>>>(body)?
                    .into_payload()?;
                assert!(!payload.is_empty());
            }

            if let Some(body) = local_probe_body("article-trend", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<Vec<ArticleTrendItem>>>(body)?
                    .into_optional_payload()?;
                if profile == "vip" {
                    assert!(payload.as_ref().is_some_and(|items| !items.is_empty()));
                } else {
                    assert!(payload.is_none());
                }
            }

            if let Some(body) = local_probe_body("play-source", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<PlaySourceData>>(body)?
                    .into_optional_payload()?;
                assert!(payload.is_none());
            }

            if let Some(body) = local_probe_body("viewer-data", profile) {
                let payload =
                    serde_json::from_value::<ApiEnvelope<ViewerData>>(body)?.into_payload()?;
                assert!(!payload.viewer_area.fan.is_empty());
            }
        }
        Ok(())
    }
}
