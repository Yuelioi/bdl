//! B站视频分区一览 (v2)
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/video)
/// 包含所有主分区及子分区信息，字段对应于 tid_v2 和 tname_v2。
pub enum VideoPartitionV2 {
    /// 动画
    Douga(Douga),
    /// 游戏
    Game(Game),
    /// 鬼畜
    Kichiku(Kichiku),
    /// 音乐
    Music(Music),
    /// 舞蹈
    Dance(Dance),
    /// 影视
    Cinephile(Cinephile),
    /// 娱乐
    Ent(Ent),
    /// 知识
    Knowledge(Knowledge),
    /// 科技数码
    Tech(Tech),
    /// 资讯
    Information(Information),
    /// 美食
    Food(Food),
    /// 小剧场
    Shortplay(Shortplay),
    /// 汽车
    Car(Car),
    /// 时尚美妆
    Fashion(Fashion),
    /// 体育运动
    Sports(Sports),
    /// 动物
    Animal(Animal),
    /// vlog
    Vlog(Vlog),
    /// 绘画
    Painting(Painting),
    /// 人工智能
    Ai(Ai),
    /// 家装房产
    Home(Home),
    /// 户外潮流
    Outdoors(Outdoors),
    /// 健身
    Gym(Gym),
    /// 手工
    Handmake(Handmake),
    /// 旅游出行
    Travel(Travel),
    /// 三农
    Rural(Rural),
    /// 亲子
    Parenting(Parenting),
    /// 健康
    Health(Health),
    /// 情感
    Emotion(Emotion),
    /// 生活兴趣
    LifeJoy(LifeJoy),
    /// 生活经验
    LifeExperience(LifeExperience),
    /// 神秘学
    Mysticism(Mysticism),
}

/// 动画分区
pub enum Douga {
    /// 动画 (主分区)
    Douga,
    /// 同人动画
    FanAnime,
    /// 模玩周边
    GarageKit,
    /// cosplay
    Cosplay,
    /// 二次元线下
    Offline,
    /// 动漫剪辑
    Editing,
    /// 动漫评论
    Commentary,
    /// 动漫速读
    QuickView,
    /// 动漫配音
    Voice,
    /// 动漫资讯
    Information,
    /// 网文解读
    Interpret,
    /// 虚拟up主
    Vup,
    /// 特摄
    Tokusatsu,
    /// 布袋戏
    Puppetry,
    /// 漫画·动态漫
    Comic,
    /// 广播剧
    Motion,
    /// 动漫reaction
    Reaction,
    /// 动漫教学
    Tutorial,
    /// 二次元其他
    Other,
}

/// 游戏分区
pub enum Game {
    /// 游戏 (主分区)
    Game,
    /// 单人RPG游戏
    Rpg,
    /// MMORPG游戏
    MmorpG,
    /// 单机主机类游戏
    StandAlone,
    /// SLG游戏
    Slg,
    /// 回合制策略游戏
    Tbs,
    /// 即时策略游戏
    Rts,
    /// MOBA游戏
    Moba,
    /// 射击游戏
    Stg,
    /// 体育竞速游戏
    Spg,
    /// 动作竞技游戏
    Act,
    /// 音游舞游
    Msc,
    /// 模拟经营游戏
    Sim,
    /// 女性向游戏
    Otome,
    /// 休闲/小游戏
    Puz,
    /// 沙盒类
    Sandbox,
    /// 其他游戏
    Other,
}

/// 鬼畜分区
pub enum Kichiku {
    /// 鬼畜 (主分区)
    Kichiku,
    /// 鬼畜调教
    Guide,
    /// 鬼畜剧场
    Theatre,
    /// 人力VOCALOID
    ManualVocaloid,
    /// 音MAD
    Mad,
    /// 鬼畜综合
    Other,
}

/// 音乐分区
pub enum Music {
    /// 音乐 (主分区)
    Music,
    /// 原创音乐
    Original,
    /// MV
    Mv,
    /// 音乐现场
    Live,
    /// 乐迷饭拍
    FanVideos,
    /// 翻唱
    Cover,
    /// 演奏
    Perform,
    /// VOCALOID
    Vocaloid,
    /// AI音乐
    AiMusic,
    /// 电台·歌单
    Radio,
    /// 音乐教学
    Tutorial,
    /// 乐评盘点
    Commentary,
    /// 音乐综合
    Other,
}

/// 舞蹈分区
pub enum Dance {
    /// 舞蹈 (主分区)
    Dance,
    /// 宅舞
    Otaku,
    /// 街舞
    Hiphop,
    /// 颜值·网红舞
    Gestures,
    /// 明星舞蹈
    Star,
    /// 国风舞蹈
    China,
    /// 舞蹈教学
    Tutorial,
    /// 芭蕾舞
    Ballet,
    /// wota艺
    Wota,
    /// 舞蹈综合
    Other,
}

/// 影视分区
pub enum Cinephile {
    /// 影视 (主分区)
    Cinephile,
    /// 影视解读
    Commentary,
    /// 影视剪辑
    Montage,
    /// 影视资讯
    Information,
    /// 影视正片搬运
    Porterage,
    /// 短剧短片
    Shortfilm,
    /// AI影视
    Ai,
    /// 影视reaction
    Reaction,
    /// 影视综合
    Other,
}

/// 娱乐分区
pub enum Ent {
    /// 娱乐 (主分区)
    Ent,
    /// 娱乐评论
    Commentary,
    /// 明星剪辑
    Montage,
    /// 娱乐饭拍&现场
    FansVideo,
    /// 娱乐资讯
    Information,
    /// 娱乐reaction
    Reaction,
    /// 娱乐综艺正片
    Variety,
    /// 娱乐综合
    Other,
}

/// 知识分区
pub enum Knowledge {
    /// 知识 (主分区)
    Knowledge,
    /// 应试教育
    Exam,
    /// 非应试语言学习
    LangSkill,
    /// 大学专业知识
    Campus,
    /// 商业财经
    Business,
    /// 社会观察
    SocialObservation,
    /// 时政解读
    Politics,
    /// 人文历史
    HumanityHistory,
    /// 设计艺术
    Design,
    /// 心理杂谈
    Psychology,
    /// 职场发展
    Career,
    /// 科学科普
    Science,
    /// 其他知识杂谈
    Other,
}

