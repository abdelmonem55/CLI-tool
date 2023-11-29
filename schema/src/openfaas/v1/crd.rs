use crate::metadata::Metadata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utility::faas::types::model::FunctionResources;

//APIVersionLatest latest API version of CRD
pub const API_VERSION_LATEST: &str = "openfaas.com/v1";

///Spec describe characteristics of the object
#[derive(Serialize, Deserialize, Debug)]
pub struct Spec {
    ///Name name of the function
    pub name: String, //`yaml:"name"`
    ///Image docker image name of the function
    pub image: String, //`yaml:"image"`

    pub environment: HashMap<String, String>, //`yaml:"environment,omitempty"`

    pub labels: HashMap<String, String>, //`yaml:"labels,omitempty"`

    ///Limits for the function
    pub limits: FunctionResources, //`yaml:"limits,omitempty"`

    ///Requests of resources requested by function
    pub requests: FunctionResources, //`yaml:"requests,omitempty"`

    pub constraints: Vec<String>, //`yaml:"constraints,omitempty"`

    //Secrets list of secrets to be made available to function
    pub secrets: Vec<String>, //`yaml:"secrets,omitempty"`
}

///CRD root level YAML definition for the object
#[derive(Deserialize, Serialize, Debug)]
pub struct CRD {
    ///APIVersion CRD API version
    #[serde(rename = "apiVersion")]
    pub api_version: String, //`yaml:"apiVersion"`

    //Kind kind of the object
    pub kind: String,       //`yaml:"kind"`
    pub metadata: Metadata, //`yaml:"metadata"`
    pub spec: Spec,         //`yaml:"spec"`
}
