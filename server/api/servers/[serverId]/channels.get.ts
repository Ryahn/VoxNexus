import { defineEventHandler } from 'h3';
import { connectToDatabase } from '../../../utils/mongoose';
import { getUserFromEvent } from '../../../utils/auth';
import Server from '../../../models/Server';
import Channel from '../../../models/Channel';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let user;
  try {
    user = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const serverId = event.context.params?.serverId;
  if (!serverId) {
    return { status: 400, error: 'Server ID is required.' };
  }

  const server = await Server.findById(serverId);
  if (!server) {
    return { status: 404, error: 'Server not found.' };
  }
  if (!server.members.map(id => id.toString()).includes(user.id)) {
    return { status: 403, error: 'You are not a member of this server.' };
  }

  const channels = await Channel.find({ serverId })
    .select('_id name type createdAt')
    .lean();

  return { status: 200, channels };
}); 