use std::collections::HashMap;
use utility::{Error, Result};

pub struct VcsCmd {
    pub name: &'static str,
    /// name of binary to invoke command
    pub cmd: &'static str,
    /// commands to execute with the binary
    pub cmds: &'static [&'static str],
    // uri schemes the command can apply to
    pub scheme: &'static [&'static str],
}

/// Invoke executes the vcsCmd replacing variables in the cmds with the keyval
/// variables passed.

impl VcsCmd {
    pub fn invoke(&self, dir: &str, args: HashMap<String, String>) -> Result<()> {
        for cmd in self.cmds {
            self.run(dir, cmd, args.clone(), true)?;
        }
        Ok(())
    }
    fn run(
        &self,
        dir: &str,
        cmd: &str,
        key_val: HashMap<String, String>,
        verbose: bool,
    ) -> Result<Vec<u8>> {
        let args: Vec<&str> = cmd.split(" ").collect();
        let mut out_args: Vec<String> = Vec::with_capacity(args.len());
        // println!("len of out{} and args {}",out_args.si(),args.len());
        for arg in args {
            out_args.push(replace_vars(&key_val, arg));
        }
        //assert that command is exsited
        find_it(&self.cmd).ok_or(Error::Custom(format!("command {} is not found", &self.cmd)))?;

        let mut cmd = std::process::Command::new(&self.cmd);
        let envs = env_with_pwd(dir.into());
        let output = cmd.args(&out_args).current_dir(dir).envs(envs).output()?;

        if !output.status.success() {
            if verbose {
                std::io::stderr().write_all(output.stderr.as_slice())?;
            }
            return Err(Error::Custom(format!(
                "output is {},error is :{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(output.stdout)
    }
}

/// env_with_pwd creates a new ENV slice from the existing ENV, updating or adding
/// the PWD flag to the specified dir. Our commands are usually given abs paths,
/// but just this is set just incase the command is sensitive to the value.
fn env_with_pwd(dir: String) -> HashMap<String, String> {
    let mut env: HashMap<String, String> = std::env::vars().collect();
    env.insert("PWD".into(), dir);
    env
}
// func envWithPWD(dir string) []string {
// env := os.Environ()
// updated := false
// for i, envVar := range env {
// if strings.HasPrefix(envVar, "PWD") {
// env[i] = "PWD=" + dir
// updated = true
// }
// }
//
// if !updated {
// env = append(env, "PWD="+dir)
// }
//
// return env
// }

use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};

fn find_it<P>(exe_name: P) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .filter_map(|dir| {
                let full_path = dir.join(&exe_name);
                if full_path.is_file() {
                    Some(full_path)
                } else {
                    None
                }
            })
            .next()
    })
}
fn replace_vars(map: &HashMap<String, String>, s: &str) -> String {
    let mut s = s.to_owned();

    for (k, val) in map {
        s = s
            .as_str()
            .replace(format!("{{+{}+}}", k).as_str(), val.as_str());
    }
    s
}
