//! B站用户信息相关接口
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/user)
use crate::models::{LevelInfo, Nameplate, Official, OfficialVerify, Pendant, Vip, VipLabel};
use serde::{Deserialize, Serialize};

/// 用户空间详细信息响应结构体
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserSpaceInfo {
    /// 用户mid
    pub mid: u64,
    /// 昵称
    pub name: String,
    /// 性别 男/女/保密
    pub sex: String,
    /// 头像链接
    pub face: String,
    /// 是否为NFT头像 0：不是NFT头像 1：是NFT头像
    pub face_nft: u8,
    /// NFT头像类型
    pub face_nft_type: Option<u8>,
    /// 签名
    pub sign: String,
    /// 用户权限等级
    pub rank: u32,
    /// 当前等级 0-6级
    pub level: u8,
    /// 注册时间 此接口返回恒为0
    pub jointime: u64,
    /// 节操值 此接口返回恒为0
    pub moral: u64,
    /// 封禁状态 0：正常 1：被封
    pub silence: u8,
    /// 硬币数 需要登录(Cookie) 只能查看自己的 默认为0
    pub coins: f64,
    /// 是否具有粉丝勋章
    pub fans_badge: bool,
    /// 粉丝勋章信息
    pub fans_medal: Option<FansMedal>,
    /// 认证信息
    pub official: Official,
    /// 会员信息
    pub vip: Vip,
    /// 头像框信息
    pub pendant: Pendant,
    /// 勋章信息
    pub nameplate: Nameplate,
    /// 用户荣誉信息
    pub user_honour_info: UserHonourInfo,
    /// 是否关注此用户 需要登录(Cookie) 未登录恒为false
    pub is_followed: bool,
    /// 主页头图链接
    pub top_photo: String,
    /// 主题信息
    pub theme: serde_json::Value,
    /// 系统通知
    pub sys_notice: SysNotice,
    /// 直播间信息（部分账号/接口返回为 `null`）
    #[serde(default)]
    pub live_room: Option<LiveRoom>,
    /// 生日 MM-DD 如设置隐私为空
    pub birthday: String,
    /// 学校（部分账号接口返回 `null`）
    #[serde(default)]
    pub school: Option<School>,
    /// 专业资质信息
    pub profession: Option<Profession>,
    /// 个人标签
    pub tags: Option<Vec<String>>,
    /// 系列信息
    pub series: Series,
    /// 是否为硬核会员 0：否 1：是
    pub is_senior_member: u8,
    /// MCN信息
    pub mcn_info: Option<serde_json::Value>,
    /// Gaia资源类型
    pub gaia_res_type: Option<u8>,
    /// Gaia数据
    pub gaia_data: Option<serde_json::Value>,
    /// 是否存在风险
    pub is_risk: bool,
    /// 充电信息
    pub elec: Elec,
    /// 是否显示老粉计划
    pub contract: Contract,
    /// 证书显示
    pub certificate_show: Option<bool>,
    /// 昵称渲染信息
    pub name_render: Option<serde_json::Value>,
}

/// 粉丝勋章信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FansMedal {
    /// 是否展示
    pub show: bool,
    /// 是否佩戴了粉丝勋章
    pub wear: bool,
    /// 粉丝勋章详细信息
    pub medal: Option<Medal>,
}

/// 粉丝勋章详细信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Medal {
    /// 粉丝勋章等级
    pub level: u8,
    /// 粉丝勋章等级
    pub guard_level: u8,
    /// 粉丝勋章名称
    pub medal_name: String,
}

/// 用户荣誉信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserHonourInfo {
    /// 用户mid
    pub mid: u64,
    /// 颜色
    pub colour: Option<String>,
    /// 标签
    pub tags: Option<Vec<serde_json::Value>>,
}

/// 系统通知
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SysNotice {
    /// 通知id
    pub id: Option<u32>,
    /// 显示文案
    pub content: Option<String>,
    /// 跳转地址
    pub url: Option<String>,
    /// 提示类型
    pub notice_type: Option<u8>,
    /// 前缀图标
    pub icon: Option<String>,
    /// 文字颜色
    pub text_color: Option<String>,
    /// 背景颜色
    pub bg_color: Option<String>,
}