/// 科技数码分区
pub enum Tech {
    /// 科技数码 (主分区)
    Tech,
    /// 电脑
    Computer,
    /// 手机
    Phone,
    /// 平板电脑
    Pad,
    /// 摄影摄像
    Photography,
    /// 工程机械
    Machine,
    /// 自制发明/设备
    Create,
    /// 科技数码综合
    Other,
}

/// 资讯分区
pub enum Information {
    /// 资讯 (主分区)
    Information,
    /// 时政资讯
    Politics,
    /// 海外资讯
    Overseas,
    /// 社会资讯
    Social,
    /// 综合资讯
    Other,
}

/// 美食分区
pub enum Food {
    /// 美食 (主分区)
    Food,
    /// 美食制作
    Make,
    /// 美食探店
    Detective,
    /// 美食测评
    Commentary,
    /// 美食记录
    Record,
    /// 美食综合
    Other,
}

/// 小剧场分区
pub enum Shortplay {
    /// 小剧场 (主分区)
    Shortplay,
    /// 剧情演绎
    Plot,
    /// 语言类小剧场
    Lang,
    /// UP主小综艺
    UpVariety,
    /// 街头采访
    Interview,
}

/// 汽车分区
pub enum Car {
    /// 汽车 (主分区)
    Car,
    /// 汽车测评
    Commentary,
    /// 汽车文化
    Culture,
    /// 汽车生活
    Life,
    /// 汽车技术
    Tech,
    /// 汽车综合
    Other,
}

/// 时尚美妆分区
pub enum Fashion {
    /// 时尚美妆 (主分区)
    Fashion,
    /// 美妆
    Makeup,
    /// 护肤
    Skincare,
    /// 仿装cos
    Cos,
    /// 鞋服穿搭
    Outfits,
    /// 箱包配饰
    Accessories,
    /// 珠宝首饰
    Jewelry,
    /// 三坑
    Trick,
    /// 时尚解读
    Commentary,
    /// 时尚综合
    Other,
}

/// 体育运动分区
pub enum Sports {
    /// 体育运动 (主分区)
    Sports,
    /// 潮流运动
    Trend,
    /// 足球
    Football,
    /// 篮球
    Basketball,
    /// 跑步
    Running,
    /// 武术
    Kungfu,
    /// 格斗
    Fighting,
    /// 羽毛球
    Badminton,
    /// 体育资讯
    Information,
    /// 体育赛事
    Match,
    /// 体育综合
    Other,
}

/// 动物分区
pub enum Animal {
    /// 动物 (主分区)
    Animal,
    /// 猫
    Cat,
    /// 狗
    Dog,
    /// 小宠异宠
    Reptiles,
    /// 野生动物·动物解说科普
    Science,
    /// 动物综合·二创
    Other,
}

/// vlog分区
pub enum Vlog {
    /// vlog (主分区)
    Vlog,
    /// 中外生活vlog
    Life,
    /// 学生vlog
    Student,
    /// 职业vlog
    Career,
    /// 其他vlog
    Other,
}

/// 绘画分区
pub enum Painting {
    /// 绘画 (主分区)
    Painting,
    /// 二次元绘画
    Acg,
    /// 非二次元绘画
    NoneAcg,
    /// 绘画学习
    Tutorial,
    /// 绘画综合
    Other,
}

/// 人工智能分区
pub enum Ai {
    /// 人工智能 (主分区)
    Ai,
    /// AI学习
    Tutorial,
    /// AI资讯
    Information,
    /// AI杂谈
    Other,
}

/// 家装房产分区
pub enum Home {
    /// 家装房产 (主分区)
    Home,
    /// 买房租房
    Trade,
    /// 家庭装修
    Renovation,
    /// 家居展示
    Furniture,
    /// 家用电器
    Appliances,
}

/// 户外潮流分区
pub enum Outdoors {
    /// 户外潮流 (主分区)
    Outdoors,
    /// 露营
    Camping,
    /// 徒步
    Hiking,
    /// 户外探秘
    Explore,
    /// 户外综合
    Other,
}

/// 健身分区
pub enum Gym {
    /// 健身 (主分区)
    Gym,
    /// 健身科普
    Science,
    /// 健身跟练教学
    Tutorial,
    /// 健身记录
    Record,
    /// 健身身材展示
    Figure,
    /// 健身综合
    Other,
}

/// 手工分区
pub enum Handmake {
    /// 手工 (主分区)
    Handmake,
    /// 文具手帐
    Handbook,
    /// 轻手作
    Light,
    /// 传统手工艺
    Traditional,
    /// 解压手工
    Relief,
    /// DIY玩具
    Diy,
    /// 其他手工
    Other,
}

/// 旅游出行分区
pub enum Travel {
    /// 旅游出行 (主分区)
    Travel,
    /// 旅游记录
    Record,
    /// 旅游攻略
    Strategy,
    /// 城市出行
    City,
    /// 公共交通
    Transport,
}

/// 三农分区
pub enum Rural {
    /// 三农 (主分区)
    Rural,
    /// 农村种植
    Planting,
    /// 赶海捕鱼
    Fishing,
    /// 打野采摘
    Harvest,
    /// 农业技术
    Tech,
    /// 农村生活
    Life,
}

/// 亲子分区
pub enum Parenting {
    /// 亲子 (主分区)
    Parenting,
    /// 孕产护理
    PregnantCare,
    /// 婴幼护理
    InfantCare,
    /// 儿童才艺
    Talent,
    /// 萌娃
    Cute,
    /// 亲子互动
    Interaction,
    /// 亲子教育
    Education,
    /// 亲子综合
    Other,
}

/// 健康分区
pub enum Health {
    /// 健康 (主分区)
    Health,
    /// 健康科普
    Science,
    /// 养生
    Regimen,
    /// 两性知识
    Sexes,
    /// 心理健康
    Psychology,
    /// 助眠视频·ASMR
    Asmr,
    /// 医疗保健综合
    Other,
}

/// 情感分区
pub enum Emotion {
    /// 情感 (主分区)
    Emotion,
    /// 家庭关系
    Family,
    /// 恋爱关系
    Romantic,
    /// 人际关系
    Interpersonal,
    /// 自我成长
    Growth,
}

/// 生活兴趣分区
pub enum LifeJoy {
    /// 生活兴趣 (主分区)
    LifeJoy,
    /// 休闲玩乐
    Leisure,
    /// 线下演出
    OnSite,
    /// 文玩文创
    ArtisticProducts,
    /// 潮玩玩具
    TrendyToys,
    /// 兴趣综合
    Other,
}

