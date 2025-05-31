import express from 'express';
import http from 'http';
import { Server } from 'socket.io';
import cors from 'cors';
import dotenv from 'dotenv';
import csrf from '@dr.pogodin/csurf';
import cookieParser from 'cookie-parser';
import authRoute from './routes/authRoute.js';
import fileRoutes from './routes/fileRoute.js';
import { apiLimiter, authLimiter, wsRateLimiter, cleanupRateLimiters } from './src/middleware/rateLimiter.js';
import emailTemplateManager from './libs/templates/email/EmailTemplateManager.js';
import { database } from './libs/database/db.js';
dotenv.config();

async function start() {
  // Initialize database connection and load models first
  try {
    console.log('Connecting to database...');
    await database.connect();
    console.log('Loading database models...');
    await database.loadModels();
    console.log('Database initialization complete');
  } catch (error) {
    console.error('Failed to initialize database:', error);
    process.exit(1);
  }

  const app = express();
  const server = http.createServer(app);

  // Trust proxy configuration
  app.set('trust proxy', 1);

  const io = new Server(server, {
    cors: {
      origin: process.env.NODE_ENV === 'production' ? 'https://voxnexus.test' : 'http://localhost:8080',
      methods: ["GET", "POST"],
      credentials: true
    }
  });

  // Basic middleware
  app.use(cors({
    origin: process.env.NODE_ENV === 'production' 
      ? ['https://voxnexus.test', 'http://voxnexus.test'] 
      : ['http://localhost:8080', 'https://voxnexus.test', 'http://voxnexus.test'],
    credentials: true
  }));
  app.use(express.json());
  app.use(cookieParser());

  // Global request logger
  app.use((req, res, next) => {
    console.log(`[REQUEST] ${req.method} ${req.originalUrl}`);
    console.log('Headers:', req.headers);
    if (req.method !== 'GET') {
      console.log('Body:', req.body);
    }
    next();
  });

  // CSRF middleware configuration - TEMPORARILY DISABLED
  /*
  app.use(csrf({
    cookie: {
      httpOnly: false, // Allow JavaScript to read the cookie
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      key: '_csrf',
      path: '/'
    }
  }));
  */

  await emailTemplateManager.loadTemplates();
  console.log('Email templates loaded successfully');


  // Apply rate limiters after CSRF
  app.use(apiLimiter); // Apply to all routes
  app.use('/api/auth', authLimiter); // Stricter limits for auth routes

  // CSRF error handler - TEMPORARILY DISABLED
  /*
  app.use((err, req, res, next) => {
    if (err.code === 'EBADCSRFTOKEN') {
      console.error('CSRF Token Error:', {
        error: err.message,
        code: err.code,
        headers: {
          'x-csrf-token': req.headers['x-csrf-token'],
          'cookie': req.headers.cookie
        }
      });
      res.status(403).json({
        error: 'Invalid CSRF token'
      });
    } else {
      next(err);
    }
  });
  */

  // Routes
  app.use('/api/auth', authRoute);
  app.use('/api/files', fileRoutes);

  // Socket.io connection with rate limiting
  io.use((socket, next) => {
    wsRateLimiter(socket.request, socket.request.res, next);
  });

  io.on('connection', (socket) => {
    console.log('New client connected:', socket.id);

    socket.on('joinRoom', (channelId) => {
      socket.join(channelId);
      console.log(`User joined room: ${channelId}`);
    });

    socket.on('leaveRoom', (channelId) => {
      socket.leave(channelId);
      console.log(`User left room: ${channelId}`);
    });

    socket.on('sendMessage', (data) => {
      io.to(data.channelId).emit('message', data);
    });

    socket.on('disconnect', () => {
      console.log('Client disconnected:', socket.id);
    });
  });

  // Global error handler (at the end, after all routes and middleware)
  app.use((err, req, res, next) => {
    console.error('[ERROR]', err.stack || err);
    res.status(err.status || 500).json({ error: err.message || 'Internal server error' });
  });

  const PORT = process.env.PORT || 5000;
  server.listen(PORT, () => console.log(`Server running on port ${PORT}`));

  // Cleanup on server shutdown
  process.on('SIGTERM', () => {
    console.log('SIGTERM received. Cleaning up...');
    cleanupRateLimiters();
    server.close(() => {
      console.log('Server closed');
      process.exit(0);
    });
  });
}
start();