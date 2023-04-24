use serde::{Deserialize, Serialize};
use k8s_openapi::api::core::v1 as v1;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as metav1;
use k8s_openapi::api::core::v1::PersistentVolumeClaimSpec;
use std::collections::HashMap;
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


pub const DEFAULT_TERMINATION_GRACE_PERIOD: i64 = 30;
pub const DEFAULT_ZOOKEEPER_CACHE_VOLUME_SIZE: &str = "20Gi";
pub const DEFAULT_READINESS_PROBE_INITIAL_DELAY_SECONDS: i32 = 10;
pub const DEFAULT_READINESS_PROBE_PERIOD_SECONDS: i32 = 10;
pub const DEFAULT_READINESS_PROBE_FAILURE_THRESHOLD: i32 = 3;
pub const DEFAULT_READINESS_PROBE_SUCCESS_THRESHOLD: i32 = 1;
pub const DEFAULT_READINESS_PROBE_TIMEOUT_SECONDS: i32 = 10;
pub const DEFAULT_LIVENESS_PROBE_INITIAL_DELAY_SECONDS: i32 = 10;
pub const DEFAULT_LIVENESS_PROBE_PERIOD_SECONDS: i32 = 10;
pub const DEFAULT_LIVENESS_PROBE_FAILURE_THRESHOLD: i32 = 3;
pub const DEFAULT_LIVENESS_PROBE_TIMEOUT_SECONDS: i32 = 10;


// Implement the ContainerImage struct
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
struct ContainerImage {
    repository: Option<String>,
    tag: Option<String>,
    #[serde(rename = "pullPolicy")]
    pull_policy: Option<String>,
}
impl ContainerImage {
    fn new() -> Self {
        ContainerImage {
            repository: None,
            tag: None,
            pull_policy: None,
        }
    }
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

