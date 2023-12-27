#!/bin/bash

echo "----------------- START Nautilus Tests -----------------"
NAUTILUS_ENVIRONMENT=test docker compose --project-name nautilus-testing -f docker-compose-test.yml run server cargo build & cargo nextest run
NAUTILUS_ENVIRONMENT=test docker compose --project-name nautilus-testing -f docker-compose-test.yml down --volumes
echo "----------------- DONE Nautilus Tests -----------------"
