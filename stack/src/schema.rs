use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utility::faas::types::model::*;

/// Provider for the FaaS set of functions.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Provider {
    //#[serde(rename="name")]
    pub name: String, //`yaml:"name"`
    #[serde(rename = "gateway")]
    pub gateway_url: String, //`yaml:"gateway"`
}

///Function as deployed or built on FaaS
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Function {
    // Name of deployed function
    #[serde(skip_deserializing)]
    pub name: String, //`yaml:"-"`
    #[serde(rename = "lang")]
    #[serde(default)]
    pub language: String, //`yaml:"lang"`

    // Handler Local folder to use for function
    //#[serde(rename="handler")]
    #[serde(default)]
    pub handler: String, //`yaml:"handler"`

    // Image Docker image name
    // #[serde(rename="")]
    #[serde(default)]
    pub image: String, //`yaml:"image"`

    pub fprocess: Option<String>, //`yaml:"fprocess"`

    pub environment: Option<HashMap<String, String>>, //`yaml:"environment"`

    // Secrets list of secrets to be made available to function
    #[serde(default)]
    pub secrets: Vec<String>, //`yaml:"secrets,omitempty"`
    #[serde(default)]
    pub skip_build: bool, //`yaml:"skip_build,omitempty"`
    #[serde(default)]
    pub constraints: Vec<String>, //`yaml:"constraints,omitempty"`

    // EnvironmentFile is a list of files to import and override environmental variables.
    // These are overriden in order.
    //#[serde(rename="")]
    #[serde(default)]
    pub environment_file: Vec<String>, //`yaml:"environment_file,omitempty"`
    #[serde(default)]
    pub labels: HashMap<String, String>, //`yaml:"labels,omitempty"`

    // Limits for function
    #[serde(default)]
    pub limits: FunctionResources, //`yaml:"limits,omitempty"`

    // Requests of resources requested by function
    #[serde(default)]
    pub requests: FunctionResources, //`yaml:"requests,omitempty"`

    // ReadOnlyRootFilesystem is used to set the container filesystem to read-only
    #[serde(default)]
    pub readonly_root_filesystem: bool, //`yaml:"readonly_root_filesystem,omitempty"`

    // BuildOptions to determine native packages
    #[serde(default)]
    pub build_options: Vec<String>, //`yaml:"build_options,omitempty"`

    // Annotations
    #[serde(default)]
    pub annotations: HashMap<String, String>, //`yaml:"annotations,omitempty"`

    // Namespace of the function
    #[serde(default)]
    pub namespace: String, //`yaml:"namespace,omitempty"`

    // BuildArgs for providing build-args
    #[serde(default)]
    pub build_args: HashMap<String, String>, //`yaml:"build_args,omitempty"`

    // Platforms for use with buildx and faas-cli publish
    #[serde(default)]
    pub platforms: String, //`yaml:"platforms,omitempty"`
}

/// Configuration for the tests.yml file
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Configuration {
    #[serde(rename = "configuration")]
    pub stack_config: StackConfiguration, //`yaml:"configuration"`
}

/// StackConfiguration for the overall tests.yml
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StackConfiguration {
    #[serde(rename = "templates")]
    pub template_configs: Vec<TemplateSource>, //`yaml:"templates"`

    // CopyExtraPaths specifies additional paths (relative to the tests file) that will be copied
    // into the functions build context, e.g. specifying `"common"` will look for and copy the
    // "common/" folder of file in the same root as the tests file.  All paths must be contained
    // within the project root defined by the location of the tests file.
    //
    // The yaml uses the shorter name `copy` to make it easier for developers to read and use
    #[serde(rename = "copy")]
    pub copy_extra_paths: Vec<String>, //`yaml:"copy"`
}

// TemplateSource for build templates
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TemplateSource {
    pub name: String, //`yaml:"name"`
    #[serde(default)]
    pub source: String, //`yaml:"source,omitempty"`
}

// FunctionResources Memory and CPU
// #[derive(Serialize, Deserialize, Debug, Clone, Default)]
// pub struct FunctionResources {
//     pub memory: String, //`yaml:"memory"`
//     pub cpu: String,    //`yaml:"cpu"`
// }

// EnvironmentFile represents external file for environment data
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EnvironmentFile {
    pub environment: HashMap<String, String>, //`yaml:"environment"`
}

// Services root level YAML file to define FaaS function-set
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Services {
    #[serde(default)]
    pub version: String, //`yaml:"version,omitempty"`
    #[serde(default)]
    pub functions: HashMap<String, Function>, //`yaml:"functions,omitempty"`
    #[serde(default)]
    pub provider: Provider, //`yaml:"provider,omitempty"`
    #[serde(default)]
    #[serde(rename = "configuration")]
    pub stack_configuration: StackConfiguration, //`yaml:"configuration,omitempty"`
}

// LanguageTemplate read from template.yml within root of a language template folder
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct LanguageTemplate {
    #[serde(default)]
    pub language: String, //`yaml:"language,omitempty"`
    #[serde(default)]
    pub fprocess: Option<String>, //`yaml:"fprocess,omitempty"`
    #[serde(default)]
    pub build_options: Vec<BuildOption>, //`yaml:"build_options,omitempty"`
    /// WelcomeMessage is printed to the user after generating a function
    #[serde(default)]
    pub welcome_message: String, //`yaml:"welcome_message,omitempty"`
    /// HandlerFolder to copy the function code into
    #[serde(default)]
    pub handler_folder: String, //`yaml:"handler_folder,omitempty"`
}

// BuildOption a named build option for one or more packages
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct BuildOption {
    pub name: String,          //`yaml:"name"`
    pub packages: Vec<String>, //`yaml:"packages"`
}
