import { defineEventHandler, readBody, setHeader } from 'h3';
import bcrypt from 'bcrypt';
import jwt from 'jsonwebtoken';
import { connectToDatabase } from '../../utils/mongoose';
import User from '../../models/User';
import { rateLimit } from '../../utils/rateLimiter';
import { serialize } from 'cookie'

const JWT_SECRET = process.env.JWT_SECRET as string;

export default defineEventHandler(async (event) => {
  await rateLimit(event, { key: 'login', window: 60, max: 5 });
  await connectToDatabase();
  const body = await readBody(event);
  const { email, password } = body;

  if (!email || !password) {
    return { status: 400, error: 'Email and password are required.' };
  }

  const user = await User.findOne({ email });
  if (!user) {
    return { status: 401, error: 'Invalid credentials.' };
  }

  const isMatch = await bcrypt.compare(password, user.password);
  if (!isMatch) {
    return { status: 401, error: 'Invalid credentials.' };
  }

  const token = jwt.sign({ id: user._id, username: user.username }, JWT_SECRET, { expiresIn: '7d' });

  setHeader(event, 'Set-Cookie', serialize('auth_token', token, {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax',
    path: '/',
    maxAge: 60 * 60 * 24 * 7 // 1 week
  }))

  return { status: 200, user: { id: user._id, username: user.username, email: user.email } };
}); 