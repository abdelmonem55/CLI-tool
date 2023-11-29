mod log_format;
mod time_format;
mod timestamp;

pub use log_format::*;
pub use time_format::*;
pub use timestamp::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub trait FormatT {
    /// Type implements pflag.Value
    fn get_type() -> &'static str;
    // String implements Stringer
    fn string(&self) -> String;
    // Set implements pflag.Value
    fn set(&mut self, val: &str) -> utility::Result<()>;
}
