import { defineEventHandler, readBody } from 'h3';
import bcrypt from 'bcrypt';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';
import User from '../../models/User';
import { indexUser } from '../../utils/zincsearch';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const body = await readBody(event);
  const { username, email, password } = body;
  if (!username && !email && !password) {
    return { status: 400, error: 'At least one field (username, email, password) is required.' };
  }

  const user = await User.findById(userPayload.id);
  if (!user) {
    return { status: 404, error: 'User not found.' };
  }

  if (username && username !== user.username) {
    const existing = await User.findOne({ username });
    if (existing) {
      return { status: 409, error: 'Username already exists.' };
    }
    user.username = username;
  }
  if (email && email !== user.email) {
    const existing = await User.findOne({ email });
    if (existing) {
      return { status: 409, error: 'Email already exists.' };
    }
    user.email = email;
  }
  if (password) {
    user.password = await bcrypt.hash(password, 10);
  }

  await user.save();
  // Index user in ZincSearch
  await indexUser({ id: user._id.toString(), username: user.username, email: user.email, avatarUrl: user.avatarUrl, bio: user.bio });

  return { status: 200, user: { id: user._id, username: user.username, email: user.email, createdAt: user.createdAt } };
}); 