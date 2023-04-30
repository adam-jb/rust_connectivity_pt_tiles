
# Upload serialised files to GCS
# python3 upload_serialised_files.py

from google.cloud import storage
import os

# Set your Google Cloud Storage bucket name
BUCKET_NAME = "tiles-api-serialised-files"

# Set the path to your serialised_data directory
LOCAL_PATH = "serialised_data/"

# Authenticate with Google Cloud Storage
storage_client = storage.Client()

# Get a reference to the bucket
bucket = storage_client.bucket(BUCKET_NAME)

# Loop through each file in the directory and upload it to the bucket
for file_name in os.listdir(LOCAL_PATH):
    # Create a blob object for the file
    blob = bucket.blob(file_name)

    # Upload the file to the bucket
    blob.upload_from_filename(os.path.join(LOCAL_PATH, file_name))

    # Print a message to indicate that the file was uploaded
    print(f"Uploaded {file_name} to {BUCKET_NAME}")