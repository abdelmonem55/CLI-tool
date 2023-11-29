use crate::client::{Client, ClientAuth};
use config::config_file::{decode_auth, BASIC_AUTH_TYPE};
use reqwest::RequestBuilder;
use utility::{Error, Result};

///auth struct for the CLI
pub struct CLIAuth<'s> {
    pub username: &'s str,
    pub password: &'s str,
    pub token: &'s str,
}

///basic authentication type
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

///bearer token
pub struct BearerToken {
    token: String,
}

impl<'s> ClientAuth for BasicAuth {
    fn set(&self, req: &mut RequestBuilder) -> Result<()> {
        let req2 = req
            .try_clone()
            .ok_or(Error::Custom(format!("can't clone request :{:?}", req)))?;
        *req = req2.basic_auth(self.username.as_str(), Some(self.password.as_str()));
        Ok(())
    }
    // fn set(&self, req: &Request) -> Result<()> {
    //     req.SetBasicAuth(auth.username, auth.password)
    //     return nil
    // }
}
impl<'s> ClientAuth for BearerToken {
    fn set(&self, req: &mut RequestBuilder) -> Result<()> {
        let req2 = req
            .try_clone()
            .ok_or(Error::Custom(format!("can't clone request :{:?}", req)))?;
        *req = req2.header("Authorization", "Bearer ".to_string() + self.token.as_str());
        Ok(())
    }
}

pub enum ClientAuthE {
    BasicAuth(BasicAuth),
    BearerToken(BearerToken),
}
impl ClientAuthE {
    ///returns a new CLI Auth
    pub fn new(token: &str, gateway: &str) -> Result<ClientAuthE> {
        let config = config::config_file::lookup_auth_config(gateway)?;
        if config.auth == BASIC_AUTH_TYPE {
            let (username, password) = decode_auth(config.token.as_str())?;

            Ok(ClientAuthE::BasicAuth(BasicAuth { username, password }))
        } else {
            // User specified token gets priority
            let bearer_token = if !token.is_empty() {
                token.to_string()
            } else {
                config.token
            };
            Ok(ClientAuthE::BearerToken(BearerToken {
                token: bearer_token,
            }))
        }
    }

    pub fn set(&self, req: &mut RequestBuilder) -> Result<()> {
        match self {
            ClientAuthE::BasicAuth(b) => b.set(req),
            ClientAuthE::BearerToken(t) => t.set(req),
        }
    }

    pub fn get_client(
        &self,
        gateway: &str, /*,basic:&mut BasicAuth,bearer:&mut BearerToken)*/
    ) -> Result<Client> {
        match self {
            ClientAuthE::BasicAuth(basic) => {
                // basic = b;
                Client::new(Box::new(basic), gateway)
            }
            ClientAuthE::BearerToken(bearer) => {
                // bearer = b;
                Client::new(Box::new(bearer), gateway)
            }
        }
    }
}
