import express from 'express';
import http from 'http';
import { Server } from 'socket.io';
import cors from 'cors';
import dotenv from 'dotenv';
import csrf from 'csurf';
import cookieParser from 'cookie-parser';
import authRoute from './routes/authRoute.js';
import fileRoutes from './routes/fileRoute.js';
dotenv.config();

const app = express();
const server = http.createServer(app);
const io = new Server(server, {
  cors: {
    origin: "http://localhost:8080",
    methods: ["GET", "POST"],
    credentials: true
  }
});

// Middleware
app.use(cors({
  origin: 'http://localhost:8080',
  credentials: true
}));
app.use(express.json());
app.use(cookieParser());
app.use(csrf({ 
  cookie: {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax'
  }
}));

// CSRF error handler
app.use((err, req, res, next) => {
  if (err.code === 'EBADCSRFTOKEN') {
    res.status(403).json({
      error: 'Invalid CSRF token'
    });
  } else {
    next(err);
  }
});

// Routes
app.use('/api/auth', authRoute);
app.use('/api/files', fileRoutes);

// Socket.io connection
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

const PORT = process.env.PORT || 5000;
server.listen(PORT, () => console.log(`Server running on port ${PORT}`));