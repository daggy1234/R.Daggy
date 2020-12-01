use num_integer::Integer;
use serenity::prelude::*;
use time::{self, OffsetDateTime};
pub struct UptimerKey;

impl TypeMapKey for UptimerKey {
    type Value = Uptimer;
}

pub struct Uptimer {
    started_at: OffsetDateTime,
}

impl Uptimer {
    pub fn new() -> Uptimer {
        Uptimer {
            started_at: OffsetDateTime::now_utc(),
        }
    }
    pub fn uptime_string(&self) -> String {
        let seconds = (OffsetDateTime::now_utc() - self.started_at).whole_seconds();
        let (minutes, seconds) = seconds.div_rem(&60);
        let (hours, minutes) = minutes.div_rem(&60);
        let (days, hours) = hours.div_rem(&24);
        format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
    }
}
