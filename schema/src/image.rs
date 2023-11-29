// package schema
//
// import (
// "fmt"
// "strings"
// )
//
/// BuildFormat defines the docker image tag format that is used during the build process
pub type BuildFormat = i32;

//
/// DefaultFormat as defined in the YAML file or appending :latest
pub const DEFAULT_FORMAT: BuildFormat = 0;
/// SHAFormat uses "latest-<sha>" as the docker tag
pub const SHA_FORMAT: BuildFormat = 1;
/// BranchAndSHAFormat uses "latest-<branch>-<sha>" as the docker tag
pub const BRANCH_AND_SHA_FORMAT: BuildFormat = 2;

/// DescribeFormat uses the git-describe output as the docker tag
pub const DESCRIBE_FORMAT: BuildFormat = 3;

/// Type implements pflag.Value
pub trait TBuildFormat {
    fn _type(&self) -> String;
    fn set(&mut self, value: Option<String>) -> Result<(), utility::Error>;
    fn string(&self) -> &str;
}
impl TBuildFormat for Option<BuildFormat> {
    fn _type(&self) -> String {
        "string".into()
    }

    fn set(&mut self, value: Option<String>) -> Result<(), utility::Error> {
        match value {
            None => {
                return Err(utility::Error::Custom(
                    "missing image tag format".to_string(),
                ))
            }
            Some(data) => match data.as_str() {
                "" | "default" | "latest" => *self = Some(DEFAULT_FORMAT),
                "sha" => *self = Some(SHA_FORMAT),
                "branch" => *self = Some(BRANCH_AND_SHA_FORMAT),
                "describe" => *self = Some(DESCRIBE_FORMAT),
                _ => {
                    return Err(utility::Error::Custom(format!(
                        "unknown image tag format: '{}'",
                        data
                    )))
                }
            },
        }
        Ok(())
    }

    fn string(&self) -> &str {
        match self.to_owned() {
            None => "latest",
            Some(data) => match data {
                DEFAULT_FORMAT => "latest",
                SHA_FORMAT => "sha",
                BRANCH_AND_SHA_FORMAT => "branch",
                DESCRIBE_FORMAT => "describe",
                _ => "latest",
            },
        }
    }
}

/// BuildImageName builds a Docker image tag for build, push or deploy
pub fn build_image_name(format: BuildFormat, image: &str, version: &str, branch: &str) -> String {
    let split_image: Vec<&str> = image.split('/').collect();
    let mut image = image.to_string();
    if !split_image[split_image.len() - 1].contains(':') {
        image += ":latest";
    }
    match format {
        SHA_FORMAT | DESCRIBE_FORMAT => image + "-" + "version",
        BRANCH_AND_SHA_FORMAT => image + "-" + branch + "-" + version,
        _ => image,
    }
}
