import { database, models } from '../database/db.js';
import bcrypt from 'bcrypt';
import { generate } from 'generate-password';

/**
 * Validates the input arguments
 * @param {string[]} args - Command line arguments
 * @returns {Object} Object containing userIdOrEmailOrUsername and newPassword
 * @throws {Error} If required arguments are missing
 */
const validateArgs = (args) => {
    const userIdOrEmailOrUsername = args[0];
    let newPassword = args[1];

    if (!userIdOrEmailOrUsername) {
        throw new Error('Usage: node reset_user_password.js <userIdOrEmailOrUsername> [newPassword]');
    }

    if (!newPassword) {
        newPassword = generate({
            length: 12,
            numbers: true,
            symbols: true,
            uppercase: true,
            lowercase: true,
            excludeSimilarCharacters: true,
            strict: true,
        });
    }

    return { userIdOrEmailOrUsername, newPassword };
};

/**
 * Resets a user's password
 * @param {string} userIdOrEmailOrUsername - User ID, email, or username
 * @param {string} newPassword - New password to set
 * @returns {Promise<string>} The new password
 * @throws {Error} If user is not found or password reset fails
 */
const resetUserPassword = async (userIdOrEmailOrUsername, newPassword) => {
    try {
        const user = await models.User.findOne({
            $or: [
                { _id: userIdOrEmailOrUsername },
                { email: userIdOrEmailOrUsername },
                { username: userIdOrEmailOrUsername },
            ],
        });

        if (!user) {
            throw new Error(`User not found: ${userIdOrEmailOrUsername}`);
        }

        // Hash the password before saving
        const saltRounds = 10;
        const hashedPassword = await bcrypt.hash(newPassword, saltRounds);
        
        user.password = hashedPassword;
        await user.save();
        
        return newPassword;
    } catch (error) {
        console.error('Error resetting password:', error.message);
        throw error;
    }
};

/**
 * Main execution function
 */
const main = async () => {
    try {
        // Wait for database connection and model loading
        await database.connect();
        await database.loadModels();

        const { userIdOrEmailOrUsername, newPassword } = validateArgs(process.argv.slice(2));
        const password = await resetUserPassword(userIdOrEmailOrUsername, newPassword);
        console.log(`Password successfully reset for user: ${userIdOrEmailOrUsername}`);
        console.log(`New password: ${password}`);
    } catch (error) {
        console.error('Failed to reset password:', error.message);
        process.exit(1);
    } finally {
        // Disconnect from database after we're done
        await database.disconnect();
    }
};

// Execute the script
main();