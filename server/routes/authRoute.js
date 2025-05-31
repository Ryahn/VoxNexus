import express from 'express';
import multer from 'multer';
import { 
    register, 
    login, 
    logout, 
    logoutAll, 
    getCurrentUser, 
    refreshToken,
    updateProfile,
    updatePassword,
    uploadAvatar,
    uploadBanner,
    getSessions,
    logoutSession,
    getCsrfToken,
    forgotPassword,
    verifyPasswordResetToken,
    resetPassword
} from '../controllers/AuthController.js';
import { authenticate } from '../middleware/auth.js';
import { passwordResetLimiter } from '../src/middleware/rateLimiter.js';

const router = express.Router();
const upload = multer({
    storage: multer.memoryStorage(),
    limits: {
        fileSize: 5 * 1024 * 1024 // 5MB limit
    },
    fileFilter: (req, file, cb) => {
        const allowedTypes = ['image/jpeg', 'image/png', 'image/gif'];
        if (allowedTypes.includes(file.mimetype)) {
            cb(null, true);
        } else {
            cb(new Error('Invalid file type. Only JPEG, PNG and GIF are allowed.'), false);
        }
    }
});

// Public routes
router.post('/register', (req, res, next) => { console.log('[ROUTE] /api/auth/register hit'); next(); }, register);
router.post('/login', (req, res, next) => { console.log('[ROUTE] /api/auth/login hit'); next(); }, login);
router.post('/refresh-token', (req, res, next) => { console.log('[ROUTE] /api/auth/refresh-token hit'); next(); }, refreshToken);
router.get('/csrf-token', (req, res, next) => { console.log('[ROUTE] /api/auth/csrf-token hit'); next(); }, getCsrfToken);

// Password reset routes with rate limiting
router.post('/forgot-password', (req, res, next) => { console.log('[ROUTE] /api/auth/forgot-password hit'); next(); }, passwordResetLimiter, forgotPassword);
router.post('/verify-password-reset-token', (req, res, next) => { console.log('[ROUTE] /api/auth/verify-password-reset-token hit'); next(); }, passwordResetLimiter, verifyPasswordResetToken);
router.post('/reset-password', (req, res, next) => { console.log('[ROUTE] /api/auth/reset-password hit'); next(); }, passwordResetLimiter, resetPassword);

// Protected routes
router.use(authenticate);
router.get('/me', getCurrentUser);
router.post('/logout', logout);
router.post('/logout-all', logoutAll);

// Profile management
router.put('/profile', updateProfile);
router.put('/password', updatePassword);
router.post('/avatar', upload.single('avatar'), uploadAvatar);
router.post('/banner', upload.single('banner'), uploadBanner);

// Session management
router.get('/sessions', getSessions);
router.delete('/sessions/:sessionId', logoutSession);

export default router;