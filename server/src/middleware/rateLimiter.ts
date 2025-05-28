import rateLimit from 'express-rate-limit'
import RedisStore from 'rate-limit-redis'
import Redis from 'ioredis'
import { Request, Response, NextFunction } from 'express'

// Create Redis client
const redis = new Redis(process.env.REDIS_URL || 'redis://localhost:6379')

// General API rate limiter
export const apiLimiter = rateLimit({
  store: new RedisStore({
    sendCommand: (...args: string[]) => redis.call(...args),
  }),
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100, // Limit each IP to 100 requests per windowMs
  message: 'Too many requests from this IP, please try again later.',
  standardHeaders: true, // Return rate limit info in the `RateLimit-*` headers
  legacyHeaders: false, // Disable the `X-RateLimit-*` headers
})

// Auth endpoints rate limiter (more strict)
export const authLimiter = rateLimit({
  store: new RedisStore({
    sendCommand: (...args: string[]) => redis.call(...args),
  }),
  windowMs: 60 * 60 * 1000, // 1 hour
  max: 5, // Limit each IP to 5 requests per windowMs
  message: 'Too many login attempts, please try again later.',
  standardHeaders: true,
  legacyHeaders: false,
})

// WebSocket rate limiter
export const wsRateLimiter = (req: Request, res: Response, next: NextFunction) => {
  const ip = req.ip
  const key = `ws:${ip}`
  
  redis.incr(key)
    .then((count) => {
      if (count === 1) {
        redis.expire(key, 60) // Set expiry for 1 minute
      }
      
      if (count > 100) { // 100 messages per minute
        res.status(429).json({
          error: 'Too many WebSocket messages, please slow down.'
        })
        return
      }
      
      next()
    })
    .catch((err) => {
      console.error('Rate limit error:', err)
      next()
    })
}

// Cleanup function
export const cleanupRateLimiters = () => {
  redis.quit()
} 