/// 生活经验分区
pub enum LifeExperience {
    /// 生活经验 (主分区)
    LifeExperience,
    /// 生活技能
    Skills,
    /// 办事流程
    Procedures,
    /// 婚嫁
    Marriage,
}

/// 神秘学分区 (未公开)
pub enum Mysticism {
    /// 神秘学 (主分区)
    Mysticism,
    /// 塔罗占卜
    Tarot,
    /// 星座占星
    Horoscope,
    /// 传统玄学
    Metaphysics,
    /// 疗愈成长
    Healing,
    /// 其他神秘学
    Other,
}

impl VideoPartitionV2 {
    /// 获取分区代号（tname_v2）
    pub fn alias(&self) -> &'static str {
        match self {
            VideoPartitionV2::Douga(d) => d.alias(),
            VideoPartitionV2::Game(g) => g.alias(),
            VideoPartitionV2::Kichiku(k) => k.alias(),
            VideoPartitionV2::Music(m) => m.alias(),
            VideoPartitionV2::Dance(d) => d.alias(),
            VideoPartitionV2::Cinephile(c) => c.alias(),
            VideoPartitionV2::Ent(e) => e.alias(),
            VideoPartitionV2::Knowledge(k) => k.alias(),
            VideoPartitionV2::Tech(t) => t.alias(),
            VideoPartitionV2::Information(i) => i.alias(),
            VideoPartitionV2::Food(f) => f.alias(),
            VideoPartitionV2::Shortplay(s) => s.alias(),
            VideoPartitionV2::Car(c) => c.alias(),
            VideoPartitionV2::Fashion(f) => f.alias(),
            VideoPartitionV2::Sports(s) => s.alias(),
            VideoPartitionV2::Animal(a) => a.alias(),
            VideoPartitionV2::Vlog(v) => v.alias(),
            VideoPartitionV2::Painting(p) => p.alias(),
            VideoPartitionV2::Ai(a) => a.alias(),
            VideoPartitionV2::Home(h) => h.alias(),
            VideoPartitionV2::Outdoors(o) => o.alias(),
            VideoPartitionV2::Gym(g) => g.alias(),
            VideoPartitionV2::Handmake(h) => h.alias(),
            VideoPartitionV2::Travel(t) => t.alias(),
            VideoPartitionV2::Rural(r) => r.alias(),
            VideoPartitionV2::Parenting(p) => p.alias(),
            VideoPartitionV2::Health(h) => h.alias(),
            VideoPartitionV2::Emotion(e) => e.alias(),
            VideoPartitionV2::LifeJoy(l) => l.alias(),
            VideoPartitionV2::LifeExperience(l) => l.alias(),
            VideoPartitionV2::Mysticism(m) => m.alias(),
        }
    }

    /// 获取分区ID（tid_v2）
    pub fn tid(&self) -> u32 {
        match self {
            VideoPartitionV2::Douga(d) => d.tid(),
            VideoPartitionV2::Game(g) => g.tid(),
            VideoPartitionV2::Kichiku(k) => k.tid(),
            VideoPartitionV2::Music(m) => m.tid(),
            VideoPartitionV2::Dance(d) => d.tid(),
            VideoPartitionV2::Cinephile(c) => c.tid(),
            VideoPartitionV2::Ent(e) => e.tid(),
            VideoPartitionV2::Knowledge(k) => k.tid(),
            VideoPartitionV2::Tech(t) => t.tid(),
            VideoPartitionV2::Information(i) => i.tid(),
            VideoPartitionV2::Food(f) => f.tid(),
            VideoPartitionV2::Shortplay(s) => s.tid(),
            VideoPartitionV2::Car(c) => c.tid(),
            VideoPartitionV2::Fashion(f) => f.tid(),
            VideoPartitionV2::Sports(s) => s.tid(),
            VideoPartitionV2::Animal(a) => a.tid(),
            VideoPartitionV2::Vlog(v) => v.tid(),
            VideoPartitionV2::Painting(p) => p.tid(),
            VideoPartitionV2::Ai(a) => a.tid(),
            VideoPartitionV2::Home(h) => h.tid(),
            VideoPartitionV2::Outdoors(o) => o.tid(),
            VideoPartitionV2::Gym(g) => g.tid(),
            VideoPartitionV2::Handmake(h) => h.tid(),
            VideoPartitionV2::Travel(t) => t.tid(),
            VideoPartitionV2::Rural(r) => r.tid(),
            VideoPartitionV2::Parenting(p) => p.tid(),
            VideoPartitionV2::Health(h) => h.tid(),
            VideoPartitionV2::Emotion(e) => e.tid(),
            VideoPartitionV2::LifeJoy(l) => l.tid(),
            VideoPartitionV2::LifeExperience(l) => l.tid(),
            VideoPartitionV2::Mysticism(m) => m.tid(),
        }
    }

    /// 获取分区名称
    pub fn name(&self) -> &'static str {
        match self {
            VideoPartitionV2::Douga(d) => d.name(),
            VideoPartitionV2::Game(g) => g.name(),
            VideoPartitionV2::Kichiku(k) => k.name(),
            VideoPartitionV2::Music(m) => m.name(),
            VideoPartitionV2::Dance(d) => d.name(),
            VideoPartitionV2::Cinephile(c) => c.name(),
            VideoPartitionV2::Ent(e) => e.name(),
            VideoPartitionV2::Knowledge(k) => k.name(),
            VideoPartitionV2::Tech(t) => t.name(),
            VideoPartitionV2::Information(i) => i.name(),
            VideoPartitionV2::Food(f) => f.name(),
            VideoPartitionV2::Shortplay(s) => s.name(),
            VideoPartitionV2::Car(c) => c.name(),
            VideoPartitionV2::Fashion(f) => f.name(),
            VideoPartitionV2::Sports(s) => s.name(),
            VideoPartitionV2::Animal(a) => a.name(),
            VideoPartitionV2::Vlog(v) => v.name(),
            VideoPartitionV2::Painting(p) => p.name(),
            VideoPartitionV2::Ai(a) => a.name(),
            VideoPartitionV2::Home(h) => h.name(),
            VideoPartitionV2::Outdoors(o) => o.name(),
            VideoPartitionV2::Gym(g) => g.name(),
            VideoPartitionV2::Handmake(h) => h.name(),
            VideoPartitionV2::Travel(t) => t.name(),
            VideoPartitionV2::Rural(r) => r.name(),
            VideoPartitionV2::Parenting(p) => p.name(),
            VideoPartitionV2::Health(h) => h.name(),
            VideoPartitionV2::Emotion(e) => e.name(),
            VideoPartitionV2::LifeJoy(l) => l.name(),
            VideoPartitionV2::LifeExperience(l) => l.name(),
            VideoPartitionV2::Mysticism(m) => m.name(),
        }
    }
}

