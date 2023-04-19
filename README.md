# zookeeper-rust-controller
This project aims to write a controller for zookeeper on kubernetes in rust.

2023.4.1 -- Initialize two simple strruct. Add a simple controller for CRD podSet.

2023.4.17 -- implement zookeeper types and zookeeper client go(using zookeeper-rust create)

2023.4.18 -- install CRDs definition and run a test yaml to create zk instance(no container image now)

TODO -- 1. fix zookeeper client 2. the trait bound `PersistentVolumeClaimSpec: JsonSchema` is not satisfied 3. pull a container image