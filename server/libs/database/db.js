import mongoose from "mongoose";
import { glob } from "glob";
import path from "path";
import { fileURLToPath } from 'url';

// Get the equivalent of __dirname in ES modules
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

class Database {
    /**
     * @param {Object} config
     * @param {string} config.uri
     */
    constructor(config) {
        this.mongoose = mongoose;
        this.config = config;
        this.models = {};
    }

    /**
     * Connect to the database
     * @returns {Promise<void>}
     */
    async connect() {
        try {
            await this.mongoose.connect(this.config.uri);
            console.log('Connected to MongoDB successfully');
        } catch (error) {
            console.error('Failed to connect to MongoDB:', error);
            throw error;
        }
    }

    /**
     * Disconnect from the database
     * @returns {Promise<void>}
     */
    async disconnect() {
        await this.mongoose.disconnect();
    }

    /**
     * Get the database connection
     * @returns {mongoose.Connection}
     */
    async getConnection() {
        return this.mongoose.connection;
    }

    /**
     * Get a model
     * @param {string} name
     * @returns {mongoose.Model}
     */
    async getModel(name) {
        return this.mongoose.model(name);
    }

    /**
     * Load models from the models directory
     * @returns {Promise<void>}
     */
    async loadModels() {
        try {
            const models = await glob('*.js', { cwd: path.join(__dirname, 'models') });
            for (const model of models) {
                try {
                    const modelPath = path.join(__dirname, 'models', model);
                    const modelModule = await import(modelPath);
                    const modelName = path.basename(model, '.js').replace('Schema', '');
                    this.models[modelName] = modelModule.default;
                } catch (error) {
                    console.error(`Failed to load model ${model}:`, error);
                }
            }
            console.log('Models loaded successfully');
        } catch (error) {
            console.error('Failed to load models:', error);
            throw error;
        }
    }
}

// Create singleton instance
const database = new Database({
    uri: process.env.MONGODB_URI || 'mongodb://localhost:27017/voxnexus'
});

// Export the database instance and models
export { database };
export const models = database.models;

// Initialize database connection and load models
database.connect()
    .then(() => database.loadModels())
    .catch(err => console.error('Failed to connect to database:', err));

export default database;