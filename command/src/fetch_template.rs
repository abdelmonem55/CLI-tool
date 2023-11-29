use builder::copy_files;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::sync::{Arc, Mutex};
use tempdir::TempDir;
use utility::{Error, Result};
use versioncontol::git::{GIT_CHECKOUT_REF_NAME, GIT_CLONE, GIT_CLONE_DEFAULT};
use versioncontol::parse::{is_git_remote, is_pinned_git_remote, parse_panned_remote};

/// DefaultTemplateRepository contains the Git repo for the official templates
pub(crate) const DEFAULT_TEMPLATE_REPOSITORY: &str = "https://github.com/openfaas/templates.git";
pub(crate) const TEMPLATE_DIRECTORY: &str = "./template/";

lazy_static! {
    ///to make the life time of temp folder longer than function scope
    static ref TEMP_FOLDERS:Arc<Mutex<Vec<TempDir>>> = Arc::new(Mutex::new(Vec::new()));
}

/// fetchTemplates fetch code templates using git clone.
pub(crate) fn fetch_templates(
    template_url: &str,
    ref_name: &str,
    overwrite: bool,
    pull_debug: bool,
) -> Result<()> {
    if template_url.is_empty() {
        return Err(Error::Custom("pass valid template_url".to_string()));
    }
    let dir = tempdir::TempDir::new("openFaasTemplates")?;
    let dir_path = dir.path().to_string_lossy().to_string();

    colour::green!("Attempting to expand templates from {}\n", template_url);
    if pull_debug {
        //don't clean up
        let mut lock = TEMP_FOLDERS
            .lock()
            .map_err(|e| Error::Custom(format!("failed to access mutux:{:?}", e)))?;
        lock.push(dir);
        println!("Temp files in {:?}", dir_path);
    }

    let mut args: HashMap<String, String> = HashMap::new();

    args.insert("dir".to_string(), dir_path.clone());
    args.insert("repo".to_string(), template_url.to_string());

    let cmd = if !ref_name.is_empty() {
        args.insert("ref_name".to_string(), ref_name.to_string());
        GIT_CLONE
    } else {
        GIT_CLONE_DEFAULT
    };

    cmd.invoke(".", args)?;

    let (pre_existing_languages, fetched_languages) = move_templates(dir_path.as_str(), overwrite)?;

    if !pre_existing_languages.is_empty() {
        colour::yellow!(
            "Cannot overwrite the following {} template(s): {:?}\n",
            pre_existing_languages.len(),
            pre_existing_languages
        );
    }

    colour::green!(
        "Fetched {} template(s) : {:?} from {}\n",
        fetched_languages.len(),
        fetched_languages,
        template_url
    );

    Ok(())
}

fn move_templates(repo_path: &str, overwrite: bool) -> Result<(Vec<String>, Vec<String>)> {
    let mut existing_languages: Vec<String> = Vec::new();
    let mut fetched_languages: Vec<String> = Vec::new();
    let mut available_languages: HashMap<String, bool> = HashMap::new();

    let temp_dir = format!(
        "{}/{}",
        repo_path.trim_end_matches('/').trim_end_matches("\\"),
        TEMPLATE_DIRECTORY.trim_start_matches("./")
    );

    let templates = std::fs::read_dir(&temp_dir)
        .map_err(|_e| Error::Custom(format!("can't find templates in: {}", repo_path)))?;

    for file in templates {
        let file = file?.path();
        if file.is_file() {
            let language = file.file_name().unwrap().to_string_lossy().to_string();
            let can_write =
                can_write_language(&mut available_languages, language.as_str(), overwrite)?;

            if can_write {
                fetched_languages.push(language.clone());
                // Do cp here
                let language_src = format!("{}/{}", temp_dir.trim_end_matches('/'), language);
                let language_dest = format!(
                    "{}/{}",
                    TEMPLATE_DIRECTORY
                        .trim_start_matches("./")
                        .trim_end_matches('/'),
                    language
                );
                copy_files(language_src.as_str(), language_dest.as_str())?;
            } else {
                existing_languages.push(language);
                // continue
            }
        }
    }

    Ok((existing_languages, fetched_languages))
}

/// tells whether the language can be expanded from the zip or not.
/// availableLanguages map keeps track of which languages we know to be okay to copy.
/// overwrite flag will allow to force copy the language template
fn can_write_language(
    available_languages: &mut HashMap<String, bool>,
    language: &str,
    overwrite: bool,
) -> Result<bool> {
    let mut can_write = false;

    if !available_languages.is_empty() && !language.is_empty() {
        if let Some(existed) = available_languages.get(language) {
            return Ok(*existed);
        }

        can_write = template_folder_exists(language, overwrite)?;
        available_languages.insert(language.to_string(), can_write);
    }

    Ok(can_write)
}
/// Takes a language input (e.g. "node"), tells whether or not it is OK to download
fn template_folder_exists(language: &str, overwrite: bool) -> Result<bool> {
    let dir = TEMPLATE_DIRECTORY.to_string() + language;
    match std::fs::metadata(dir) {
        Ok(_) => {
            if !overwrite {
                Ok(false)
            } else {
                Ok(true)
            }
        }
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                Ok(true)
            } else {
                Err(Error::Io(e))
            }
        }
    }
}

pub(crate) fn pull_template(repository: &str, overwrite: bool, pull_debug: bool) -> Result<()> {
    if std::fs::metadata(repository).is_err() {
        if !is_git_remote(repository) && !is_pinned_git_remote(repository) {
            return Err(Error::Custom(
                "The repository URL must be a valid git repo uri".to_string(),
            ));
        }
    }

    let (repository, ref_name) = parse_panned_remote(repository);

    if !ref_name.is_empty() {
        let map: HashMap<String, String> = [("refname".to_string(), ref_name.clone())]
            .iter()
            .cloned()
            .collect();
        GIT_CHECKOUT_REF_NAME.invoke("",map)
            .map_err(|e|{
                colour::red!("Invalid tag or branch name `{}`\n", ref_name);
                colour::red!("See https://git-scm.com/docs/git-check-ref-format for more details of the rules Git enforces
                 on branch and reference names.");

                e
            })?;
    }
    colour::green!(
        "Fetch templates from repository: {} at {}\n",
        repository,
        ref_name
    );

    fetch_templates(
        repository.as_str(),
        ref_name.as_str(),
        overwrite,
        pull_debug,
    )
    .map_err(|e| Error::Custom(format!("error while fetching templates : {}", e)))
}
