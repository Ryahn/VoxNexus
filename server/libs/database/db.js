import mongoose from "mongoose";
import glob from "glob";
import path from "path";

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
        await this.mongoose.connect(this.config.uri);
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
        const models = glob.sync(path.join(__dirname, 'models', '*.js'));
        for (const model of models) {
            const modelModule = await import(model);
            const modelName = path.basename(model, '.js');
            this.models[modelName] = modelModule.default;
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