/// 直播间信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LiveRoom {
    /// 直播间状态 0：无房间 1：有房间
    #[serde(rename = "roomStatus")]
    pub room_status: u8,
    /// 直播状态 0：未开播 1：直播中
    #[serde(rename = "liveStatus")]
    pub live_status: u8,
    /// 直播间网页url
    pub url: String,
    /// 直播间标题
    pub title: String,
    /// 直播间封面url
    pub cover: String,
    /// 观看显示信息
    pub watched_show: WatchedShow,
    /// 直播间id
    pub roomid: u64,
    /// 轮播状态 0：未轮播 1：轮播
    #[serde(rename = "roundStatus")]
    pub round_status: u8,
    /// 广播类型
    pub broadcast_type: u8,
}

/// 观看显示信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WatchedShow {
    /// 开关
    pub switch: bool,
    /// 观看人数
    pub num: u64,
    /// 小文本
    pub text_small: String,
    /// 大文本
    pub text_large: String,
    /// 图标
    pub icon: String,
    /// 图标位置
    pub icon_location: String,
    /// 网页图标
    pub icon_web: String,
}

/// 学校信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct School {
    /// 就读大学名称 没有则为空
    pub name: String,
}

/// 专业资质信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Profession {
    /// 资质名称
    pub name: String,
    /// 职位
    pub department: String,
    /// 所属机构
    pub title: String,
    /// 是否显示 0：不显示 1：显示
    pub is_show: u8,
}

/// 系列信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Series {
    /// 用户升级状态
    pub user_upgrade_status: u8,
    /// 是否显示升级窗口
    pub show_upgrade_window: bool,
}

/// 充电信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Elec {
    /// 显示的充电信息
    pub show_info: ShowInfo,
}

/// 显示的充电信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShowInfo {
    /// 是否显示充电按钮
    pub show: bool,
    /// 充电功能开启状态 -1：未开通充电功能 1：已开通自定义充电 2：已开通包月、自定义充电 3：已开通包月高档、自定义充电
    pub state: i8,
    /// 充电按钮显示文字
    pub title: String,
    /// 充电图标
    pub icon: String,
    /// 跳转url
    pub jump_url: String,
}

/// 老粉计划信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Contract {
    /// 是否显示
    pub is_display: bool,
    /// 是否在显示老粉计划 true：显示 false：不显示
    pub is_follow_display: bool,
}

/// 用户名片信息响应结构体
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserCardInfo {
    /// 卡片信息
    pub card: Card,
    /// 是否关注此用户 true：已关注 false：未关注 需要登录(Cookie) 未登录为false
    pub following: bool,
    /// 用户稿件数
    pub archive_count: u32,
    /// 作用尚不明确
    pub article_count: u32,
    /// 粉丝数
    pub follower: u32,
    /// 点赞数
    pub like_num: u32,
}

/// 用户卡片详细信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Card {
    /// 用户mid
    pub mid: String,
    /// 用户昵称
    pub name: String,
    /// 用户性别 男/女/保密
    pub sex: String,
    /// 用户头像链接
    pub face: String,
    /// 显示排名 作用尚不明确
    #[serde(rename = "DisplayRank")]
    pub display_rank: String,
    /// 注册时间 作用尚不明确
    pub regtime: u64,
    /// 用户状态 0：正常 -2：被封禁
    pub spacesta: i32,
    /// 生日 作用尚不明确
    pub birthday: String,
    /// 地点 作用尚不明确
    pub place: String,
    /// 描述 作用尚不明确
    pub description: String,
    /// 文章数 作用尚不明确
    pub article: u32,
    /// 关注列表 作用尚不明确
    pub attentions: Vec<serde_json::Value>,
    /// 粉丝数
    pub fans: u32,
    /// 好友数
    pub friend: u32,
    /// 关注数
    pub attention: u32,
    /// 签名
    pub sign: String,
    /// 等级信息
    pub level_info: LevelInfo,
    /// 挂件信息
    pub pendant: Pendant,
    /// 勋章信息
    pub nameplate: Nameplate,
    /// 认证信息
    #[serde(rename = "Official")]
    pub official: Official,
    /// 认证信息2
    pub official_verify: OfficialVerify,
    /// 大会员状态
    pub vip: Vip,
    /// 主页头图
    pub space: Option<Space>,
}

/// 主页头图信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Space {
    /// 主页头图url 小图
    pub s_img: String,
    /// 主页头图url 正常
    pub l_img: String,
}

