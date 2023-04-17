# Getting started

```
curl https://sh.rustup.rs -sSf | sh && \
source "$HOME/.cargo/env"
```

1. Run `./download_input.sh` once to download input data

2. Flip the `if false` part of `serialise_files` and `create_graph_walk_len` in `src/main.rs` to `true` so the files are serialised

3. Run with`cargo run --release` to serialise all files. End the process once the API is listening

4. Flip the `if false` part of `serialise_files` and `create_graph_walk_len` in `src/main.rs` to `false` to run without serialising any files

Then build the docker container, or run with `cargo run --release`


# On querying the API

Check it's listening:
```
curl http://0.0.0.0:7328/
```
    
    
Run PT algorithm on 3 start nodes: 
```
wget -O- --post-data='{"start_nodes_user_input": [9380647, 9183046, 2420336], "init_travel_times_user_input": [16, 10, 10], "trip_start_seconds": 28800}' \
  --header='Content-Type:application/json' \
  'http://0.0.0.0:7328/floodfill_pt/'
```


# Deploying with Docker

To make and run docker image.
```
docker build --progress=plain -t rust_connectivity:latest .
docker run -p 0.0.0.0:7328:7328 rust_connectivity:latest
```

To push build image to dockerhub
```
docker tag connectivity_rust:latest adambricknell/connectivity_rust
docker push adambricknell/connectivity_rust
```

To deploy with Cloud Run do the below, then use Cloud Run UI in GCP to deploy
```
docker build --progress=plain -t rust_connectivity:latest . && \
docker tag rust_connectivity:latest gcr.io/dft-dst-prt-connectivitymetric/adambricknell/connectivity_rust:latest && \
docker push gcr.io/dft-dst-prt-connectivitymetric/adambricknell/connectivity_rust:latest
```



