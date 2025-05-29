#!/bin/bash
set -e

# Start cron service
service cron start

# Change to app directory
cd /app

# Execute the command passed to docker run
exec "$@" 