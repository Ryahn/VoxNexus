#!/bin/bash

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if npm is installed
if ! command_exists npm; then
    echo "Error: npm is not installed"
    exit 1
fi

# Check if yarn is installed
if ! command_exists yarn; then
    echo "Error: yarn is not installed"
    exit 1
fi

# Install dependencies if node_modules doesn't exist
if [ ! -d "node_modules" ]; then
    echo "Installing root dependencies..."
    yarn install
fi

if [ ! -d "client/node_modules" ]; then
    echo "Installing client dependencies..."
    cd client && yarn install && cd ..
fi

if [ ! -d "server/node_modules" ]; then
    echo "Installing server dependencies..."
    cd server && yarn install && cd ..
fi

docker compose down
docker compose build --no-cache
docker compose up -d
