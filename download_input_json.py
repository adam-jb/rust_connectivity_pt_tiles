# python3 download_input_json.py

# For use by internal devs: downloads files while keeping them internal only

import os
from google.cloud import storage

YEAR = 2022

client = storage.Client()

# Create directories if they don't exist
os.makedirs('data', exist_ok=True)
os.makedirs('serialised_data', exist_ok=True)

bucket_name = 'hack-bucket-8204707942'
bucket = client.bucket(bucket_name)

# Download files from Google Cloud Storage
data_files = [
    'travel_time_relationships_10.json',
    'travel_time_relationships_16.json',
    'travel_time_relationships_19.json',
    'travel_time_relationships_7.json',
    'subpurpose_purpose_lookup.json',
    'number_of_destination_categories.json',
    'subpurpose_to_purpose_integer.json',
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

for trip_start_hour in [7, 10, 16, 19]:
    for file in [
        f'graph_car_{trip_start_hour}.json',
        f'sparse_node_values_car_{trip_start_hour}.json',
        f'car_travel_time_relationships_{trip_start_hour}.json',
    ]:
        blob = bucket.blob(file)
        blob.download_to_filename(f"data/{file}")
        
for mode_simpler in ['bus', 'walk', 'cycling', 'car']:
    file = f'score_multipliers_{mode_simpler}.json'
    blob = bucket.blob(file)
    blob.download_to_filename(f"data/{file}")
    
        
