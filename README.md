
Code for all rust-side APIs to calculate Connectivity scores and origin-destination (OD) pairs. This is to be queried from a front end or python batch processing.


# Contents of services in this repo

Service Change API

Planning app public transport API

Public Transport batch

Walk cycling car batch




# Getting started

Install rust: 
```
curl https://sh.rustup.rs -sSf | sh && \
source "$HOME/.cargo/env"
```

1. Run `python3 download_input_json.py` once to download input data

2. Build all services `cargo run --build`

3. Run `./target/release/do_serialisation`. Then all data will be ready for each service

4. (Optional: only needed if running Planning app public transport API) Run `./target/release/find_nodes_near_each_other`. To create dataset of which nodes are near each other. Used by planning_app_public_transport_api; can skip this if using other apps. Takes 128gb RAM and ~1 day with 16cores




# Service Change API

### On querying Service Change API

Example payloads to cloud run and local host
```
wget -O- --post-data='{"start_nodes": [4380647, 4183046, 5420336], "init_travel_times": [16, 10, 10], "trip_start_seconds": 28800, "graph_walk_additions": [], "graph_routes_additions": [], "graph_walk_updates_keys": [], "graph_walk_updates_additions": [], "year": 2022, "new_build_additions": [], "target_destinations": [], "nodes_to_remove_routes_from": [1,2,3,4,5,6]}' \
  --header='Content-Type:application/json' \
  'https://service-change-api-y3gbqriuaq-nw.a.run.app/floodfill_pt/'
  
wget -O- --post-data='{"start_nodes": [4380647, 4183046, 5420336], "init_travel_times": [16, 10, 10], "trip_start_seconds": 28800, "graph_walk_additions": [], "graph_routes_additions": [], "graph_walk_updates_keys": [], "graph_walk_updates_additions": [], "year": 2022, "new_build_additions": [], "target_destinations": [], "nodes_to_remove_routes_from": [1,2,3,4,5,6]}' \
  --header='Content-Type:application/json' \
  'http://0.0.0.0:7328/floodfill_pt/'
```

And with some new routes added:
```
wget -O- --post-data='{"start_nodes": [8522168, 3938703, 9643896, 9082987, 3218374, 3432456, 5085784, 1940916, 2934474, 2956552], "init_travel_times": [19, 22, 9, 18, 2, 54, 15, 18, 51, 32], "trip_start_seconds": 21600, "graph_walk_additions": [[[1, 0], [1, 2170979]], [[1, 0], [1, 2170955]], [[1, 0], [1, 2169635]], [[0, 0], [1, 2170939]]], "graph_routes_additions": [[[12065676, 0], [25200, 600], [32400, 600]], [[12065677, 0], [25800, 600], [33000, 600]], [[12065678, 0], [26400, 600], [33600, 600]], [[0, 0]]], "graph_walk_updates_keys": [2170979, 2170955, 2169635, 2170939], "graph_walk_updates_additions": [[[0, 12065675]], [[0, 12065676]], [[0, 12065677]], [[0, 12065678]]], "year": 2022, "new_build_additions": [], "target_destinations": [], "nodes_to_remove_routes_from": [1,2,3,4,5,6]}' \
  --header='Content-Type:application/json' \
  'http://0.0.0.0:7328/floodfill_pt/'
```



### Deploying Service Change API with Docker

To make and run docker image.
```
docker build --file Dockerfile_service_change_api --progress=plain -t service_change_api:latest .
docker run -p 0.0.0.0:7328:7328 service_change_api:latest
```

To deploy with Cloud Run do the below, then use Cloud Run UI in GCP to deploy

```
docker build --file Dockerfile_service_change_api --progress=plain -t service_change_api:latest . && \
docker tag service_change_api:latest gcr.io/dft-dst-prt-connectivitymetric/adambricknell/service_change_api:latest && \
docker push gcr.io/dft-dst-prt-connectivitymetric/adambricknell/service_change_api:latest
```

Cloud Run settings to choose:
```
europe-west-2
CPU is only allocated during request processing
Minimum number of instances = 0
Maximum number of instances = 300
Internal only
Allow unauthenticated invocations

# Container
Container port: 7328
Container command and arguments: leave blank
Memory: 8GiB
vCPUs: 8
Request timeout: 600 seconds
Maximum requests per instance: 1

# Networking
VPC: connectivity1
Only route requests to private IPs through the VPC connector: tick this
```




# Planning app public transport api

### On querying API

Check it's listening:
```
curl http://0.0.0.0:7328/
```

To specify the number of top scoring node clusters returned, change TOP_CLUSTERS_COUNT in src/shared.rs

