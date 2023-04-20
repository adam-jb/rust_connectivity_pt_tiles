#!/bin/bash

set -e
mkdir -p serialised_data

cd serialised_data
for x in graph_walk_len_2022.bin padded_node_values_6am_2022.bin \
    travel_time_relationships_16.bin node_values_padding_row_count_6am_2022.bin \
    rust_lookup_long_lat_list.bin travel_time_relationships_19.bin \
    nodes_to_neighbouring_nodes.bin sparse_node_values_6am_2022_2d.bin \
    travel_time_relationships_7.bin p1_main_nodes_vector_6am_2022.bin \
    subpurpose_purpose_lookup.bin p2_main_nodes_vector_6am_2022.bin \
    travel_time_relationships_10.bin;

do
    wget https://storage.googleapis.com/april-2023-hack-rust-files/$x
done
