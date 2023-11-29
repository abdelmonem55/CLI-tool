use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
use utility::{Error, Result};

pub const USER_AGENT: &str = "OpenFaaS CLI";

lazy_static! {
    static ref GIT_COMMIT: Arc<Mutex<String>> = Arc::new(Mutex::new("".into()));
    static ref VERSION: Arc<Mutex<String>> = Arc::new(Mutex::new("".into()));
}

pub fn get_version() -> Result<String> {
    let guard = VERSION
        .lock()
        .map_err(|e| Error::Custom(format!("{:?}", e)))?;
    Ok(guard.clone())
}
pub fn set_version(version: String) -> Result<()> {
    let mut guard = VERSION
        .lock()
        .map_err(|e| Error::Custom(format!("{:?}", e)))?;
    *guard = version;
    Ok(())
}

pub fn get_git_commit() -> Result<String> {
    let guard = GIT_COMMIT
        .lock()
        .map_err(|e| Error::Custom(format!("{:?}", e)))?;
    Ok(guard.clone())
}
pub fn set_git_commit(commit: String) -> Result<()> {
    let mut guard = GIT_COMMIT
        .lock()
        .map_err(|e| Error::Custom(format!("{:?}", e)))?;
    *guard = commit;
    Ok(())
}

pub fn build_version() -> Result<String> {
    let guard = VERSION
        .lock()
        .map_err(|e| Error::Custom(format!("{:?}", e)))?;
    if guard.len() == 0 {
        Ok("dev".into())
    } else {
        Ok(guard.clone())
    }
}
