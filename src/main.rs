use kube::{
    api::{Api, DeleteParams, ListParams, Patch, PatchParams, PostParams, ResourceExt},
    core::crd::CustomResourceExt,
    Client, CustomResource,
    runtime::controller::{Controller, Action}
};
use serde::{Deserialize, Serialize};
use tokio::time::Duration;

use futures::StreamExt;

use schemars::JsonSchema;
use std::sync::Arc;
use thiserror::Error;
use tracing::*;
mod zookeeper_type;
mod status;
mod zookeeper_client_go;
use zookeeper_type::{ZookeeperClusterSpec, ZookeeperCluster};
use zookeeper_client_go as zk;

#[derive(Debug, Error)]
enum Error {}
const RECONCILE_TIME: Duration = Duration::from_secs(30);

struct ZookeeperClusterReconciler {
    client: kube::Client,
    log: tracing_subscriber::fmt::Subscriber,
    // scheme: kube::runtime::Scheme, can not find same in rust
    zk_client: zk::DefaultZookeeperClient,
}


async fn reconcile(g: Arc<ZookeeperCluster>, _ctx: Arc<()>) -> Result<Action, Error> {
    // .. use api here to reconcile a child ConfigMap with ownerreferences
    // see configmapgen_controller example for full info
    println!("reconciling {:?}", g);
    Ok(Action::requeue(Duration::from_secs(300)))
}

/// object that caused the failure and the actual error
fn error_policy(obj: Arc<ZookeeperCluster>, _error: &Error, _ctx: Arc<()>) -> Action {
    Action::requeue(Duration::from_secs(60))
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    tracing_subscriber::fmt::init(); // init logging
    let client = Client::try_default().await?;
    let context = Arc::new(()); // bad empty context - put client in here

    let zk_cluster = Api::<ZookeeperCluster>::all(client.clone());
    Controller::new(zk_cluster, ListParams::default())
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
