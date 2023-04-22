use kube::{
    api::{Api, DeleteParams, ListParams, Patch, PatchParams, PostParams, ResourceExt},
    core::crd::CustomResourceExt,
    Client, CustomResource,
    runtime::controller::{Controller, Action}
};
use serde::{Deserialize, Serialize};
use tokio::time::Duration;
use tokio::time::sleep;
use futures::StreamExt;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use k8s_openapi::api::core::v1::Pod;
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


async fn reconcile(g: Arc<ZookeeperCluster>, _ctx: Arc<ZookeeperClusterReconciler>) -> Result<Action, Error> {
    // .. use api here to reconcile a child ConfigMap with ownerreferences
    // see configmapgen_controller example for full info
    let zk_client = &_ctx.zk_client;
    let client = _ctx.client.clone();
    println!("reconciling {:?}", g);
    
    let changed = g.with_defaults();
    
    if changed {
        // todo 
    }
    Ok(Action::requeue(Duration::from_secs(300)))
}

/// object that caused the failure and the actual error
fn error_policy(obj: Arc<ZookeeperCluster>, _error: &Error, _ctx: Arc<ZookeeperClusterReconciler>) -> Action {
    Action::requeue(Duration::from_secs(60))
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {


    {
        // create CRD definition
        let client = Client::try_default().await?;
        // Manage CRDs first
        let crds: Api<CustomResourceDefinition> = Api::all(client.clone());
    
        // Delete any old versions of it first:
        let dp = DeleteParams::default();
        // but ignore delete err if not exists
        let _ = crds.delete("zookeepercluster.zookeeper.pravega.io", &dp).await.map(|res| {
            res.map_left(|o| {
                info!(
                    "Deleting {}: ({:?})",
                    o.name_any(),
                    o.status.unwrap().conditions.unwrap().last()
                );
            })
            .map_right(|s| {
                // it's gone.
                info!("Deleted old version: ({:?})", s);
            })
        });
        // Wait for the delete to take place (map-left case or delete from previous run)
        sleep(Duration::from_secs(2)).await;
    
        // Create the CRD so we can create Foos in kube
        let zkcrd = ZookeeperCluster::crd();
        info!("Creating zk CRD: {}", serde_json::to_string_pretty(&zkcrd)?);
        let pp = PostParams::default();
        match crds.create(&pp, &zkcrd).await {
            Ok(o) => {
                info!("Created {} ({:?})", o.name_any(), o.status.unwrap());
                debug!("Created CRD: {:?}", o.spec);
            }
            Err(kube::Error::Api(ae)) => assert_eq!(ae.code, 409), // if you skipped delete, for instance
            Err(e) => return Err(e.into()),                        // any other case is probably bad
        }
        // Wait for the api to catch up
        sleep(Duration::from_secs(1)).await;
    }

    tracing_subscriber::fmt::init(); // init logging
    let client = Client::try_default().await?;
    let zk_cluster = Api::<ZookeeperCluster>::all(client.clone());
    let zk_client = zk::DefaultZookeeperClient::new("localhost:2181");
    let pods = Api::<Pod>::all(client.clone());

    let log = tracing_subscriber::fmt::Subscriber::new();
    let context = Arc::new(ZookeeperClusterReconciler{ client, log, zk_client }); // context with zookeeperclusterReconciler


    Controller::new(zk_cluster, ListParams::default())
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
