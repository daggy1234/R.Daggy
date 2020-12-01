#[allow(unused_imports)]
use num_integer::Integer;
use time::Duration;

pub fn humanise_time(interval: Duration) -> String {
    if interval.whole_days() > 0 {
        return format!("Started {} days ago", interval.whole_days());
    } else {
        if interval.whole_hours() > 0 {
            return format!("Started {} hours ago", interval.whole_hours());
        } else {
            if interval.whole_minutes() > 0 {
                return format!(
                    "Started {} minutes and {} seconds ago",
                    interval.whole_minutes(),
                    interval.whole_seconds() - (interval.whole_minutes() * 60)
                );
            } else {
                return format!("Started {} seconds ago", interval.whole_seconds());
            }
        }
    }
}

pub fn min_sec_parse(interval: Duration) -> String {
    return format!(
        "{}:{}",
        interval.whole_minutes(),
        interval.whole_seconds() - (interval.whole_minutes() * 60)
    );
}
