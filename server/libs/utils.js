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
    }
}

export default utility;