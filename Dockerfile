# Build stage for client
FROM node:18-alpine as client-builder
WORKDIR /app/client
COPY client/package*.json ./
COPY client/yarn.lock ./
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
RUN yarn install
COPY client/ .
RUN yarn build

# Build stage for server
FROM node:18-alpine as server-builder
WORKDIR /app/server
COPY server/package*.json ./
RUN npm install
COPY server/ .

# Production stage
FROM node:18-alpine
WORKDIR /app

# Copy built client files
COPY --from=client-builder /app/client/dist ./client/dist

# Copy server files
COPY --from=server-builder /app/server ./server
COPY --from=server-builder /app/server/package*.json ./server/

# Install production dependencies
WORKDIR /app/server
RUN npm install --production

# Set environment variables
ENV NODE_ENV=production
ENV PORT=3000
ENV API_PORT=3001
ENV API_HOST=0.0.0.0
ENV MONGODB_URI=mongodb://mongo:27017/discord-clone
ENV BOT_API_ENABLED=true
ENV WEBRTC_SIGNALING_PORT=3002
ENV TURN_SERVER=turn:3478
ENV TURN_USERNAME=your_turn_username
ENV TURN_PASSWORD=your_turn_password
ENV WEBRTC_ICE_SERVERS=[{"urls":["stun:turn:3478","turn:turn:3478"],"username":"your_turn_username","credential":"your_turn_password"}]

# Expose ports
EXPOSE 3000
EXPOSE 3001
EXPOSE 3002
EXPOSE 3478
EXPOSE 5349

# Start the server
CMD ["node", "server.js"]
