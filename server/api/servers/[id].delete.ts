import { defineEventHandler } from 'h3';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';
import Server from '../../models/Server';
import { removeServerFromIndex } from '../../utils/zincsearch';

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

  const server = await Server.findById(serverId);
  if (!server) {
    return { status: 404, error: 'Server not found.' };
  }
  if (server.ownerId.toString() !== user.id) {
    return { status: 403, error: 'Only the server owner can delete the server.' };
  }

  await server.deleteOne();
  try {
    await removeServerFromIndex(server._id.toString());
  } catch (err) {
    console.error('Failed to remove server from ZincSearch:', err);
    // Do not block deletion if ZincSearch fails
  }
  return { status: 200, message: 'Server deleted successfully.' };
}); 