// 动画分区实现
impl Douga {
    pub fn alias(&self) -> &'static str {
        match self {
            Douga::Douga => "douga",
            Douga::FanAnime => "fan_anime",
            Douga::GarageKit => "garage_kit",
            Douga::Cosplay => "cosplay",
            Douga::Offline => "offline",
            Douga::Editing => "editing",
            Douga::Commentary => "commentary",
            Douga::QuickView => "quick_view",
            Douga::Voice => "voice",
            Douga::Information => "information",
            Douga::Interpret => "interpret",
            Douga::Vup => "vup",
            Douga::Tokusatsu => "tokusatsu",
            Douga::Puppetry => "puppetry",
            Douga::Comic => "comic",
            Douga::Motion => "motion",
            Douga::Reaction => "reaction",
            Douga::Tutorial => "tutorial",
            Douga::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Douga::Douga => 1005,
            Douga::FanAnime => 2037,
            Douga::GarageKit => 2038,
            Douga::Cosplay => 2039,
            Douga::Offline => 2040,
            Douga::Editing => 2041,
            Douga::Commentary => 2042,
            Douga::QuickView => 2043,
            Douga::Voice => 2044,
            Douga::Information => 2045,
            Douga::Interpret => 2046,
            Douga::Vup => 2047,
            Douga::Tokusatsu => 2048,
            Douga::Puppetry => 2049,
            Douga::Comic => 2050,
            Douga::Motion => 2051,
            Douga::Reaction => 2052,
            Douga::Tutorial => 2053,
            Douga::Other => 2054,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Douga::Douga => "动画 (主分区)",
            Douga::FanAnime => "同人动画",
            Douga::GarageKit => "模玩周边",
            Douga::Cosplay => "cosplay",
            Douga::Offline => "二次元线下",
            Douga::Editing => "动漫剪辑",
            Douga::Commentary => "动漫评论",
            Douga::QuickView => "动漫速读",
            Douga::Voice => "动漫配音",
            Douga::Information => "动漫资讯",
            Douga::Interpret => "网文解读",
            Douga::Vup => "虚拟up主",
            Douga::Tokusatsu => "特摄",
            Douga::Puppetry => "布袋戏",
            Douga::Comic => "漫画·动态漫",
            Douga::Motion => "广播剧",
            Douga::Reaction => "动漫reaction",
            Douga::Tutorial => "动漫教学",
            Douga::Other => "二次元其他",
        }
    }
}

// 游戏分区实现
impl Game {
    pub fn alias(&self) -> &'static str {
        match self {
            Game::Game => "game",
            Game::Rpg => "rpg",
            Game::MmorpG => "mmorpg",
            Game::StandAlone => "stand_alone",
            Game::Slg => "slg",
            Game::Tbs => "tbs",
            Game::Rts => "rts",
            Game::Moba => "moba",
            Game::Stg => "stg",
            Game::Spg => "spg",
            Game::Act => "act",
            Game::Msc => "msc",
            Game::Sim => "sim",
            Game::Otome => "otome",
            Game::Puz => "puz",
            Game::Sandbox => "sandbox",
            Game::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Game::Game => 1008,
            Game::Rpg => 2064,
            Game::MmorpG => 2065,
            Game::StandAlone => 2066,
            Game::Slg => 2067,
            Game::Tbs => 2068,
            Game::Rts => 2069,
            Game::Moba => 2070,
            Game::Stg => 2071,
            Game::Spg => 2072,
            Game::Act => 2073,
            Game::Msc => 2074,
            Game::Sim => 2075,
            Game::Otome => 2076,
            Game::Puz => 2077,
            Game::Sandbox => 2078,
            Game::Other => 2079,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Game::Game => "游戏 (主分区)",
            Game::Rpg => "单人RPG游戏",
            Game::MmorpG => "MMORPG游戏",
            Game::StandAlone => "单机主机类游戏",
            Game::Slg => "SLG游戏",
            Game::Tbs => "回合制策略游戏",
            Game::Rts => "即时策略游戏",
            Game::Moba => "MOBA游戏",
            Game::Stg => "射击游戏",
            Game::Spg => "体育竞速游戏",
            Game::Act => "动作竞技游戏",
            Game::Msc => "音游舞游",
            Game::Sim => "模拟经营游戏",
            Game::Otome => "女性向游戏",
            Game::Puz => "休闲/小游戏",
            Game::Sandbox => "沙盒类",
            Game::Other => "其他游戏",
        }
    }
}

// 鬼畜分区实现
impl Kichiku {
    pub fn alias(&self) -> &'static str {
        match self {
            Kichiku::Kichiku => "kichiku",
            Kichiku::Guide => "guide",
            Kichiku::Theatre => "theatre",
            Kichiku::ManualVocaloid => "manual_vocaloid",
            Kichiku::Mad => "mad",
            Kichiku::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Kichiku::Kichiku => 1007,
            Kichiku::Guide => 2059,
            Kichiku::Theatre => 2060,
            Kichiku::ManualVocaloid => 2061,
            Kichiku::Mad => 2062,
            Kichiku::Other => 2063,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Kichiku::Kichiku => "鬼畜 (主分区)",
            Kichiku::Guide => "鬼畜调教",
            Kichiku::Theatre => "鬼畜剧场",
            Kichiku::ManualVocaloid => "人力VOCALOID",
            Kichiku::Mad => "音MAD",
            Kichiku::Other => "鬼畜综合",
        }
    }
}

