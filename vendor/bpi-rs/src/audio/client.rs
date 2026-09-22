use crate::audio::info::{AudioInfoData, AudioMemberType, AudioTag};
use crate::audio::music_list::{
    AudioCollection, AudioCollectionsListData, AudioHotMenuData, AudioRankMenuData,
};
use crate::audio::musicstream_url::{AudioStreamUrlData, AudioStreamUrlWebData};
use crate::audio::params::{
    AudioCollectionInfoParams, AudioPageParams, AudioRankListParams, AudioRankPeriodParams,
    AudioSongParams, AudioStreamUrlParams, AudioStreamUrlWebParams,
};
use crate::audio::rank::{AudioRankDetailData, AudioRankMusicListData, AudioRankPeriodData};
use crate::audio::status_number::AudioStatusNumberData;
use crate::{BilibiliRequest, BpiClient, BpiResult};

const INFO_ENDPOINT: &str = "https://www.bilibili.com/audio/music-service-c/web/song/info";
const TAGS_ENDPOINT: &str = "https://www.bilibili.com/audio/music-service-c/web/tag/song";
const MEMBERS_ENDPOINT: &str = "https://www.bilibili.com/audio/music-service-c/web/member/song";
const LYRIC_ENDPOINT: &str = "https://www.bilibili.com/audio/music-service-c/web/song/lyric";
const STATUS_NUMBER_ENDPOINT: &str = "https://www.bilibili.com/audio/music-service-c/web/stat/song";
const COLLECTION_STATUS_ENDPOINT: &str =
    "https://www.bilibili.com/audio/music-service-c/web/collections/songs-coll";
const COIN_COUNT_ENDPOINT: &str = "https://www.bilibili.com/audio/music-service-c/web/coin/audio";
const STREAM_URL_WEB_ENDPOINT: &str = "https://www.bilibili.com/audio/music-service-c/web/url";
const STREAM_URL_ENDPOINT: &str = "https://api.bilibili.com/audio/music-service-c/url";
const COLLECTIONS_LIST_ENDPOINT: &str =
    "https://www.bilibili.com/audio/music-service-c/web/collections/list";
const COLLECTION_INFO_ENDPOINT: &str =
    "https://www.bilibili.com/audio/music-service-c/web/collections/info";
const HOT_MENU_ENDPOINT: &str = "https://www.bilibili.com/audio/music-service-c/web/menu/hit";
const RANK_MENU_ENDPOINT: &str = "https://www.bilibili.com/audio/music-service-c/web/menu/rank";
const RANK_PERIOD_ENDPOINT: &str =
    "https://api.bilibili.com/x/copyright-music-publicity/toplist/all_period";
const RANK_DETAIL_ENDPOINT: &str =
    "https://api.bilibili.com/x/copyright-music-publicity/toplist/detail";
const RANK_MUSIC_LIST_ENDPOINT: &str =
    "https://api.bilibili.com/x/copyright-music-publicity/toplist/music_list";

