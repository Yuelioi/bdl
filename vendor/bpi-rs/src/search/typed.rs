//! 搜索
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/search/search_request.md)

#[cfg(test)]
use super::result::{
    Article, Bangumi, BiliUser, LiveData, LiveRoom, LiveUser, Movie, SearchData, Video,
};
#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::{
        CategoryId, Duration, OrderSort, SearchArticleParams, SearchBangumiParams,
        SearchBiliUserParams, SearchLiveRoomParams, SearchLiveUserParams, SearchMovieParams,
        SearchOrder, SearchVideoParams, UserType,
    };
    use crate::{
        ApiEnvelope, BpiClient, BpiError, BpiResult,
        probe::{contract::HttpMethod, endpoint_contract::EndpointContract},
    };
    use serde::de::DeserializeOwned;
    use tracing::info;

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes: &[u8] = match endpoint {
            "article" => include_bytes!("../../tests/contracts/search/read/article/contract.json"),
            "bangumi" => include_bytes!("../../tests/contracts/search/read/bangumi/contract.json"),
            "bili-user" => {
                include_bytes!("../../tests/contracts/search/read/bili-user/contract.json")
            }
            "live" => include_bytes!("../../tests/contracts/search/read/live/contract.json"),
            "live-room" => {
                include_bytes!("../../tests/contracts/search/read/live-room/contract.json")
            }
            "live-user" => {
                include_bytes!("../../tests/contracts/search/read/live-user/contract.json")
            }
            "movie" => include_bytes!("../../tests/contracts/search/read/movie/contract.json"),
            "video" => include_bytes!("../../tests/contracts/search/read/video/contract.json"),
            _ => {
                return Err(BpiError::invalid_parameter(
                    "endpoint",
                    "unknown search typed contract",
                ));
            }
        };

        EndpointContract::from_slice(bytes)
    }

    fn fixture_payload<T>(bytes: &[u8]) -> BpiResult<SearchData<T>>
    where
        T: DeserializeOwned,
    {
        ApiEnvelope::<SearchData<T>>::from_slice(bytes)?.into_payload()
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/search/read/{endpoint}/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    fn parse_local_probe_outputs<T>(endpoint: &str) -> BpiResult<()>
    where
        T: DeserializeOwned,
    {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(endpoint, profile) else {
                continue;
            };

            let _payload =
                serde_json::from_value::<ApiEnvelope<SearchData<T>>>(body)?.into_payload()?;
        }

        Ok(())
    }

    #[test]
    fn typed_search_contracts_match_endpoint_requests() -> BpiResult<()> {
        let expectations = [
            (
                "article",
                "search.article",
                "article",
                "SearchData<Vec<Article>>",
            ),
            (
                "bangumi",
                "search.bangumi",
                "media_bangumi",
                "SearchData<Vec<Bangumi>>",
            ),
            (
                "bili-user",
                "search.bili_user",
                "bili_user",
                "SearchData<Vec<BiliUser>>",
            ),
            ("live", "search.live", "live", "SearchData<LiveData>"),
            (
                "live-room",
                "search.live_room",
                "live_room",
                "SearchData<Vec<LiveRoom>>",
            ),
            (
                "live-user",
                "search.live_user",
                "live_user",
                "SearchData<Vec<LiveUser>>",
            ),
            (
                "movie",
                "search.movie",
                "media_ft",
                "SearchData<Vec<Movie>>",
            ),
            ("video", "search.video", "video", "SearchData<Vec<Video>>"),
        ];

        for (endpoint, name, search_type, rust_model) in expectations {
            let contract = contract(endpoint)?;

            assert_eq!(contract.name, name);
            assert_eq!(contract.request.method, HttpMethod::Get);
            assert_eq!(
                contract.request.url.as_str(),
                "https://api.bilibili.com/x/web-interface/wbi/search/type"
            );
            assert_eq!(
                contract
                    .request
                    .query
                    .get("search_type")
                    .map(String::as_str),
                Some(search_type)
            );
            assert!(contract.request.auth.requires_wbi());
            assert_eq!(contract.cases.len(), 3);
            for case in &contract.cases {
                assert_eq!(case.response.http_status, Some(200));
                assert_eq!(case.response.api_code, Some(0));
                assert_eq!(case.response.rust_model.as_deref(), Some(rust_model));
            }
        }

        Ok(())
    }

    #[test]
    fn typed_search_response_fixtures_parse_declared_models() -> BpiResult<()> {
        let article = fixture_payload::<Vec<Article>>(include_bytes!(
            "../../tests/contracts/search/read/article/responses/success.json"
        ))?;
        let bangumi = fixture_payload::<Vec<Bangumi>>(include_bytes!(
            "../../tests/contracts/search/read/bangumi/responses/success.json"
        ))?;
        let bili_user = fixture_payload::<Vec<BiliUser>>(include_bytes!(
            "../../tests/contracts/search/read/bili-user/responses/success.json"
        ))?;
        let live = fixture_payload::<LiveData>(include_bytes!(
            "../../tests/contracts/search/read/live/responses/success.json"
        ))?;
        let live_room = fixture_payload::<Vec<LiveRoom>>(include_bytes!(
            "../../tests/contracts/search/read/live-room/responses/success.json"
        ))?;
        let live_user = fixture_payload::<Vec<LiveUser>>(include_bytes!(
            "../../tests/contracts/search/read/live-user/responses/success.json"
        ))?;
        let movie = fixture_payload::<Vec<Movie>>(include_bytes!(
            "../../tests/contracts/search/read/movie/responses/success.json"
        ))?;
        let video = fixture_payload::<Vec<Video>>(include_bytes!(
            "../../tests/contracts/search/read/video/responses/success.json"
        ))?;

        assert!(article.result.unwrap_or_default().is_empty());
        assert!(bangumi.result.unwrap_or_default().is_empty());
        assert!(bili_user.result.unwrap_or_default().is_empty());
        assert!(live.result.is_some());
        assert!(live_room.result.unwrap_or_default().is_empty());
        assert!(live_user.result.unwrap_or_default().is_empty());
        assert!(movie.result.unwrap_or_default().is_empty());
        assert!(video.result.unwrap_or_default().is_empty());
        Ok(())
    }

    #[test]
    fn typed_search_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        parse_local_probe_outputs::<Vec<Article>>("article")?;
        parse_local_probe_outputs::<Vec<Bangumi>>("bangumi")?;
        parse_local_probe_outputs::<Vec<BiliUser>>("bili-user")?;
        parse_local_probe_outputs::<LiveData>("live")?;
        parse_local_probe_outputs::<Vec<LiveRoom>>("live-room")?;
        parse_local_probe_outputs::<Vec<LiveUser>>("live-user")?;
        parse_local_probe_outputs::<Vec<Movie>>("movie")?;
        parse_local_probe_outputs::<Vec<Video>>("video")?;
        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_search_article() {
        let bpi = BpiClient::new().expect("client should build");
        let params = SearchArticleParams::new("Rust")
            .expect("keyword should be valid")
            .with_order(SearchOrder::PubDate)
            .with_category_id(CategoryId::Technology);
        let resp = bpi.search().article(params).await;
        assert!(resp.is_ok());
        if let Ok(data) = resp {
            info!("搜索文章返回: {:?}", data);
            if let Some(results) = data.result {
                assert!(!results.is_empty());
                for article in results {
                    info!("文章标题: {}", article.title);
                }
            } else {
                info!("未找到任何文章结果。");
            }
        }
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_search_bangumi() {
        let bpi = BpiClient::new().expect("client should build");
        let params = SearchBangumiParams::new("天气之子").expect("keyword should be valid");
        let resp = bpi.search().bangumi(params).await;
        assert!(resp.is_ok());
        if let Ok(data) = resp {
            info!("搜索番剧返回: {:?}", data);
            if let Some(results) = data.result {
                assert!(!results.is_empty());
                for bangumi in results {
                    info!("番剧标题: {}", bangumi.title);
                }
            } else {
                info!("未找到任何番剧结果。");
            }
        }
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_search_bili_user() {
        let bpi = BpiClient::new().expect("client should build");
        let params = SearchBiliUserParams::new("老番茄")
            .expect("keyword should be valid")
            .with_order_sort(OrderSort::Descending)
            .with_user_type(UserType::All);
        let resp = bpi.search().bili_user(params).await;
        assert!(resp.is_ok());
        if let Ok(data) = resp {
            info!("搜索用户返回: {:?}", data);
            if let Some(results) = data.result {
                assert!(!results.is_empty());
                for user in results {
                    info!("用户昵称: {}", user.uname);
                }
            } else {
                info!("未找到任何用户结果。");
            }
        }
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_search_live_room() {
        let bpi = BpiClient::new().expect("client should build");
        let params = SearchLiveRoomParams::new("游戏").expect("keyword should be valid");
        let resp = bpi.search().live_room(params).await;
        assert!(resp.is_ok());
        if let Ok(data) = resp {
            info!("搜索直播间返回: {:?}", data);
            if let Some(results) = data.result {
                assert!(!results.is_empty());
                for room in results {
                    info!("直播间标题: {}", room.title);
                }
            } else {
                info!("未找到任何直播间结果。");
            }
        }
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_search_live_user() {
        let bpi = BpiClient::new().expect("client should build");
        let params = SearchLiveUserParams::new("散人")
            .expect("keyword should be valid")
            .with_order_sort(OrderSort::Descending)
            .with_user_type(UserType::All);
        let resp = bpi.search().live_user(params).await;
        assert!(resp.is_ok());
        if let Ok(data) = resp {
            info!("搜索主播返回: {:?}", data);
            if let Some(results) = data.result {
                assert!(!results.is_empty());
                for user in results {
                    info!("主播昵称: {}", user.uname);
                }
            } else {
                info!("未找到任何主播结果。");
            }
        }
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_search_movie() {
        let bpi = BpiClient::new().expect("client should build");
        let params = SearchMovieParams::new("哈利波特").expect("keyword should be valid");
        let resp = bpi.search().movie(params).await;
        assert!(resp.is_ok());
        if let Ok(data) = resp {
            info!("搜索影视返回: {:?}", data);
            if let Some(results) = data.result {
                assert!(!results.is_empty());
                for movie in results {
                    info!("影视标题: {}", movie.title);
                }
            } else {
                info!("未找到任何影视结果。");
            }
        }
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_search_video() {
        let bpi = BpiClient::new().expect("client should build");
        let params = SearchVideoParams::new("Rust 教程")
            .expect("keyword should be valid")
            .with_order(SearchOrder::Online)
            .with_duration(Duration::From10To30)
            .with_tid(171);
        let resp = bpi.search().video(params).await;
        assert!(resp.is_ok());
        if let Ok(data) = resp {
            info!("搜索视频返回: {:?}", data);
            if let Some(results) = data.result {
                assert!(!results.is_empty());
                for video in results {
                    info!("视频标题: {}", video.title);
                }
            } else {
                info!("未找到任何视频结果。");
            }
        }
    }
}
