use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// FunctionDeployment represents a request to create or update a Function.
#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct FunctionDeployment {
    // Service is the name of the function deployment
    #[serde(default)]
    pub service: String, //`json:"service"`
    // Image is a fully-qualified container image
    #[serde(default)]
    pub image: String, //`json:"image"`

    // Namespace for the function, if supported by the faas_provider
    #[serde(default)]
    pub namespace: String, //`json:"namespace,omitempty"`

    // EnvProcess overrides the fprocess environment variable and can be used
    // with the watchdog
    #[serde(rename = "envProcess")]
    #[serde(default)]
    pub env_process: String, //`json:"envProcess,omitempty"`

    // EnvVars can be provided to set environment variables for the function runtime.
    #[serde(rename = "envVars")]
    #[serde(default)]
    pub env_vars: HashMap<String, String>, //`json:"envVars,omitempty"`

    // Constraints are specific to the faas_provider.
    #[serde(default)]
    pub constraints: Vec<String>, //`json:"constraints,omitempty"`

    // Secrets list of secrets to be made available to function
    #[serde(default)]
    pub secrets: Vec<String>, //`json:"secrets,omitempty"`

    // Labels are metadata for functions which may be used by the
    // faas_provider or the gateway
    #[serde(default)]
    pub labels: HashMap<String, String>, //`json:"labels,omitempty"`

    // Annotations are metadata for functions which may be used by the
    // faas_provider or the gateway
    #[serde(default)]
    pub annotations: HashMap<String, String>, //`json:"annotations,omitempty"`

    // Limits for function
    #[serde(default)]
    pub limits: Option<FunctionResources>, //`json:"limits,omitempty"`

    // Requests of resources requested by function
    #[serde(default)]
    pub requests: Option<FunctionResources>, //`json:"requests,omitempty"`

    // ReadOnlyRootFilesystem removes write-access from the root filesystem
    // mount-point.
    #[serde(rename = "readOnlyRootFilesystem")]
    #[serde(default)]
    pub read_only_root_filesystem: bool, //`json:"readOnlyRootFilesystem,omitempty"`
}

// /Secret for underlying orchestrator
#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct Secret {
    #[serde(default)]
    pub name: String, //`json:"name"`
    #[serde(default)]
    pub namespace: String, //`json:"namespace,omitempty"`
    #[serde(default)]
    pub value: String, //`json:"value,omitempty"`
}

// FunctionResources Memory and CPU
#[derive(Serialize, Deserialize, Debug, PartialEq, Default, Clone)]
pub struct FunctionResources {
    #[serde(default)]
    pub memory: String, //`json:"memory,omitempty"`
    #[serde(default)]
    pub cpu: String, //`json:"cpu,omitempty"`
}

// FunctionStatus exported for system/functions endpoint
#[derive(Serialize, Deserialize, Debug, PartialEq, Default, Clone)]
pub struct FunctionStatus {
    // Name is the name of the function deployment
    #[serde(default)]
    pub name: String, //`json:"name"`

    // Image is a fully-qualified container image
    #[serde(default)]
    pub image: String, //`json:"image"`

    // Namespace for the function, if supported by the faas_provider
    #[serde(default)]
    pub namespace: String, //`json:"namespace,omitempty"`

    // EnvProcess overrides the fprocess environment variable and can be used
    // with the watchdog
    #[serde(rename = "envProcess")]
    #[serde(default)]
    pub env_process: String, //`json:"envProcess,omitempty"`

    // EnvVars set environment variables for the function runtime
    #[serde(rename = "envVars")]
    #[serde(default)]
    pub env_vars: HashMap<String, String>, //`json:"envVars,omitempty"`

    // Constraints are specific to the faas_provider
    #[serde(default)]
    pub constraints: Vec<String>, //`json:"constraints,omitempty"`

    // secrets list of secrets to be made available to function
    #[serde(default)]
    pub secrets: Vec<String>, //`json:"secrets,omitempty"`

    // Labels are metadata for functions which may be used by the
    // faas_provider or the gateway
    #[serde(default)]
    pub labels: HashMap<String, String>, //`json:"labels,omitempty"`

    // Annotations are metadata for functions which may be used by the
    // faas_provider or the gateway
    #[serde(default)]
    pub annotations: HashMap<String, String>, //`json:"annotations,omitempty"`

    // Limits for function
    #[serde(default)]
    pub limits: FunctionResources, //`json:"limits,omitempty"`

    // Requests of resources requested by function
    #[serde(default)]
    pub requests: FunctionResources, //`json:"requests,omitempty"`

    // ReadOnlyRootFilesystem removes write-access from the root filesystem
    // mount-point.
    #[serde(rename = "readOnlyRootFilesystem")]
    #[serde(default)]
    pub read_only_root_filesystem: bool, //`json:"readOnlyRootFilesystem,omitempty"`

    // ================
    // Fields for status
    // ================

    // InvocationCount count of invocations
    #[serde(rename = "invocationCount")]
    #[serde(default)]
    pub invocation_count: f64, //`json:"invocationCount,omitempty"`

    // Replicas desired within the cluster
    #[serde(default)]
    pub replicas: u64, //`json:"replicas,omitempty"`

    // AvailableReplicas is the count of replicas ready to receive
    // invocations as reported by the faas_provider
    #[serde(rename = "availableReplicas")]
    #[serde(default)]
    pub available_replicas: u64, //`json:"availableReplicas,omitempty"`

    // CreatedAt is the time read back from the faas backend's
    // data store for when the function or its container was created.
    //rfc3339
    #[serde(rename = "createdAt")]
    #[serde(default)]
    pub created_at: String, //`json:"createdAt,omitempty"`
}
