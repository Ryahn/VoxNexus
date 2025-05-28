import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

class EmailTemplateManager {
    constructor() {
        this.templates = new Map();
        this.templateDir = path.join(__dirname, 'templates');
        this.logoBase64 = null;
    }

    /**
     * Load and convert logo to base64
     */
    async loadLogo() {
        try {
            const logoPath = path.join(process.cwd(), 'client', 'public', 'assets', 'logo.png');
            const logoBuffer = await fs.readFile(logoPath);
            this.logoBase64 = `data:image/png;base64,${logoBuffer.toString('base64')}`;
        } catch (error) {
            console.error('Error loading logo:', error);
            this.logoBase64 = null;
        }
    }

    /**
     * Load all email templates from the templates directory
     */
    async loadTemplates() {
        try {
            // Load logo first
            await this.loadLogo();

            const files = await fs.readdir(this.templateDir);
            for (const file of files) {
                if (file.endsWith('.html')) {
                    const templateName = path.basename(file, '.html');
                    const content = await fs.readFile(path.join(this.templateDir, file), 'utf-8');
                    this.templates.set(templateName, content);
                }
            }
        } catch (error) {
            console.error('Error loading email templates:', error);
            throw error;
        }
    }

    /**
     * Get a template by name
     * @param {string} templateName - Name of the template
     * @returns {string} Template content
     */
    getTemplate(templateName) {
        const template = this.templates.get(templateName);
        if (!template) {
            throw new Error(`Template '${templateName}' not found`);
        }
        return template;
    }

    /**
     * Render a template with variables
     * @param {string} templateName - Name of the template
     * @param {Object} variables - Variables to replace in the template
     * @returns {string} Rendered template
     */
    renderTemplate(templateName, variables = {}) {
        let template = this.getTemplate(templateName);
        
        // Add logo to variables if available
        if (this.logoBase64) {
            variables.logoUrl = this.logoBase64;
        }
        
        // Replace variables in the template
        Object.entries(variables).forEach(([key, value]) => {
            const regex = new RegExp(`{{${key}}}`, 'g');
            template = template.replace(regex, value);
        });

        return template;
    }
}

// Create and export a singleton instance
const emailTemplateManager = new EmailTemplateManager();
export default emailTemplateManager; 