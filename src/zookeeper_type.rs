use serde::{Deserialize, Serialize};
use k8s_openapi::api::core::v1 as v1;



const DEFAULT_ZK_CONTAINER_REPOSITORY: &str = "default_zk_container_repository";
const DEFAULT_ZK_CONTAINER_VERSION: &str = "default_zk_container_version";
const DEFAULT_ZK_CONTAINER_POLICY: &str = "default_zk_container_policy";

// Implement the ContainerImage struct
#[derive(Default, Serialize, Deserialize)]
struct ContainerImage {
    repository: Option<String>,
    tag: Option<String>,
    #[serde(rename = "pullPolicy")]
    pull_policy: Option<v1::PullPolicy>,
}
impl ContainerImage {
    fn with_defaults(&mut self) -> bool {
        let mut changed = false;
        if self.repository.is_empty() {
            self.repository = String::from(DEFAULT_ZK_CONTAINER_REPOSITORY);
            changed = true;
        }
        if self.tag.is_empty() {
            self.tag = String::from(DEFAULT_ZK_CONTAINER_VERSION);
            changed = true;
        }
        if self.pull_policy.is_none() {
            self.pull_policy = Some(String::from(DEFAULT_ZK_CONTAINER_POLICY));
            changed = true;
        }
        changed
    }

    fn to_string(&self) -> String {
        format!("{}:{}", self.repository, self.tag)
    }
}


// Implement the PodPolicy struct
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct PodPolicy {
    #[serde(rename = "labels", skip_serializing_if = "Option::is_none")]
    labels: Option<std::collections::BTreeMap<String, String>>,

    #[serde(rename = "nodeSelector", skip_serializing_if = "Option::is_none")]
    node_selector: Option<std::collections::BTreeMap<String, String>>,

    #[serde(rename = "affinity", skip_serializing_if = "Option::is_none")]
    affinity: Option<Affinity>,

    #[serde(rename = "topologySpreadConstraints", skip_serializing_if = "Vec::is_empty")]
    topology_spread_constraints: Vec<TopologySpreadConstraint>,

    #[serde(rename = "resources", skip_serializing_if = "Option::is_none")]
    resources: Option<ResourceRequirements>,

    #[serde(rename = "tolerations", skip_serializing_if = "Vec::is_empty")]
    tolerations: Vec<Toleration>,

    #[serde(rename = "env", skip_serializing_if = "Vec::is_empty")]
    env: Vec<EnvVar>,

    #[serde(rename = "annotations", skip_serializing_if = "Option::is_none")]
    annotations: Option<std::collections::BTreeMap<String, String>>,

    #[serde(rename = "securityContext", skip_serializing_if = "Option::is_none")]
    security_context: Option<PodSecurityContext>,

    #[serde(rename = "terminationGracePeriodSeconds", skip_serializing_if = "Option::is_none")]
    termination_grace_period_seconds: Option<i64>,

    #[serde(rename = "serviceAccountName", skip_serializing_if = "Option::is_none")]
    service_account_name: Option<String>,

    #[serde(rename = "imagePullSecrets", skip_serializing_if = "Vec::is_empty")]
    image_pull_secrets: Vec<LocalObjectReference>,
}



// Implement the persistent struct
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct Persistence {
    #[serde(rename = "reclaimPolicy", skip_serializing_if = "Option::is_none")]
    volume_reclaim_policy: Option<String>,

    #[serde(rename = "spec", skip_serializing_if = "Option::is_none")]
    persistent_volume_claim_spec: Option<v1::PersistentVolumeClaimSpec>,

    #[serde(rename = "annotations", skip_serializing_if = "Option::is_none")]
    annotations: Option<std::collections::BTreeMap<String, String>>,
}

// Implement the ZookeeperClusterSpec struct
#[derive(Default, Serialize, Deserialize)]
struct ZookeeperClusterSpec{
    #[serde(rename = "image", skip_serializing_if = "Option::is_none")]
    image: Option<ContainerImage>,

    #[serde(rename = "replicas", skip_serializing_if = "Option::is_none")]
    replicas: Option<i32>,

    #[serde(rename = "storagetype", skip_serializing_if = "Option::is_none")]
    storagetype: Option<string>,

    #[serde(rename = "persistence", skip_serializing_if = "Option::is_none")]
    persistence: Option<Persistence>,
}