import type { H3Event } from 'h3'
import redis from './redis'

interface RateLimitOptions {
  key: string // unique key per user/IP/route
  window: number // window in seconds
  max: number // max requests per window
}

export async function rateLimit(event: H3Event, options: RateLimitOptions) {
  const ip = event.node.req.headers['x-forwarded-for']?.toString().split(',')[0] || event.node.req.socket.remoteAddress || 'unknown'
  const key = `ratelimit:${options.key}:${ip}`
  const ttl = options.window
  const max = options.max

  const current = await redis.incr(key)
  if (current === 1) {
    await redis.expire(key, ttl)
  }
  if (current > max) {
    const retryAfter = await redis.ttl(key)
    event.node.res.statusCode = 429
    event.node.res.setHeader('Retry-After', retryAfter)
    throw new Error(`Rate limit exceeded. Try again in ${retryAfter} seconds.`)
  }
} 