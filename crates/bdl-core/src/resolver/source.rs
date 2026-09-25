use std::sync::Arc;
use std::time::Duration;

use bpi_rs::BpiClient;

use crate::error::{BdlError, BdlResult};
use crate::ids::ItemId;
use crate::input::{ClassifiedInput, classify_input};
use crate::model::{NormalizedItem, NormalizedSourceTree, SourceKind};
use crate::resolver::Resolver;
use crate::resolver::bangumi::BangumiResolver;
use crate::resolver::cheese::CheeseResolver;
use crate::resolver::collection::{
    CollectionResolver, SeriesResolver, collection_ids_from_url, series_ids_from_url,
};
use crate::resolver::favorite::{FavoriteResolver, favorite_media_id_from_url};
use crate::resolver::paged::{PageRequest, append_source_page};
use crate::resolver::uploader::UploaderResolver;
use crate::resolver::video::VideoResolver;

use super::ResolveOptions;
use super::pacing::{ResolveGate, ResolvePolicy};

#[derive(Clone)]
pub struct SourceResolver {
    client: Arc<BpiClient>,
    gate: Arc<ResolveGate>,
    http_guard: Option<Arc<super::http_guard::HttpGuard>>,
}

impl SourceResolver {
    pub fn with_http_guard(mut self, guard: Arc<super::http_guard::HttpGuard>) -> Self {
        self.http_guard = Some(guard);
        self
    }
    async fn controlled<T>(
        &self,
        operation: impl std::future::Future<Output = BdlResult<T>>,
    ) -> BdlResult<T> {
        let operation = self.gate.run(operation);
        if let Some(guard) = &self.http_guard {
            bpi_rs::transport::observer::scope(guard.clone(), operation).await
        } else {
            operation.await
        }
    }

    pub fn with_policy(mut self, policy: ResolvePolicy) -> Self {
        self.gate = Arc::new(ResolveGate::new(policy));
        self
    }
    /// Load metadata until the requested number of items is available, or all pages end.
    pub async fn load_until(
        &self,
        tree: &mut NormalizedSourceTree,
        limit: Option<usize>,
    ) -> BdlResult<()> {
        while tree.source.has_more && limit.is_none_or(|limit| tree.source.loaded_count < limit) {
            self.load_more(tree).await?;
        }
        Ok(())
    }

    pub async fn load_more(&self, tree: &mut NormalizedSourceTree) -> BdlResult<()> {
        if !tree.source.has_more {
            return Ok(());
        }
        self.controlled(self.load_more_unpaced(tree)).await
    }

    async fn load_more_unpaced(&self, tree: &mut NormalizedSourceTree) -> BdlResult<()> {
        let page = tree
            .groups
            .iter()
            .find_map(|group| group.page.as_ref())
            .ok_or_else(|| source_error("source has no page state"))?;
        let request = PageRequest {
            page_number: page
                .page_number
                .checked_add(1)
                .ok_or_else(|| source_error("page number overflow"))?,
            page_size: page.page_size,
        };
        let input = classify_input(&tree.source.input)?;
        let next = match input {
            ClassifiedInput::Favorite { raw_url } => {
                FavoriteResolver::from_bpi_client(self.client.clone())
                    .resolve_page(favorite_media_id_from_url(&raw_url)?, request)
                    .await?
            }
            ClassifiedInput::Uploader { mid } => {
                UploaderResolver::from_bpi_client(self.client.clone())
                    .resolve_page(mid, request)
                    .await?
            }
            ClassifiedInput::Collection { raw_url } => {
                CollectionResolver::from_bpi_client(self.client.clone())
                    .resolve_page(collection_ids_from_url(&raw_url)?, request)
                    .await?
            }
            ClassifiedInput::Series { raw_url } => {
                SeriesResolver::from_bpi_client(self.client.clone())
                    .resolve_page(series_ids_from_url(&raw_url)?, request)
                    .await?
            }
            ClassifiedInput::Cheese { .. } => {
                let season_id = tree
                    .source
                    .id
                    .0
                    .strip_prefix("cheese:")
                    .and_then(|id| id.parse().ok())
                    .ok_or_else(|| source_error("course has no season ID"))?;
                CheeseResolver::from_bpi_client(self.client.clone())
                    .resolve_page(season_id, request)
                    .await?
            }
            _ => return Err(source_error("source does not support paging")),
        };
        append_source_page(tree, next)
    }

