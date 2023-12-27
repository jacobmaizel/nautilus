#!/bin/sh

# until diesel migration run --locked-schema; do
#   echo "Migrations failed, retrying in 5 seconds..."
#   sleep 5
# done

RUSTFLAGS="-Z threads=8" cargo +nightly watch -x run