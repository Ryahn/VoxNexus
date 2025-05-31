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

  const body = await readBody(event);
  const { name } = body;
  if (!name) {
    return { status: 400, error: 'Server name is required.' };
  }

  const existing = await Server.findOne({ name });
  if (existing) {
    return { status: 409, error: 'Server name already exists.' };
  }

  const server = await Server.create({
    name,
    ownerId: user.id,
    members: [user.id],
  });

  return { status: 201, server: { id: server._id, name: server.name, ownerId: server.ownerId, createdAt: server.createdAt } };
}); 