    /// Hydrate one list item immediately before planning its download.
    pub async fn expand_item(
        &self,
        tree: &mut NormalizedSourceTree,
        item_id: &ItemId,
    ) -> BdlResult<()> {
        let item = tree
            .groups
            .iter()
            .flat_map(|g| &g.items)
            .find(|i| &i.id == item_id)
            .ok_or_else(|| source_error("item missing"))?;
        let input = item_media_input(tree.source.kind, item)?;
        let loaded = self
            .resolve(
                &input,
                ResolveOptions {
                    fetch_streams: false,
                },
            )
            .await?;
        let item = tree
            .groups
            .iter_mut()
            .flat_map(|g| &mut g.items)
            .find(|i| &i.id == item_id)
            .ok_or_else(|| source_error("item missing"))?;
        merge_item_parts(item, loaded, tree.source.kind, false)
    }

    pub async fn hydrate_item(
        &self,
        tree: &mut NormalizedSourceTree,
        item_id: &ItemId,
    ) -> BdlResult<()> {
        let item = tree
            .groups
            .iter()
            .flat_map(|group| &group.items)
            .find(|item| &item.id == item_id)
            .ok_or_else(|| source_error("selected item was not found"))?;
        let input = item_media_input(tree.source.kind, item)?;
        let hydrated = self
            .resolve(
                &input,
                ResolveOptions {
                    fetch_streams: true,
                },
            )
            .await?;
        let item = tree
            .groups
            .iter_mut()
            .flat_map(|group| &mut group.items)
            .find(|item| &item.id == item_id)
            .ok_or_else(|| source_error("selected item was not found"))?;
        merge_item_media(item, hydrated, tree.source.kind)
    }

    pub fn new() -> BdlResult<Self> {
        Self::from_optional_cookie(None)
    }

    pub fn from_cookie(cookie: &str) -> BdlResult<Self> {
        Self::from_optional_cookie(Some(cookie))
    }

