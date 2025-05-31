import { H3Event } from 'h3';
import jwt from 'jsonwebtoken';

const JWT_SECRET = process.env.JWT_SECRET as string;

export interface AuthPayload {
  id: string;
  username: string;
}

export function getUserFromEvent(event: H3Event): AuthPayload {
  const authHeader = event.node.req.headers['authorization'];
  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    throw new Error('Authorization header missing or malformed');
  }
  const token = authHeader.split(' ')[1];
  try {
    return jwt.verify(token, JWT_SECRET) as AuthPayload;
  } catch {
    throw new Error('Invalid or expired token');
  }
} 