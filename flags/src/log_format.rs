use crate::FormatT;
use utility::{Error, Result};

/// LogFormat determines the output format of the log stream
pub struct LogFormat(String);

pub const PLAIN_LOG_FORMAT: &'static str = "plain";
pub const KEY_VALUE_LOG_FORMAT: &str = "keyvalue";
pub const JSON_LOG_FORMAT: &str = "json";

impl FormatT for Option<LogFormat> {
    /// get_type implements pflag.Value
    fn get_type() -> &'static str {
        "logformat"
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
        match val.to_ascii_lowercase().as_str() {
            "plain" | "keyvalue" | "json" => {
                *self = Some(LogFormat(val.into()));
                Ok(())
            }
            _ => Err(Error::Custom(format!("unknown log format: {}", val))),
        }
    }
    // Set implements pflag.Value
}