    pub fn from_optional_cookie(cookie: Option<&str>) -> BdlResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .build()
            .map_err(|error| BdlError::Bpi(error.to_string()))?;
        let mut builder = BpiClient::builder().reqwest_client(http);
        if let Some(cookie) = cookie.filter(|value| !value.trim().is_empty()) {
            builder = builder.cookie(cookie);
        }
        let client = builder
            .build()
            .map_err(|error| BdlError::Bpi(error.to_string()))?;
        Ok(Self::from_bpi_client(client))
    }

    pub fn from_bpi_client(client: impl Into<Arc<BpiClient>>) -> Self {
        Self {
            client: client.into(),
            gate: Arc::new(ResolveGate::new(ResolvePolicy::default())),
            http_guard: None,
        }
    }

    pub async fn resolve(
        &self,
        input: &str,
        options: ResolveOptions,
    ) -> BdlResult<NormalizedSourceTree> {
        self.resolve_classified(classify_input(input)?, options)
            .await
    }

    pub async fn resolve_classified(
        &self,
        input: ClassifiedInput,
        options: ResolveOptions,
    ) -> BdlResult<NormalizedSourceTree> {
        let input = if let ClassifiedInput::ShortUrl(url) = input {
            self.expand_short_url(&url).await?
        } else {
            input
        };
        if options.fetch_streams && input.source_kind() == SourceKind::Video {
            let mut tree = self
                .controlled(self.resolve_unpaced(
                    input,
                    ResolveOptions {
                        fetch_streams: false,
                    },
                ))
                .await?;
            let source_input = tree.source.input.clone();
            for part in tree
                .groups
                .iter_mut()
                .flat_map(|group| &mut group.items)
                .flat_map(|item| &mut item.parts)
            {
                let cid = part
                    .cid
                    .ok_or_else(|| source_error("video part has no CID"))?;
                let loaded = self
                    .resolve_video_target_streams(&source_input, cid)
                    .await?;
                *part = loaded
                    .groups
                    .into_iter()
                    .flat_map(|group| group.items)
                    .flat_map(|item| item.parts)
                    .find(|candidate| candidate.id == part.id)
                    .ok_or_else(|| source_error("video part disappeared"))?;
            }
            return Ok(tree);
        }
        self.controlled(self.resolve_unpaced(input, options)).await
    }

    async fn resolve_unpaced(
        &self,
        input: ClassifiedInput,
        options: ResolveOptions,
    ) -> BdlResult<NormalizedSourceTree> {
        match input.source_kind() {
            SourceKind::Video => {
                VideoResolver::from_bpi_client(self.client.clone())
                    .resolve(input, options)
                    .await
            }
            SourceKind::Favorite => {
                FavoriteResolver::from_bpi_client(self.client.clone())
                    .resolve(input, options)
                    .await
            }
            SourceKind::Uploader => {
                UploaderResolver::from_bpi_client(self.client.clone())
                    .resolve(input, options)
                    .await
            }
            SourceKind::Collection => {
                CollectionResolver::from_bpi_client(self.client.clone())
                    .resolve(input, options)
                    .await
            }
            SourceKind::Series => {
                SeriesResolver::from_bpi_client(self.client.clone())
                    .resolve(input, options)
                    .await
            }
            SourceKind::Bangumi => {
                BangumiResolver::from_bpi_client(self.client.clone())
                    .resolve(input, options)
                    .await
            }
            SourceKind::Cheese => {
                CheeseResolver::from_bpi_client(self.client.clone())
                    .resolve(input, options)
                    .await
            }
            SourceKind::Unknown => {
                VideoResolver::from_bpi_client(self.client.clone())
                    .resolve(input, options)
                    .await
            }
        }
    }

    pub async fn resolve_video_target_streams(
        &self,
        input: &str,
        cid: u64,
    ) -> BdlResult<NormalizedSourceTree> {
        self.controlled(async {
            VideoResolver::from_bpi_client(self.client.clone())
                .resolve_target_streams(classify_input(input)?, cid)
                .await
        })
        .await
    }

    /// Expand a share link before dispatching to a source-specific resolver.
    pub async fn expand_short_url(&self, input: &str) -> BdlResult<ClassifiedInput> {
        use bpi_rs::transport::observer::RequestObserver;
        let client = reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(20))
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .build()
            .map_err(|error| BdlError::Bpi(error.to_string()))?;
        let mut url = url::Url::parse(input).map_err(|_| source_error("invalid short link"))?;
        url.set_scheme("https")
            .map_err(|_| source_error("invalid short link scheme"))?;
        for _ in 0..5 {
            validate_redirect_target(&url)?;
            if let Ok(classified) = classify_input(url.as_str())
                && !matches!(classified, ClassifiedInput::ShortUrl(_))
            {
                return Ok(classified);
            }
            if !matches!(url.host_str(), Some("b23.tv" | "www.b23.tv")) {
                return Err(source_error(
                    "short link did not resolve to a supported source",
                ));
            }
            let mut permit = if let Some(guard) = &self.http_guard {
                Some(
                    guard
                        .before()
                        .await
                        .map_err(|error| BdlError::Bpi(error.to_string()))?,
                )
            } else {
                None
            };
            let response = client
                .get(url.clone())
                .send()
                .await
                .map_err(|error| BdlError::Bpi(error.to_string()))?;
            if let Some(permit) = &mut permit {
                permit
                    .observe(response.status().as_u16(), response.headers(), &[])
                    .map_err(|error| BdlError::Bpi(error.to_string()))?;
            }
            if !response.status().is_redirection() {
                return Err(source_error(
                    "short link requires an HTTP redirect; verification pages are not followed",
                ));
            }
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| source_error("short link has no valid redirect target"))?;
            url = url
                .join(location)
                .map_err(|_| source_error("invalid redirect URL"))?;
        }
        validate_redirect_target(&url)?;
        let classified = classify_input(url.as_str())?;
        if matches!(classified, ClassifiedInput::ShortUrl(_)) {
            return Err(source_error("short link exceeded five redirects"));
        }
        Ok(classified)
    }
}

fn validate_redirect_target(url: &url::Url) -> BdlResult<()> {
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port_or_known_default() != Some(443)
        || !matches!(
            url.host_str(),
            Some(
                "b23.tv"
                    | "www.b23.tv"
                    | "bilibili.com"
                    | "www.bilibili.com"
                    | "m.bilibili.com"
                    | "space.bilibili.com"
            )
        )
    {
        return Err(source_error(
            "short link redirected outside supported Bilibili HTTPS hosts",
        ));
    }
    Ok(())
}

fn source_error(message: &str) -> BdlError {
    BdlError::Planning {
        message: message.to_owned(),
    }
}

