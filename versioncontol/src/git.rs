use crate::core::VcsCmd;
use exec::command_with_output;
use utility::{Error, Result};
/// GitClone defines the command to clone a repo into a directory
pub const GIT_CLONE: VcsCmd = VcsCmd {
    name: "Git",
    cmd: "git",
    cmds: &["clone {repo} {dir} --depth=1 --config core.autocrlf=false -b {refname}"],
    scheme: &["git", "https", "http", "git+ssh", "ssh"],
};

/// GitClone defines the command to clone the default branch of a repo into a directory
pub const GIT_CLONE_DEFAULT: VcsCmd = VcsCmd {
    name: "Git",
    cmd: "git",
    cmds: &["clone {repo} {dir} --depth=1 --config core.autocrlf=false"],
    scheme: &["git", "https", "http", "git+ssh", "ssh"],
};

/// GitCheckout defines the command to clone a specific REF of repo into a directory
pub const GIT_CHECKOUT: VcsCmd = VcsCmd {
    name: "Git",
    cmd: "git",
    cmds: &["-C {dir} checkout {refname}"],
    scheme: &["git", "https", "http", "git+ssh", "ssh"],
};

/// GitCheckRefName defines the command that validates if a string is a valid reference name or sha
pub const GIT_CHECKOUT_REF_NAME: VcsCmd = VcsCmd {
    name: "Git",
    cmd: "git",
    cmds: &["check-ref-format --allow-onelevel {refname}"],
    scheme: &["git", "https", "http", "git+ssh", "ssh"],
};

// GitInitRepo initializes the working directory add commit all files & directories
pub const GIT_INIT_REPO: VcsCmd = VcsCmd {
    name: "Git",
    cmd: "git",
    cmds: &[
        "init {dir}",
        "config core.autocrlf false",
        "config user.email \"contact@openfaas.com\"",
        "config user.name \"OpenFaaS\"",
        "add {dir}",
        "commit -m \"Test-commit\"",
    ],
    scheme: &["git", "https", "http", "git+ssh", "ssh"],
};

/// get_git_describe returns the human readable name for the current commit using `git-describe`
pub fn get_git_describe() -> Result<String> {
    // git-describe - Give an object a human readable name based on an available ref
    // --tags                use any tag, even unannotated
    // --always              show abbreviated commit object as fallback

    // using --tags, means that the output should look like v1.2.2-1-g3443110 where the last
    // <most-recent-parent-tag>-<number-of-commits-to-that-tag>-g<short-sha>
    // using --always, means that if the repo does not use tags, then we will still get the <short-sha>
    // as output, similar to GetGitSHA
    let get_describe_command = vec!["git", "describe", "--tags", "--always"];
    let sha: String = command_with_output(get_describe_command, true)?;
    if sha.contains("Not a git repository") {
        Err(Error::Custom("Not a git repository".to_string()))
    } else {
        let sha = sha.trim_end_matches("\n");
        Ok(sha.into())
    }
}

// get_git_sha returns the short Git commit SHA from local repo
pub fn get_git_sha() -> Result<String> {
    let get_sha_command = vec!["git", "rev-parse", "--short", "HEAD"];
    let sha: String = command_with_output(get_sha_command, true)?;
    if sha.contains("Not a git repository") {
        Err(Error::Custom("Not a git repository".to_string()))
    } else {
        let sha = sha.trim_end_matches("\n");
        Ok(sha.into())
    }
}

pub fn get_git_branch() -> Result<String> {
    let get_branch_command = vec!["git", "rev-parse", "--short", "HEAD"];
    let branch: String = command_with_output(get_branch_command, true)?;
    if branch.contains("Not a git repository") {
        Err(Error::Custom("Not a git repository".to_string()))
    } else {
        let branch = branch.trim_end_matches("\n");
        Ok(branch.into())
    }
}
