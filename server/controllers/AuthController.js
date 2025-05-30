import { models } from '../libs/database/db.js';
import { auth } from '../libs/utils.js';
import jwt from 'jsonwebtoken';
import crypto from 'crypto';
import bcrypt from 'bcrypt';
import AttachmentService from '../services/attachment/AttachmentService.js';
import MailerService from '../services/mailer/MailerService.js';

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
    delete sanitized.verificationToken;
    delete sanitized.verificationTokenExpiry;
    delete sanitized.passwordResetToken;
    delete sanitized.passwordResetTokenExpiry;
    delete sanitized.__v;
    return sanitized;
};

export const register = async (req, res) => {
    console.log('[REGISTER] Entry');
    console.log('[REGISTER] Request body:', req.body);
    try {
        const { username, email, password, confirmPassword } = req.body;

        // Validation
        if (!username || !email || !password) {
            console.log('[REGISTER] Missing fields');
            return res.status(400).json({ error: 'All fields are required' });
        }

        if (password !== confirmPassword) {
            console.log('[REGISTER] Passwords do not match');
            return res.status(400).json({ error: 'Passwords do not match' });
        }

        // Check if user already exists
        console.log('[REGISTER] Checking for existing user');
        const existingUser = await models.User.findOne({ 
            $or: [{ email }, { username }] 
        });

        if (existingUser) {
            console.log('[REGISTER] User already exists');
            return res.status(400).json({ 
                error: 'User with this email or username already exists' 
            });
        }

        console.log('[REGISTER] Hashing password');
        const hashedPassword = await auth.hashPassword(password);
        
        console.log('[REGISTER] Generating verification token');
        const verificationToken = crypto.randomBytes(32).toString('hex');
        const verificationTokenExpiry = new Date(Date.now() + 24 * 60 * 60 * 1000); // 24 hours

        console.log('[REGISTER] Creating user');
        const user = await models.User.create({ 
            username, 
            email, 
            password: hashedPassword,
            verificationToken,
            verificationTokenExpiry
        });

        console.log('[REGISTER] User created successfully, generating tokens');
        const deviceInfo = req.headers['user-agent'] || 'Unknown';
        const tokens = await generateTokens(user, deviceInfo);
        const sanitizedUser = sanitizeUser(user);
        
        // Only send verification email in production
        if (process.env.NODE_ENV === 'production') {
            console.log('[REGISTER] Sending verification email');
            await MailerService.sendVerificationEmail(email, verificationToken, username);
        } else {
            console.log('[REGISTER] Skipping email in development mode. Verification token:', verificationToken);
        }

        console.log('[REGISTER] Success:', sanitizedUser);
        return res.status(201).json({ 
            message: 'User created successfully', 
            user: sanitizedUser,
            token: tokens.accessToken,
            refreshToken: tokens.refreshToken,
            expiresIn: 15 * 60 // 15 minutes in seconds
        });
    } catch (error) {
        console.error('[REGISTER] Error:', error);
        console.error('[REGISTER] Error stack:', error.stack);
        
        // Handle specific MongoDB errors
        if (error.code === 11000) {
            const field = Object.keys(error.keyPattern)[0];
            return res.status(400).json({ 
                error: `${field.charAt(0).toUpperCase() + field.slice(1)} already exists` 
            });
        }
        
        // Handle validation errors
        if (error.name === 'ValidationError') {
            const errors = Object.values(error.errors).map(err => err.message);
            return res.status(400).json({ 
                error: 'Validation failed',
                errors: errors
            });
        }
        
        return res.status(500).json({ 
            error: 'Internal server error',
            message: process.env.NODE_ENV === 'development' ? error.message : 'An unexpected error occurred'
        });
    }
};

export const verifyEmail = async (req, res) => {
    try {
        const { token } = req.params;
        const user = await models.User.findOne({ 
            verificationToken: token,
            verificationTokenExpiry: { $gt: new Date() }
        });
        
        if (!user) {
            return res.status(400).json({ error: 'Invalid or expired verification token' });
        }
        
        user.is_verified = true;
        user.verificationToken = undefined;
        user.verificationTokenExpiry = undefined;
        await user.save();
        
        return res.status(200).json({ message: 'Email verified successfully' });
    } catch (error) {
        console.error('Verify email error:', error);
        return res.status(500).json({ error: 'Failed to verify email' });
    }
}

