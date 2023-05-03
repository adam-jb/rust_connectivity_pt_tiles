# Getting started

```
curl https://sh.rustup.rs -sSf | sh && \
source "$HOME/.cargo/env"
```

1. Run `./download_input_json.py` once to download input data

2. Flip the `if false` part of main() in `src/main.rs` to `true` so the files are serialised and all nodes within 120s are found for all start nodes

3. Run with`cargo run --release` to serialise all files and find nearby nodes. End the process once the API is listening

4. Flip the `if false` part of main() in `src/main.rs` to `false` to run without serialising any files

Then build the docker container, or run with `cargo run --release`


# On querying the API

Check it's listening:
```
curl http://0.0.0.0:7328/
```

The payload send to the API consists of 3 lists; each should be of length 1; subsequent values will be ignored

Run PT algorithm on one start node and save the output: 
```
wget -O- --post-data='{"start_nodes_user_input": [9380647], "init_travel_times_user_input": [16], "trip_start_seconds": 28800}' \
  --header='Content-Type:application/json' \
  'http://0.0.0.0:7328/floodfill_pt/' > example_returned_payload_May1st_API.txt
  
# Returns larger payload
wget -O- --post-data='{"start_nodes_user_input": [2780647], "init_travel_times_user_input": [16], "trip_start_seconds": 28800}' \
  --header='Content-Type:application/json' \
  'http://0.0.0.0:7328/floodfill_pt/' > example_returned_payload_May1st_API.txt
```




# Deploying with Docker

To make and run docker image.
```
docker build --progress=plain -t rust_connectivity_pt_tiles:latest .
docker run -p 0.0.0.0:7328:7328 rust_connectivity_pt_tiles:latest
```


# FOR DEV: Significant commits

Last one using hashmap and hashset instead of the vec sparse_node_values_contributed: e9ce405a5b4fb0f8eaeee2b460a5349fee9234bf
