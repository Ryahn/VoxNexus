import { Server as IOServer, Socket } from 'socket.io';
import type { Nitro } from 'nitropack';
import jwt from 'jsonwebtoken';
import type {
  GroupDMMessagePayload,
  GroupDMMessageEditPayload,
  GroupDMMessageDeletePayload,
  GroupDMMessageReactionPayload,
  ChannelMessageEditPayload,
  ChannelMessageDeletePayload,
  ChannelMessageReactionPayload
} from '../../types/socket-events';

const JWT_SECRET = process.env.JWT_SECRET as string;

interface AuthPayload {
  id: string;
  username: string;
}

interface ChannelMessage {
  channelId: string;
  content: string;
  userId: string;
  username: string;
  createdAt: string;
}

interface DirectMessage {
  from: string;
  to: string;
  content: string;
  createdAt: string;
}

const userSocketMap = new Map<string, string>(); // userId -> socketId

export function setupSocketIO(nitroApp: Nitro) {
  if ((globalThis as any).io) return (globalThis as any).io;
  // @ts-ignore: Accessing internal node server for Socket.io
  const io = new IOServer((nitroApp as any).node?.server, {
    cors: { origin: '*', methods: ['GET', 'POST'] },
  });

  io.use((socket: Socket, next: (err?: Error) => void) => {
    const token = socket.handshake.auth?.token;
    if (!token) return next(new Error('Authentication required'));
    try {
      const payload = jwt.verify(token, JWT_SECRET) as AuthPayload;
      (socket as any).user = payload;
      next();
    } catch {
      next(new Error('Invalid token'));
    }
  });

  io.on('connection', (socket: Socket) => {
    const user = (socket as any).user as AuthPayload;
    userSocketMap.set(user.id, socket.id);
    io.emit('user:online', { userId: user.id });

    socket.on('disconnect', () => {
      userSocketMap.delete(user.id);
      io.emit('user:offline', { userId: user.id });
    });

    // Real-time channel message event
    socket.on('channel:message', (msg: ChannelMessage) => {
      io.to(msg.channelId).emit('channel:message', msg);
    });
    socket.on('channel:edit-message', (payload: ChannelMessageEditPayload) => {
      io.to(payload.channelId).emit('channel:edit-message', payload);
    });
    socket.on('channel:delete-message', (payload: ChannelMessageDeletePayload) => {
      io.to(payload.channelId).emit('channel:delete-message', payload);
    });
    socket.on('channel:reaction', (payload: ChannelMessageReactionPayload) => {
      io.to(payload.channelId).emit('channel:reaction', payload);
    });

    // Join/leave channel rooms
    socket.on('channel:join', (channelId: string) => {
      socket.join(channelId);
    });
    socket.on('channel:leave', (channelId: string) => {
      socket.leave(channelId);
    });

    // --- Direct Message Events ---
    socket.on('dm:message', (msg: DirectMessage) => {
      // Emit to both sender and recipient if online
      const toSocket = userSocketMap.get(msg.to);
      const fromSocket = userSocketMap.get(msg.from);
      if (toSocket) io.to(toSocket).emit('dm:message', msg);
      if (fromSocket && fromSocket !== toSocket) io.to(fromSocket).emit('dm:message', msg);
    });
    socket.on('dm:typing', ({ to }: { to: string }) => {
      const toSocket = userSocketMap.get(to);
      if (toSocket) io.to(toSocket).emit('dm:typing', { from: user.id });
    });
    socket.on('dm:stopTyping', ({ to }: { to: string }) => {
      const toSocket = userSocketMap.get(to);
      if (toSocket) io.to(toSocket).emit('dm:stopTyping', { from: user.id });
    });
    // ---

    // --- Group DM Events ---
    socket.on('group-dm:join', (groupId: string) => {
      socket.join(`group-dm:${groupId}`);
    });
    socket.on('group-dm:leave', (groupId: string) => {
      socket.leave(`group-dm:${groupId}`);
    });
    socket.on('group-dm:message', (msg: GroupDMMessagePayload) => {
      io.to(`group-dm:${msg.groupId}`).emit('group-dm:message', msg);
    });
    socket.on('group-dm:edit-message', (payload: GroupDMMessageEditPayload) => {
      io.to(`group-dm:${payload.groupId}`).emit('group-dm:edit-message', payload);
    });
    socket.on('group-dm:delete-message', (payload: GroupDMMessageDeletePayload) => {
      io.to(`group-dm:${payload.groupId}`).emit('group-dm:delete-message', payload);
    });
    socket.on('group-dm:reaction', (payload: GroupDMMessageReactionPayload) => {
      io.to(`group-dm:${payload.groupId}`).emit('group-dm:reaction', payload);
    });
    socket.on('group-dm:typing', ({ groupId }: { groupId: string }) => {
      socket.to(`group-dm:${groupId}`).emit('group-dm:typing', { from: user.id });
    });
    socket.on('group-dm:stopTyping', ({ groupId }: { groupId: string }) => {
      socket.to(`group-dm:${groupId}`).emit('group-dm:stopTyping', { from: user.id });
    });
    // ---

    // Further events (typing, reactions, etc.) can be added here
  });

  (globalThis as any).io = io;
  return io;
}

// Helper: emit to a user by userId
export function emitToUser(userId: string, event: string, payload: any) {
  const io = (globalThis as any).io as IOServer | undefined
  if (!io) return
  const socketId = userSocketMap.get(userId)
  if (socketId) io.to(socketId).emit(event, payload)
} 