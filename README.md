# zookeeper-rust-controller
This project aims to write a controller for zookeeper on kubernetes in rust.

2023.4.1 -- Initialize two simple strruct. Add a simple controller for CRD podSet.

2023.4.17 -- implement zookeeper types and zookeeper client go(using zookeeper-rust create)

2023.4.18 -- install CRDs definition and run a test yaml to create zk instance(no container image now)

2023.4.19/20 -- apply zkclient into recounciler

2023.4.21 -- implement with_default() for zkcluster.. (used to judge state)

2023.4.23 -- implement rollingTriggerStart

2023.4.24 -- almost finish all with_defaults() check    But immutable/mutable reference is not solved.

### Supporot

- [x] Create CRD ZookeeperCluster in k8s API
- [x] Create ZookeeperCluster resources/API in k8s
- [x] Controller detects a ZookeeperCluster resources has been created
- [x] Test the state of current ZookeeperCluster

### TODO

- [ ] using client to modify ConfigMap, SateFulSet, Service 
- [ ] figure out how to connect zookeeper/ open zookeeper cluster 
- [ ] pull a container image 

*Not Sure*: 1. Whether flags=0 in zk-Go means CreateMode::Persistent in zk-rust. 2. What is storage.is_Zero() with_defaults for Persistence