use super::error::ErrorCategory;
use std::collections::HashMap;
use std::sync::OnceLock;

// 错误码映射表
static ERROR_MAP: OnceLock<HashMap<i32, ErrorInfo>> = OnceLock::new();

#[derive(Clone)]
struct ErrorInfo {
    message: &'static str,
    category: ErrorCategory,
}

impl ErrorInfo {
    fn new(message: &'static str, category: ErrorCategory) -> Self {
        Self { message, category }
    }
}

pub fn get_error_message(code: i32) -> String {
    get_error_map()
        .get(&code)
        .map(|info| info.message.to_string())
        .unwrap_or_else(|| "未知错误".to_string())
}

pub fn categorize_error(code: i32) -> ErrorCategory {
    get_error_map()
        .get(&code)
        .map(|info| info.category.clone())
        .unwrap_or(ErrorCategory::Unknown)
}

// 宏：自动生成错误码映射
macro_rules! define_errors {
  ($($category:ident: [$(($code:expr, $msg:expr)),* $(,)?]),* $(,)?) => {
        fn get_error_map() -> &'static HashMap<i32, ErrorInfo> {
            ERROR_MAP.get_or_init(|| {
                let mut map = HashMap::new();

                $(
                    // 为每个分类添加错误码
                    $(
                        map.insert($code, ErrorInfo::new($msg, ErrorCategory::$category));
                    )*
                )*

                map
            })
        }
  };
}

// 使用宏定义所有错误码
define_errors! {
    Auth: [
        (-1, "应用程序不存在或已被封禁"),
        (-2, "Access Key 错误"),
        (-3, "API 校验密匙错误"),
        (-4, "调用方对该 Method 没有权限"),
        (-101, "账号未登录"),
        (-102, "账号被封停"),
        (-105, "验证码错误"),
        (-107, "应用不存在或者被封禁"),
        (-108, "未绑定手机"),
        (-110, "未绑定手机"),
        (-111, "csrf 校验失败"),
        (-113, "账号尚未实名认证"),
        (-114, "请先绑定手机"),
        (-115, "请先完成实名认证"),
    ],

    Business: [
        (-103, "积分不足"),
        (-104, "硬币不足"),
        (-106, "账号非正式会员或在适应期"),
        (-509, "超出限制"),
        (-650, "用户等级太低"),
        (-688, "地理区域限制"),
        (-689, "版权限制"),
        (-701, "扣节操失败"),
    ],

    Request: [
        (-304, "木有改动"),
        (-307, "撞车跳转"),
        (-352, "风控校验失败 (UA 或 wbi 参数不合法)"),
        (-400, "请求错误"),
        (-401, "未认证 (或非法请求)"),
        (-403, "访问权限不足"),
        (-404, "啥都木有"),
        (-405, "不支持该方法"),
        (-409, "冲突"),
        (-412, "请求被拦截 (客户端 ip 被服务端风控)"),
        (-616, "上传文件不存在"),
        (-617, "上传文件太大"),
        (-625, "登录失败次数太多"),
        (-626, "用户不存在"),
        (-628, "密码太弱"),
        (-629, "用户名或密码错误"),
        (-632, "操作对象数量限制"),
        (-643, "被锁定"),
        (-652, "重复的用户"),
        (-658, "Token 过期"),
        (-662, "密码时间戳过期"),
    ],

    Server: [
        (-112, "系统升级中"),
        (-500, "服务器错误"),
        (-503, "过载保护,服务暂不可用"),
        (-504, "服务调用超时"),
        (-799, "请求过于频繁，请稍后再试"),
        (-8888, "对不起，服务器开小差了~ (ಥ﹏ಥ)"),
    ],
}
