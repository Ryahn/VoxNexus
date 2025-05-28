import bcrypt from 'bcrypt';

/**
 * Utility functions module
 * @module utility
 * @description A collection of utility functions for the application
 * @author Ryahn
 * @version 1.0.0
 * @since 1.0.0
 * @license MIT
 * @copyright 2025 Ryahn
 */

/**
 * @typedef {Object} Utility
 * @property {function(): BigInt} generateSnowflake - Generates a unique snowflake ID
 */

/**
 * Collection of utility functions
 * @type {Utility}
 */
const utility = {
    /**
     * Generates a unique snowflake ID using timestamp, worker ID, process ID, and sequence
     * @returns {BigInt} A unique snowflake ID
     * @example
     * const id = utility.generateSnowflake();
     */
    generateSnowflake: () => {
        const timestamp = Date.now();
        const workerId = 1; // You can make this dynamic based on your server instance
        const processId = process.pid;
        const sequence = Math.floor(Math.random() * 4096);
        
        return BigInt(timestamp) << 22n | BigInt(workerId) << 17n | BigInt(processId) << 12n | BigInt(sequence);
    },

    getHumanReadableSize: (size) => {
        if (size < 1024) {
            return `${size} B`;
        } else if (size < 1024 * 1024) {
            return `${(size / 1024).toFixed(2)} KB`;
        } else if (size < 1024 * 1024 * 1024) {
            return `${(size / (1024 * 1024)).toFixed(2)} MB`;
        } else {
            return `${(size / (1024 * 1024 * 1024)).toFixed(2)} GB`;
        }
    },

    getHumanReadableDuration: (duration) => {
        const hours = Math.floor(duration / 3600);
        const minutes = Math.floor((duration % 3600) / 60);
        const seconds = duration % 60;
        
        if (hours > 0) {
            return `${hours}h ${minutes}m ${seconds}s`;
        } else if (minutes > 0) {
            return `${minutes}m ${seconds}s`;
        } else {
            return `${seconds}s`;
        }
    },

}

const auth = {
    hashPassword: (password) => {
        return bcrypt.hash(password, 10);
    },

    comparePassword: (password, hash) => {
        return bcrypt.compare(password, hash);
    }
}

const ALLOWED_MIME_TYPES = ['image/png', 'image/jpeg', 'image/gif', 'image/webp'];
const MAX_FILE_SIZE = 10 * 1024 * 1024; // 10MB

const file = {
    validateFile: (file) => {
        if (!file || Object.keys(file).length === 0) {
            return false;
        }
        if (file.size > MAX_FILE_SIZE || file.size < 1024) {
            return false;
        }
        if (!ALLOWED_MIME_TYPES.includes(file.mimetype)) {
            return false;
        }
        return true;
    }
}

export { utility, auth };

// Add default export
const utils = {
    ...utility,
    ...auth
};

export default utils;