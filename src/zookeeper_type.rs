use serde::{Deserialize, Serialize};
use k8s_openapi::api::core::v1 as v1;
use k8s_openapi::api::core::v1::PersistentVolumeClaimSpec;

use kube::api::{TypeMeta, ObjectMeta};
use kube::{
    api::{Api, DeleteParams, ListParams, Patch, PatchParams, PostParams, ResourceExt},
    core::crd::CustomResourceExt,
    Client, CustomResource,
    runtime::controller::{Controller, Action}
};
use schemars::JsonSchema;
use super::status::ZookeeperClusterStatus;



const DEFAULT_ZK_CONTAINER_REPOSITORY: &str = "default_zk_container_repository";
const DEFAULT_ZK_CONTAINER_VERSION: &str = "default_zk_container_version";
const DEFAULT_ZK_CONTAINER_POLICY: &str = "default_zk_container_policy";

const PULL_ALWAYS: &str = "Always";
const PULL_NEVER: &str = "Never";
const PULL_IF_NOT_PRESENT: &str = "IfNotPresent";

// Implement the ContainerImage struct
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
struct ContainerImage {
    repository: Option<String>,
    tag: Option<String>,
    #[serde(rename = "pullPolicy")]
    pull_policy: Option<String>,
}
impl ContainerImage {
    fn with_defaults(&mut self) -> bool {
        let mut changed = false;
        if self.repository.is_none() {
            self.repository = Some(String::from(DEFAULT_ZK_CONTAINER_REPOSITORY));
            changed = true;
        }
        if self.tag.is_none() {
            self.tag = Some(String::from(DEFAULT_ZK_CONTAINER_VERSION));
            changed = true;
        }
        if self.pull_policy.is_none() {
            self.pull_policy = Some(String::from(DEFAULT_ZK_CONTAINER_POLICY));
            changed = true;
        }
        changed
    }

    fn to_string(&self) -> String {
        if self.repository.is_none() && self.tag.is_none() {
            return format!("{}:{}", self.repository.as_ref().unwrap(), self.tag.as_ref().unwrap());
        }else {
            return String::from("");
        }
    }
}


// Implement the PodPolicy struct
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
struct PodPolicy {
    #[serde(rename = "labels", skip_serializing_if = "Option::is_none")]
    labels: Option<std::collections::BTreeMap<String, String>>,

    #[serde(rename = "nodeSelector", skip_serializing_if = "Option::is_none")]
    node_selector: Option<std::collections::BTreeMap<String, String>>,

    // #[serde(rename = "affinity", skip_serializing_if = "Option::is_none")]
    // affinity: Option<Affinity>,

    // #[serde(rename = "topologySpreadConstraints", skip_serializing_if = "Vec::is_empty")]
    // topology_spread_constraints: Vec<TopologySpreadConstraint>,

    // #[serde(rename = "resources", skip_serializing_if = "Option::is_none")]
    // resources: Option<ResourceRequirements>,

    // #[serde(rename = "tolerations", skip_serializing_if = "Vec::is_empty")]
    // tolerations: Vec<Toleration>,

    // #[serde(rename = "env", skip_serializing_if = "Vec::is_empty")]
    // env: Vec<EnvVar>,

    #[serde(rename = "annotations", skip_serializing_if = "Option::is_none")]
    annotations: Option<std::collections::BTreeMap<String, String>>,

    // #[serde(rename = "securityContext", skip_serializing_if = "Option::is_none")]
    // security_context: Option<PodSecurityContext>,

    #[serde(rename = "terminationGracePeriodSeconds", skip_serializing_if = "Option::is_none")]
    termination_grace_period_seconds: Option<i64>,

    #[serde(rename = "serviceAccountName", skip_serializing_if = "Option::is_none")]
    service_account_name: Option<String>,

    // #[serde(rename = "imagePullSecrets", skip_serializing_if = "Vec::is_empty")]
    // image_pull_secrets: Vec<LocalObjectReference>,
}



// Implement the persistent struct
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
struct Persistence {
    #[serde(rename = "reclaimPolicy", skip_serializing_if = "Option::is_none")]
    volume_reclaim_policy: Option<String>,

    #[serde(rename = "spec", skip_serializing_if = "Option::is_none")]
    persistent_volume_claim_spec: Option<v1::PersistentVolumeClaimSpec>,

    #[serde(rename = "annotations", skip_serializing_if = "Option::is_none")]
    annotations: Option<std::collections::BTreeMap<String, String>>,
}


