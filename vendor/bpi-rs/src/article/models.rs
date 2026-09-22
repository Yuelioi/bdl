use crate::article::view::AuthorVip;
use crate::models::{Nameplate, Pendant};
use serde::{Deserialize, Serialize};

/// 专栏统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleStats {
    /// 投币数
    pub coin: i64,
    /// 点踩数
    pub dislike: i64,
    /// 动态转发数
    pub dynamic: i64,
    /// 收藏数
    pub favorite: i64,
    /// 点赞数
    pub like: i64,
    /// 评论数
    pub reply: i64,
    /// 分享数
    pub share: i64,
    /// 阅读数
    pub view: i64,
}

/// 专栏作者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleAuthor {
    /// 用户ID
    pub mid: i64,
    /// 用户昵称
    pub name: String,
    /// 用户头像
    pub face: String,
    /// 用户等级
    pub level: i32,
    /// 粉丝数
    pub fans: i64,
    /// 认证信息
    pub official_verify: AuthorOfficialVerify,
    /// 勋章信息
    pub nameplate: Nameplate,
    /// 头像框
    pub pendant: Pendant,
    /// VIP信息
    pub vip: AuthorVip,
}

/// 专栏分类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleCategory {
    /// 分类ID
    pub id: i32,
    /// 分类名称
    pub name: String,
    /// 父分类ID
    pub parent_id: i32,
}

/// 作者认证信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorOfficialVerify {
    /// 认证类型
    pub r#type: i32,
    /// 认证描述
    pub desc: String,
}

/// 专栏媒体信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleMedia {
    /// 地区
    pub area: String,
    /// 封面
    pub cover: String,
    /// 媒体ID
    pub media_id: i64,
    /// 评分
    pub score: i32,
    /// 季ID
    pub season_id: i64,
    /// 剧透
    pub spoiler: i32,
    /// 标题
    pub title: String,
    /// 类型ID
    pub type_id: i32,
    /// 类型名称
    pub type_name: String,
}
