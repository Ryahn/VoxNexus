#! /bin/bash

generate_random_token() {
    openssl rand -hex 32
}

ENV_FILE=".env"
ENV_PATH="$(pwd)/$ENV_FILE"

TURN_CONFIG_FILE="turnserver.conf"
TURN_CONFIG_PATH="$(pwd)/$TURN_CONFIG_FILE"

if [ ! -f "$ENV_PATH" ]; then
    echo "Environment file does not exist"
    exit 1
fi

TURN_USERNAME=$(generate_random_token)
TURN_PASSWORD=$(generate_random_token)

echo "Generating random tokens"
sed -i "s/TURN_USERNAME=.*/TURN_USERNAME=$TURN_USERNAME/" "$ENV_PATH"
sed -i "s/TURN_PASSWORD=.*/TURN_PASSWORD=$TURN_PASSWORD/" "$ENV_PATH"
sed -i "s/JWT_SECRET=.*/JWT_SECRET=$(generate_random_token)/" "$ENV_PATH"
sed -i "s/SESSION_SECRET=.*/SESSION_SECRET=$(generate_random_token)/" "$ENV_PATH"
sed -i "s/ZINC_FIRST_ADMIN_USER=.*/ZINC_FIRST_ADMIN_USER=$(generate_random_token)/" "$ENV_PATH"
sed -i "s/ZINC_FIRST_ADMIN_PASSWORD=.*/ZINC_FIRST_ADMIN_PASSWORD=$(generate_random_token)/" "$ENV_PATH"
sed -i "0,/^user=.*/s//user=$TURN_USERNAME:$TURN_PASSWORD/" "$TURN_CONFIG_PATH"