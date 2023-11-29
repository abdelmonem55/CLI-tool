use std::io::{Read, Write};
use utility::Result;

pub(crate) fn update_gitignore() -> Result<()> {
    let mut file;
    // update .gitignore file if it already present othewise creates it
    #[cfg(target_os = "windows")]
    {
        let mut options = std::fs::File::options();
        options.write(true).read(true);
        file = options.open(".gitignore")?;
    }
    //todo fix this
    #[cfg(target_os = "unix")]
    {
        let mut options = std::fs::File::options();
        options.write(true).read(true);
        file = options.open(".gitignore")?;
    }

    // f, err := os.OpenFile(".gitignore", os.O_RDWR|os.O_CREATE, 0644)
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let write_content = update_content(content);
    file.write_all((write_content + "\n").as_bytes())?;

    Ok(())
}

pub(crate) fn contains(strings: &Vec<&str>, e: &str) -> bool {
    for a in strings {
        if a.to_owned() == e {
            return true;
        }
    }
    false
}

pub(crate) fn update_content(content: String) -> String {
    // append files to ignore to file content if it is not already ignored

    let files_to_ignore = ["template", "build"];
    let mut lines: Vec<&str> = content.split('\n').collect();

    for file in files_to_ignore.iter() {
        if !contains(&lines, file.to_owned()) {
            lines.push(file.to_owned());
        }
    }

    let update_content: String = lines.join("\n");

    update_content.trim_matches('\n').to_string()
}
