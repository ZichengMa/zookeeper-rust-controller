use std::{fmt, time::Duration};
use std::str::FromStr;
use std::convert::TryInto;
use std::collections::HashMap;
use zookeeper::{CreateMode, Watcher, WatchedEvent, ZkResult, ZooKeeper};
use zookeeper as zk;
use super::zookeeper_type::ZookeeperCluster;

pub struct DefaultZookeeperClient {
    conn: ZooKeeper,
}

struct MyWatcher; // This is a custom watcher that must be implemented if using zk-rust. Still not sure how to use it.
impl Watcher for MyWatcher {
    fn handle(&self, event: WatchedEvent) {
        println!("Node event: {:?}", event);
    }
}


impl DefaultZookeeperClient {
    // fn new() -> Result<Self, Box<dyn std::error::Error>> {
    //     let conn = ZooKeeper::connect("localhost:2181", Duration::from_secs(5), WatcherFn)?;
    //     Ok(Self { conn })
    // }
    
    fn connect(&mut self, zk_uri: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = ZooKeeper::connect(zk_uri, Duration::from_secs(5), MyWatcher)?;
        self.conn = conn;
        Ok(())
    }

    fn create_node(&self, zoo: &ZookeeperCluster, z_node_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // @TODO: Not sure whether flags=0 in zk-Go means CreateMode::Persistent in zk-rust.
        let paths = z_node_path.split('/').filter(|&p| !p.is_empty()).collect::<Vec<_>>();
        let path_length = paths.len();
        let mut parent_path = String::new();
        for i in 1..(path_length - 1) {
            parent_path.push('/');
            parent_path.push_str(paths[i]);
            match self.conn.create(&parent_path, vec![], zk::Acl::open_unsafe().clone(),CreateMode::Persistent) {
                Ok(_) => {},
                Err(e) if e == zk::ZkError::NodeExists => {}, // Ignore if node already exists.
                Err(e) => return Err(Box::new(e))
            }
        }
        let data = format!("CLUSTER_SIZE={}", zoo.spec.replicas);
        let child_node = format!("{}{}", parent_path, paths[path_length-1]);
        self.conn.create(&child_node, data.as_bytes().to_vec(), zk::Acl::open_unsafe().clone(),CreateMode::Persistent)?;
        Ok(())
    }

    // fn update_node(&self, path: &str, data: &str, version: i32) -> Result<(), Box<dyn std::error::Error>> {
    //     let version: i32 = version.try_into().unwrap_or(-1);
    //     self.conn.set_data(path, data.as_bytes().to_vec(), version)?;
    //     Ok(())
    // }

    // fn node_exists(&self, z_node_path: &str) -> Result<i32, Box<dyn std::error::Error>> {
    //     match self.conn.exists(z_node_path, false) {
    //         Ok(Some(stat)) => Ok(stat.version()),
    //         Ok(None) => Err(Box::new(ZkError::NoNode)),
    //         Err(e) => Err(Box::new(e)),
    //     }
    // }

    // fn close(&self) {
    //     self.conn.close();
    // }
}