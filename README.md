# VoxNexus - Discord Clone chat app

A real-time chat application built with a client-server architecture, inspired by Discord.

## Prerequisites

- Node.js (v14 or higher)
- npm (Node Package Manager)

## Project Structure

```
voxnexus/
├── client/         # Frontend application
├── server/         # Backend server
├── package.json    # Root package configuration
└── start.sh        # Setup and start script
```

## Setup and Installation

### Option 1: Using the start script (Recommended)

1. Make the start script executable:
   ```bash
   chmod +x start.sh
   ```

2. Run the start script:
   ```bash
   ./start.sh
   ```
   This will:
   - Check for npm installation
   - Install all dependencies (root, client, and server)
   - Start both client and server applications

### Option 2: Manual Setup

1. Install root dependencies:
   ```bash
   npm install
   ```

2. Install client dependencies:
   ```bash
   cd client && npm install && cd ..
   ```

3. Install server dependencies:
   ```bash
   cd server && npm install && cd ..
   ```

## Running the Application

### Using npm scripts

Start both client and server concurrently:
```bash
npm start
```

Or start them separately:
```bash
# Start server only
npm run start:server

# Start client only
npm run start:client
```

## Development

- The client application will be available at `http://localhost:8080`
- The server will run on `http://localhost:5000`

## Docker Support

The project includes Docker configuration for containerized deployment:

- `Dockerfile` - Container configuration
- `docker-compose.yml` - Multi-container setup

To run with Docker:
```bash
docker-compose up -d
```

## License

This project is open source and available under the MIT License.
