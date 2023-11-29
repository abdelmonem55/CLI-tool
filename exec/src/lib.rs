#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

use utility::{Error, Result};
/// Command run a system command and return output
pub fn command_exec(
    temp_path: &str,
    cmd: &str,
    args: &Vec<&str>,
) -> Result<std::process::ExitStatus> {
    // let args: Vec<&&str> = builder.iter().skip(1).collect();
    let out = std::process::Command::new(cmd)
        .args(args)
        .current_dir(temp_path)
        .output()
        //.status()
        .map_err(|e| {
            Error::Custom(format!(
                "ERROR - Could not execute command: {} {:?}\n{:?}",
                cmd, args, e
            ))
        })?;
    if !out.status.success() {
        println!("out ####  {:?}", out);
    }
    Ok(out.status)
}

/// Command run a system command
pub fn command(temp_path: &str, builder: Vec<&str>) -> Result<()> {
    let args: Vec<&&str> = builder.iter().skip(1).collect();
    let _ = std::process::Command::new(builder[0])
        .args(args)
        .current_dir(temp_path)
        .output()
        .map_err(|e| {
            Error::Custom(format!(
                "ERROR - Could not execute command: {:?}\n{:?}",
                builder, e
            ))
        })?;
    Ok(())
}

pub fn command_with_output(builder: Vec<&str>, skip_failure: bool) -> Result<String> {
    let args: Vec<&&str> = builder.iter().skip(1).collect();
    let output = std::process::Command::new(builder[0])
        .args(args)
        .output()
        .map_err(|e| {
            let err = Error::Custom(format!(
                "ERROR - Could not execute command: {:?}\n{:?}",
                builder, e
            ));

            if !skip_failure {
                //log the error
            }
            err
        })?;

    Ok(String::from_utf8_lossy(output.stdout.as_slice()).to_string())
}
