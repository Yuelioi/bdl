//! 专栏分类
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/article/category.md)

use crate::article::models::ArticleCategory;

/// 专栏分类常量
pub struct ArticleCategories;

impl ArticleCategories {
    /// 获取所有专栏分类
    pub fn all() -> Vec<ArticleCategory> {
        vec![
            // 游戏分类
            ArticleCategory {
                id: 1,
                parent_id: 0,
                name: "游戏".to_string(),
            },
            ArticleCategory {
                id: 6,
                parent_id: 1,
                name: "单机游戏".to_string(),
            },
            ArticleCategory {
                id: 7,
                parent_id: 1,
                name: "电子竞技".to_string(),
            },
            ArticleCategory {
                id: 8,
                parent_id: 1,
                name: "手机游戏".to_string(),
            },
            ArticleCategory {
                id: 9,
                parent_id: 1,
                name: "网络游戏".to_string(),
            },
            ArticleCategory {
                id: 10,
                parent_id: 1,
                name: "桌游棋牌".to_string(),
            },
            // 动画分类
            ArticleCategory {
                id: 2,
                parent_id: 0,
                name: "动画".to_string(),
            },
            ArticleCategory {
                id: 4,
                parent_id: 2,
                name: "动漫杂谈".to_string(),
            },
            ArticleCategory {
                id: 5,
                parent_id: 2,
                name: "动漫资讯".to_string(),
            },
            ArticleCategory {
                id: 31,
                parent_id: 2,
                name: "动画技术".to_string(),
            },
            // 生活分类
            ArticleCategory {
                id: 3,
                parent_id: 0,
                name: "生活".to_string(),
            },
            ArticleCategory {
                id: 13,
                parent_id: 3,
                name: "美食".to_string(),
            },
            ArticleCategory {
                id: 14,
                parent_id: 3,
                name: "时尚".to_string(),
            },
            ArticleCategory {
                id: 15,
                parent_id: 3,
                name: "日常".to_string(),
            },
            ArticleCategory {
                id: 21,
                parent_id: 3,
                name: "萌宠".to_string(),
            },
            ArticleCategory {
                id: 22,
                parent_id: 3,
                name: "运动".to_string(),
            },
            // 轻小说分类
            ArticleCategory {
                id: 16,
                parent_id: 0,
                name: "轻小说".to_string(),
            },
            ArticleCategory {
                id: 18,
                parent_id: 16,
                name: "原创连载".to_string(),
            },
            ArticleCategory {
                id: 19,
                parent_id: 16,
                name: "同人连载".to_string(),
            },
            ArticleCategory {
                id: 20,
                parent_id: 16,
                name: "小说杂谈".to_string(),
            },
            ArticleCategory {
                id: 32,
                parent_id: 16,
                name: "短篇小说".to_string(),
            },
            // 科技分类
            ArticleCategory {
                id: 17,
                parent_id: 0,
                name: "科技".to_string(),
            },
            ArticleCategory {
                id: 25,
                parent_id: 17,
                name: "人文历史".to_string(),
            },
            ArticleCategory {
                id: 26,
                parent_id: 17,
                name: "数码".to_string(),
            },
            ArticleCategory {
                id: 27,
                parent_id: 17,
                name: "汽车".to_string(),
            },
            ArticleCategory {
                id: 33,
                parent_id: 17,
                name: "自然".to_string(),
            },
            ArticleCategory {
                id: 34,
                parent_id: 17,
                name: "学习".to_string(),
            },
            // 影视分类
            ArticleCategory {
                id: 28,
                parent_id: 0,
                name: "影视".to_string(),
            },
            ArticleCategory {
                id: 12,
                parent_id: 28,
                name: "电影".to_string(),
            },
            ArticleCategory {
                id: 35,
                parent_id: 28,
                name: "电视剧".to_string(),
            },
            ArticleCategory {
                id: 36,
                parent_id: 28,
                name: "纪录片".to_string(),
            },
            ArticleCategory {
                id: 37,
                parent_id: 28,
                name: "综艺".to_string(),
            },
            // 兴趣分类
            ArticleCategory {
                id: 29,
                parent_id: 0,
                name: "兴趣".to_string(),
            },
            ArticleCategory {
                id: 11,
                parent_id: 29,
                name: "模型手办".to_string(),
            },
            ArticleCategory {
                id: 23,
                parent_id: 29,
                name: "绘画".to_string(),
            },
            ArticleCategory {
                id: 24,
                parent_id: 29,
                name: "手工".to_string(),
            },
            ArticleCategory {
                id: 38,
                parent_id: 29,
                name: "摄影".to_string(),
            },
            ArticleCategory {
                id: 39,
                parent_id: 29,
                name: "音乐舞蹈".to_string(),
            },
            // 笔记分类
            ArticleCategory {
                id: 41,
                parent_id: 0,
                name: "笔记".to_string(),
            },
            ArticleCategory {
                id: 42,
                parent_id: 41,
                name: "全部笔记".to_string(),
            },
        ]
    }

    /// 获取顶级分类
    pub fn top_level() -> Vec<ArticleCategory> {
        Self::all()
            .into_iter()
            .filter(|cat| cat.parent_id == 0)
            .collect()
    }

    /// 根据ID获取分类
    pub fn find_by_id(id: i32) -> Option<ArticleCategory> {
        Self::all().into_iter().find(|cat| cat.id == id)
    }

    /// 根据名称获取分类
    pub fn find_by_name(name: &str) -> Option<ArticleCategory> {
        Self::all().into_iter().find(|cat| cat.name == name)
    }

    /// 获取子分类
    pub fn children_of(parent_id: i32) -> Vec<ArticleCategory> {
        Self::all()
            .into_iter()
            .filter(|cat| cat.parent_id == parent_id)
            .collect()
    }

    /// 获取游戏分类
    pub fn game() -> Vec<ArticleCategory> {
        Self::children_of(1)
    }

    /// 获取动画分类
    pub fn animation() -> Vec<ArticleCategory> {
        Self::children_of(2)
    }

    /// 获取生活分类
    pub fn life() -> Vec<ArticleCategory> {
        Self::children_of(3)
    }

    /// 获取轻小说分类
    pub fn light_novel() -> Vec<ArticleCategory> {
        Self::children_of(16)
    }

    /// 获取科技分类
    pub fn technology() -> Vec<ArticleCategory> {
        Self::children_of(17)
    }

    /// 获取影视分类
    pub fn film() -> Vec<ArticleCategory> {
        Self::children_of(28)
    }

    /// 获取兴趣分类
    pub fn interest() -> Vec<ArticleCategory> {
        Self::children_of(29)
    }

    /// 获取笔记分类
    pub fn note() -> Vec<ArticleCategory> {
        Self::children_of(41)
    }
}