export const resendVerificationEmail = async (req, res) => {
    try {
        const { email } = req.body;
        const user = await models.User.findOne({ email });
        
        if (!user) {
            return res.status(400).json({ error: 'User not found' });
        }
        
        if (user.is_verified) {
            return res.status(400).json({ error: 'Email already verified' });
        }

        const verificationToken = crypto.randomBytes(32).toString('hex');
        const verificationTokenExpiry = new Date(Date.now() + 24 * 60 * 60 * 1000); // 24 hours
        
        user.verificationToken = verificationToken;
        user.verificationTokenExpiry = verificationTokenExpiry;
        await user.save();
        
        // Only send verification email in production
        if (process.env.NODE_ENV === 'production') {
            await MailerService.sendVerificationEmail(email, verificationToken, user.username);
        } else {
            console.log('[RESEND_VERIFICATION] Skipping email in development mode. Verification token:', verificationToken);
        }
        
        return res.status(200).json({ message: 'Verification email sent successfully' });
    } catch (error) {
        console.error('Resend verification email error:', error);
        return res.status(500).json({ error: 'Failed to resend verification email' });
    }
}

export const forgotPassword = async (req, res) => {
    try {
        const { email } = req.body;
        const user = await models.User.findOne({ email });
        
        if (!user) {
            return res.status(400).json({ error: 'User not found' });
        }
        
        const passwordResetToken = crypto.randomBytes(32).toString('hex');
        const passwordResetTokenExpiry = new Date(Date.now() + 1 * 60 * 60 * 1000); // 1 hour
        
        user.passwordResetToken = passwordResetToken;
        user.passwordResetTokenExpiry = passwordResetTokenExpiry;
        await user.save();
        
        // Only send password reset email in production
        if (process.env.NODE_ENV === 'production') {
            await MailerService.sendPasswordResetEmail(email, passwordResetToken);
        } else {
            console.log('[FORGOT_PASSWORD] Skipping email in development mode. Reset token:', passwordResetToken);
        }
        
        return res.status(200).json({ message: 'Password reset email sent successfully' });
    } catch (error) {
        console.error('Forgot password error:', error);
        return res.status(500).json({ error: 'Failed to process password reset request' });
    }
}

export const verifyPasswordResetToken = async (req, res) => {
    try {
        const { token } = req.params;
        const user = await models.User.findOne({ 
            passwordResetToken: token,
            passwordResetTokenExpiry: { $gt: new Date() }
        });
        
        if (!user) {
            return res.status(400).json({ error: 'Invalid or expired password reset token' });
        }
        
        return res.status(200).json({ message: 'Password reset token verified successfully' });
    } catch (error) {
        console.error('Verify password reset token error:', error);
        return res.status(500).json({ error: 'Failed to verify password reset token' });
    }
}

export const login = async (req, res) => {
    try {
        const { email, password } = req.body;

        // Validation
        if (!email || !password) {
            return res.status(400).json({ error: 'Email and password are required' });
        }

        console.log('Login attempt:', { email, password });

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
        console.log('Is password valid:', isPasswordValid);

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
            await AttachmentService.delete(user.avatar);
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
            await AttachmentService.delete(user.banner);
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

export const getCsrfToken = async (req, res) => {
    try {
        const token = req.csrfToken();
        console.log('Generating CSRF token:', {
            token,
            cookies: req.cookies,
            headers: req.headers,
            url: req.url,
            method: req.method
        });
        res.json({ csrfToken: token });
    } catch (error) {
        console.error('Error generating CSRF token:', error);
        res.status(500).json({ error: 'Failed to generate CSRF token' });
    }
};

export const resetPassword = async (req, res) => {
    try {
        const { token, newPassword, confirmPassword } = req.body;

        // Validation
        if (!token || !newPassword || !confirmPassword) {
            return res.status(400).json({ error: 'All fields are required' });
        }

        if (newPassword !== confirmPassword) {
            return res.status(400).json({ error: 'Passwords do not match' });
        }

        // Password strength validation
        if (newPassword.length < 8) {
            return res.status(400).json({ error: 'Password must be at least 8 characters long' });
        }

        // Find user with valid reset token
        const user = await models.User.findOne({
            passwordResetToken: token,
            passwordResetTokenExpiry: { $gt: new Date() }
        });

        if (!user) {
            return res.status(400).json({ error: 'Invalid or expired password reset token' });
        }

        // Hash new password
        const hashedPassword = await auth.hashPassword(newPassword);

        // Update user's password and clear reset token
        user.password = hashedPassword;
        user.passwordResetToken = undefined;
        user.passwordResetTokenExpiry = undefined;
        await user.save();

        // Invalidate all existing refresh tokens for security
        user.refreshTokens = [];
        await user.save();

        return res.status(200).json({ message: 'Password reset successful' });
    } catch (error) {
        console.error('Reset password error:', error);
        return res.status(500).json({ error: 'Failed to reset password' });
    }
};