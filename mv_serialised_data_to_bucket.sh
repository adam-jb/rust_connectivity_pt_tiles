#!/bin/bash

FILES="/home/jupyter/rust_connectivity/serialised_data/*"
for f in $FILES
do
  echo "Moving $f file..."
  gsutil cp $f gs://april-2023-hack-rust-files/
done

