# python3 /home/jupyter/rust_connectivity/download_serialised_files.py

# For use by internal devs: downloads files while keeping them internal only

import os
from google.cloud import storage

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
    'rust_nodes_long_lat.json'
]

for file in data_files:
    blob = bucket.blob(file)
    blob.download_to_filename(f"data/{file}")

for year in range(2022, 2023):
    for file in [
        f"p1_main_nodes_updated_6am_{year}.json",
        f"p2_main_nodes_updated_6am_{year}.json",
        f"padded_node_values_6am_{year}.json",
        f"sparse_node_values_6am_{year}_2d.json",
        f"node_values_padding_row_count_6am_{year}.json",
        f"routes_info_{year}.json",
    ]:
        blob = bucket.blob(file)
        blob.download_to_filename(f"data/{file}")
