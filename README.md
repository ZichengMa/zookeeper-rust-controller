# zookeeper-rust-controller
This project aims to write a controller for zookeeper on kubernetes in rust.

2023.4.1 -- Initialize two simple strruct. Add a simple controller for CRD podSet.

2023.4.17 -- implement zookeeper types and zookeeper client go(using zookeeper-rust create)

2023.4.18 -- install CRDs definition and run a test yaml to create zk instance(no container image now)

2023.4.19/20 -- apply zkclient into recounciler

2023.4.21 -- implement with_default() for zkcluster.. (used to judge state)

TODO -- 1. based on Go controller, write logic for different states 2. figure out how to connect zookeeper/ open zookeeper cluster 3. pull a container image

Not Sure: 1. Whether flags=0 in zk-Go means CreateMode::Persistent in zk-rust.