// 音乐分区实现
impl Music {
    pub fn alias(&self) -> &'static str {
        match self {
            Music::Music => "music",
            Music::Original => "original",
            Music::Mv => "mv",
            Music::Live => "live",
            Music::FanVideos => "fan_videos",
            Music::Cover => "cover",
            Music::Perform => "perform",
            Music::Vocaloid => "vocaloid",
            Music::AiMusic => "ai_music",
            Music::Radio => "radio",
            Music::Tutorial => "tutorial",
            Music::Commentary => "commentary",
            Music::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Music::Music => 1003,
            Music::Original => 2016,
            Music::Mv => 2017,
            Music::Live => 2018,
            Music::FanVideos => 2019,
            Music::Cover => 2020,
            Music::Perform => 2021,
            Music::Vocaloid => 2022,
            Music::AiMusic => 2023,
            Music::Radio => 2024,
            Music::Tutorial => 2025,
            Music::Commentary => 2026,
            Music::Other => 2027,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Music::Music => "音乐 (主分区)",
            Music::Original => "原创音乐",
            Music::Mv => "MV",
            Music::Live => "音乐现场",
            Music::FanVideos => "乐迷饭拍",
            Music::Cover => "翻唱",
            Music::Perform => "演奏",
            Music::Vocaloid => "VOCALOID",
            Music::AiMusic => "AI音乐",
            Music::Radio => "电台·歌单",
            Music::Tutorial => "音乐教学",
            Music::Commentary => "乐评盘点",
            Music::Other => "音乐综合",
        }
    }
}

// 舞蹈分区实现
impl Dance {
    pub fn alias(&self) -> &'static str {
        match self {
            Dance::Dance => "dance",
            Dance::Otaku => "otaku",
            Dance::Hiphop => "hiphop",
            Dance::Gestures => "gestures",
            Dance::Star => "star",
            Dance::China => "china",
            Dance::Tutorial => "tutorial",
            Dance::Ballet => "ballet",
            Dance::Wota => "wota",
            Dance::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Dance::Dance => 1004,
            Dance::Otaku => 2028,
            Dance::Hiphop => 2029,
            Dance::Gestures => 2030,
            Dance::Star => 2031,
            Dance::China => 2032,
            Dance::Tutorial => 2033,
            Dance::Ballet => 2034,
            Dance::Wota => 2035,
            Dance::Other => 2036,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Dance::Dance => "舞蹈 (主分区)",
            Dance::Otaku => "宅舞",
            Dance::Hiphop => "街舞",
            Dance::Gestures => "颜值·网红舞",
            Dance::Star => "明星舞蹈",
            Dance::China => "国风舞蹈",
            Dance::Tutorial => "舞蹈教学",
            Dance::Ballet => "芭蕾舞",
            Dance::Wota => "wota艺",
            Dance::Other => "舞蹈综合",
        }
    }
}

// 影视分区实现
impl Cinephile {
    pub fn alias(&self) -> &'static str {
        match self {
            Cinephile::Cinephile => "cinephile",
            Cinephile::Commentary => "commentary",
            Cinephile::Montage => "montage",
            Cinephile::Information => "information",
            Cinephile::Porterage => "porterage",
            Cinephile::Shortfilm => "shortfilm",
            Cinephile::Ai => "ai",
            Cinephile::Reaction => "reaction",
            Cinephile::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Cinephile::Cinephile => 1001,
            Cinephile::Commentary => 2001,
            Cinephile::Montage => 2002,
            Cinephile::Information => 2003,
            Cinephile::Porterage => 2004,
            Cinephile::Shortfilm => 2005,
            Cinephile::Ai => 2006,
            Cinephile::Reaction => 2007,
            Cinephile::Other => 2008,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Cinephile::Cinephile => "影视 (主分区)",
            Cinephile::Commentary => "影视解读",
            Cinephile::Montage => "影视剪辑",
            Cinephile::Information => "影视资讯",
            Cinephile::Porterage => "影视正片搬运",
            Cinephile::Shortfilm => "短剧短片",
            Cinephile::Ai => "AI影视",
            Cinephile::Reaction => "影视reaction",
            Cinephile::Other => "影视综合",
        }
    }
}

// 娱乐分区实现
impl Ent {
    pub fn alias(&self) -> &'static str {
        match self {
            Ent::Ent => "ent",
            Ent::Commentary => "commentary",
            Ent::Montage => "montage",
            Ent::FansVideo => "fans_video",
            Ent::Information => "information",
            Ent::Reaction => "reaction",
            Ent::Variety => "variety",
            Ent::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Ent::Ent => 1002,
            Ent::Commentary => 2009,
            Ent::Montage => 2010,
            Ent::FansVideo => 2011,
            Ent::Information => 2012,
            Ent::Reaction => 2013,
            Ent::Variety => 2014,
            Ent::Other => 2015,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Ent::Ent => "娱乐 (主分区)",
            Ent::Commentary => "娱乐评论",
            Ent::Montage => "明星剪辑",
            Ent::FansVideo => "娱乐饭拍&现场",
            Ent::Information => "娱乐资讯",
            Ent::Reaction => "娱乐reaction",
            Ent::Variety => "娱乐综艺正片",
            Ent::Other => "娱乐综合",
        }
    }
}

// 知识分区实现
impl Knowledge {
    pub fn alias(&self) -> &'static str {
        match self {
            Knowledge::Knowledge => "knowledge",
            Knowledge::Exam => "exam",
            Knowledge::LangSkill => "lang_skill",
            Knowledge::Campus => "campus",
            Knowledge::Business => "business",
            Knowledge::SocialObservation => "social_observation",
            Knowledge::Politics => "politics",
            Knowledge::HumanityHistory => "humanity_history",
            Knowledge::Design => "design",
            Knowledge::Psychology => "psychology",
            Knowledge::Career => "career",
            Knowledge::Science => "science",
            Knowledge::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Knowledge::Knowledge => 1010,
            Knowledge::Exam => 2084,
            Knowledge::LangSkill => 2085,
            Knowledge::Campus => 2086,
            Knowledge::Business => 2087,
            Knowledge::SocialObservation => 2088,
            Knowledge::Politics => 2089,
            Knowledge::HumanityHistory => 2090,
            Knowledge::Design => 2091,
            Knowledge::Psychology => 2092,
            Knowledge::Career => 2093,
            Knowledge::Science => 2094,
            Knowledge::Other => 2095,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Knowledge::Knowledge => "知识 (主分区)",
            Knowledge::Exam => "应试教育",
            Knowledge::LangSkill => "非应试语言学习",
            Knowledge::Campus => "大学专业知识",
            Knowledge::Business => "商业财经",
            Knowledge::SocialObservation => "社会观察",
            Knowledge::Politics => "时政解读",
            Knowledge::HumanityHistory => "人文历史",
            Knowledge::Design => "设计艺术",
            Knowledge::Psychology => "心理杂谈",
            Knowledge::Career => "职场发展",
            Knowledge::Science => "科学科普",
            Knowledge::Other => "其他知识杂谈",
        }
    }
}

