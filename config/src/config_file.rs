use path_clean::PathClean;
use serde::{Deserialize, Serialize};
use std::env::VarError;
use std::io::Write;
use std::path::PathBuf;
use utility::{Error, Result};

//AuthType auth type
pub type AuthType = String;

///BasicAuthType basic authentication type
pub const BASIC_AUTH_TYPE: &str = "basic";
///OAUTH_2AUTH_TYPE oauth2 authentication type
pub const OAUTH_2AUTH_TYPE: &str = "oauth2";

/// CONFIG_LOCATION_ENV is the name of he env variable used
/// to configure the location of the faas-cli config folder.
/// When not set, DEFAULT_DIR location is used.
pub const CONFIG_LOCATION_ENV: &str = "OPENFAAS_CONFIG";

pub const DEFAULT_DIR: &str = "~/.openfaas";
pub const DEFAULT_FILE: &str = "config.yml";
pub const DEFAULT_PERMISSION: usize = 0700;

/// DEFAULT_CI_DIR creates the 'openfaas' directory in the current directory
/// if running in a CI environment.
pub const DEFAULT_CI_DIR: &str = "./openfaas";
/// DEFAULT_CI_PERMISSION creates the config file with elevated permissions
/// for it to be read by multiple users when running in a CI environment.
pub const DEFAULT_CI_PERMISSION: usize = 0744;

/// ConfigFile for OpenFaaS CLI exclusively.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct ConfigFile<'s> {
    #[serde(default)]
    #[serde(rename = "auths")]
    auth_configs: Vec<AuthConfig>, //`yaml:"auths"`
    #[serde(skip_deserializing)]
    #[serde(skip_serializing)]
    file_path: &'s str, //`yaml:"-"`
}
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AuthConfig {
    #[serde(default)]
    pub gateway: String, //`yaml:"gateway,omitempty"`
    #[serde(default)]
    pub auth: AuthType, //`yaml:"auth,omitempty"`
    #[serde(default)]
    pub token: String, //`yaml:"token,omitempty"`
}

