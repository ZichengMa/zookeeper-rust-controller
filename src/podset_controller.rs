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

/// The reconciler that will be called when either object change
async fn reconcile(g: Arc<PodSet>, _ctx: Arc<()>) -> Result<Action, Error> {
    // .. use api here to reconcile a child ConfigMap with ownerreferences
    // see configmapgen_controller example for full info
    println!("reconciling {:?}", g);
    Ok(Action::requeue(Duration::from_secs(300)))
}
/// an error handler that will be called when the reconciler fails with access to both the
/// object that caused the failure and the actual error
fn error_policy(obj: Arc<PodSet>, _error: &Error, _ctx: Arc<()>) -> Action {
    Action::requeue(Duration::from_secs(60))
}

/// something to drive the controller
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init(); // init logging
    let client = Client::try_default().await?;
    let context = Arc::new(()); // bad empty context - put client in here
    let podset = Api::<PodSet>::all(client.clone());
    let pods = Api::<Pod>::all(client.clone());
    Controller::new(podset, ListParams::default())
        .owns(pods, ListParams::default())
        .run(reconcile, error_policy, context)
        .for_each(|res| async move {
            match res {
                Ok(o) => println!("reconciled {:?}", o),
                Err(e) => println!("reconcile failed: {:?}", e),
            }
        })
        .await; // controller does nothing unless polled
    Ok(())
}