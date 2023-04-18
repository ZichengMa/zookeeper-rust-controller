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
    create_zk_cluster_crd();
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


fn create_zk_cluster_crd() -> bool {
    let client = Client::try_default().await?;

    // Manage CRDs first
    let crds: Api<CustomResourceDefinition> = Api::all(client.clone());

    // Delete any old versions of it first:
    let dp = DeleteParams::default();
    // but ignore delete err if not exists
    let _ = crds.delete("zookeepercluster.pravega.io", &dp).await.map(|res| {
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
    info!("Creating zk CRD: {}", serde_json::to_string_pretty(&zkrcd)?);
    let pp = PostParams::default();
    let patch_params = PatchParams::default();
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
    Ok(())
}