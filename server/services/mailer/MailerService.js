/**
 * Service for handling email operations
 * Uses different mailer adapters based on configuration
 */
class MailerService {
  constructor() {
    this.adapter = null;
    this.initializeAdapter();
  }

  /**
   * Create the appropriate mailer adapter based on configuration
   * @private
   * @returns {Promise<BaseMailerAdapter>} The configured mailer adapter
   */
  async createAdapter() {
    const mailType = process.env.MAIL_TYPE || 'mailpace';
    const adapterPath = `./adapters/${mailType.toLowerCase()}.js`;

    try {
      const module = await import(adapterPath);
      return new module.default();
    } catch (error) {
      // Fallback to mailpace if specified adapter fails to load
      const mailpaceModule = await import('./adapters/mailpace.js');
      return new mailpaceModule.default();
    }
  }

  async initializeAdapter() {
    this.adapter = await this.createAdapter();
  }

  /**
   * Send an email
   * @param {string} to - Recipient email address
   * @param {string} subject - Email subject
   * @param {string} html - HTML content of the email
   * @returns {Promise<any>} The result of sending the email
   */
  async send(to, subject, html) {
    if (!this.adapter) {
      await this.initializeAdapter();
    }
    return this.adapter.send(to, subject, html);
  }

  /**
   * Send a verification email
   * @param {string} to - Recipient email address
   * @param {string} token - Verification token
   * @param {string} username - Username of the recipient
   * @returns {Promise<any>} The result of sending the email
   */
  async sendVerificationEmail(to, token, username) {
    if (!this.adapter) {
      await this.initializeAdapter();
    }
    return this.adapter.sendVerificationEmail(to, token, username);
  }

  /**
   * Send a password reset email
   * @param {string} to - Recipient email address
   * @param {string} token - Password reset token
   * @returns {Promise<any>} The result of sending the email
   */
  async sendPasswordResetEmail(to, token) {
    if (!this.adapter) {
      await this.initializeAdapter();
    }
    return this.adapter.sendPasswordResetEmail(to, token);
  }
}

// Export a singleton instance
export default new MailerService();