// Implement the ZookeeperClusterSpec struct
#[derive(CustomResource, Serialize, Deserialize, Default, Clone, Debug, PartialEq, JsonSchema)]
#[kube(
    group = "zookeeper.pravega.io",
    version = "v1beta1",
    kind = "ZookeeperCluster",
    plural = "zookeeperclusters",
    shortname = "zk",
    status = "ZookeeperClusterStatus",
    namespaced
    // printcolumn = r#"[
    //     {"name": "Replicas", "type": "integer", "jsonPath": ".spec.replicas", "description": "The number of ZooKeeper servers in the ensemble"},
    //     {"name": "Ready Replicas", "type": "integer", "jsonPath": ".status.readyReplicas", "description": "The number of ZooKeeper servers in the ensemble that are in a Ready state"},
    //     {"name": "Version", "type": "string", "jsonPath": ".status.currentVersion", "description": "The current Zookeeper version"},
    //     {"name": "Desired Version", "type": "string", "jsonPath": ".spec.image.tag", "description": "The desired Zookeeper version"},
    //     {"name": "Internal Endpoint", "type": "string", "jsonPath": ".status.internalClientEndpoint", "description": "Client endpoint internal to cluster network"},
    //     {"name": "External Endpoint", "type": "string", "jsonPath": ".status.externalClientEndpoint", "description": "Client endpoint external to cluster network via LoadBalancer"},
    //     {"name": "Age", "type": "date", "jsonPath": ".metadata.creationTimestamp"}
    // ]"#
)]
pub struct ZookeeperClusterSpec{
    #[serde(rename = "image", skip_serializing_if = "Option::is_none")]
    image: Option<ContainerImage>,

    #[serde(rename = "replicas", default)]
    pub replicas: i32,

    #[serde(rename = "storageType", skip_serializing_if = "Option::is_none")]
    storagetype: Option<String>,

    #[serde(rename = "persistence", skip_serializing_if = "Option::is_none")]
    persistence: Option<Persistence>,

    #[serde(rename = "triggerRollingRestart", skip_serializing_if = "Option::is_none")]
    triggerRollingRestart: Option<Bool>,
}



// pub struct ZookeeperCluster{
//     #[serde(flatten)]
//     pub type_meta: TypeMeta,
//     #[serde(rename = "metadata", skip_serializing_if = "Option::is_none")]
//     objectdata: Option<ObjectMeta>,
//     #[serde(rename = "spec", skip_serializing_if = "Option::is_none")]
//     spec: Option<ZookeeperClusterSpec>,
//     #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
//     status: Option<ZookeeperClusterStatus>,
// }

impl ZookeeperClusterSpec {
    pub fn new() -> ZookeeperClusterSpec {
        ZookeeperClusterSpec {
            image: None,
            replicas: 3,
            storagetype: None,
            persistence: None,
        }
    }
    pub fn with_defaults() -> bool{
        let mut changed = false;
        if self.image.is_none() {
            self.image = Some(ContainerImage::new());
            changed = true;
        }
        if self.image.as_ref().unwrap().with_defaults() {
            changed = true;
        }
        if self.replicas == 0 {
            self.replicas = 3;
            changed = true;
        }
        if self.storagetype.is_none() {
            self.storagetype = Some(String::from(DEFAULT_ZK_STORAGE_TYPE));
            changed = true;
        }
        if self.persistence.is_none() {
            self.persistence = Some(Persistence::new());
            changed = true;
        }
        if self.persistence.as_ref().unwrap().with_defaults() {
            changed = true;
        }
        changed
    }
}

impl Persistence {
    pub fn new() -> Persistence {
        Persistence {
            volume_reclaim_policy: None,
            persistent_volume_claim_spec: None,
            annotations: None,
        }
    }
    pub fn with_defaults() -> bool {
        let mut changed = false;
        if self.volume_reclaim_policy.is_none() {
            self.volume_reclaim_policy = Some(String::from(DEFAULT_ZK_VOLUME_RECLAIM_POLICY));
            changed = true;
        }
        if self.persistent_volume_claim_spec.is_none() {
            self.persistent_volume_claim_spec = Some(v1::PersistentVolumeClaimSpec::new());
            changed = true;
        }
        if self.persistent_volume_claim_spec.as_ref().unwrap().with_defaults() {
            changed = true;
        }
        if self.annotations.is_none() {
            self.annotations = Some(std::collections::BTreeMap::new());
            changed = true;
        }
        changed
    }
    pub fn get_trigger_rolling_restart(&self) -> bool {
        self.spec.triggerRollingRestart
    }
}