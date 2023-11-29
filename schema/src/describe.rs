use std::collections::HashMap;

///FunctionDescription information related to a function

#[derive(Debug)]
pub struct FunctionDescription<'s> {
    pub name: &'s str,
    pub status: &'s str,
    pub replicas: i32,
    pub available_replicas: i32,
    pub invocation_count: i32,
    pub image: &'s str,
    pub env_process: &'s str,
    pub url: &'s str,
    pub async_url: &'s str,
    pub labels: &'s HashMap<String, String>,
    pub annotations: &'s HashMap<String, String>,
}
