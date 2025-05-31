import { defineEventHandler, getCookie } from 'h3';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';
import FriendRequest from '../../models/FriendRequest';
import User from '../../models/User';

interface FriendCardUser {
  id: string;
  username: string;
  tag: string;
  avatarUrl: string;
  bio?: string;
  status?: 'online' | 'idle' | 'dnd' | 'offline';
  statusEmoji?: string;
  statusText?: string;
}

function getTag(user: any) {
  // Use discriminator if present, else fallback to #0000
  return user.tag || (user.discriminator ? `#${user.discriminator}` : '#0000');
}

// TODO: Replace with real DB lookup and auth
const mockFriends: FriendCardUser[] = [
  {
    id: '1',
    username: 'JaneDoe',
    tag: '#1234',
    avatarUrl: 'https://cdn.discordapp.com/avatars/1/abc.png',
    bio: 'I love coding!',
    status: 'online',
    statusEmoji: '💻',
    statusText: 'Working on VoxNexus'
  },
  {
    id: '2',
    username: 'JohnSmith',
    tag: '#5678',
    avatarUrl: 'https://cdn.discordapp.com/avatars/2/def.png',
    bio: 'Let\'s play!',
    status: 'idle',
    statusEmoji: '🎮',
    statusText: 'Gaming'
  }
];

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  // Accepted friends (either direction)
  const accepted = await FriendRequest.find({
    $or: [
      { from: userPayload.id },
      { to: userPayload.id },
    ],
    status: 'accepted',
  }).lean();

  // Get friend user IDs
  const friendIds = accepted.map(r =>
    r.from.toString() === userPayload.id ? r.to : r.from
  );
  const friends = await User.find({ _id: { $in: friendIds } })
    .select('_id username avatarUrl bio status tag discriminator')
    .lean();

  // Map to FriendCardUser
  const friendCardUsers: FriendCardUser[] = friends.map((u: any) => ({
    id: u._id.toString(),
    username: u.username,
    tag: getTag(u),
    avatarUrl: u.avatarUrl || '',
    bio: u.bio || '',
    status: u.status || 'offline',
    // Optionally, parse statusEmoji/statusText from a status object if present
    statusEmoji: u.statusEmoji || '',
    statusText: u.statusText || '',
  }));

  // Incoming pending requests
  const incoming = await FriendRequest.find({
    to: userPayload.id,
    status: 'pending',
  })
    .populate('from', '_id username avatarUrl bio status tag discriminator')
    .lean();

  // Outgoing pending requests
  const outgoing = await FriendRequest.find({
    from: userPayload.id,
    status: 'pending',
  })
    .populate('to', '_id username avatarUrl bio status tag discriminator')
    .lean();

  return {
    status: 200,
    friends: friendCardUsers,
    incoming,
    outgoing,
  };
}); 