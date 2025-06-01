import { defineEventHandler, setHeader } from 'h3';
import { serialize } from 'cookie';

export default defineEventHandler(async (event) => {
  setHeader(event, 'Set-Cookie', serialize('auth_token', '', {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax',
    path: '/',
    maxAge: 0
  }))
  return { status: 200, message: 'Logged out' };
}); 