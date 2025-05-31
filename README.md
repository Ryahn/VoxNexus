# VoxNexus Discord Clone

[![Discord](https://i.redd.it/e7hb6p5nx3g11.png)](https://discord.com/)

A full-stack Discord clone built with Nuxt 3, Vue 3, TypeScript, MongoDB, and real-time features.

---

## Features
- Real-time chat (Socket.io)
- User authentication (JWT/session)
- Server & channel management
- Direct messages
- File uploads (local, S3, SFTP, FTP)
- Modern UI with Tailwind CSS
- MongoDB for persistent storage
- Production-ready Docker setup

---

## Local Development

```bash
# Install dependencies (from project root)
pnpm install

# Start the app with hot reload (from project root)
pnpm dev --filter ./server
# Or, from server directory:
cd server && pnpm dev
```

The app will be available at http://localhost:3000 by default.

---

## Production Build

```bash
# Build the app (from project root)
pnpm build --filter ./server
# Or, from server directory:
cd server && pnpm build

# Start the production server
pnpm start --filter ./server
# Or, from server directory:
cd server && pnpm start
```

---

## Docker & Docker Compose

### Build and Run (Development)

```bash
# Start all services with hot reload
docker-compose up --build
# The app will be available at http://localhost:3555
```

### Build and Run (Production)

```bash
# Set NODE_ENV=production to use the production build
NODE_ENV=production docker-compose up --build
# The app will be available at http://localhost:3555
```

- The Nuxt 3 server serves both the frontend and backend.
- All configuration is handled via environment variables (see `.env-example`).
- MongoDB, Redis, TURN, and ZincSearch are included as services.

---

## Environment Variables

Copy `.env-example` to `.env` and fill in the required values:

```bash
cp .env-example .env
```

- `NODE_ENV` - Set to `development` or `production`
- `MONGODB_URI` - MongoDB connection string
- `JWT_SECRET`, `SESSION_SECRET` - Secrets for authentication
- `TURN_USERNAME`, `TURN_PASSWORD` - For WebRTC/voice
- ...and more (see `.env-example`)

---

## Project Structure

```
voxnexus/
├── server/          # Nuxt 3 full-stack app (frontend + backend)
├── docker-compose.yml
├── Dockerfile.server
├── .env-example
└── ...
```

---

## Useful Commands

- `pnpm dev` - Start in development mode (hot reload)
- `pnpm build` - Build for production
- `pnpm start` - Start production server
- `docker-compose up --build` - Start all services with Docker

---

For more details, see the [Nuxt 3 documentation](https://nuxt.com).
