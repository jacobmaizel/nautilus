#!/bin/sh

until diesel migration run --database-url postgresql://postgres:postgres@localhost:5432/postgres --locked-schema; do
  echo "Migrations failed, retrying in 5 seconds..."
  sleep 5
done
