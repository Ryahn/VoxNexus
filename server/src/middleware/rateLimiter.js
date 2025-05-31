import rateLimit from 'express-rate-limit';
import RedisStore from 'rate-limit-redis';
import Redis from 'ioredis';

const redis = new Redis(process.env.REDIS_URL || 'redis://localhost:6379');

// API rate limiter (100 requests per minute per IP)
export const apiLimiter = rateLimit({
    store: new RedisStore({
        sendCommand: (...args) => redis.call(...args),
        prefix: 'api:'
    }),
    windowMs: 60 * 1000, // 1 minute
    max: 100, // 100 requests per minute
    message: (req, res) => {
        console.warn(`[RATE LIMIT] API limiter triggered for IP: ${req.ip} at ${new Date().toISOString()}`);
        return { error: 'Too many requests, please try again later.' };
    },
    standardHeaders: true,
    legacyHeaders: false
});

// Auth rate limiter (5 attempts per minute per IP)
export const authLimiter = rateLimit({
    store: new RedisStore({
        sendCommand: (...args) => redis.call(...args),
        prefix: 'auth:'
    }),
    windowMs: 60 * 1000, // 1 minute
    max: 5, // 5 attempts per minute
    message: (req, res) => {
        console.warn(`[RATE LIMIT] Auth limiter triggered for IP: ${req.ip} at ${new Date().toISOString()}`);
        return { error: 'Too many authentication attempts, please try again later.' };
    },
    standardHeaders: true,
    legacyHeaders: false
});

// WebSocket rate limiter (100 messages per minute per connection)
export const wsRateLimiter = (req, res, next) => {
    const ip = req.ip;
    const key = `ws:${ip}`;
    
    redis.incr(key)
        .then((count) => {
            if (count === 1) {
                redis.expire(key, 60); // Set expiry for 1 minute
            }
            
            if (count > 100) { // 100 messages per minute
                res.status(429).json({
                    error: 'Too many WebSocket messages, please slow down.'
                });
                return;
            }
            
            next();
        })
        .catch((err) => {
            console.error('Rate limit error:', err);
            next();
        });
};

// Password reset rate limiter (3 attempts per hour per IP)
export const passwordResetLimiter = rateLimit({
    store: new RedisStore({
        sendCommand: (...args) => redis.call(...args),
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

// Cleanup function to remove expired rate limit entries
export const cleanupRateLimiters = async () => {
    try {
        // Get all keys with rate limit prefixes
        const keys = await redis.keys('*:rate-limit:*');
        if (keys.length > 0) {
            await redis.del(...keys);
        }
    } catch (error) {
        console.error('Error cleaning up rate limiters:', error);
    }
}; 