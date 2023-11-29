use flags::{JSON_LOG_FORMAT, KEY_VALUE_LOG_FORMAT};
use utility::faas_provider::logs::Message;
use utility::{Error, Result};

/// LogFormatter is a function that converts a log message to a string based on the supplied options
//type LogFormatter func(msg logs.Message, timeFormat string, includeName, includeInstance bool) string

pub type LogFormatter = fn(&Message, &str, bool, bool) -> utility::Result<String>;

/// GetLogFormatter maps a formatter name to a LogFormatter method
pub fn get_log_formatter(name: &str) -> LogFormatter {
    match name {
        JSON_LOG_FORMAT => json_format_message,
        KEY_VALUE_LOG_FORMAT => key_value_format_message,
        _ => plain_format_message,
    }
}

/// JSONFormatMessage is a JSON formatting for log messages, the options are ignored and the entire log
/// message json serialized
pub fn json_format_message(
    msg: &Message,
    _time_format: &str,
    _include_name: bool,
    _include_instance: bool,
) -> utility::Result<String> {
    // error really can't happen here because of how simple the msg object is
    let res = serde_json::to_string(msg).map_err(|e| Error::Custom(e.to_string()))?;
    Ok(res)
}

/// returns the message in the format "timestamp=<> name=<> instance=<> message=<>"
pub fn key_value_format_message(
    msg: &Message,
    time_format: &str,
    include_name: bool,
    include_instance: bool,
) -> Result<String> {
    let mut b = String::new();

    if !time_format.is_empty() {
        b.push_str("timestamp=\"");
        let time = chrono::DateTime::parse_from_str(msg.timestamp.as_str(), time_format)
            .map_err(|e| Error::Custom(e.to_string()))?;
        b.push_str(time.to_string().as_str());
        b.push_str("\" ");
    }

    if include_name {
        b.push_str("name=\"");
        b.push_str(msg.name.as_str());
        b.push_str("\" ");
    }

    if include_instance {
        b.push_str("instance=\"");
        b.push_str(msg.instance.as_str());
        b.push_str("\" ");
    }

    b.push_str("text=\"");
    let txt = msg.text.trim_end_matches('\n');
    b.push_str(txt);
    b.push_str("\" ");

    Ok(b)
}

/// formats a log message as "<timestamp> <name> (<instance>) <text>"
pub fn plain_format_message(
    msg: &Message,
    time_format: &str,
    include_name: bool,
    include_instance: bool,
) -> Result<String> {
    let mut b = String::new();

    // note that push_str's error is always nil and safe to ignore here
    if !time_format.is_empty() {
        let time = chrono::DateTime::parse_from_str(msg.timestamp.as_str(), time_format)
            .map_err(|e| Error::Custom(e.to_string()))?;
        b.push_str(time.to_string().as_str());

        b.push(' ');
    }

    if include_name {
        b.push_str(msg.name.as_str());
        b.push(' ');
    }

    if include_instance {
        b.push('(');
        b.push_str(msg.instance.as_str());
        b.push(')');
        b.push(' ');
    }

    let txt = msg.text.trim_end_matches('\n');
    b.push_str(txt);

    Ok(b)
}