/// 用户卡片（精简版）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserCard {
    pub mid: u64,
    pub name: String,
    pub face: String,
    pub sign: String,
    pub rank: i32,
    pub level: i32,
    pub silence: i32,
}

/// 用户详细信息（完整版）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserInfo {
    pub mid: u64,
    pub name: String,
    pub sign: String,
    pub rank: i32,
    pub level: i32,
    pub silence: i32,

    pub sex: Option<String>,
    pub face: String,
    pub vip: Option<UserVip>,
    pub official: Option<UserOfficial>,
    pub is_fake_account: Option<u32>,
    pub expert_info: Option<serde_json::Value>,
}

/// 大会员信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserVip {
    pub r#type: i32,
    pub status: i32,
    pub due_date: i64,
    pub vip_pay_type: i32,
    pub theme_type: i32,
    pub label: Option<VipLabel>,
}

/// 认证信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserOfficial {
    #[serde(default)]
    pub role: i32,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub r#type: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BpiClient;
    use crate::ids::Mid;
    use crate::user::params::{
        UserCardParams, UserCardPhoto, UserCardsParams, UserInfosParams, UserSpaceParams,
    };

    fn live_user_tests_enabled() -> bool {
        std::env::var("BPI_LIVE_TEST").ok().as_deref() == Some("1")
    }

    #[test]
    fn user_space_info_accepts_null_live_room_and_school() {
        #[derive(serde::Deserialize)]
        struct NullableFields {
            #[serde(default)]
            live_room: Option<LiveRoom>,
            #[serde(default)]
            school: Option<School>,
        }

        let parsed: NullableFields = serde_json::from_str(
            r#"{
                "live_room": null,
                "school": null
            }"#,
        )
        .expect("nullable user-space fields should deserialize");

        assert!(parsed.live_room.is_none());
        assert!(parsed.school.is_none());
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_user_space_info() {
        if !live_user_tests_enabled() {
            return;
        }

        tracing::info!("开始测试获取用户空间详细信息");

        let bpi = BpiClient::new().expect("client should build");
        let mid = 2; // 测试用户ID

        tracing::info!("测试用户ID: {}", mid);

        let resp = bpi
            .user()
            .space_info(UserSpaceParams::new(Mid::new(mid).expect("valid mid")))
            .await;

        match &resp {
            Ok(data) => {
                tracing::info!("用户昵称: {}", data.name);
                tracing::info!("用户等级: {}", data.level);
                tracing::info!(
                    "是否为会员: {}",
                    data.vip.as_ref().is_some_and(|vip| vip.status > 0)
                );
                tracing::info!("是否有粉丝勋章: {}", data.fans_badge);
            }
            Err(e) => {
                tracing::error!("请求失败: {:?}", e);
            }
        }

        assert!(resp.is_ok());
        tracing::info!("测试完成");
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_user_space_info_nonexistent() {
        if !live_user_tests_enabled() {
            return;
        }

        tracing::info!("开始测试获取不存在用户的空间详细信息");

        let bpi = BpiClient::new().expect("client should build");
        let mid = 0; // 不存在的用户ID

        tracing::info!("测试用户ID: {}", mid);

        let resp = bpi
            .user()
            .space_info(UserSpaceParams::new(Mid::new(mid).expect("valid mid")))
            .await;

        match &resp {
            Ok(data) => {
                tracing::info!("意外返回用户: {}", data.name);
            }
            Err(e) => {
                tracing::error!("请求失败: {:?}", e);
            }
        }

        tracing::info!("测试完成");
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_user_card_info() {
        if !live_user_tests_enabled() {
            return;
        }

        tracing::info!("开始测试获取用户名片信息");

        let bpi = BpiClient::new().expect("client should build");
        let mid = 2; // 测试用户ID

        tracing::info!("测试用户ID: {}", mid);

        let resp = bpi
            .user()
            .card(UserCardParams::new(Mid::new(mid).expect("valid mid")))
            .await;

        match &resp {
            Ok(data) => {
                tracing::info!("用户昵称: {}", data.card.name);
                tracing::info!("用户性别: {:?}", data.card.sex);
                tracing::info!("是否关注: {}", data.following);
                tracing::info!("稿件数: {}", data.archive_count);
                tracing::info!("粉丝数: {}", data.follower);
                tracing::info!("点赞数: {}", data.like_num);
                tracing::info!("用户签名: {}", data.card.sign);
            }
            Err(e) => {
                tracing::error!("请求失败: {:?}", e);
            }
        }

        assert!(resp.is_ok());
        tracing::info!("测试完成");
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_user_card_info_with_photo() {
        if !live_user_tests_enabled() {
            return;
        }

        tracing::info!("开始测试获取用户名片信息（包含主页头图）");

        let bpi = BpiClient::new().expect("client should build");
        let mid = 2; // 测试用户ID

        tracing::info!("测试用户ID: {}", mid);

        let resp = bpi
            .user()
            .card(
                UserCardParams::new(Mid::new(mid).expect("valid mid"))
                    .with_photo(UserCardPhoto::Include),
            )
            .await;

        match &resp {
            Ok(data) => {
                tracing::info!("用户昵称: {}", data.card.name);
                tracing::info!("粉丝数: {}", data.card.fans);
            }
            Err(e) => {
                tracing::error!("请求失败: {:?}", e);
            }
        }

        assert!(resp.is_ok());
        tracing::info!("测试完成");
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_user_card_info_without_photo() {
        if !live_user_tests_enabled() {
            return;
        }

        tracing::info!("开始测试获取用户名片信息（不包含主页头图）");

        let bpi = BpiClient::new().expect("client should build");
        let mid = 123456; // 测试用户ID

        tracing::info!("测试用户ID: {}", mid);

        let resp = bpi
            .user()
            .card(
                UserCardParams::new(Mid::new(mid).expect("valid mid"))
                    .with_photo(UserCardPhoto::Exclude),
            )
            .await;

        match &resp {
            Ok(data) => {
                tracing::info!("用户昵称: {}", data.card.name);
                tracing::info!("粉丝数: {}", data.card.fans);
                tracing::info!("关注数: {}", data.card.attention);
            }
            Err(e) => {
                tracing::error!("请求失败: {:?}", e);
            }
        }

        assert!(resp.is_ok());
        tracing::info!("测试完成");
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_user_card_info_invalid_user() {
        if !live_user_tests_enabled() {
            return;
        }

        tracing::info!("开始测试获取不存在用户的名片信息");

        let bpi = BpiClient::new().expect("client should build");
        let mid = 0; // 不存在的用户ID

        tracing::info!("测试用户ID: {}", mid);

        let resp = bpi
            .user()
            .card(UserCardParams::new(Mid::new(mid).expect("valid mid")))
            .await;

        match &resp {
            Ok(data) => {
                tracing::info!("意外返回用户: {}", data.card.name);
            }
            Err(e) => {
                tracing::error!("请求失败: {:?}", e);
            }
        }

        tracing::info!("测试完成");
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_user_card_info_banned_user() {
        if !live_user_tests_enabled() {
            return;
        }

        tracing::info!("开始测试获取被封禁用户的名片信息");

        let bpi = BpiClient::new().expect("client should build");
        let mid = 999999999; // 假设的被封禁用户ID

        tracing::info!("测试用户ID: {}", mid);

        let resp = bpi
            .user()
            .card(UserCardParams::new(Mid::new(mid).expect("valid mid")))
            .await;

        match &resp {
            Ok(data) => {
                tracing::info!("请求成功，用户昵称: {}", data.card.name);
            }
            Err(e) => {
                tracing::error!("请求失败: {:?}", e);
            }
        }

        tracing::info!("测试完成");
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_user_cards_and_infos() {
        if !live_user_tests_enabled() {
            return;
        }

        let bpi = BpiClient::new().expect("client should build");

        // 测试精简版
        let cards = bpi
            .user()
            .cards(
                UserCardsParams::new([
                    Mid::new(2).expect("valid mid"),
                    Mid::new(3).expect("valid mid"),
                ])
                .expect("valid params"),
            )
            .await
            .unwrap();
        tracing::info!("用户卡片: {:?}", cards);

        // 测试完整版
        let infos = bpi
            .user()
            .infos(
                UserInfosParams::new([
                    Mid::new(2).expect("valid mid"),
                    Mid::new(3).expect("valid mid"),
                ])
                .expect("valid params"),
            )
            .await
            .unwrap();
        tracing::info!("用户详细信息: {:?}", infos);
    }
}
