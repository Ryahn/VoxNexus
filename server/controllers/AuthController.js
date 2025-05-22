import { models } from '../libs/database/db.js';
import { auth } from '../libs/utils.js';
import jwt from 'jsonwebtoken';
import crypto from 'crypto';
import bcrypt from 'bcrypt';
import AttachmentService from '../services/AttachmentService.js';

// Helper function to generate tokens
const generateTokens = async (user, deviceInfo = 'Unknown') => {
    // Generate access token (short-lived)
    const accessToken = jwt.sign(
        { 
            id: user._id,
            email: user.email,
            username: user.username
        },
        process.env.JWT_SECRET || 'your-secret-key',
        { expiresIn: '15m' }
    );

    // Generate refresh token (long-lived)
    const refreshToken = crypto.randomBytes(40).toString('hex');
    const refreshTokenExpiry = new Date(Date.now() + 7 * 24 * 60 * 60 * 1000); // 7 days

    // Store refresh token
    user.refreshTokens.push({
        token: refreshToken,
        expiresAt: refreshTokenExpiry,
        deviceInfo
    });

    // Keep only the last 5 refresh tokens
    if (user.refreshTokens.length > 5) {
        user.refreshTokens = user.refreshTokens.slice(-5);
    }

    await user.save();

    return {
        accessToken,
        refreshToken,
        refreshTokenExpiry
    };
};

// Helper function to sanitize user data
const sanitizeUser = (user) => {
    const sanitized = user.toObject();
    delete sanitized.password;
    delete sanitized.refreshTokens;
    delete sanitized.__v;
    return sanitized;
};

