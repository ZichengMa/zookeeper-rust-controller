use kube::{
Client, CustomResource,
api::{Api, ListParams},
runtime::controller::{Controller, Action}
};
use serde::{Deserialize, Serialize};
use tokio::time::Duration;
use futures::StreamExt;
use k8s_openapi::api::core::v1::Pod;
use schemars::JsonSchema;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
enum Error {}

/// A custom resource
#[derive(CustomResource, Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[kube(group = "batch.tutorial.kubebuilder.io", version = "v1", kind = "PodSet", namespaced)]
#[kube(status = "PodSetStatus")]
pub struct PodSetSpec {
    replicas: i32,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub struct PodSetStatus {
    podNames: Vec<String>,
    readyReplicas: i32,
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init(); // init logging
    let client = Client::try_default().await?;
    let context = Arc::new(()); // bad empty context - put client in here
    let podset = Api::<PodSet>::all(client.clone());
    let pods = Api::<Pod>::all(client.clone());
    Ok(())
}