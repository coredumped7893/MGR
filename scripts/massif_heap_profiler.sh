cargo build --release
valgrind --tool=massif target/release/mgr_map_visualiser

# Export heaptrack data to a massif file
heaptrack_print --file /home/.../heaptrack.....883446.zst -M heaptrack.....883446.massif
