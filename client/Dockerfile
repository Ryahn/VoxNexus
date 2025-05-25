# Build stage
FROM node:18-alpine as build-stage

# Install build dependencies
RUN apk add --no-cache \
    autoconf \
    automake \
    build-base \
    gcc \
    g++ \
    make \
    python3 \
    zlib-dev \
    libpng-dev \
    nasm

WORKDIR /app

# Copy package files
COPY package.web.json ./package.json
COPY yarn.lock ./

# Install dependencies
RUN yarn install

# Copy source files
COPY . .

# Build the app
RUN yarn build

# Production stage
FROM nginx:stable-alpine as production-stage

# Copy built files from build stage
COPY --from=build-stage /app/dist /usr/share/nginx/html

# Copy nginx configuration
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Expose port
EXPOSE 80

# Start nginx
CMD ["nginx", "-g", "daemon off;"] 