The payload send to the API consists of 3 lists; each should be of length 1; subsequent values will be ignored

Run PT algorithm on one start node and save the output: 
```
wget -O- --post-data='{"start_nodes_user_input": [9380647], "init_travel_times_user_input": [16], "trip_start_seconds": 28800}' \
  --header='Content-Type:application/json' \
  'http://0.0.0.0:7328/floodfill_pt/' > example_returned_payload_May1st_API.txt
  
# Returns larger payload: 4Bb
wget -O- --post-data='{"start_nodes_user_input": [2780647], "init_travel_times_user_input": [16], "trip_start_seconds": 28800}' \
  --header='Content-Type:application/json' \
  'http://0.0.0.0:7328/floodfill_pt/' > example_returned_payload_May1st_API.txt
  
# Start point close to Bethnal Green tube station. Returns 20Mb payload
wget -O- --post-data='{"start_nodes_user_input": [5850631], "init_travel_times_user_input": [14], "trip_start_seconds": 28800}' \
  --header='Content-Type:application/json' \
  'http://0.0.0.0:7328/floodfill_pt/' > example_returned_payload_May1st_API.txt
```

### Querying api

Check it's listening:
```
curl http://0.0.0.0:7328/
```

To specify the number of top scoring node clusters returned, change TOP_CLUSTERS_COUNT in src/shared.rs

The payload send to the API consists of 3 lists; each should be of length 1; subsequent values will be ignored

Run PT algorithm on one start node and save the output: 
```
wget -O- --post-data='{"start_nodes_user_input": [9380647], "init_travel_times_user_input": [16], "trip_start_seconds": 28800}' \
  --header='Content-Type:application/json' \
  'http://0.0.0.0:7328/floodfill_pt/' > example_returned_payload_May1st_API.txt
  
# Returns larger payload: 4Bb
wget -O- --post-data='{"start_nodes_user_input": [2780647], "init_travel_times_user_input": [16], "trip_start_seconds": 28800}' \
  --header='Content-Type:application/json' \
  'http://0.0.0.0:7328/floodfill_pt/' > example_returned_payload_May1st_API.txt
  
# Start point close to Bethnal Green tube station. Returns 20Mb payload
wget -O- --post-data='{"start_nodes_user_input": [5850631], "init_travel_times_user_input": [14], "trip_start_seconds": 28800}' \
  --header='Content-Type:application/json' \
  'http://0.0.0.0:7328/floodfill_pt/' > example_returned_payload_May1st_API.txt
```





# Public Transport batch

### Docker and Cloud Run

To deploy with Cloud Run do the below, then use Cloud Run UI in GCP to deploy

```
docker build --file Dockerfile_public_transport_batch --progress=plain -t public_transport_batch:latest . && \
docker tag public_transport_batch:latest gcr.io/dft-dst-prt-connectivitymetric/connectivity/public_transport_batch:latest && \
docker push gcr.io/dft-dst-prt-connectivitymetric/connectivity/public_transport_batch:latest

docker run -p 0.0.0.0:7328:7328 public_transport_batch:latest

```

Cloud Run settings to choose are slightly different to Service Change API: more RAM as storing OD pairs, and fewer max instances as assume these will mostly run one at a time
```
europe-west-2
CPU is only allocated during request processing
Minimum number of instances = 0
Maximum number of instances = 10
Internal only
Allow unauthenticated invocations

# Container
Container port: 7328
Container command and arguments: leave blank
Memory: 16GiB
vCPUs: 8
Request timeout: 600 seconds
Maximum requests per instance: 1

# Networking
VPC: connectivity1
Only route requests to private IPs through the VPC connector: tick this
```

Example request:
```
wget -O- --post-data='{"start_nodes": [9380647, 9183046, 2420336], "init_travel_times": [16, 10, 10], "trip_start_seconds": 28800, "destination_nodes": [1,2,3,4]}' \
  --header='Content-Type:application/json' \
  'https://public-transport-batch-y3gbqriuaq-nw.a.run.app/floodfill_pt/'
```



# Walk cycling car batch

To run:
```
cargo run --release --bin walk_cycling_car_batch
```

Example query which returns number of destinations reached, by subpurpose and size of destination (small, medium and large) both 600 and 1200 seconds into the process, and also looking for OD pairs where destination nodes are reached from the start nodes 
```
wget -O- --post-data='{"start_nodes_user_input": [1, 2, 3, 4, 5], "init_travel_times_user_input": [16, 10, 10, 23, 99], "mode": "walk", "destination_nodes": [1,2,3,4,55,6,7,8,9,10], "trip_start_seconds": 28800, "builds_to_remove": [], "time_or_distance": "time",  "track_pt_nodes_reached": 0, "seconds_reclaimed_when_pt_stop_reached": 0, "target_node": 0, "count_destinations_at_intervals": 1, "original_time_intervals_to_store_destination_counts": [600, 1200]}' \
  --header='Content-Type:application/json' \
  'http://0.0.0.0:7328/floodfill_endpoint/'
```

