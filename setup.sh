#! /bin/bash

generate_random_token() {
    openssl rand -hex 32
}

ENV_FILE=".env"
ENV_PATH="$(pwd)/$ENV_FILE"

if [ ! -f "$ENV_PATH" ]; then
    echo "Environment file does not exist"
    exit 1
fi

echo "Generating random tokens"
sed -i "s/TURN_USERNAME=.*/TURN_USERNAME=$(generate_random_token)/" "$ENV_PATH"
sed -i "s/TURN_PASSWORD=.*/TURN_PASSWORD=$(generate_random_token)/" "$ENV_PATH"
sed -i "s/JWT_SECRET=.*/JWT_SECRET=$(generate_random_token)/" "$ENV_PATH"
sed -i "s/SESSION_SECRET=.*/SESSION_SECRET=$(generate_random_token)/" "$ENV_PATH"