fn item_media_input(kind: SourceKind, item: &NormalizedItem) -> BdlResult<String> {
    let part = item
        .parts
        .first()
        .ok_or_else(|| source_error("item has no parts"))?;
    if matches!(kind, SourceKind::Bangumi | SourceKind::Cheese) {
        let prefix = if kind == SourceKind::Bangumi {
            "bangumi"
        } else {
            "cheese"
        };
        let mut segments = part.id.0.split(':');
        if segments.next() != Some("part") || segments.next() != Some(prefix) {
            return Err(source_error("episode has an invalid part ID"));
        }
        let episode = segments
            .nth(1)
            .and_then(|id| id.parse::<u64>().ok())
            .filter(|id| *id > 0)
            .ok_or_else(|| source_error("episode has no episode ID"))?;
        return Ok(format!(
            "https://www.bilibili.com/{prefix}/play/ep{episode}"
        ));
    }
    part.bvid
        .clone()
        .filter(|id| !id.is_empty())
        .or_else(|| part.aid.filter(|id| *id > 0).map(|id| format!("av{id}")))
        .ok_or_else(|| source_error("list item has no BV/AV identity (it may be unavailable)"))
}

fn merge_item_media(
    item: &mut NormalizedItem,
    hydrated: NormalizedSourceTree,
    kind: SourceKind,
) -> BdlResult<()> {
    merge_item_parts(item, hydrated, kind, true)
}

