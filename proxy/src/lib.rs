use crate::client::ClientAuth;

pub mod auth;
pub mod client;
pub mod delete;
pub mod deploy;
pub mod describe;
pub mod function_store;
pub mod invoke;
pub mod list;
pub mod logs;
pub mod namespace;
pub mod proxy;
pub mod scale;
pub mod secret;
pub mod utils;
pub mod version;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

#[derive(Clone)]
pub struct TestAuth;

impl ClientAuth for TestAuth {
    fn set(&self, _req: &mut reqwest::RequestBuilder) -> utility::Result<()> {
        Ok(())
    }
}
