# python3 download_input_json.py

# For use by internal devs: downloads files while keeping them internal only

# Do not interrupt this script when running halfway! This will overwrite existing data with an incomplete text file

import os
from google.cloud import storage
import json

def read_json_file(filename, bucket_name='hack-bucket-8204707942'):
    client = storage.Client()
    project_id='dft-dst-prt-connectivitymetric'
    bucket = client.get_bucket(bucket_name)
    blob = bucket.blob(filename)
    pickle_in = blob.download_as_string()
    return json.loads(pickle_in)

YEAR = 2022

client = storage.Client()

# Create directories if they don't exist
os.makedirs('data', exist_ok=True)
os.makedirs('serialised_data', exist_ok=True)

bucket_name = 'hack-bucket-8204707942'
bucket = client.bucket(bucket_name)


# Download files from Google Cloud Storage
for trip_start_hour in [1, 7, 10, 16, 19]:
    for file in [
        f'graph_car_{trip_start_hour}.json',
        f'sparse_node_values_car_{trip_start_hour}.json',
        f'car_travel_time_relationships_{trip_start_hour}.json',
    ]:
        blob = bucket.blob(file)
        blob.download_to_filename(f"data/{file}")
    print(f'Downloaded car files for trip_start_hour {trip_start_hour}')

data_files = [
    'travel_time_relationships_10.json',
    'travel_time_relationships_16.json',
    'travel_time_relationships_19.json',
    'travel_time_relationships_7.json',
    'subpurpose_purpose_lookup.json',
    'number_of_destination_categories.json',
    'small_medium_large_subpurpose_destinations_walk.json',
    'small_medium_large_subpurpose_destinations_cycling.json',
    'small_medium_large_subpurpose_destinations_car.json',
]

for file in data_files:
    blob = bucket.blob(file)
    blob.download_to_filename(f"data/{file}")

for file in [
    f"graph_pt_walk_6am_{YEAR}.json",
    f"graph_pt_routes_6am_{YEAR}.json",
    f"sparse_node_values_6am_{YEAR}_2d.json",
    f"node_values_padding_row_count_6am_{YEAR}.json",
    f"routes_info_{YEAR}.json",
    f'rust_nodes_long_lat_{YEAR}.json',
    f'stop_rail_statuses_{YEAR}.json',
    f'car_nodes_is_closest_to_pt.json',
]:
    blob = bucket.blob(file)
    blob.download_to_filename(f"data/{file}")

for mode in ['cycling', 'walk']:
    for file in [
        f'graph_{mode}.json',
        f'sparse_node_values_{mode}.json',
        f'{mode}_travel_time_relationships_7.json',
    ]:
        blob = bucket.blob(file)
        blob.download_to_filename(f"data/{file}")


## These files we save directly to serialised_data, as Rust reads then directly without serialising them. 
## It does this because they are small files, and .dockerignore prevents files in 'data' folder
## from being used
for mode_simpler in ['car', 'bus', 'walk', 'cycling']:
    file = f'score_multipliers_{mode_simpler}.json'
    blob = bucket.blob(file)
    blob.download_to_filename(f"serialised_data/{file}")
    
    # checking all valueare above zero: ensures correct file has been moved across
    multipliers = read_json_file(file)
    for multiplier in multipliers:
        if multiplier < 0.00000001:
            print(f'multiplier: {multiplier} in {file}')
            raise ValueError("Multiplier is a zero: shouldnt be the case")


blob = bucket.blob('subpurpose_to_purpose_integer.json')
blob.download_to_filename("serialised_data/subpurpose_to_purpose_integer.json")
print('All files read in')
