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
    logoutSession
} from '../controllers/AuthController.js';
import { authenticate } from '../middleware/auth.js';

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
router.post('/register', register);
router.post('/login', login);
router.post('/refresh-token', refreshToken);

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