// 科技数码分区实现
impl Tech {
    pub fn alias(&self) -> &'static str {
        match self {
            Tech::Tech => "tech",
            Tech::Computer => "computer",
            Tech::Phone => "phone",
            Tech::Pad => "pad",
            Tech::Photography => "photography",
            Tech::Machine => "machine",
            Tech::Create => "create",
            Tech::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Tech::Tech => 1012,
            Tech::Computer => 2099,
            Tech::Phone => 2100,
            Tech::Pad => 2101,
            Tech::Photography => 2102,
            Tech::Machine => 2103,
            Tech::Create => 2104,
            Tech::Other => 2105,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Tech::Tech => "科技数码 (主分区)",
            Tech::Computer => "电脑",
            Tech::Phone => "手机",
            Tech::Pad => "平板电脑",
            Tech::Photography => "摄影摄像",
            Tech::Machine => "工程机械",
            Tech::Create => "自制发明/设备",
            Tech::Other => "科技数码综合",
        }
    }
}

// 资讯分区实现
impl Information {
    pub fn alias(&self) -> &'static str {
        match self {
            Information::Information => "information",
            Information::Politics => "politics",
            Information::Overseas => "overseas",
            Information::Social => "social",
            Information::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Information::Information => 1009,
            Information::Politics => 2080,
            Information::Overseas => 2081,
            Information::Social => 2082,
            Information::Other => 2083,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Information::Information => "资讯 (主分区)",
            Information::Politics => "时政资讯",
            Information::Overseas => "海外资讯",
            Information::Social => "社会资讯",
            Information::Other => "综合资讯",
        }
    }
}

// 美食分区实现
impl Food {
    pub fn alias(&self) -> &'static str {
        match self {
            Food::Food => "food",
            Food::Make => "make",
            Food::Detective => "detective",
            Food::Commentary => "commentary",
            Food::Record => "record",
            Food::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Food::Food => 1020,
            Food::Make => 2149,
            Food::Detective => 2150,
            Food::Commentary => 2151,
            Food::Record => 2152,
            Food::Other => 2153,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Food::Food => "美食 (主分区)",
            Food::Make => "美食制作",
            Food::Detective => "美食探店",
            Food::Commentary => "美食测评",
            Food::Record => "美食记录",
            Food::Other => "美食综合",
        }
    }
}

// 小剧场分区实现
impl Shortplay {
    pub fn alias(&self) -> &'static str {
        match self {
            Shortplay::Shortplay => "shortplay",
            Shortplay::Plot => "plot",
            Shortplay::Lang => "lang",
            Shortplay::UpVariety => "up_variety",
            Shortplay::Interview => "interview",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Shortplay::Shortplay => 1021,
            Shortplay::Plot => 2154,
            Shortplay::Lang => 2155,
            Shortplay::UpVariety => 2156,
            Shortplay::Interview => 2157,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Shortplay::Shortplay => "小剧场 (主分区)",
            Shortplay::Plot => "剧情演绎",
            Shortplay::Lang => "语言类小剧场",
            Shortplay::UpVariety => "UP主小综艺",
            Shortplay::Interview => "街头采访",
        }
    }
}

// 汽车分区实现
impl Car {
    pub fn alias(&self) -> &'static str {
        match self {
            Car::Car => "car",
            Car::Commentary => "commentary",
            Car::Culture => "culture",
            Car::Life => "life",
            Car::Tech => "tech",
            Car::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Car::Car => 1013,
            Car::Commentary => 2106,
            Car::Culture => 2107,
            Car::Life => 2108,
            Car::Tech => 2109,
            Car::Other => 2110,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Car::Car => "汽车 (主分区)",
            Car::Commentary => "汽车测评",
            Car::Culture => "汽车文化",
            Car::Life => "汽车生活",
            Car::Tech => "汽车技术",
            Car::Other => "汽车综合",
        }
    }
}

// 时尚美妆分区实现
impl Fashion {
    pub fn alias(&self) -> &'static str {
        match self {
            Fashion::Fashion => "fashion",
            Fashion::Makeup => "makeup",
            Fashion::Skincare => "skincare",
            Fashion::Cos => "cos",
            Fashion::Outfits => "outfits",
            Fashion::Accessories => "accessories",
            Fashion::Jewelry => "jewelry",
            Fashion::Trick => "trick",
            Fashion::Commentary => "commentary",
            Fashion::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Fashion::Fashion => 1014,
            Fashion::Makeup => 2111,
            Fashion::Skincare => 2112,
            Fashion::Cos => 2113,
            Fashion::Outfits => 2114,
            Fashion::Accessories => 2115,
            Fashion::Jewelry => 2116,
            Fashion::Trick => 2117,
            Fashion::Commentary => 2118,
            Fashion::Other => 2119,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Fashion::Fashion => "时尚美妆 (主分区)",
            Fashion::Makeup => "美妆",
            Fashion::Skincare => "护肤",
            Fashion::Cos => "仿装cos",
            Fashion::Outfits => "鞋服穿搭",
            Fashion::Accessories => "箱包配饰",
            Fashion::Jewelry => "珠宝首饰",
            Fashion::Trick => "三坑",
            Fashion::Commentary => "时尚解读",
            Fashion::Other => "时尚综合",
        }
    }
}

// 体育运动分区实现
impl Sports {
    pub fn alias(&self) -> &'static str {
        match self {
            Sports::Sports => "sports",
            Sports::Trend => "trend",
            Sports::Football => "football",
            Sports::Basketball => "basketball",
            Sports::Running => "running",
            Sports::Kungfu => "kungfu",
            Sports::Fighting => "fighting",
            Sports::Badminton => "badminton",
            Sports::Information => "information",
            Sports::Match => "match",
            Sports::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Sports::Sports => 1018,
            Sports::Trend => 2133,
            Sports::Football => 2134,
            Sports::Basketball => 2135,
            Sports::Running => 2136,
            Sports::Kungfu => 2137,
            Sports::Fighting => 2138,
            Sports::Badminton => 2139,
            Sports::Information => 2140,
            Sports::Match => 2141,
            Sports::Other => 2142,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Sports::Sports => "体育运动 (主分区)",
            Sports::Trend => "潮流运动",
            Sports::Football => "足球",
            Sports::Basketball => "篮球",
            Sports::Running => "跑步",
            Sports::Kungfu => "武术",
            Sports::Fighting => "格斗",
            Sports::Badminton => "羽毛球",
            Sports::Information => "体育资讯",
            Sports::Match => "体育赛事",
            Sports::Other => "体育综合",
        }
    }
}