/// 音频 API 客户端。
#[derive(Clone, Copy)]
pub struct AudioClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> AudioClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn info_endpoint(&self) -> &'static str {
        INFO_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn tags_endpoint(&self) -> &'static str {
        TAGS_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn members_endpoint(&self) -> &'static str {
        MEMBERS_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn lyric_endpoint(&self) -> &'static str {
        LYRIC_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn status_number_endpoint(&self) -> &'static str {
        STATUS_NUMBER_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn collection_status_endpoint(&self) -> &'static str {
        COLLECTION_STATUS_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn coin_count_endpoint(&self) -> &'static str {
        COIN_COUNT_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn stream_url_web_endpoint(&self) -> &'static str {
        STREAM_URL_WEB_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn stream_url_endpoint(&self) -> &'static str {
        STREAM_URL_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn collections_list_endpoint(&self) -> &'static str {
        COLLECTIONS_LIST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn collection_info_endpoint(&self) -> &'static str {
        COLLECTION_INFO_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn hot_menu_endpoint(&self) -> &'static str {
        HOT_MENU_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn rank_menu_endpoint(&self) -> &'static str {
        RANK_MENU_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn rank_period_endpoint(&self) -> &'static str {
        RANK_PERIOD_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn rank_detail_endpoint(&self) -> &'static str {
        RANK_DETAIL_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn rank_music_list_endpoint(&self) -> &'static str {
        RANK_MUSIC_LIST_ENDPOINT
    }

    /// 获取音频条目的基本信息。
    pub async fn info(&self, params: AudioSongParams) -> BpiResult<AudioInfoData> {
        self.client
            .get(INFO_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("audio.info")
            .await
    }

    /// 获取音频条目的标签。
    pub async fn tags(&self, params: AudioSongParams) -> BpiResult<Vec<AudioTag>> {
        self.client
            .get(TAGS_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("audio.tags")
            .await
    }

    /// 获取音频条目的创作成员。
    pub async fn members(&self, params: AudioSongParams) -> BpiResult<Vec<AudioMemberType>> {
        self.client
            .get(MEMBERS_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("audio.members")
            .await
    }

    /// 获取音频条目的歌词正文。
    pub async fn lyric(&self, params: AudioSongParams) -> BpiResult<String> {
        self.client
            .get(LYRIC_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("audio.lyric")
            .await
    }

    /// 获取音频条目的状态计数。
    pub async fn status_number(&self, params: AudioSongParams) -> BpiResult<AudioStatusNumberData> {
        self.client
            .get(STATUS_NUMBER_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("audio.status_number")
            .await
    }

    /// 获取当前账号是否已收藏某个音频条目。
    pub async fn collection_status(&self, params: AudioSongParams) -> BpiResult<bool> {
        self.client
            .get(COLLECTION_STATUS_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("audio.collection_status")
            .await
    }

    /// 获取当前账号对某个音频条目的投币数量。
    pub async fn coin_count(&self, params: AudioSongParams) -> BpiResult<i32> {
        self.client
            .get(COIN_COUNT_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("audio.coin_count")
            .await
    }

    /// 获取 Web 音频流 URL payload。
    pub async fn stream_url_web(
        &self,
        params: AudioStreamUrlWebParams,
    ) -> BpiResult<AudioStreamUrlWebData> {
        self.client
            .get(STREAM_URL_WEB_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("audio.stream_url_web")
            .await
    }

    /// 获取 app 风格的音频流 URL payload。
    pub async fn stream_url(&self, params: AudioStreamUrlParams) -> BpiResult<AudioStreamUrlData> {
        self.client
            .get(STREAM_URL_ENDPOINT)
            .with_bilibili_headers()
            .query(&params.query_pairs())
            .send_bpi_payload("audio.stream_url")
            .await
    }

    /// 获取当前账号创建的音频收藏夹。
    pub async fn collections_list(
        &self,
        params: AudioPageParams,
    ) -> BpiResult<AudioCollectionsListData> {
        self.client
            .get(COLLECTIONS_LIST_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("audio.collections_list")
            .await
    }

    /// 获取音频收藏夹信息。
    pub async fn collection_info(
        &self,
        params: AudioCollectionInfoParams,
    ) -> BpiResult<Option<AudioCollection>> {
        self.client
            .get(COLLECTION_INFO_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_optional_payload("audio.collection_info")
            .await
    }

    /// 获取热门音频收藏夹。
    pub async fn hot_menu(&self, params: AudioPageParams) -> BpiResult<AudioHotMenuData> {
        self.client
            .get(HOT_MENU_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("audio.hot_menu")
            .await
    }

    /// 获取音频榜单菜单。
    pub async fn rank_menu(&self, params: AudioPageParams) -> BpiResult<AudioRankMenuData> {
        self.client
            .get(RANK_MENU_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("audio.rank_menu")
            .await
    }

    /// 获取音频榜单可用期数。
    pub async fn rank_period(
        &self,
        params: AudioRankPeriodParams,
    ) -> BpiResult<AudioRankPeriodData> {
        let csrf = self.client.csrf().unwrap_or_default();

        self.client
            .get(RANK_PERIOD_ENDPOINT)
            .query(&params.query_pairs(&csrf))
            .send_bpi_payload("audio.rank_period")
            .await
    }

    /// 获取单个音频榜单期数详情。
    pub async fn rank_detail(&self, params: AudioRankListParams) -> BpiResult<AudioRankDetailData> {
        let csrf = self.client.csrf().unwrap_or_default();

        self.client
            .get(RANK_DETAIL_ENDPOINT)
            .query(&params.query_pairs(&csrf))
            .send_bpi_payload("audio.rank_detail")
            .await
    }

    /// 获取单个音频榜单期数的音乐条目。
    pub async fn rank_music_list(
        &self,
        params: AudioRankListParams,
    ) -> BpiResult<AudioRankMusicListData> {
        let csrf = self.client.csrf().unwrap_or_default();

        self.client
            .get(RANK_MUSIC_LIST_ENDPOINT)
            .query(&params.query_pairs(&csrf))
            .send_bpi_payload("audio.rank_music_list")
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::audio::info::{AudioInfoData, AudioMemberType, AudioTag};
    use crate::audio::music_list::{
        AudioCollection, AudioCollectionsListData, AudioHotMenuData, AudioRankMenuData,
    };
    use crate::audio::musicstream_url::{AudioQuality, AudioStreamUrlData, AudioStreamUrlWebData};
    use crate::audio::params::{
        AudioCollectionInfoParams, AudioPageParams, AudioRankListParams, AudioRankListType,
        AudioRankPeriodParams, AudioSongParams, AudioStreamUrlParams, AudioStreamUrlWebParams,
    };
    use crate::audio::rank::{AudioRankDetailData, AudioRankMusicListData, AudioRankPeriodData};
    use crate::audio::status_number::AudioStatusNumberData;
    use crate::ids::AudioId;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{BpiClient, BpiResult};

    fn assert_info_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<AudioInfoData>>,
    {
    }

    fn assert_tags_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<Vec<AudioTag>>>,
    {
    }

    fn assert_members_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<Vec<AudioMemberType>>>,
    {
    }

    fn assert_lyric_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<String>>,
    {
    }

    fn assert_status_number_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<AudioStatusNumberData>>,
    {
    }

    fn assert_bool_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<bool>>,
    {
    }

    fn assert_coin_count_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<i32>>,
    {
    }

    fn assert_stream_url_web_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<AudioStreamUrlWebData>>,
    {
    }

    fn assert_stream_url_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<AudioStreamUrlData>>,
    {
    }

    fn assert_collections_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<AudioCollectionsListData>>,
    {
    }

    fn assert_collection_info_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<Option<AudioCollection>>>,
    {
    }

    fn assert_hot_menu_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<AudioHotMenuData>>,
    {
    }

    fn assert_rank_menu_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<AudioRankMenuData>>,
    {
    }

    fn assert_rank_period_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<AudioRankPeriodData>>,
    {
    }

    fn assert_rank_detail_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<AudioRankDetailData>>,
    {
    }

    fn assert_rank_music_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<AudioRankMusicListData>>,
    {
    }

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "info" => include_bytes!("../../tests/contracts/audio/info/contract.json").as_slice(),
            "tags" => include_bytes!("../../tests/contracts/audio/tags/contract.json").as_slice(),
            "members" => {
                include_bytes!("../../tests/contracts/audio/members/contract.json").as_slice()
            }
            "lyric" => include_bytes!("../../tests/contracts/audio/lyric/contract.json").as_slice(),
            "status-number" => {
                include_bytes!("../../tests/contracts/audio/status-number/contract.json").as_slice()
            }
            "collection-status" => {
                include_bytes!("../../tests/contracts/audio/collection-status/contract.json")
                    .as_slice()
            }
            "coin-count" => {
                include_bytes!("../../tests/contracts/audio/coin-count/contract.json").as_slice()
            }
            "stream-url-web" => {
                include_bytes!("../../tests/contracts/audio/stream-url-web/contract.json")
                    .as_slice()
            }
            "stream-url" => {
                include_bytes!("../../tests/contracts/audio/stream-url/contract.json").as_slice()
            }
            "collections-list" => {
                include_bytes!("../../tests/contracts/audio/collections-list/contract.json")
                    .as_slice()
            }
            "collection-info" => {
                include_bytes!("../../tests/contracts/audio/collection-info/contract.json")
                    .as_slice()
            }
            "hot-menu" => {
                include_bytes!("../../tests/contracts/audio/hot-menu/contract.json").as_slice()
            }
            "rank-menu" => {
                include_bytes!("../../tests/contracts/audio/rank-menu/contract.json").as_slice()
            }
            "rank-period" => {
                include_bytes!("../../tests/contracts/audio/rank-period/contract.json").as_slice()
            }
            "rank-detail" => {
                include_bytes!("../../tests/contracts/audio/rank-detail/contract.json").as_slice()
            }
            "rank-music-list" => {
                include_bytes!("../../tests/contracts/audio/rank-music-list/contract.json")
                    .as_slice()
            }
            _ => unreachable!("unknown audio contract"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn audio_client_exposes_promoted_endpoint_urls() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let audio = client.audio();

        assert_eq!(
            audio.info_endpoint(),
            "https://www.bilibili.com/audio/music-service-c/web/song/info"
        );
        assert_eq!(
            audio.tags_endpoint(),
            "https://www.bilibili.com/audio/music-service-c/web/tag/song"
        );
        assert_eq!(
            audio.members_endpoint(),
            "https://www.bilibili.com/audio/music-service-c/web/member/song"
        );
        assert_eq!(
            audio.lyric_endpoint(),
            "https://www.bilibili.com/audio/music-service-c/web/song/lyric"
        );
        assert_eq!(
            audio.status_number_endpoint(),
            "https://www.bilibili.com/audio/music-service-c/web/stat/song"
        );
        assert_eq!(
            audio.collection_status_endpoint(),
            "https://www.bilibili.com/audio/music-service-c/web/collections/songs-coll"
        );
        assert_eq!(
            audio.coin_count_endpoint(),
            "https://www.bilibili.com/audio/music-service-c/web/coin/audio"
        );
        assert_eq!(
            audio.stream_url_web_endpoint(),
            "https://www.bilibili.com/audio/music-service-c/web/url"
        );
        assert_eq!(
            audio.stream_url_endpoint(),
            "https://api.bilibili.com/audio/music-service-c/url"
        );
        assert_eq!(
            audio.collections_list_endpoint(),
            "https://www.bilibili.com/audio/music-service-c/web/collections/list"
        );
        assert_eq!(
            audio.collection_info_endpoint(),
            "https://www.bilibili.com/audio/music-service-c/web/collections/info"
        );
        assert_eq!(
            audio.hot_menu_endpoint(),
            "https://www.bilibili.com/audio/music-service-c/web/menu/hit"
        );
        assert_eq!(
            audio.rank_menu_endpoint(),
            "https://www.bilibili.com/audio/music-service-c/web/menu/rank"
        );
        assert_eq!(
            audio.rank_period_endpoint(),
            "https://api.bilibili.com/x/copyright-music-publicity/toplist/all_period"
        );
        assert_eq!(
            audio.rank_detail_endpoint(),
            "https://api.bilibili.com/x/copyright-music-publicity/toplist/detail"
        );
        assert_eq!(
            audio.rank_music_list_endpoint(),
            "https://api.bilibili.com/x/copyright-music-publicity/toplist/music_list"
        );
        Ok(())
    }

    #[test]
    fn audio_methods_return_payload_futures() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let audio = client.audio();
        let sid = AudioId::new(13603)?;
        let stream_sid = AudioId::new(15664)?;

        assert_info_future(audio.info(AudioSongParams::new(sid)));
        assert_tags_future(audio.tags(AudioSongParams::new(sid)));
        assert_members_future(audio.members(AudioSongParams::new(sid)));
        assert_lyric_future(audio.lyric(AudioSongParams::new(sid)));
        assert_status_number_future(audio.status_number(AudioSongParams::new(sid)));
        assert_bool_future(audio.collection_status(AudioSongParams::new(sid)));
        assert_coin_count_future(audio.coin_count(AudioSongParams::new(sid)));
        assert_stream_url_web_future(audio.stream_url_web(AudioStreamUrlWebParams::new(sid)));
        assert_stream_url_future(audio.stream_url(AudioStreamUrlParams::new(
            stream_sid,
            AudioQuality::HighQuality,
        )));
        assert_collections_list_future(audio.collections_list(AudioPageParams::new(1, 2)?));
        assert_collection_info_future(
            audio.collection_info(AudioCollectionInfoParams::new(15_967_839)?),
        );
        assert_hot_menu_future(audio.hot_menu(AudioPageParams::new(1, 3)?));
        assert_rank_menu_future(audio.rank_menu(AudioPageParams::new(1, 6)?));
        assert_rank_period_future(
            audio.rank_period(AudioRankPeriodParams::new(AudioRankListType::Original)),
        );
        assert_rank_detail_future(audio.rank_detail(AudioRankListParams::new(76)?));
        assert_rank_music_list_future(audio.rank_music_list(AudioRankListParams::new(76)?));
        Ok(())
    }

    #[test]
    fn audio_contracts_match_module_client_endpoints() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let audio = client.audio();

        let expectations = [
            ("info", "audio.info", audio.info_endpoint()),
            ("tags", "audio.tags", audio.tags_endpoint()),
            ("members", "audio.members", audio.members_endpoint()),
            ("lyric", "audio.lyric", audio.lyric_endpoint()),
            (
                "status-number",
                "audio.status_number",
                audio.status_number_endpoint(),
            ),
            (
                "collection-status",
                "audio.collection_status",
                audio.collection_status_endpoint(),
            ),
            (
                "coin-count",
                "audio.coin_count",
                audio.coin_count_endpoint(),
            ),
            (
                "stream-url-web",
                "audio.stream_url_web",
                audio.stream_url_web_endpoint(),
            ),
            (
                "stream-url",
                "audio.stream_url",
                audio.stream_url_endpoint(),
            ),
            (
                "collections-list",
                "audio.collections_list",
                audio.collections_list_endpoint(),
            ),
            (
                "collection-info",
                "audio.collection_info",
                audio.collection_info_endpoint(),
            ),
            ("hot-menu", "audio.hot_menu", audio.hot_menu_endpoint()),
            ("rank-menu", "audio.rank_menu", audio.rank_menu_endpoint()),
            (
                "rank-period",
                "audio.rank_period",
                audio.rank_period_endpoint(),
            ),
            (
                "rank-detail",
                "audio.rank_detail",
                audio.rank_detail_endpoint(),
            ),
            (
                "rank-music-list",
                "audio.rank_music_list",
                audio.rank_music_list_endpoint(),
            ),
        ];

        for (endpoint, name, url) in expectations {
            let contract = contract(endpoint)?;

            assert_eq!(contract.name, name);
            assert_eq!(contract.request.method, HttpMethod::Get);
            assert_eq!(contract.request.url.as_str(), url);
        }
        Ok(())
    }
}
