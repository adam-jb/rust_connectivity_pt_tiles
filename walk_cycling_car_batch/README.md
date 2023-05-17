
To run:
```
cargo run --release --bin walk_cycling_car_batch
```

Example query:
```
wget -O- --post-data='{"start_nodes_user_input": [1, 2, 3, 4, 5], "init_travel_times_user_input": [16, 10, 10, 23, 99], "mode": "walk", "destination_nodes": [1,2,3,4,55,6,7,8,9,10], "trip_start_seconds": 28800}' \
  --header='Content-Type:application/json' \
  'http://0.0.0.0:7329/floodfill_endpoint/'
```

