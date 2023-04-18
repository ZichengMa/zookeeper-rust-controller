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

mod zookeeper_type;
mod status;
mod zookeeper_client_go;
use zookeeper_type::ZookeeperCluster;
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




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init(); // init logging
    let client = Client::try_default().await?;
    let context = Arc::new(()); // bad empty context - put client in here
    let pods = Api::<Pod>::all(client.clone());
    Ok(())
}