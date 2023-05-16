

Initialise
```
cargo run --release --bin public_transport_batch
```


Example payload
```
wget -O- --post-data='{"start_nodes_user_input": [9380647, 9183046, 2420336], "init_travel_times_user_input": [16, 10, 10], "trip_start_seconds": 28800, "destination_nodes": [1,2,3,4]}' \
  --header='Content-Type:application/json' \
  'http://0.0.0.0:6000/floodfill_pt/'
```

