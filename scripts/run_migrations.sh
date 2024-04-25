#!/bin/sh

until diesel migration run --database-url $1 --locked-schema; do
  echo "Migrations failed for $1, retrying in 5 seconds..."
  sleep 5
done