fn merge_item_parts(
    item: &mut NormalizedItem,
    hydrated: NormalizedSourceTree,
    kind: SourceKind,
    require_streams: bool,
) -> BdlResult<()> {
    let is_episode = matches!(kind, SourceKind::Bangumi | SourceKind::Cheese);
    let part = item
        .parts
        .first()
        .ok_or_else(|| source_error("item has no parts"))?;
    let loaded = hydrated
        .groups
        .into_iter()
        .flat_map(|group| group.items)
        .find(|candidate| {
            candidate.parts.iter().any(|candidate_part| {
                if is_episode {
                    candidate_part.id == part.id
                } else {
                    part.bvid
                        .as_ref()
                        .is_some_and(|bvid| candidate_part.bvid.as_ref() == Some(bvid))
                        || part.aid.is_some_and(|aid| candidate_part.aid == Some(aid))
                }
            })
        })
        .ok_or_else(|| source_error("media resolution did not return the selected item"))?;
    if loaded.parts.is_empty()
        || (require_streams && loaded.parts.iter().any(|part| part.streams.is_empty()))
    {
        return Err(source_error("selected item has no playable streams"));
    }
    // Keep the list's item identity, title and position for naming and selection.
    item.owner_name = item.owner_name.take().or(loaded.owner_name);
    item.owner_mid = item.owner_mid.or(loaded.owner_mid);
    item.publish_date = item.publish_date.take().or(loaded.publish_date);
    item.cover_url = item.cover_url.take().or(loaded.cover_url);
    item.parts = loaded.parts;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::PartId;
    use crate::planner::{DownloadOptions, plan_selected_parts};

    #[test]
    fn short_link_redirect_policy_rejects_external_hosts_credentials_and_downgrades() {
        for target in [
            "https://www.bilibili.com/video/BV1ZA411g7Sb",
            "https://b23.tv/abc",
            "https://space.bilibili.com/123",
        ] {
            validate_redirect_target(&url::Url::parse(target).unwrap()).unwrap();
        }
        for target in [
            "http://www.bilibili.com/video/x",
            "https://www.bilibili.com.evil.example/x",
            "https://user:secret@b23.tv/abc",
            "https://b23.tv:444/abc",
            "https://127.0.0.1/x",
        ] {
            assert!(validate_redirect_target(&url::Url::parse(target).unwrap()).is_err());
        }
    }

    fn item(id: &str, part_id: &str, playable: bool) -> NormalizedItem {
        let streams = if playable {
            vec![
                serde_json::json!({
                    "id": "video", "kind": "video", "quality": {"kind":"quality", "value":80},
                    "codec":"avc", "bandwidth":1000, "urls":["https://example.invalid/video"],
                    "headers":[], "acquired_at":"2026-09-22T00:00:00Z"
                }),
                serde_json::json!({
                    "id": "audio", "kind": "audio", "quality": {"kind":"quality", "value":30280},
                    "codec":"unknown", "bandwidth":100, "urls":["https://example.invalid/audio"],
                    "headers":[], "acquired_at":"2026-09-22T00:00:00Z"
                }),
            ]
        } else {
            Vec::new()
        };
        serde_json::from_value(serde_json::json!({
            "id":id, "title":"List title", "owner_name":null, "cover_url":null, "duration_seconds":10,
            "parts":[{"id":part_id,"title":"Part", "bvid":"BV1xx411c7mD", "aid":170001,
                "cid": if playable {Some(62131)} else {None}, "duration_seconds":10,"streams":streams,"assets":[]}]
        })).unwrap()
    }

    fn tree(items: Vec<NormalizedItem>) -> NormalizedSourceTree {
        serde_json::from_value(serde_json::json!({
            "source":{"id":"favorite:1","kind":"favorite","input":"unused","title":"List", "loaded_count":items.len(),"total_count":items.len(),"has_more":false},
            "groups":[{"id":"group:favorite:1","kind":"favorite","title":"List","items":items,"page":null}]
        })).unwrap()
    }

    #[test]
    fn list_hydration_expands_parts_and_keeps_list_identity_and_position() {
        let placeholder = item("item:favorite:1:BV1xx411c7mD", "placeholder", false);
        let mut source = tree(vec![
            item("previous", "previous", false),
            placeholder.clone(),
        ]);
        let mut video = item("item:video", "part:BV1xx411c7mD:62131", true);
        video.owner_name = Some("Video owner".into());
        let mut second = video.parts[0].clone();
        second.id = PartId("part:BV1xx411c7mD:62132".into());
        second.cid = Some(62132);
        video.parts.push(second);
        merge_item_media(
            &mut source.groups[0].items[1],
            tree(vec![video]),
            SourceKind::Favorite,
        )
        .unwrap();
        let hydrated = &source.groups[0].items[1];
        assert_eq!(hydrated.id, placeholder.id);
        assert_eq!(hydrated.title, placeholder.title);
        assert_eq!(hydrated.parts.len(), 2);
        assert_eq!(hydrated.owner_name.as_deref(), Some("Video owner"));
        let ids = hydrated
            .parts
            .iter()
            .map(|part| part.id.clone())
            .collect::<Vec<_>>();
        let mut options =
            DownloadOptions::new(std::env::temp_dir().join(uuid::Uuid::new_v4().to_string()));
        options.naming_template = "{index}-P{part_index}.{ext}".into();
        let tasks = plan_selected_parts(&source, &ids, &options).unwrap();
        assert_eq!(tasks.len(), 2);
        assert!(tasks[0].output_path.ends_with("2-P1.mp4"));
        assert!(tasks[1].output_path.ends_with("2-P2.mp4"));
    }

    #[test]
    fn episodes_resolve_by_episode_id_and_merge_only_the_matching_episode() {
        for (kind, prefix) in [
            (SourceKind::Bangumi, "bangumi"),
            (SourceKind::Cheese, "cheese"),
        ] {
            let id = format!("part:{prefix}:100:456:62131");
            let mut placeholder = item("episode", &id, false);
            assert_eq!(
                item_media_input(kind, &placeholder).unwrap(),
                format!("https://www.bilibili.com/{prefix}/play/ep456")
            );
            let other = item("other", &format!("part:{prefix}:100:455:1"), false);
            let matching = item("episode", &id, true);
            merge_item_media(&mut placeholder, tree(vec![other, matching]), kind).unwrap();
            assert_eq!(placeholder.parts[0].id.0, id);
            assert!(!placeholder.parts[0].streams.is_empty());
        }
    }

    #[test]
    fn unavailable_or_mismatched_hydration_does_not_replace_metadata() {
        let original = item("list", "placeholder", false);
        for mut video in [
            item("video", "part:video:1", true),
            item("video", "part:video:1", false),
        ] {
            if !video.parts[0].streams.is_empty() {
                video.parts[0].bvid = Some("different".into());
                video.parts[0].aid = Some(999);
            }
            let mut placeholder = original.clone();
            assert!(
                merge_item_media(&mut placeholder, tree(vec![video]), SourceKind::Collection)
                    .is_err()
            );
            assert_eq!(placeholder, original);
        }
        let mut missing = original;
        missing.parts[0].bvid = None;
        missing.parts[0].aid = None;
        assert!(item_media_input(SourceKind::Favorite, &missing).is_err());
    }

    #[tokio::test]
    async fn bounded_loading_does_not_request_more_pages_after_selection_is_available() {
        let resolver = SourceResolver::new().unwrap();
        let mut source = tree(vec![item("item", "part", false)]);
        source.source.has_more = true;
        // No valid input or page state: an accidental extra request would fail.
        resolver.load_until(&mut source, Some(1)).await.unwrap();
        assert!(source.source.has_more);
    }
}
