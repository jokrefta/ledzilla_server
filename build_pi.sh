#!/bin/bash
set -e
SCRIPT_DIR="$(dirname $(realpath "$0"))"

cd "$SCRIPT_DIR"

TARFILE="pi_files.tar"

echo "Building server..."
cargo build --release  --features led --target arm-unknown-linux-gnueabihf

echo "Building typescript for web GUI..."
cd "$SCRIPT_DIR/web_gui"
npm run build


echo "Packaging..."
cd "$SCRIPT_DIR"
mkdir -p pi_files/

cp target/arm-unknown-linux-gnueabihf/release/ledzilla_server pi_files/
cp example_server_config.toml pi_files/
cp -r web_gui/web_content/ pi_files/

sed -i -E "s/(use_sim *= *)true/\1false/" pi_files/example_server_config.toml

tar -cvf "$TARFILE" pi_files/* && echo "Created $TARFILE"
rm -r pi_files/
