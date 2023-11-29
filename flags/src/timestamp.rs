use crate::FormatT;
use chrono::{DateTime, FixedOffset};
use utility::{Error, Result};

pub trait TimestampFormat {
    fn as_time(&self) -> Result<DateTime<FixedOffset>>;
}
/// TimestampFlag implements the Value interface to accept and validate a
/// RFC3339 timestamp string as a flag
pub struct TimestampFlag(String);

impl FormatT for Option<TimestampFlag> {
    fn get_type() -> &'static str {
        "timestamp"
    }

    ///string implements Stringer
    fn string(&self) -> String {
        if let Some(val) = self {
            String::from(val.0.clone())
        } else {
            return String::new();
        }
    }

    fn set(&mut self, val: &str) -> Result<()> {
        if let Err(e) = chrono::DateTime::parse_from_rfc3339(val) {
            Err(Error::Custom(format!("{:?}", e)))
        } else {
            *self = Some(TimestampFlag(val.into()));
            Ok(())
        }
    }
}

impl TimestampFormat for Option<TimestampFlag> {
    fn as_time(&self) -> Result<DateTime<FixedOffset>> {
        match self {
            None => Err(Error::Custom(format!("empty timestamp"))),
            Some(time) => DateTime::parse_from_rfc3339(time.0.as_str())
                .map_err(|e| Error::Custom(format!("{:?}", e))),
        }
    }
}
