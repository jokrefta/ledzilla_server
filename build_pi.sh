#!/bin/bash
set -e
cd "$(dirname "$0")"

TARFILE="pi_files.tar"

cargo build --release  --features led --target arm-unknown-linux-gnueabihf


mkdir -p pi_files/

cp target/arm-unknown-linux-gnueabihf/release/ledzilla_server pi_files/
cp example_server_config.toml pi_files/
cp -r web_content/ pi_files/

sed -i -E "s/(use_sim *= *)true/\1false/" pi_files/example_server_config.toml

tar -cvf "$TARFILE" pi_files/* && echo "Created $TARFILE"
rm -r pi_files/