    #[serde(rename = "affinity", skip_serializing_if = "Option::is_none")]
    affinity: Option<v1::Affinity>,

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

impl PodPolicy {
    fn new() -> Self {
        PodPolicy {
            labels: None,
            node_selector: None,
            affinity: None,
            annotations: None,
            termination_grace_period_seconds: None,
            service_account_name: None,
        }
    }
    fn with_defaults(&mut self, z: &ZookeeperCluster) -> bool {
        let mut changed = false;

        if self.labels.is_none() {
            self.labels = Some(std::collections::BTreeMap::new());
            changed = true;
        }

        if self.termination_grace_period_seconds.is_none() {
            self.termination_grace_period_seconds = Some(DEFAULT_TERMINATION_GRACE_PERIOD);
            changed = true;
        }

        if self.service_account_name.is_none() {
            self.service_account_name = Some("default".to_owned());
            changed = true;
        }

        if z.spec.pod.as_ref().unwrap().labels.is_none() {
            self.labels = Some(std::collections::BTreeMap::new());
            changed = true;
        }

        if !self.labels.as_ref().unwrap().contains_key("app") {
            self.labels.as_mut().unwrap().insert("app".to_owned(), z.metadata.name.unwrap());
            changed = true;
        }

        if !self.labels.as_ref().unwrap().contains_key("release") {
            self.labels.as_mut().unwrap().insert("release".to_owned(), z.metadata.name.unwrap());
            changed = true;
        }

        if self.affinity.is_none() {
            self.affinity = Some(v1::Affinity {
                pod_anti_affinity: Some(v1::PodAntiAffinity {
                    preferred_during_scheduling_ignored_during_execution: Some(vec![
                        v1::WeightedPodAffinityTerm {
                            weight: 20,
                            pod_affinity_term: v1::PodAffinityTerm {
                                topology_key: "kubernetes.io/hostname".to_owned(),
                                label_selector: Some(metav1::LabelSelector {
                                    match_expressions:Some( vec![
                                        metav1::LabelSelectorRequirement {
                                            key: "app".to_owned(),
                                            operator: "In".to_owned(),
                                            values: Some(vec![z.metadata.name.unwrap()]),
                                        },
                                    ]),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                        },
                    ]),
                    ..Default::default()
                }),
                ..Default::default()
            });
            changed = true;
        }

        changed
    }
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

impl Persistence {
    pub fn new() -> Persistence {
        Persistence {
            volume_reclaim_policy: None,
            persistent_volume_claim_spec: None,
            annotations: None,
        }
    }
    pub fn with_defaults(&mut self) -> bool {
        let mut changed = false;
        if self.volume_reclaim_policy.is_none() {
            self.volume_reclaim_policy = Some(String::from("Retain"));
            changed = true;
        }
        if self.persistent_volume_claim_spec.is_none() {
            self.persistent_volume_claim_spec = Some(v1::PersistentVolumeClaimSpec {
                access_modes: Some(vec![String::from("ReadWriteOnce")]),
                ..Default::default()
            });
        }
        let mut spec = self.persistent_volume_claim_spec.take().unwrap();
        spec.access_modes = Some(vec![String::from("ReadWriteOnce")]);
        self.persistent_volume_claim_spec = Some(spec);
        let storage = self.persistent_volume_claim_spec.as_ref().unwrap().resources.as_ref().unwrap().requests.as_ref().unwrap().get("storage");
        // @TODO: What is storage == 0?
        // if storage.is_none() || storage.unwrap().is_zero() {
        //     self.persistent_volume_claim_spec.resources.requests =
        //         std::collections::BTreeMap::from_iter(vec![(
        //             v1::ResourceStorage,
        //             resource::Quantity(DefaultZookeeperCacheVolumeSize.to_owned()),
        //         )]);
        //     changed = true;
        // }
        changed
    }
}

// Implement zk config
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ZookeeperConfig {
    #[serde(rename = "initLimit", skip_serializing_if = "Option::is_none")]
    pub init_limit: Option<i32>,

    #[serde(rename = "tickTime", skip_serializing_if = "Option::is_none")]
    pub tick_time: Option<i32>,

    #[serde(rename = "syncLimit", skip_serializing_if = "Option::is_none")]
    pub sync_limit: Option<i32>,

    #[serde(rename = "globalOutstandingLimit", skip_serializing_if = "Option::is_none")]
    pub global_outstanding_limit: Option<i32>,

    #[serde(rename = "preAllocSize", skip_serializing_if = "Option::is_none")]
    pub pre_alloc_size: Option<i32>,

    #[serde(rename = "snapCount", skip_serializing_if = "Option::is_none")]
    pub snap_count: Option<i32>,

    #[serde(rename = "commitLogCount", skip_serializing_if = "Option::is_none")]
    pub commit_log_count: Option<i32>,

    #[serde(rename = "snapSizeLimitInKb", skip_serializing_if = "Option::is_none")]
    pub snap_size_limit_in_kb: Option<i32>,

    #[serde(rename = "maxCnxns", skip_serializing_if = "Option::is_none")]
    pub max_cnxns: Option<i32>,

    #[serde(rename = "maxClientCnxns", skip_serializing_if = "Option::is_none")]
    pub max_client_cnxns: Option<i32>,

    #[serde(rename = "minSessionTimeout", skip_serializing_if = "Option::is_none")]
    pub min_session_timeout: Option<i32>,

    #[serde(rename = "maxSessionTimeout", skip_serializing_if = "Option::is_none")]
    pub max_session_timeout: Option<i32>,

    #[serde(rename = "autoPurgeSnapRetainCount", skip_serializing_if = "Option::is_none")]
    pub auto_purge_snap_retain_count: Option<i32>,

    #[serde(rename = "autoPurgePurgeInterval", skip_serializing_if = "Option::is_none")]
    pub auto_purge_purge_interval: Option<i32>,

    #[serde(rename = "quorumListenOnAllIPs", skip_serializing_if = "Option::is_none")]
    pub quorum_listen_on_all_ips: Option<bool>,

    #[serde(rename = "additionalConfig", skip_serializing_if = "Option::is_none")]
    pub additional_config: Option<HashMap<String, String>>,
}

impl ZookeeperConfig {
    pub fn new() -> Self {
        ZookeeperConfig {
            init_limit: None,
            tick_time: None,
            sync_limit: None,
            global_outstanding_limit: None,
            pre_alloc_size: None,
            snap_count: None,
            commit_log_count: None,
            snap_size_limit_in_kb: None,
            max_cnxns: None,
            max_client_cnxns: None,
            min_session_timeout: None,
            max_session_timeout: None,
            auto_purge_snap_retain_count: None,
            auto_purge_purge_interval: None,
            quorum_listen_on_all_ips: None,
            additional_config: None,
        }
    }
    pub fn with_defaults(&mut self) -> bool {
        let mut changed = false;
        if self.init_limit.is_none() {
            changed = true;
            self.init_limit = Some(10);
        }
        if self.tick_time.is_none() {
            changed = true;
            self.tick_time = Some(2000);
        }
        if self.sync_limit.is_none() {
            changed = true;
            self.sync_limit = Some(2);
        }
        if self.global_outstanding_limit.is_none() {
            changed = true;
            self.global_outstanding_limit = Some(1000);
        }
        if self.pre_alloc_size.is_none() {
            changed = true;
            self.pre_alloc_size = Some(65536);
        }
        if self.snap_count.is_none() {
            changed = true;
            self.snap_count = Some(10000);
        }
        if self.commit_log_count.is_none() {
            changed = true;
            self.commit_log_count = Some(500);
        }
        if self.snap_size_limit_in_kb.is_none() {
            changed = true;
            self.snap_size_limit_in_kb = Some(4194304);
        }
        if self.max_client_cnxns.is_none() {
            changed = true;
            self.max_client_cnxns = Some(60);
        }
        if self.min_session_timeout.is_none() {
            changed = true;
            self.min_session_timeout = Some(2 * self.tick_time.unwrap());
        }
        if self.max_session_timeout.is_none() {
            changed = true;
            self.max_session_timeout = Some(20 * self.tick_time.unwrap());
        }
        if self.auto_purge_snap_retain_count.is_none() {
            changed = true;
            self.auto_purge_snap_retain_count = Some(3);
        }
        if self.auto_purge_purge_interval.is_none() {
            changed = true;
            self.auto_purge_purge_interval = Some(1);
        }
        changed
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Probe {
    #[serde(rename = "initialDelaySeconds", skip_serializing_if = "Option::is_none")]
    pub initial_delay_seconds: Option<i32>,
    #[serde(rename = "periodSeconds", skip_serializing_if = "Option::is_none")]
    pub period_seconds: Option<i32>,
    #[serde(rename = "failureThreshold", skip_serializing_if = "Option::is_none")]
    pub failure_threshold: Option<i32>,
    #[serde(rename = "successThreshold", skip_serializing_if = "Option::is_none")]
    pub success_threshold: Option<i32>,
    #[serde(rename = "timeoutSeconds", skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<i32>,
}

impl Probe {
    fn default() -> Self {
        Probe {
            initial_delay_seconds: None,
            period_seconds: None,
            failure_threshold: None,
            success_threshold: None,
            timeout_seconds: None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
struct Probes {
    #[serde(rename="readinessProbe", skip_serializing_if = "Option::is_none")]
    readiness_probe: Option<Probe>,
    #[serde(rename="livenessProbe", skip_serializing_if = "Option::is_none")]
    liveness_probe: Option<Probe>,
}

impl Probes {
    fn new() -> Self {
        Probes {
            readiness_probe: None,
            liveness_probe: None,
        }
    }
    fn with_defaults(&mut self) -> bool{
        let mut changed = false;
        if self.readiness_probe.is_none() {
            changed = true;
            self.readiness_probe = Some(Probe {
                initial_delay_seconds: Some(DEFAULT_READINESS_PROBE_INITIAL_DELAY_SECONDS),
                period_seconds: Some(DEFAULT_READINESS_PROBE_PERIOD_SECONDS),
                failure_threshold: Some(DEFAULT_READINESS_PROBE_FAILURE_THRESHOLD),
                success_threshold: Some(DEFAULT_READINESS_PROBE_SUCCESS_THRESHOLD),
                timeout_seconds: Some(DEFAULT_READINESS_PROBE_TIMEOUT_SECONDS),
            });
        }
        if self.liveness_probe.is_none() {
            changed = true;
            self.liveness_probe = Some(Probe {
                initial_delay_seconds: Some(DEFAULT_LIVENESS_PROBE_INITIAL_DELAY_SECONDS),
                period_seconds: Some(DEFAULT_LIVENESS_PROBE_PERIOD_SECONDS),
                failure_threshold: Some(DEFAULT_LIVENESS_PROBE_FAILURE_THRESHOLD),
                success_threshold: None,
                timeout_seconds: Some(DEFAULT_LIVENESS_PROBE_TIMEOUT_SECONDS),
            });
        }
        changed
    }
}



#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
struct Ephemeral{
    #[serde(rename="emptydirvolumesource", skip_serializing_if = "Option::is_none")]
    emptydirvolumesource: Option<v1::EmptyDirVolumeSource>,
}
impl Ephemeral {
    fn new() -> Self {
        Ephemeral {
            emptydirvolumesource: None,
        }
    }
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
    triggerRollingRestart: Option<bool>,

    #[serde(rename = "config", skip_serializing_if = "Option::is_none")]
    zkconfig: Option<ZookeeperConfig>,

    #[serde(rename = "probes", skip_serializing_if = "Option::is_none")]
    probes: Option<Probes>,

    #[serde(rename = "ports", skip_serializing_if = "Option::is_none")]
    ports: Option<Vec<v1::ContainerPort>>,

    #[serde(rename = "pod", skip_serializing_if = "Option::is_none")]
    pod: Option<PodPolicy>,

    #[serde(rename = "ephemeral", skip_serializing_if = "Option::is_none")]
    ephemeral: Option<Ephemeral>,
}









impl ZookeeperClusterSpec {
    pub fn new() -> ZookeeperClusterSpec {
        ZookeeperClusterSpec {
            image: None,
            replicas: 3,
            storagetype: None,
            persistence: None,
            triggerRollingRestart: None,
            zkconfig: None,
            probes: None,
            ports: None,
            pod: None,
            ephemeral: None,
        }
    }
    pub fn with_defaults(&mut self, z: & ZookeeperCluster) -> bool{
        let mut changed = false;
        if self.image.is_none() {
            self.image = Some(ContainerImage::new()); // Initialize the ContainerImage struct
        }
        if self.image.as_mut().unwrap().with_defaults() {
            changed = true;
        }

        if self.zkconfig.is_none() {
            self.zkconfig = Some(ZookeeperConfig::new()); // Initialize the ZookeeperConfig struct
        }
        if self.zkconfig.as_mut().unwrap().with_defaults() {
            changed = true;
        }

        if self.replicas == 0 {
            self.replicas = 3;
            changed = true;
        }

        if self.probes.is_none() {
            self.probes = Some(Probes::new()); // Initialize the Probes struct
        }
        if self.probes.as_mut().unwrap().with_defaults() {
            changed = true;
        }


        if self.ports.is_none() {
            self.ports = Some(vec![
                v1::ContainerPort {
                    name: Some("client".to_owned()),
                    container_port: 2181,
                    ..Default::default()
                },
                v1::ContainerPort {
                    name: Some("quorum".to_owned()),
                    container_port: 2888,
                    ..Default::default()
                },
                v1::ContainerPort {
                    name: Some("leader-election".to_owned()),
                    container_port: 3888,
                    ..Default::default()
                },
                v1::ContainerPort {
                    name: Some("metrics".to_owned()),
                    container_port: 7000,
                    ..Default::default()
                },
                v1::ContainerPort {
                    name: Some("admin-server".to_owned()),
                    container_port: 8080,
                    ..Default::default()
                },
            ]);
            changed = true;
        } else {
            let mut found_client = false;
            let mut found_quorum = false;
            let mut found_leader = false;
            let mut found_metrics = false;
            let mut found_admin = false;
        
            if let Some(ports) = self.ports.as_mut() {
                for p in ports.iter() {
                    match p.name {
                        Some(ref n) if n == "client" => found_client = true,
                        Some(ref n) if n == "quorum" => found_quorum = true,
                        Some(ref n) if n == "leader-election" => found_leader = true,
                        Some(ref n) if n == "metrics" => found_metrics = true,
                        Some(ref n) if n == "admin-server" => found_admin = true,
                        _ => (),
                    }
                }
                if !found_client {
                    let port = v1::ContainerPort {
                        name: Some("client".to_owned()),
                        container_port: 2181,
                        ..Default::default()
                    };
                    ports.push(port);
                    changed = true;
                }
                if !found_quorum {
                    let port = v1::ContainerPort {
                        name: Some("quorum".to_owned()),
                        container_port: 2888,
                        ..Default::default()
                    };
                    ports.push(port);
                    changed = true;
                }
                if !found_leader {
                    let port = v1::ContainerPort {
                        name: Some("leader-election".to_owned()),
                        container_port: 3888,
                        ..Default::default()
                    };
                    ports.push(port);
                    changed = true;
                }
                if !found_metrics {
                    let port = v1::ContainerPort {
                        name: Some("metrics".to_owned()),
                        container_port: 7000,
                        ..Default::default()
                    };
                    ports.push(port);
                    changed = true;
                }
                if !found_admin {
                    let port = v1::ContainerPort {
                        name: Some("admin-server".to_owned()),
                        container_port: 8080,
                        ..Default::default()
                    };
                    ports.push(port);
                    changed = true;
                }
            }
        }
        



        if self.pod.is_none() {
            self.pod = Some(PodPolicy::new()); // Initialize the PodPolicy struct
        }
        if self.pod.as_mut().unwrap().with_defaults(z) {
            changed = true;
        }


        if self.storagetype.is_none() {
            self.storagetype = Some("".to_owned());
            changed = true;
        }
        if  self.storagetype.as_mut().unwrap() == "ephemeral" {
            if self.ephemeral.is_none() {
                self.ephemeral = Some(Ephemeral::new()); // Initialize the Ephemeral struct
                self.ephemeral.as_mut().unwrap().emptydirvolumesource = Some(v1::EmptyDirVolumeSource {..Default::default()});
                changed = true;
            }
        } else {
            if self.persistence.is_none() {
                self.storagetype = Some("persistence".to_owned());
                self.persistence = Some(Persistence::new()); // Initialize the Persistence struct
                changed = true;
            }
            if self.persistence.as_mut().unwrap().with_defaults() {
                self.storagetype = Some("persistence".to_owned());
                changed = true;
            }
        }
        changed
    }
}


impl ZookeeperCluster {
    pub fn with_defaults(&mut self)->bool{
        let mut changed = false;
        let spec = &mut self.spec;
        let temp = self;
        if spec.with_defaults(temp) {
            changed = true;
        }
        changed
    }
    pub fn get_trigger_rolling_restart(&self) -> bool {
        if self.spec.triggerRollingRestart.is_none() {
            return false;
        }
        self.spec.triggerRollingRestart.unwrap()
    }
}