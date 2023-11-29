use crate::metadata::Metadata;
use serde::{Deserialize, Serialize};

//#[derive(Serialize, Deserialize, Debug)]

pub const API_VERSION_LATEST: &str = "serving.knative.dev/v1";

///ServingServiceCRD root level YAML definition for the object
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ServingServiceCRD {
    /// APIVersion CRD API version
    #[serde(rename = "apiVersion")]
    pub api_version: String, //`yaml:"apiVersion"`
    //Kind kind of the object
    pub kind: String,             //`yaml:"kind"`
    pub metadata: Metadata,       //`yaml:"metadata,omitempty"`
    pub spec: ServingServiceSpec, //`yaml:"spec"`
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ServingServiceSpec {
    #[serde(rename = "template")]
    pub serving_service_spec_template: ServingServiceSpecTemplate, //`yaml:"template"`
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ServingServiceSpecTemplateSpec {
    #[serde(default)]
    #[serde(skip_serializing_if = "utility::is_default")]
    pub containers: Vec<ServingSpecContainersContainerSpec>, //`yaml:"containers"`
    #[serde(default)]
    #[serde(skip_serializing_if = "utility::is_default")]
    pub volumes: Vec<Volume>, //yaml:"volumes,omitempty"`
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ServingServiceSpecTemplate {
    pub template: ServingServiceSpecTemplateSpec, //`yaml:"spec"`
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct ServingSpecContainersContainerSpec {
    pub image: String,     //`yaml:"image"`
    pub env: Vec<EnvPair>, //`yaml:"env,omitempty"`
    #[serde(rename = "volumMounts")]
    pub volume_mounts: Vec<VolumeMount>, //`yaml:"volumeMounts,omitempty"`
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct VolumeMount {
    pub name: String, //`yaml:"name"`
    #[serde(rename = "mountPath")]
    pub mount_path: String, //yaml:"mountPath"`
    #[serde(rename = "readOnly")]
    pub read_only: bool, //yaml:"readOnly"`
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Volume {
    #[serde(default)]
    #[serde(skip_serializing_if = "utility::is_default")]
    pub name: String, //`yaml:"name"`
    #[serde(default)]
    #[serde(skip_serializing_if = "utility::is_default")]
    pub secret: Secret, //`yaml:"secret"`
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct Secret {
    #[serde(rename = "secretName")]
    pub secret_name: String, //`yaml:"secretName"`
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct EnvPair {
    pub name: String,  //`yaml:"name"`
    pub value: String, //`yaml:"value"`
}
