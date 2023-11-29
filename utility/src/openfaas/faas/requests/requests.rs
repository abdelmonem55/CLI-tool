use serde::{Deserialize, Serialize};

/// AsyncReport is the report from a function executed on a queue worker.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AsyncReport {
    #[serde(rename = "name")]
    #[serde(default)]
    pub function_name: String, //`json:"name"`
    #[serde(rename = "statusCode")]
    #[serde(default)]
    pub status_code: i32, //`json:"statusCode"`
    #[serde(rename = "timeTaken")]
    #[serde(default)]
    pub time_taken: f64, //`json:"timeTaken"`
}

// // DeleteFunctionRequest delete a deployed function
// #[derive(Serialize, Deserialize, Debug, PartialEq)]
// pub struct DeleteFunctionRequest {
//     #[serde(rename = "functionName")]
//     #[serde(default)]
//     pub function_name: String, //`json:"functionName"`
// }
