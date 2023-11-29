use crate::FormatT;
use utility::Result;
/// TimeFormat is a timestamp format string that also accepts the following RFC names as shortcuts
///
///  ANSIC       = "Mon Jan _2 15:04:05 2006"
/// 	UnixDate    = "Mon Jan _2 15:04:05 MST 2006"
/// 	RubyDate    = "Mon Jan 02 15:04:05 -0700 2006"
/// 	RFC822      = "02 Jan 06 15:04 MST"
/// 	RFC822Z     = "02 Jan 06 15:04 -0700" // RFC822 with numeric zone
/// 	RFC850      = "Monday, 02-Jan-06 15:04:05 MST"
/// 	RFC1123     = "Mon, 02 Jan 2006 15:04:05 MST"
/// 	RFC1123Z    = "Mon, 02 Jan 2006 15:04:05 -0700" // RFC1123 with numeric zone
/// 	RFC3339     = "2006-01-02T15:04:05Z07:00"
/// 	RFC3339Nano = "2006-01-02T15:04:05.999999999Z07:00"

pub const ANSIC: &str = "Mon Jan _2 15:04:05 2006";
pub const UNIX_DATE: &str = "Mon Jan _2 15:04:05 MST 2006";
pub const RUBY_DATE: &str = "Mon Jan 02 15:04:05 -0700 2006";
pub const RFC822: &str = "02 Jan 06 15:04 MST";
pub const RFC822Z: &str = "02 Jan 06 15:04 -0700"; // RFC822 with numeric zone
pub const RFC850: &str = "Monday, 02-Jan-06 15:04:05 MST";
pub const RFC1123: &str = "Mon, 02 Jan 2006 15:04:05 MST";
pub const RFC1123Z: &str = "Mon, 02 Jan 2006 15:04:05 -0700"; // RFC1123 with numeric zone
pub const RFC3339: &str = "2006-01-02T15:04:05Z07:00";
pub const RFC3339_NANO: &str = "2006-01-02T15:04:05.999999999Z07:00";
pub const KITCHEN: &str = "3:04PM";
// Handy time stamps.
pub const STAMP: &str = "Jan _2 15:04:05";
pub const STAMP_MILL: &str = "Jan _2 15:04:05.000";
pub const STAMP_MICRO: &str = "Jan _2 15:04:05.000000";
pub const STAMP_NANO: &str = "Jan _2 15:04:05.000000000";

/// Any string is accepted
pub struct TimeFormat(String);

impl FormatT for Option<TimeFormat> {
    fn get_type() -> &'static str {
        "timeformat"
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
        match val {
            "ansic" => *self = Some(TimeFormat(String::from(ANSIC))),
            "unixdate" => *self = Some(TimeFormat(String::from(UNIX_DATE))),
            "rubydate" => *self = Some(TimeFormat(String::from(RUBY_DATE))),
            "rfc822" => *self = Some(TimeFormat(String::from(RFC822))),
            "rfc822z" => *self = Some(TimeFormat(String::from(RFC822Z))),
            "rfc850" => *self = Some(TimeFormat(String::from(RFC850))),
            "rfc1123" => *self = Some(TimeFormat(String::from(RFC1123))),
            "rfc1123z" => *self = Some(TimeFormat(String::from(RFC1123Z))),
            "rfc3339" => *self = Some(TimeFormat(String::from(RFC3339))),
            "rfc3339nano" => *self = Some(TimeFormat(String::from(RFC3339_NANO))),
            _ => *self = Some(TimeFormat(String::from(val))),
        }
        Ok(())
    }
}
