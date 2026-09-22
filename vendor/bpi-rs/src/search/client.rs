use crate::search::hot::{DefaultSearchData, HotWordDataResponse};
use crate::search::result::{
    Article, Bangumi, BiliUser, LiveData, LiveRoom, LiveUser, Movie, SearchData, Video,
};
use crate::search::search_params::{
    SearchArticleParams, SearchBangumiParams, SearchBiliUserParams, SearchLiveParams,
    SearchLiveRoomParams, SearchLiveUserParams, SearchMovieParams, SearchVideoParams,
};
use crate::search::suggest::{SearchSuggest, SearchSuggestParams};
use crate::{BilibiliRequest, BpiClient, BpiResult};

const TYPED_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/wbi/search/type";
const DEFAULT_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/wbi/search/default";
const SUGGEST_ENDPOINT: &str = "https://s.search.bilibili.com/main/suggest";
const HOTWORDS_ENDPOINT: &str = "https://s.search.bilibili.com/main/hotword";

/// 搜索 API 客户端。
#[derive(Clone, Copy)]
pub struct SearchClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> SearchClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn typed_endpoint(&self) -> &'static str {
        TYPED_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn default_endpoint(&self) -> &'static str {
        DEFAULT_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn suggest_endpoint(&self) -> &'static str {
        SUGGEST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn hotwords_endpoint(&self) -> &'static str {
        HOTWORDS_ENDPOINT
    }

    /// 搜索专栏结果。
    pub async fn article(
        &self,
        params: SearchArticleParams,
    ) -> BpiResult<SearchData<Vec<Article>>> {
        self.typed_search(params.query_pairs(), "search.article")
            .await
    }

    /// 搜索番剧结果。
    pub async fn bangumi(
        &self,
        params: SearchBangumiParams,
    ) -> BpiResult<SearchData<Vec<Bangumi>>> {
        self.typed_search(params.query_pairs(), "search.bangumi")
            .await
    }

    /// 搜索 Bilibili 用户结果。
    pub async fn bili_user(
        &self,
        params: SearchBiliUserParams,
    ) -> BpiResult<SearchData<Vec<BiliUser>>> {
        self.typed_search(params.query_pairs(), "search.bili_user")
            .await
    }

    /// 搜索合并的直播间和直播用户结果。
    pub async fn live(&self, params: SearchLiveParams) -> BpiResult<SearchData<LiveData>> {
        self.typed_search(params.query_pairs(), "search.live").await
    }

    /// 搜索直播间结果。
    pub async fn live_room(
        &self,
        params: SearchLiveRoomParams,
    ) -> BpiResult<SearchData<Vec<LiveRoom>>> {
        self.typed_search(params.query_pairs(), "search.live_room")
            .await
    }

    /// 搜索直播用户结果。
    pub async fn live_user(
        &self,
        params: SearchLiveUserParams,
    ) -> BpiResult<SearchData<Vec<LiveUser>>> {
        self.typed_search(params.query_pairs(), "search.live_user")
            .await
    }

    /// 搜索影视结果。
    pub async fn movie(&self, params: SearchMovieParams) -> BpiResult<SearchData<Vec<Movie>>> {
        self.typed_search(params.query_pairs(), "search.movie")
            .await
    }

    /// 搜索视频结果。
    pub async fn video(&self, params: SearchVideoParams) -> BpiResult<SearchData<Vec<Video>>> {
        self.typed_search(params.query_pairs(), "search.video")
            .await
    }

    /// 获取默认 Web 搜索内容。
    pub async fn default(&self) -> BpiResult<DefaultSearchData> {
        let signed_params = self.client.get_wbi_sign2(vec![("foo", "bar")]).await?;

        self.client
            .get(DEFAULT_ENDPOINT)
            .query(&signed_params)
            .send_bpi_payload("search.default")
            .await
    }

    /// 获取搜索词建议。
    pub async fn suggest(&self, params: SearchSuggestParams) -> BpiResult<SearchSuggest> {
        self.client
            .get(SUGGEST_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("search.suggest")
            .await
    }

    /// 获取 Web 热词列表。
    pub async fn hotwords(&self) -> BpiResult<HotWordDataResponse> {
        let response = self.client.get(HOTWORDS_ENDPOINT).send().await?;

        Ok(response.json().await?)
    }

    async fn typed_search<T>(
        &self,
        query_pairs: Vec<(&'static str, String)>,
        endpoint_label: &'static str,
    ) -> BpiResult<SearchData<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let signed_params = self.client.get_wbi_sign2(query_pairs).await?;

        self.client
            .get(TYPED_ENDPOINT)
            .query(&signed_params)
            .send_bpi_payload(endpoint_label)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::search::hot::{DefaultSearchData, HotWordDataResponse};
    use crate::search::result::{
        Article, Bangumi, BiliUser, LiveData, LiveRoom, LiveUser, Movie, SearchData, Video,
    };
    use crate::search::search_params::{
        SearchArticleParams, SearchBangumiParams, SearchBiliUserParams, SearchLiveParams,
        SearchLiveRoomParams, SearchLiveUserParams, SearchMovieParams, SearchVideoParams,
    };
    use crate::search::suggest::{SearchSuggest, SearchSuggestParams};
    use crate::{BpiClient, BpiResult};

    fn assert_article_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<SearchData<Vec<Article>>>>,
    {
    }

    fn assert_bangumi_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<SearchData<Vec<Bangumi>>>>,
    {
    }

    fn assert_bili_user_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<SearchData<Vec<BiliUser>>>>,
    {
    }

    fn assert_live_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<SearchData<LiveData>>>,
    {
    }

    fn assert_live_room_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<SearchData<Vec<LiveRoom>>>>,
    {
    }

    fn assert_live_user_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<SearchData<Vec<LiveUser>>>>,
    {
    }

    fn assert_movie_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<SearchData<Vec<Movie>>>>,
    {
    }

    fn assert_video_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<SearchData<Vec<Video>>>>,
    {
    }

    fn assert_default_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<DefaultSearchData>>,
    {
    }

    fn assert_suggest_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<SearchSuggest>>,
    {
    }

    fn assert_hotwords_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<HotWordDataResponse>>,
    {
    }

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "article" => {
                include_bytes!("../../tests/contracts/search/read/article/contract.json").as_slice()
            }
            "bangumi" => {
                include_bytes!("../../tests/contracts/search/read/bangumi/contract.json").as_slice()
            }
            "bili-user" => {
                include_bytes!("../../tests/contracts/search/read/bili-user/contract.json")
                    .as_slice()
            }
            "live" => {
                include_bytes!("../../tests/contracts/search/read/live/contract.json").as_slice()
            }
            "live-room" => {
                include_bytes!("../../tests/contracts/search/read/live-room/contract.json")
                    .as_slice()
            }
            "live-user" => {
                include_bytes!("../../tests/contracts/search/read/live-user/contract.json")
                    .as_slice()
            }
            "movie" => {
                include_bytes!("../../tests/contracts/search/read/movie/contract.json").as_slice()
            }
            "video" => {
                include_bytes!("../../tests/contracts/search/read/video/contract.json").as_slice()
            }
            "default" => {
                include_bytes!("../../tests/contracts/search/read/default/contract.json").as_slice()
            }
            "suggest" => {
                include_bytes!("../../tests/contracts/search/read/suggest/contract.json").as_slice()
            }
            "hotwords" => {
                include_bytes!("../../tests/contracts/search/read/hotwords/contract.json")
                    .as_slice()
            }
            _ => unreachable!("unknown search contract"),
        };
        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn search_client_exposes_promoted_endpoint_urls() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let search = client.search();

        assert_eq!(
            search.typed_endpoint(),
            "https://api.bilibili.com/x/web-interface/wbi/search/type"
        );
        assert_eq!(
            search.default_endpoint(),
            "https://api.bilibili.com/x/web-interface/wbi/search/default"
        );
        assert_eq!(
            search.suggest_endpoint(),
            "https://s.search.bilibili.com/main/suggest"
        );
        assert_eq!(
            search.hotwords_endpoint(),
            "https://s.search.bilibili.com/main/hotword"
        );
        Ok(())
    }

    #[test]
    fn search_methods_return_payload_futures() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let search = client.search();

        assert_article_future(search.article(SearchArticleParams::new("rust")?));
        assert_bangumi_future(search.bangumi(SearchBangumiParams::new("天气之子")?));
        assert_bili_user_future(search.bili_user(SearchBiliUserParams::new("老番茄")?));
        assert_live_future(search.live(SearchLiveParams::new("游戏")?));
        assert_live_room_future(search.live_room(SearchLiveRoomParams::new("游戏")?));
        assert_live_user_future(search.live_user(SearchLiveUserParams::new("散人")?));
        assert_movie_future(search.movie(SearchMovieParams::new("哈利波特")?));
        assert_video_future(search.video(SearchVideoParams::new("rust")?));
        assert_default_future(search.default());
        assert_suggest_future(search.suggest(SearchSuggestParams::new("rust")?));
        assert_hotwords_future(search.hotwords());
        Ok(())
    }

    #[test]
    fn search_contracts_match_module_client_endpoints() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let search = client.search();

        let typed_expectations = [
            ("article", "search.article"),
            ("bangumi", "search.bangumi"),
            ("bili-user", "search.bili_user"),
            ("live", "search.live"),
            ("live-room", "search.live_room"),
            ("live-user", "search.live_user"),
            ("movie", "search.movie"),
            ("video", "search.video"),
        ];

        for (endpoint, name) in typed_expectations {
            let contract = contract(endpoint)?;

            assert_eq!(contract.name, name);
            assert_eq!(contract.request.method, HttpMethod::Get);
            assert_eq!(contract.request.url.as_str(), search.typed_endpoint());
            assert!(contract.request.auth.requires_wbi());
        }

        let default = contract("default")?;
        assert_eq!(default.name, "search.default");
        assert_eq!(default.request.method, HttpMethod::Get);
        assert_eq!(default.request.url.as_str(), search.default_endpoint());
        assert!(default.request.auth.requires_wbi());

        let suggest = contract("suggest")?;
        assert_eq!(suggest.name, "search.suggest");
        assert_eq!(suggest.request.method, HttpMethod::Get);
        assert_eq!(suggest.request.url.as_str(), search.suggest_endpoint());
        assert!(!suggest.request.auth.requires_wbi());

        let hotwords = contract("hotwords")?;
        assert_eq!(hotwords.name, "search.hotwords");
        assert_eq!(hotwords.request.method, HttpMethod::Get);
        assert_eq!(hotwords.request.url.as_str(), search.hotwords_endpoint());
        assert!(!hotwords.request.auth.requires_wbi());
        Ok(())
    }
}
