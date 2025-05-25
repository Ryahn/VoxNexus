import { models } from '../database/db.js';
import bcrypt from 'bcrypt';
import { generate } from 'generate-password';

/**
 * Creates an admin user
 * @returns {Promise<{username: string, email: string, password: string}>} The admin user
 * @throws {Error} If admin creation fails
 */
const makeAdmin = async () => {
    try {
        const newPassword = generate({
            length: 12,
            numbers: true,
            symbols: true,
            uppercase: true,
            lowercase: true,
            excludeSimilarCharacters: true,
            strict: true,
        });

        const saltRounds = 10;
        const hashedPassword = await bcrypt.hash(newPassword, saltRounds);

        const user = await models.User.create({
            username: 'admin',
            email: `admin@${process.env.SITE_DOMAIN}`,
            password: hashedPassword,
            isAdmin: true,
        });

        if (!user) {
            throw new Error(`User not found: ${userIdOrEmailOrUsername}`);
        }

        await user.save();
        
        return { username: 'admin', email: `admin@${process.env.SITE_DOMAIN}`, password: newPassword };
    } catch (error) {
        console.error('Error creating admin:', error.message);
        throw error;
    }
};

/**
 * Main execution function
 */
const main = async () => {
    try {
        const admin = await makeAdmin();
        console.log(`Admin created: ${admin.username} with email: ${admin.email} and password: ${admin.password}`);
    } catch (error) {
        console.error('Failed to create admin:', error.message);
        process.exit(1);
    }
};

// Execute the script
main();