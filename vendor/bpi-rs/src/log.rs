use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;

pub struct TimeFormat;

impl FormatTime for TimeFormat {
    fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO);

        // 转换为中国时区 (UTC+8)
        let china_time = now + Duration::from_secs(8 * 3600);
        let total_seconds = china_time.as_secs();

        // 计算日期和时间
        let days_since_epoch = total_seconds / 86400;
        let seconds_today = total_seconds % 86400;

        let hours = (seconds_today / 3600) % 24;
        let minutes = (seconds_today / 60) % 60;
        let seconds = seconds_today % 60;

        // 计算月份和日期 (简化版本，1970年1月1日开始)
        let (month, day) = days_to_month_day(days_since_epoch);

        // 灰色时间
        write!(
            w,
            "\x1b[90m{:02}-{:02} {:02}:{:02}:{:02}\x1b[0m",
            month, day, hours, minutes, seconds
        )
    }
}

// 简化的日期计算函数
fn days_to_month_day(days: u64) -> (u32, u32) {
    // 这是一个简化版本，基于平均月份长度
    // 对于生产环境，建议使用更精确的日期计算
    let year_days = days % 365;
    let month = ((year_days * 12) / 365) + 1;
    let day = (year_days % 30) + 1;
    (month as u32, day as u32)
}

pub fn init_log() {
    let _ = tracing_subscriber::fmt()
        .with_timer(TimeFormat)
        .with_target(false)
        .compact()
        .try_init();
}
