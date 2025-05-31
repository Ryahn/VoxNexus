import { defineEventHandler, readBody } from 'h3';
import bcrypt from 'bcrypt';
import jwt from 'jsonwebtoken';
import { connectToDatabase } from '../../utils/mongoose';
import User from '../../models/User';
import { rateLimit } from '../../utils/rateLimiter';
import { indexUser } from '../../utils/zincsearch';

const JWT_SECRET = process.env.JWT_SECRET as string;

export default defineEventHandler(async (event) => {
  await rateLimit(event, { key: 'register', window: 60, max: 5 });
  await connectToDatabase();
  const body = await readBody(event);
  const { username, email, password } = body;

  if (!username || !email || !password) {
    return { status: 400, error: 'All fields are required.' };
  }

  const existingUser = await User.findOne({ $or: [{ username }, { email }] });
  if (existingUser) {
    return { status: 409, error: 'Username or email already exists.' };
  }

  const hashedPassword = await bcrypt.hash(password, 10);
  const user = await User.create({ username, email, password: hashedPassword });

  // Index user in ZincSearch
  await indexUser({ id: user._id.toString(), username: user.username, email: user.email, avatarUrl: user.avatarUrl, bio: user.bio });

  const token = jwt.sign({ id: user._id, username: user.username }, JWT_SECRET, { expiresIn: '7d' });

  return { status: 201, token, user: { id: user._id, username: user.username, email: user.email } };
}); 