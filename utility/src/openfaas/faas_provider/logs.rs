/// logs provides the standard interface and handler for OpenFaaS providers to expose function logs.
///
/// The package defines the Requester interface that OpenFaaS providers should implement and then expose using
/// the predefined NewLogHandlerFunc. See the example folder for a minimal log provider implementation.
///
/// The Requester is where the actual specific logic for connecting to and querying the log system should be implemented.
use serde::{Deserialize, Serialize};
/// Request is the query to return the function logs.
#[derive(Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct Request<'s> {
    /// Name is the function name and is required
    pub name: &'s str, //`json:"name"`
    /// Namespace is the namespace the function is deployed to, how a namespace is defined
    /// is faas-provider specific
    pub namespace: &'s str, //`json:"namespace"`
    /// Instance is the optional container name, that allows you to request logs from a specific function instance
    #[serde(default)]
    pub instance: &'s str, //`json:"instance"`
    /// Since is the optional datetime value to start the logs from in time stamp
    #[serde(default)]
    pub since: Option<i64>, //`json:"since"`
    /// Tail sets the maximum number of log messages to return, <=0 means unlimited
    pub tail: isize, //`json:"tail"`
    /// Follow is allows the user to request a stream of logs until the timeout
    pub follow: bool, //`json:"follow"`
}

impl<'s> ToString for Request<'s> {
    fn to_string(&self) -> String {
        format!(
            "name: {} namespace: {} instance: {} since: {} tail: {} follow: {}",
            self.name,
            self.namespace,
            self.instance,
            self.since.unwrap_or(0),
            self.tail,
            self.follow
        )
    }
}

/// Message is a specific log message from a function container log stream
#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct Message {
    /// Name is the function name
    #[serde(default)]
    pub name: String, //`json:"name"`
    /// Namespace is the namespace the function is deployed to, how a namespace is defined
    /// is faas-provider specific
    #[serde(default)]
    pub namespace: String, //`json:"namespace"`
    /// instance is the name/id of the specific function instance
    #[serde(default)]
    pub instance: String, //`json:"instance"`
    /// Timestamp is the timestamp of when the log message was recorded
    #[serde(default)]
    pub timestamp: String, //`json:"timestamp"`
    /// Text is the raw log message content
    #[serde(default)]
    pub text: String, //`json:"text"`
}

impl<'s> ToString for Message {
    fn to_string(&self) -> String {
        let namespace = if !self.namespace.is_empty() {
            self.namespace.to_string() + " "
        } else {
            String::new()
        };
        format!(
            "{} {} ({}{}) {}",
            self.timestamp, self.name, namespace, self.instance, self.text
        )
    }
}
