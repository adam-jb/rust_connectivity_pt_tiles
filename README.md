


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

1. Run `./download_input_json.py` once to download input data

2. Build all services `cargo run --build`

3. Run `./target/release/do_serialisation`. Then all data will be ready for each service

Then build the docker container, or run with `cargo run --release`



# Service Change API

### On querying Service Change API

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




### Deploying Service Change API with Docker

To make and run docker image.
```
docker build --file Dockerfile_service_change_api --progress=plain -t service_change_api:latest .
docker run -p 0.0.0.0:7328:7328 service_change_api:latest
```

To deploy with Cloud Run do the below, then use Cloud Run UI in GCP to deploy

```
docker build --file Dockerfile_service_change_api --progress=plain -t service_change_api:latest .
docker tag service_change_api:latest gcr.io/dft-dst-prt-connectivitymetric/adambricknell/service_change_api:latest && \
docker push gcr.io/dft-dst-prt-connectivitymetric/adambricknell/service_change_api:latest
```

Cloud Run settings to choose:
```


```




# Planning app public transport

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



# Significant points in history

Latest commit before trying to amalgamate the APIs: 26262b764a9b06025c411cf41c8050d3f91fa1a2