Example query using the distance driving graph: a "time_or_distance": "distance" and mode of "car" tells the API to use the distance driving graph:
```
wget -O- --post-data='{"start_nodes_user_input": [1, 2, 3, 4, 5], "init_travel_times_user_input": [16, 10, 10, 23, 99], "mode": "car", "destination_nodes": [1,2,3,4,55,6,7,8,9,10], "trip_start_seconds": 1, "builds_to_remove": [], "time_or_distance": "distance", "track_pt_nodes_reached":0, "seconds_reclaimed_when_pt_stop_reached": 0, "target_node": 10, "count_destinations_at_intervals": 0, "original_time_intervals_to_store_destination_counts": []}' \
  --header='Content-Type:application/json' \
  'http://0.0.0.0:7328/floodfill_endpoint/'    
```

With a target node. An example of the payload used to find the optimal routes:
```
wget -O- --post-data='{"start_nodes_user_input": [1], "init_travel_times_user_input": [0], "mode": "car", "destination_nodes": [], "trip_start_seconds": 1, "builds_to_remove": [], "time_or_distance": "distance", "track_pt_nodes_reached":1, "seconds_reclaimed_when_pt_stop_reached": 3, "target_node": 5, "count_destinations_at_intervals": 0, "original_time_intervals_to_store_destination_counts": []}' \
  --header='Content-Type:application/json' \
  'http://0.0.0.0:7328/floodfill_endpoint/'
```



### Docker and Cloud Run

To build the docker image and push to container registry

```
docker build --file Dockerfile_walk_cycling_car_batch --progress=plain -t walk_cycling_car_batch:latest . && \
docker tag walk_cycling_car_batch:latest gcr.io/dft-dst-prt-connectivitymetric/connectivity/walk_cycling_car_batch:latest && \
docker push gcr.io/dft-dst-prt-connectivitymetric/connectivity/walk_cycling_car_batch:latest
```

Cloud Run settings are the same as Public Transport batch

Example query to Cloud Run:
```
wget -O- --post-data='{"start_nodes_user_input": [1, 2, 3, 4, 5], "init_travel_times_user_input": [16, 10, 10, 23, 99], "mode": "walk", "destination_nodes": [1,2,3,4,55,6,7,8,9,10], "trip_start_seconds": 28800, "builds_to_remove": []}' \
  --header='Content-Type:application/json' \
  'https://walk-cycling-car-batch-y3gbqriuaq-nw.a.run.app/floodfill_endpoint/'
```
The 'builds_to_remove' parameter is a vector of smaller vectors (each of length 2). Each smaller vector specifies: 0 is index_of_nearest_node, 1 is subpurpose_ix. This is to see the effect on Connectivity of removing destinations, such as a large employers, from the graph. The dummy payload below includes scores being removed for specific subpurposes for two nodes. 

```
wget -O- --post-data='{"start_nodes_user_input": [1, 2, 3, 4, 5], "init_travel_times_user_input": [16, 10, 10, 23, 99], "mode": "walk", "destination_nodes": [1,2,3,4,55,6,7,8,9,10], "trip_start_seconds": 28800, "builds_to_remove": [[10,3], [20, 6]]}' \
  --header='Content-Type:application/json' \
  'https://walk-cycling-car-batch-y3gbqriuaq-nw.a.run.app/floodfill_endpoint/'
```


# Read tests cloud run

To test different file reading strategies in Cloud Run. Inc splitting vectors into smaller files and (1) reading them in parallel, (2) appending them. Sees if this is faster than 'normal' parallel reading of files. 
```
docker build --file Dockerfile_read_tests_cloud_run --progress=plain -t read_tests_cloud_run:latest . && \
docker tag read_tests_cloud_run:latest gcr.io/dft-dst-prt-connectivitymetric/connectivity/read_tests_cloud_run:latest && \
docker push gcr.io/dft-dst-prt-connectivitymetric/connectivity/read_tests_cloud_run:latest
```

Your Cloud Run instance will need 16GiB of RAM to run: the parallel method of reading files uses more than the 'usual' amount of RAM

Then hit the below with a GET request to run tests. View the results in the Cloud Run logs. Make sure you wait a few minutes between tests: you want to compare the load speeds of cold boots.
```
curl https://read-tests-cloud-run-y3gbqriuaq-nw.a.run.app/run_tests/
```



