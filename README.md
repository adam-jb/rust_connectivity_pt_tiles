# Notes on running

Run `./download_input.sh` once to download input data. Uncomment serialise_files() in main.rs to run

Run `./download_input_mac.sh` if on a mac

The current version hosts an API, which accepts start node IDs and initial travel times. It requires about 3gb of RAM and loads in 3s on our GCE instance.


# On querying the API

Check it's listening:
```
curl http://127.0.0.1:7328/
```

Run PT algorithm on 5 start nodes: 
```
wget -O- --post-data='{"start_nodes_user_input":[3556923, 3556924, 3556925,3556926,3556927],"init_travel_times_user_input":[4,5,6,3,434],"trip_start_seconds":80000,"p1_additions":[],"p2_additions":[]}' \
  --header='Content-Type:application/json' \
  'http://127.0.0.1:7328/floodfill_pt/'
```
