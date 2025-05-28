import rateLimit from 'express-rate-limit';
import RedisStore from 'rate-limit-redis';
import Redis from 'ioredis';

const redis = new Redis(process.env.REDIS_URL || 'redis://localhost:6379');

// Password reset rate limiter (3 attempts per hour per IP)
export const passwordResetLimiter = rateLimit({
    store: new RedisStore({
        client: redis,
        prefix: 'password_reset:'
    }),
    windowMs: 60 * 60 * 1000, // 1 hour
    max: 3, // 3 attempts per hour
    message: {
        error: 'Too many password reset attempts. Please try again later.'
    },
    standardHeaders: true,
    legacyHeaders: false
}); 