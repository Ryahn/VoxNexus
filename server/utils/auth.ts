// Add this for TypeScript to recognize the 'cookie' module
// @ts-ignore
import { parse } from 'cookie'
import { H3Event } from 'h3';
import jwt from 'jsonwebtoken';

const JWT_SECRET = process.env.JWT_SECRET as string;

export interface AuthPayload {
  id: string;
  username: string;
}

export function getUserFromEvent(event: H3Event): AuthPayload {
  const cookies = parse(event.node.req.headers['cookie'] || '')
  const token = cookies.auth_token
  if (!token) {
    throw new Error('Auth token missing in cookies')
  }
  try {
    return jwt.verify(token, JWT_SECRET) as AuthPayload;
  } catch {
    throw new Error('Invalid or expired token');
  }
}

// Helper for Nuxt 3 server route middleware
export async function requireAuthFromEvent(event: H3Event): Promise<AuthPayload | null> {
  try {
    return getUserFromEvent(event)
  } catch {
    return null
  }
}