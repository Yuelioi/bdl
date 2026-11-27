//! Read-only account list. Stream/part hydration stays in the common video pipeline.
use crate::{
    BdlError, BdlResult,
    ids::{GroupId, ItemId, PartId, SourceId},
    model::{
        AssetKind, FetchPolicy, NormalizedGroup, NormalizedItem, NormalizedPart,
        NormalizedSourceTree, SourceKind, SourceSummary,
    },
};
use bpi_rs::{BpiClient, historytoview::toview::ToViewListData};

pub const WATCH_LATER_URL: &str = "https://www.bilibili.com/list/watchlater";

pub async fn resolve_watch_later(client: &BpiClient) -> BdlResult<NormalizedSourceTree> {
    let data = client
        .historytoview()
        .toview_list()
        .await
        .map_err(|e| BdlError::Bpi(e.to_string()))?;
    Ok(normalize(data))
}

fn normalize(data: ToViewListData) -> NormalizedSourceTree {
    let mut seen = std::collections::HashSet::new();
    let items: Vec<_> = data
        .list
        .into_iter()
        .filter(|v| seen.insert(v.aid))
        .map(|v| {
            let key = format!("watch_later:{}", v.aid);
            NormalizedItem {
                episode_number: None,
                collection: None,
                id: ItemId(format!("item:{key}")),
                title: v.title.clone(),
                owner_name: Some(v.owner.name),
                owner_mid: Some(v.owner.mid),
                publish_date: super::bilibili_publish_date(v.pubdate),
                cover_url: (!v.pic.is_empty()).then_some(v.pic),
                duration_seconds: Some(v.duration.into()),
                nfo_metadata: crate::nfo::NfoMetadata {
                    description: v.desc,
                    genre: v.tname,
                    ..Default::default()
                },
                parts: vec![NormalizedPart {
                    id: PartId(format!("part:{key}")),
                    title: v.title,
                    aid: Some(v.aid),
                    bvid: (!v.bvid.is_empty()).then_some(v.bvid),
                    cid: None,
                    duration_seconds: Some(v.duration.into()),
                    streams: vec![],
                    assets: vec![AssetKind::Cover.with_policy(FetchPolicy::OnDemand)],
                }],
            }
        })
        .collect();
    // This endpoint returns the complete list, not a page. Keep empty/unavailable
    // entries visible; the common hydration path gives actionable per-item errors.
    NormalizedSourceTree {
        source: SourceSummary {
            id: SourceId("watch_later:account".into()),
            kind: SourceKind::WatchLater,
            input: WATCH_LATER_URL.into(),
            title: "稍后再看".into(),
            loaded_count: items.len(),
            total_count: Some(items.len()),
            has_more: false,
        },
        groups: vec![NormalizedGroup {
            id: GroupId("group:watch_later".into()),
            kind: "watch_later".into(),
            title: "稍后再看".into(),
            items,
            page: None,
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalizes_list_without_fetching_streams_or_losing_multipart_hydration() {
        let data: ToViewListData = serde_json::from_value(serde_json::json!({"count":1,"list":[{
            "aid":1,"videos":2,"tid":1,"tname":"动画","copyright":1,"pic":"","title":"测试",
            "pubdate":0,"ctime":0,"desc":"说明","state":0,"duration":60,
            "rights":{"bp":0,"elec":0,"download":0,"movie":0,"pay":0,"hd5":0,"no_reprint":0,"autoplay":0,"ugc_pay":0,"is_cooperation":0,"ugc_pay_preview":0,"no_background":0},
            "owner":{"mid":2,"name":"UP","face":""},
            "stat":{"aid":1,"view":0,"danmaku":0,"reply":0,"favorite":0,"coin":0,"share":0,"now_rank":0,"his_rank":0,"like":0,"dislike":0,"vt":0,"vv":0},
            "dimension":{"width":1920,"height":1080,"rotate":0},"cid":3,"progress":0,"add_at":0,"bvid":"BV1xx411c7mD"
        }]})).unwrap();
        let expected = data.list.len();
        let tree = normalize(data);
        assert_eq!(tree.source.kind, SourceKind::WatchLater);
        assert_eq!(tree.source.loaded_count, expected);
        assert!(!tree.source.has_more);
        for item in &tree.groups[0].items {
            assert!(item.parts[0].cid.is_none());
            assert!(item.parts[0].streams.is_empty());
        }
        assert_eq!(
            normalize(ToViewListData {
                count: 0,
                list: vec![]
            })
            .source
            .total_count,
            Some(0)
        );
    }
}
