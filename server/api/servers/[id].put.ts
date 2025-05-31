import { defineEventHandler, readBody } from 'h3';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';
import Server from '../../models/Server';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let user;
  try {
    user = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const serverId = event.context.params?.id;
  if (!serverId) {
    return { status: 400, error: 'Server ID is required.' };
  }

  const body = await readBody(event);
  const { name } = body;
  if (!name) {
    return { status: 400, error: 'Server name is required.' };
  }

  const server = await Server.findById(serverId);
  if (!server) {
    return { status: 404, error: 'Server not found.' };
  }
  if (server.ownerId.toString() !== user.id) {
    return { status: 403, error: 'Only the server owner can update the server.' };
  }

  server.name = name;
  await server.save();

  return { status: 200, server: { id: server._id, name: server.name, ownerId: server.ownerId, createdAt: server.createdAt } };
}); 