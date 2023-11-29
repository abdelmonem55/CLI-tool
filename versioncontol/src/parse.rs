use lazy_static::lazy_static;
use regex::Regex;
use std::sync::Arc;

pub const PIN_CHARACTER: &str = "#";
const GIT_PINNED_REMOTE_STR: &str =
    r"(git|ssh|https?|git@[-\w.]+):(//)?([^#]*?(?:.git)?/?)#[-/\d\w._]+$";
const GIT_REMOTE_STR: &str = r"(git|ssh|https?|git@[-\w.]+):(//)?([^#]*?(?:.git)?/?)$";

lazy_static! {
    static ref GIT_PINNED_REMOTE_REGEX_PSTR: Arc<Regex> =
        Arc::new(Regex::new(GIT_PINNED_REMOTE_STR).unwrap());
    static ref GIT_REMOTE_REPO_REGEX_PSTR: Arc<Regex> =
        Arc::new(Regex::new(GIT_REMOTE_STR).unwrap());
}

/// is_git_remote validates if the supplied string is a valid git remote url value
pub fn is_git_remote(repo_url: &str) -> bool {
    GIT_REMOTE_REPO_REGEX_PSTR.is_match(repo_url)
}

/// is_pinned_git_remote validates if the supplied string is a valid git remote url value
pub fn is_pinned_git_remote(repo_url: &str) -> bool {
    // If using a Regexp in multiple goroutines,
    // giving each goroutine its own copy helps to avoid lock contention.
    GIT_PINNED_REMOTE_REGEX_PSTR.is_match(repo_url)
}

/// ParsePinnedRemote returns the remote url and contraint value from repository url(ref_name)
pub fn parse_panned_remote(repo_url: &str) -> (String, String) {
    //func ParsePinnedRemote(repoURL string) (remoteUrl, refName string) {
    // default refName is empty
    // the template fetcher can detect this and will pull the default when
    // the ref is empty
    let mut ref_name = String::new();
    let mut remote_url = repo_url.to_string();

    if !is_pinned_git_remote(repo_url) {
        return (remote_url, ref_name);
    }
    // handle ssh special case
    if let Some(idx) = repo_url.rfind(PIN_CHARACTER) {
        let (left, right) = repo_url.split_at(idx);
        ref_name = right.replace(PIN_CHARACTER, "");
        remote_url = left.into();
    }

    return (remote_url, ref_name);
}