export const register = async (req, res) => {
    try {
        const { username, email, password, confirmPassword } = req.body;

        // Validation
        if (!username || !email || !password) {
            return res.status(400).json({ error: 'All fields are required' });
        }

        if (password !== confirmPassword) {
            return res.status(400).json({ error: 'Passwords do not match' });
        }

        // Check if user already exists
        const existingUser = await models.User.findOne({ 
            $or: [{ email }, { username }] 
        });

        if (existingUser) {
            return res.status(400).json({ 
                error: 'User with this email or username already exists' 
            });
        }

        const hashedPassword = await auth.hashPassword(password);

        const user = await models.User.create({ 
            username, 
            email, 
            password: hashedPassword 
        });

        const deviceInfo = req.headers['user-agent'] || 'Unknown';
        const tokens = await generateTokens(user, deviceInfo);
        const sanitizedUser = sanitizeUser(user);

        return res.status(201).json({ 
            message: 'User created successfully', 
            user: sanitizedUser,
            ...tokens
        });
    } catch (error) {
        console.error('Registration error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
};

export const login = async (req, res) => {
    try {
        const { email, password } = req.body;

        // Validation
        if (!email || !password) {
            return res.status(400).json({ error: 'Email and password are required' });
        }

        const user = await models.User.findOne({ email });

        if (!user) {
            return res.status(401).json({ error: 'Invalid credentials' });
        }

        if (user.is_banned) {
            return res.status(403).json({ 
                error: 'Account is banned',
                reason: user.ban_reason
            });
        }

        const isPasswordValid = await auth.comparePassword(password, user.password);

        if (!isPasswordValid) {
            return res.status(401).json({ error: 'Invalid credentials' });
        }

        // Update last online timestamp
        user.last_online = new Date();
        await user.save();

        const deviceInfo = req.headers['user-agent'] || 'Unknown';
        const tokens = await generateTokens(user, deviceInfo);
        const sanitizedUser = sanitizeUser(user);

        return res.status(200).json({ 
            message: 'Login successful', 
            user: sanitizedUser,
            ...tokens
        });
    } catch (error) {
        console.error('Login error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
};

export const refreshToken = async (req, res) => {
    try {
        const { refreshToken } = req.body;

        if (!refreshToken) {
            return res.status(400).json({ error: 'Refresh token is required' });
        }

        const user = await models.User.findOne({
            'refreshTokens.token': refreshToken,
            'refreshTokens.isValid': true,
            'refreshTokens.expiresAt': { $gt: new Date() }
        });

        if (!user) {
            return res.status(401).json({ error: 'Invalid refresh token' });
        }

        // Generate new tokens
        const deviceInfo = req.headers['user-agent'] || 'Unknown';
        const tokens = await generateTokens(user, deviceInfo);

        // Invalidate old refresh token
        await models.User.updateOne(
            { _id: user._id, 'refreshTokens.token': refreshToken },
            { $set: { 'refreshTokens.$.isValid': false } }
        );

        return res.status(200).json(tokens);
    } catch (error) {
        console.error('Refresh token error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
};

export const getCurrentUser = async (req, res) => {
    try {
        const user = await models.User.findById(req.user.id);
        
        if (!user) {
            return res.status(404).json({ error: 'User not found' });
        }

        const sanitizedUser = sanitizeUser(user);
        return res.status(200).json({ user: sanitizedUser });
    } catch (error) {
        console.error('Get current user error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
};

export const logout = async (req, res) => {
    try {
        const { refreshToken } = req.body;
        
        if (refreshToken) {
            // Invalidate the refresh token
            await models.User.updateOne(
                { _id: req.user.id, 'refreshTokens.token': refreshToken },
                { $set: { 'refreshTokens.$.isValid': false } }
            );
        }

        // Update last online timestamp
        const user = await models.User.findById(req.user.id);
        if (user) {
            user.last_online = new Date();
            await user.save();
        }

        return res.status(200).json({ message: 'Logged out successfully' });
    } catch (error) {
        console.error('Logout error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
};

export const logoutAll = async (req, res) => {
    try {
        // Invalidate all refresh tokens
        await models.User.updateOne(
            { _id: req.user.id },
            { $set: { 'refreshTokens.$[].isValid': false } }
        );

        // Update last online timestamp
        const user = await models.User.findById(req.user.id);
        if (user) {
            user.last_online = new Date();
            await user.save();
        }

        return res.status(200).json({ message: 'Logged out from all devices' });
    } catch (error) {
        console.error('Logout all error:', error);
        return res.status(500).json({ error: 'Internal server error' });
    }
};

export const updateProfile = async (req, res) => {
    try {
        const { username, email } = req.body;
        const userId = req.user.id;

        // Check if username or email is already taken
        const existingUser = await models.User.findOne({
            $or: [
                { username, _id: { $ne: userId } },
                { email, _id: { $ne: userId } }
            ]
        });

        if (existingUser) {
            return res.status(400).json({
                error: 'Username or email is already taken'
            });
        }

        const user = await models.User.findById(userId);
        if (!user) {
            return res.status(404).json({
                error: 'User not found'
            });
        }

        user.username = username;
        user.email = email;
        await user.save();

        res.json({
            user: sanitizeUser(user)
        });
    } catch (error) {
        console.error('Update profile error:', error);
        res.status(500).json({
            error: 'Failed to update profile'
        });
    }
};

export const updatePassword = async (req, res) => {
    try {
        const { currentPassword, newPassword } = req.body;
        const userId = req.user.id;

        const user = await models.User.findById(userId);
        if (!user) {
            return res.status(404).json({
                error: 'User not found'
            });
        }

        const isPasswordValid = await bcrypt.compare(currentPassword, user.password);
        if (!isPasswordValid) {
            return res.status(400).json({
                error: 'Current password is incorrect'
            });
        }

        const hashedPassword = await bcrypt.hash(newPassword, 10);
        user.password = hashedPassword;
        await user.save();

        res.json({
            message: 'Password updated successfully'
        });
    } catch (error) {
        console.error('Update password error:', error);
        res.status(500).json({
            error: 'Failed to update password'
        });
    }
};

export const uploadAvatar = async (req, res) => {
    try {
        if (!req.file) {
            return res.status(400).json({
                error: 'No file uploaded'
            });
        }

        const userId = req.user.id;
        const user = await models.User.findById(userId);
        if (!user) {
            return res.status(404).json({
                error: 'User not found'
            });
        }

        // Delete old avatar if exists
        if (user.avatar) {
            await AttachmentService.deleteAttachment(user.avatar);
        }

        // Upload new avatar
        const avatarPath = await AttachmentService.uploadUserAvatar(
            req.file.buffer,
            userId,
            req.file.mimetype
        );
        
        user.avatar = avatarPath;
        await user.save();

        res.json({
            avatar: avatarPath
        });
    } catch (error) {
        console.error('Upload avatar error:', error);
        res.status(500).json({
            error: 'Failed to upload avatar'
        });
    }
};

export const uploadBanner = async (req, res) => {
    try {
        if (!req.file) {
            return res.status(400).json({
                error: 'No file uploaded'
            });
        }

        const userId = req.user.id;
        const user = await models.User.findById(userId);
        if (!user) {
            return res.status(404).json({
                error: 'User not found'
            });
        }

        // Delete old banner if exists
        if (user.banner) {
            await AttachmentService.deleteAttachment(user.banner);
        }

        // Upload new banner
        const bannerPath = await AttachmentService.uploadUserBanner(
            req.file.buffer,
            userId,
            req.file.mimetype
        );
        
        user.banner = bannerPath;
        await user.save();

        res.json({
            banner: bannerPath
        });
    } catch (error) {
        console.error('Upload banner error:', error);
        res.status(500).json({
            error: 'Failed to upload banner'
        });
    }
};

export const getSessions = async (req, res) => {
    try {
        const userId = req.user.id;
        const user = await models.User.findById(userId);
        if (!user) {
            return res.status(404).json({
                error: 'User not found'
            });
        }

        const sessions = user.refreshTokens.map(token => ({
            id: token._id,
            device: token.device,
            lastActive: token.lastActive,
            current: token.token === req.cookies.refreshToken
        }));

        res.json({
            sessions
        });
    } catch (error) {
        console.error('Get sessions error:', error);
        res.status(500).json({
            error: 'Failed to fetch sessions'
        });
    }
};

export const logoutSession = async (req, res) => {
    try {
        const { sessionId } = req.params;
        const userId = req.user.id;

        const user = await models.User.findById(userId);
        if (!user) {
            return res.status(404).json({
                error: 'User not found'
            });
        }

        user.refreshTokens = user.refreshTokens.filter(
            token => token._id.toString() !== sessionId
        );
        await user.save();

        res.json({
            message: 'Session logged out successfully'
        });
    } catch (error) {
        console.error('Logout session error:', error);
        res.status(500).json({
            error: 'Failed to logout session'
        });
    }
};