#!/bin/bash

# Define the Docker Compose file
DOCKER_COMPOSE_FILE="$(readlink -f $(dirname $0))/../docker-compose.yaml"

# Check if any of the services defined in the Docker Compose file are running
if docker compose -f $DOCKER_COMPOSE_FILE ps | grep -q 'Up'; then
    echo "Docker Compose services are already running."
else
    # Start the Docker Compose services
    docker compose -f $DOCKER_COMPOSE_FILE up --wait --wait-timeout 60
    echo "Docker Compose services have been started."
fi