// 动物分区实现
impl Animal {
    pub fn alias(&self) -> &'static str {
        match self {
            Animal::Animal => "animal",
            Animal::Cat => "cat",
            Animal::Dog => "dog",
            Animal::Reptiles => "reptiles",
            Animal::Science => "science",
            Animal::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Animal::Animal => 1024,
            Animal::Cat => 2167,
            Animal::Dog => 2168,
            Animal::Reptiles => 2169,
            Animal::Science => 2170,
            Animal::Other => 2171,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Animal::Animal => "动物 (主分区)",
            Animal::Cat => "猫",
            Animal::Dog => "狗",
            Animal::Reptiles => "小宠异宠",
            Animal::Science => "野生动物·动物解说科普",
            Animal::Other => "动物综合·二创",
        }
    }
}

// vlog分区实现
impl Vlog {
    pub fn alias(&self) -> &'static str {
        match self {
            Vlog::Vlog => "vlog",
            Vlog::Life => "life",
            Vlog::Student => "student",
            Vlog::Career => "career",
            Vlog::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Vlog::Vlog => 1029,
            Vlog::Life => 2194,
            Vlog::Student => 2195,
            Vlog::Career => 2196,
            Vlog::Other => 2197,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Vlog::Vlog => "vlog (主分区)",
            Vlog::Life => "中外生活vlog",
            Vlog::Student => "学生vlog",
            Vlog::Career => "职业vlog",
            Vlog::Other => "其他vlog",
        }
    }
}

// 绘画分区实现
impl Painting {
    pub fn alias(&self) -> &'static str {
        match self {
            Painting::Painting => "painting",
            Painting::Acg => "acg",
            Painting::NoneAcg => "none_acg",
            Painting::Tutorial => "tutorial",
            Painting::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Painting::Painting => 1006,
            Painting::Acg => 2055,
            Painting::NoneAcg => 2056,
            Painting::Tutorial => 2057,
            Painting::Other => 2058,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Painting::Painting => "绘画 (主分区)",
            Painting::Acg => "二次元绘画",
            Painting::NoneAcg => "非二次元绘画",
            Painting::Tutorial => "绘画学习",
            Painting::Other => "绘画综合",
        }
    }
}

// 人工智能分区实现
impl Ai {
    pub fn alias(&self) -> &'static str {
        match self {
            Ai::Ai => "ai",
            Ai::Tutorial => "tutorial",
            Ai::Information => "information",
            Ai::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Ai::Ai => 1011,
            Ai::Tutorial => 2096,
            Ai::Information => 2097,
            Ai::Other => 2098,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Ai::Ai => "人工智能 (主分区)",
            Ai::Tutorial => "AI学习",
            Ai::Information => "AI资讯",
            Ai::Other => "AI杂谈",
        }
    }
}

// 家装房产分区实现
impl Home {
    pub fn alias(&self) -> &'static str {
        match self {
            Home::Home => "home",
            Home::Trade => "trade",
            Home::Renovation => "renovation",
            Home::Furniture => "furniture",
            Home::Appliances => "appliances",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Home::Home => 1015,
            Home::Trade => 2120,
            Home::Renovation => 2121,
            Home::Furniture => 2122,
            Home::Appliances => 2123,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Home::Home => "家装房产 (主分区)",
            Home::Trade => "买房租房",
            Home::Renovation => "家庭装修",
            Home::Furniture => "家居展示",
            Home::Appliances => "家用电器",
        }
    }
}

// 户外潮流分区实现
impl Outdoors {
    pub fn alias(&self) -> &'static str {
        match self {
            Outdoors::Outdoors => "outdoors",
            Outdoors::Camping => "camping",
            Outdoors::Hiking => "hiking",
            Outdoors::Explore => "explore",
            Outdoors::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Outdoors::Outdoors => 1016,
            Outdoors::Camping => 2124,
            Outdoors::Hiking => 2125,
            Outdoors::Explore => 2126,
            Outdoors::Other => 2127,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Outdoors::Outdoors => "户外潮流 (主分区)",
            Outdoors::Camping => "露营",
            Outdoors::Hiking => "徒步",
            Outdoors::Explore => "户外探秘",
            Outdoors::Other => "户外综合",
        }
    }
}

// 健身分区实现
impl Gym {
    pub fn alias(&self) -> &'static str {
        match self {
            Gym::Gym => "gym",
            Gym::Science => "science",
            Gym::Tutorial => "tutorial",
            Gym::Record => "record",
            Gym::Figure => "figure",
            Gym::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Gym::Gym => 1017,
            Gym::Science => 2128,
            Gym::Tutorial => 2129,
            Gym::Record => 2130,
            Gym::Figure => 2131,
            Gym::Other => 2132,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Gym::Gym => "健身 (主分区)",
            Gym::Science => "健身科普",
            Gym::Tutorial => "健身跟练教学",
            Gym::Record => "健身记录",
            Gym::Figure => "健身身材展示",
            Gym::Other => "健身综合",
        }
    }
}

// 手工分区实现
impl Handmake {
    pub fn alias(&self) -> &'static str {
        match self {
            Handmake::Handmake => "handmake",
            Handmake::Handbook => "handbook",
            Handmake::Light => "light",
            Handmake::Traditional => "traditional",
            Handmake::Relief => "relief",
            Handmake::Diy => "diy",
            Handmake::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Handmake::Handmake => 1019,
            Handmake::Handbook => 2143,
            Handmake::Light => 2144,
            Handmake::Traditional => 2145,
            Handmake::Relief => 2146,
            Handmake::Diy => 2147,
            Handmake::Other => 2148,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Handmake::Handmake => "手工 (主分区)",
            Handmake::Handbook => "文具手帐",
            Handmake::Light => "轻手作",
            Handmake::Traditional => "传统手工艺",
            Handmake::Relief => "解压手工",
            Handmake::Diy => "DIY玩具",
            Handmake::Other => "其他手工",
        }
    }
}

// 旅游出行分区实现
impl Travel {
    pub fn alias(&self) -> &'static str {
        match self {
            Travel::Travel => "travel",
            Travel::Record => "record",
            Travel::Strategy => "strategy",
            Travel::City => "city",
            Travel::Transport => "transport",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Travel::Travel => 1022,
            Travel::Record => 2158,
            Travel::Strategy => 2159,
            Travel::City => 2160,
            Travel::Transport => 2161,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Travel::Travel => "旅游出行 (主分区)",
            Travel::Record => "旅游记录",
            Travel::Strategy => "旅游攻略",
            Travel::City => "城市出行",
            Travel::Transport => "公共交通",
        }
    }
}