impl<'s> ConfigFile<'s> {
    /// new initializes a config file for the given file path
    pub fn new(file_path: &'s str) -> Result<ConfigFile<'s>> {
        let res = if file_path.is_empty() {
            Err(Error::Custom(format!(
                "can't create config with empty filePath"
            )))
        } else {
            Ok(ConfigFile {
                auth_configs: vec![],
                file_path,
            })
        };
        res
    }
}

/// ConfigDir returns the path to the faas-cli config directory.
/// When
/// 1. CI = "true" and OPENFAAS_CONFIG="", then it will return `./openfaas`, which is located in the current working directory.
/// 2. CI = "true" and OPENFAAS_CONFIG="<path>", then it will return the path value in  OPENFAAS_CONFIG
/// 3. CI = "" and OPENFAAS_CONFIG="", then it will return the default location ~/.openfaas
pub fn config_dir() -> Result<String> {
    match std::env::var(CONFIG_LOCATION_ENV) {
        Ok(overrid) => {
            if overrid.is_empty() && is_running_in_cli() {
                Ok(DEFAULT_CI_DIR.into())
            } else if !overrid.is_empty() {
                Ok(overrid)
            } else {
                Ok(DEFAULT_DIR.into())
            }
        }
        Err(e) => {
            if is_running_in_cli() && e == VarError::NotPresent {
                Ok(DEFAULT_CI_DIR.into())
            } else if e == VarError::NotPresent {
                Ok(DEFAULT_DIR.into())
            } else {
                Err(Error::Custom(format!("{:?}", e)))
            }
        }
    }
}

/// is_running_in_cli checks the ENV var CI and returns true if it's set to true or 1
fn is_running_in_cli() -> bool {
    match std::env::var("CI") {
        Ok(val) => {
            if val == "1" || val == "true" {
                true
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

/// creates the root dir and config file
pub fn ensure_file() -> Result<String> {
    //permission := DefaultPermissions
    let dir = config_dir()?;

    let file_path = shellexpand::tilde(&dir);
    let file_path = PathBuf::from(file_path.into_owned())
        .join(DEFAULT_FILE)
        .clean();
    let dir = file_path
        .parent()
        .map_or_else(|| file_path.clone(), |v| v.to_owned());

    #[cfg(target_os = "unix")]
    {
        use std::fs::DirBuilder;
        use std::os::unix::fs::DirBuilderExt;
        std::os::unix::fs::PermissionsExt;

        let permission = if is_running_in_cli() {
            DEFAULT_CI_PERMISSION
        } else {
            DEFAULT_PERMISSION
        };
        let mut builder = DirBuilder::new();
        builder
            .mode(permission)
            .recursive(true)
            .create(dir)
            .map_error(|e| Error::Custom(e.to_string()))?;
        if let Err(err) = std::fs::metadata(&file_path) {
            if format!("{:?}", err).contains("kind: NotFound") {
                let mut file = std::fs::File::create(&file_path)?;
                let permissions = Permissions::from_mode(0600);
                file.set_permissions(permissions);
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        use std::fs::DirBuilder;
        let mut builder = DirBuilder::new();
        builder.recursive(true).create(dir)?;
        if let Err(err) = std::fs::metadata(&file_path) {
            if format!("{:?}", err).contains("kind: NotFound") {
                std::fs::File::create(&file_path)?;
            } else {
                return Err(Error::Io(err));
            }
        }
    }

    return Ok(file_path.to_string_lossy().to_string());
}

/// returns true if the config file is located at the default path
fn file_exists() -> Result<bool> {
    let dir = config_dir()?;
    let file_path = shellexpand::tilde(&dir);
    let file_path = PathBuf::from(file_path.into_owned())
        .join(DEFAULT_FILE)
        .clean();

    if let Err(err) = std::fs::metadata(&file_path) {
        if format!("{:?}", err).contains("kind: NotFound") {
            Ok(false)
        } else {
            Err(Error::Io(err))
        }
    } else {
        Ok(true)
    }
}

impl<'s> ConfigFile<'s> {
    // Save writes the config to disk
    fn save(&self) -> Result<()> {
        let mut file;
        #[cfg(target_os = "unix")]
        {
            std::os::unix::fs::PermissionsExt;

            file = std::fs::File::create(&file_path)?;
            let permissions = Permissions::from_mode(0600);
            file.set_permissions(permissions);
        }
        #[cfg(target_os = "windows")]
        {
            file = std::fs::File::create(&self.file_path)?;
        }

        let data = serde_yaml::to_string(self).map_err(|e| Error::Custom(format!("{:?}", e)))?;

        file.write_all(data.as_bytes())?;
        Ok(())
    }

    /// load reads the yaml file from disk
    fn load(&mut self) -> Result<()> {
        //  let conf = ConfigFile::default();

        if let Err(err) = std::fs::metadata(&self.file_path) {
            if format!("{:?}", err).contains("kind: NotFound") {
                return Err(Error::Custom(format!(
                    "can't load config from non existent filePath"
                )));
            } else {
                return Err(Error::Io(err));
            }
        }
        let data = std::fs::read_to_string(&self.file_path)?;
        println!("data: {}", data);
        let conf: ConfigFile = if data.is_empty() {
            ConfigFile::default()
        } else {
            serde_yaml::from_str(data.as_str()).map_err(|e| Error::Custom(format!("{:?}", e)))?
        };

        if !conf.auth_configs.is_empty() {
            self.auth_configs = conf.auth_configs;
        }
        Ok(())
    }
}

/// encodes the username and password strings to base64
pub fn encode_auth(username: &str, password: &str) -> String {
    let input = username.to_string() + ":" + password;

    //encoded := make([]byte, base64.StdEncoding.EncodedLen(len(msg)))
    // base64.StdEncoding.Encode(encoded, msg)
    let encoded = base64::encode(input);
    encoded
}

// decodes base64 to the username and password
pub fn decode_auth(encoded: &str) -> Result<(String, String)> {
    // base64.StdEncoding.Encode(encoded, msg)
    let decoded = base64::decode(encoded).map_err(|e| Error::Custom(format!("{:?}", e)))?;
    let decoded = std::str::from_utf8(&decoded).map_err(|e| Error::Custom(format!("{:?}", e)))?;
    let data: Vec<&str> = decoded.split(":").collect();

    let username = data[0].to_owned();
    let password = data
        .get(1)
        .ok_or(Error::Custom(format!(
            "the data decoded to format not like username:password"
        )))?
        .to_string();
    Ok((username, password))
}
/// creates or updates the username and password for a given gateway
pub fn update_auth_config(gateway: &str, token: &str, auth_type: AuthType) -> Result<()> {
    if gateway.is_empty() || url::Url::parse(gateway).is_err() {
        return Err(Error::Custom(format!("invalid gateway")));
    }

    let config_path = ensure_file()?;

    let mut cfg = ConfigFile::new(config_path.as_str())?;

    cfg.load()?;

    let auth = AuthConfig {
        gateway: gateway.to_string(),
        auth: auth_type,
        token: token.to_string(),
    };

    let mut index = -1;
    for (i, v) in cfg.auth_configs.iter().enumerate() {
        if gateway == v.gateway {
            index = i as i32;
            break;
        }
    }

    if index == -1 {
        cfg.auth_configs.push(auth);
    } else {
        cfg.auth_configs[index as usize] = auth;
    }

    cfg.save()
}

///returns the username and password for a given gateway
pub fn lookup_auth_config(gateway: &str) -> Result<AuthConfig> {
    if !file_exists()? {
        return Err(Error::Custom("config file is not found".to_string()));
    }

    let config_path = ensure_file()?;

    let mut cfg = ConfigFile::new(config_path.as_str())?;
    cfg.load()?;
    //println!("{:#?}", cfg);

    for v in cfg.auth_configs {
        if gateway == v.gateway {
            return Ok(v);
        }
    }
    Ok(AuthConfig::default())

    // return Err(Error::IoCustom(format!(
    //     "no auth config found for {}",
    //     gateway
    // )));
}

///deletes the username and password for a given gateway
pub fn remove_auth_config(gateway: &str) -> Result<()> {
    if !file_exists()? {
        return Err(Error::Custom("config file is not found".to_string()));
    }

    let config_path = ensure_file()?;

    let mut cfg = ConfigFile::new(config_path.as_str())?;
    cfg.load()?;

    let mut index = -1;
    for (i, v) in cfg.auth_configs.iter().enumerate() {
        if gateway == v.gateway {
            index = i as i32;
            break;
        }
    }

    if index > -1 {
        //cfg.AuthConfigs = remove_auth_by_index(cfg.auth_configs, index);
        cfg.auth_configs.remove(index as usize);
        cfg.save()
    } else {
        Err(Error::Custom(format!(
            "gateway {} not found in config",
            gateway
        )))
    }
}
