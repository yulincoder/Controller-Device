cargo build --release
mv target/release/untitled5 target/release/connection-service.bin
scp ./target/release/connection-service.bin root@39.105.63.97:/root/home/csod/heartbeat-server/
