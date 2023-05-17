

Example payload for service_change_api (Adam 17th May)
```
wget -O- --post-data='{"start_nodes": [4380647, 4183046, 5420336], "init_travel_times": [16, 10, 10], "trip_start_seconds": 28800, "graph_walk_additions": [], "graph_routes_additions": [], "graph_walk_updates_keys": [], "graph_walk_updates_additions": [], "year": 2022, "new_build_additions": [], "target_destinations": []}' \
  --header='Content-Type:application/json' \
  '0.0.0.0:7328/floodfill_pt/'
```




