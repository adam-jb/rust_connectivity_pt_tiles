
# python3 /home/jupyter/rust_connectivity/download_serialised_files.py

# For use by internal devs: downloads files while keeping them internal only


import os
from google.cloud import storage

os.chdir('/home/jupyter/rust_connectivity')

# Set up GCS client and bucket name
client = storage.Client()
bucket_name = 'april-2023-hack-rust-files'
bucket = client.bucket(bucket_name)

# Create local directory if it doesn't exist
local_dir = 'serialised_data'
os.makedirs(local_dir, exist_ok=True)

# List of files to download from bucket
files = ['graph_walk_len_2022.bin', 'padded_node_values_6am_2022.bin',
         'travel_time_relationships_16.bin', 'node_values_padding_row_count_6am_2022.bin',
         'rust_lookup_long_lat_list.bin', 'travel_time_relationships_19.bin',
         'nodes_to_neighbouring_nodes.bin', 'sparse_node_values_6am_2022_2d.bin',
         'travel_time_relationships_7.bin', 'p1_main_nodes_vector_6am_2022.bin',
         'subpurpose_purpose_lookup.bin', 'p2_main_nodes_vector_6am_2022.bin',
         'travel_time_relationships_10.bin', 'rust_lookup_long_lat_pt_class_list.bin']

# Download each file from the bucket and save to local directory
for file in files:
    blob = bucket.blob(file)
    blob.download_to_filename(os.path.join(local_dir, file))