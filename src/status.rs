use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ZookeeperClusterStatus {
    #[serde(rename = "members", skip_serializing_if = "Option::is_none")]
    members: Option<MembersStatus>,

    #[serde(rename = "replicas", skip_serializing_if = "Option::is_none")]
    replicas: Option<i32>,

    #[serde(rename = "readyReplicas", skip_serializing_if = "Option::is_none")]
    ready_replicas: Option<i32>,

    #[serde(rename = "internalClientEndpoint", skip_serializing_if = "Option::is_none")]
    internal_client_endpoint: Option<String>,

    #[serde(rename = "externalClientEndpoint", skip_serializing_if = "Option::is_none")]
    external_client_endpoint: Option<String>,

    #[serde(rename = "metaRootCreated", skip_serializing_if = "Option::is_none")]
    meta_root_created: Option<bool>,

    #[serde(rename = "currentVersion", skip_serializing_if = "Option::is_none")]
    current_version: Option<String>,

    #[serde(rename = "targetVersion", skip_serializing_if = "Option::is_none")]
    target_version: Option<String>,

    #[serde(rename = "conditions", skip_serializing_if = "Vec::is_empty")]
    conditions: Vec<ClusterCondition>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct MembersStatus {
    #[serde(rename = "ready", skip_serializing_if = "Vec::is_empty")]
    ready: Vec<String>,

    #[serde(rename = "unready", skip_serializing_if = "Vec::is_empty")]
    unready: Vec<String>,
}

const CONDITION_TRUE: &str = "True";
const CONDITION_FALSE: &str = "False";
const CONDITION_UNKNOWN: &str  = "Unknown";

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct ClusterCondition {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    condition_type: Option<ClusterConditionType>,

    #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
    status: Option<String>, // original type: ConditionStatus

    #[serde(rename = "reason", skip_serializing_if = "Option::is_none")]
    reason: Option<String>,

    #[serde(rename = "message", skip_serializing_if = "Option::is_none")]
    message: Option<String>,

    #[serde(rename = "lastUpdateTime", skip_serializing_if = "Option::is_none")]
    last_update_time: Option<String>,

    #[serde(rename = "lastTransitionTime", skip_serializing_if = "Option::is_none")]
    last_transition_time: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum ClusterConditionType {
    #[serde(rename = "PodsReady")]
    PodsReady,
    #[serde(rename = "Upgrading")]
    Upgrading,
    #[serde(rename = "Error")]
    Error,
}