// 三农分区实现
impl Rural {
    pub fn alias(&self) -> &'static str {
        match self {
            Rural::Rural => "rural",
            Rural::Planting => "planting",
            Rural::Fishing => "fishing",
            Rural::Harvest => "harvest",
            Rural::Tech => "tech",
            Rural::Life => "life",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Rural::Rural => 1023,
            Rural::Planting => 2162,
            Rural::Fishing => 2163,
            Rural::Harvest => 2164,
            Rural::Tech => 2165,
            Rural::Life => 2166,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Rural::Rural => "三农 (主分区)",
            Rural::Planting => "农村种植",
            Rural::Fishing => "赶海捕鱼",
            Rural::Harvest => "打野采摘",
            Rural::Tech => "农业技术",
            Rural::Life => "农村生活",
        }
    }
}

// 亲子分区实现
impl Parenting {
    pub fn alias(&self) -> &'static str {
        match self {
            Parenting::Parenting => "parenting",
            Parenting::PregnantCare => "pregnant_care",
            Parenting::InfantCare => "infant_care",
            Parenting::Talent => "talent",
            Parenting::Cute => "cute",
            Parenting::Interaction => "interaction",
            Parenting::Education => "education",
            Parenting::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Parenting::Parenting => 1025,
            Parenting::PregnantCare => 2172,
            Parenting::InfantCare => 2173,
            Parenting::Talent => 2174,
            Parenting::Cute => 2175,
            Parenting::Interaction => 2176,
            Parenting::Education => 2177,
            Parenting::Other => 2178,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Parenting::Parenting => "亲子 (主分区)",
            Parenting::PregnantCare => "孕产护理",
            Parenting::InfantCare => "婴幼护理",
            Parenting::Talent => "儿童才艺",
            Parenting::Cute => "萌娃",
            Parenting::Interaction => "亲子互动",
            Parenting::Education => "亲子教育",
            Parenting::Other => "亲子综合",
        }
    }
}

// 健康分区实现
impl Health {
    pub fn alias(&self) -> &'static str {
        match self {
            Health::Health => "health",
            Health::Science => "science",
            Health::Regimen => "regimen",
            Health::Sexes => "sexes",
            Health::Psychology => "psychology",
            Health::Asmr => "asmr",
            Health::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Health::Health => 1026,
            Health::Science => 2179,
            Health::Regimen => 2180,
            Health::Sexes => 2181,
            Health::Psychology => 2182,
            Health::Asmr => 2183,
            Health::Other => 2184,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Health::Health => "健康 (主分区)",
            Health::Science => "健康科普",
            Health::Regimen => "养生",
            Health::Sexes => "两性知识",
            Health::Psychology => "心理健康",
            Health::Asmr => "助眠视频·ASMR",
            Health::Other => "医疗保健综合",
        }
    }
}

// 情感分区实现
impl Emotion {
    pub fn alias(&self) -> &'static str {
        match self {
            Emotion::Emotion => "emotion",
            Emotion::Family => "family",
            Emotion::Romantic => "romantic",
            Emotion::Interpersonal => "interpersonal",
            Emotion::Growth => "growth",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Emotion::Emotion => 1027,
            Emotion::Family => 2185,
            Emotion::Romantic => 2186,
            Emotion::Interpersonal => 2187,
            Emotion::Growth => 2188,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Emotion::Emotion => "情感 (主分区)",
            Emotion::Family => "家庭关系",
            Emotion::Romantic => "恋爱关系",
            Emotion::Interpersonal => "人际关系",
            Emotion::Growth => "自我成长",
        }
    }
}

// 生活兴趣分区实现
impl LifeJoy {
    pub fn alias(&self) -> &'static str {
        match self {
            LifeJoy::LifeJoy => "life_joy",
            LifeJoy::Leisure => "leisure",
            LifeJoy::OnSite => "on_site",
            LifeJoy::ArtisticProducts => "artistic_products",
            LifeJoy::TrendyToys => "trendy_toys",
            LifeJoy::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            LifeJoy::LifeJoy => 1030,
            LifeJoy::Leisure => 2198,
            LifeJoy::OnSite => 2199,
            LifeJoy::ArtisticProducts => 2200,
            LifeJoy::TrendyToys => 2201,
            LifeJoy::Other => 2202,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            LifeJoy::LifeJoy => "生活兴趣 (主分区)",
            LifeJoy::Leisure => "休闲玩乐",
            LifeJoy::OnSite => "线下演出",
            LifeJoy::ArtisticProducts => "文玩文创",
            LifeJoy::TrendyToys => "潮玩玩具",
            LifeJoy::Other => "兴趣综合",
        }
    }
}

// 生活经验分区实现
impl LifeExperience {
    pub fn alias(&self) -> &'static str {
        match self {
            LifeExperience::LifeExperience => "life_experience",
            LifeExperience::Skills => "skills",
            LifeExperience::Procedures => "procedures",
            LifeExperience::Marriage => "marriage",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            LifeExperience::LifeExperience => 1031,
            LifeExperience::Skills => 2203,
            LifeExperience::Procedures => 2204,
            LifeExperience::Marriage => 2205,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            LifeExperience::LifeExperience => "生活经验 (主分区)",
            LifeExperience::Skills => "生活技能",
            LifeExperience::Procedures => "办事流程",
            LifeExperience::Marriage => "婚嫁",
        }
    }
}

// 神秘学分区实现
impl Mysticism {
    pub fn alias(&self) -> &'static str {
        match self {
            Mysticism::Mysticism => "mysticism",
            Mysticism::Tarot => "tarot",
            Mysticism::Horoscope => "horoscope",
            Mysticism::Metaphysics => "metaphysics",
            Mysticism::Healing => "healing",
            Mysticism::Other => "other",
        }
    }
    pub fn tid(&self) -> u32 {
        match self {
            Mysticism::Mysticism => 1028,
            Mysticism::Tarot => 2189,
            Mysticism::Horoscope => 2190,
            Mysticism::Metaphysics => 2191,
            Mysticism::Healing => 2192,
            Mysticism::Other => 2193,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Mysticism::Mysticism => "神秘学 (主分区)",
            Mysticism::Tarot => "塔罗占卜",
            Mysticism::Horoscope => "星座占星",
            Mysticism::Metaphysics => "传统玄学",
            Mysticism::Healing => "疗愈成长",
            Mysticism::Other => "其他神秘学",
        }
    }
}
