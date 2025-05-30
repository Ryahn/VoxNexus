/**
 * Base class for all mailer adapters
 * Defines the interface that all mailer adapters must implement
 */
class BaseMailerAdapter {
  /**
   * Send an email
   * @param {string} to - Recipient email address
   * @param {string} subject - Email subject
   * @param {string} html - HTML content of the email
   * @returns {Promise<any>} The result of sending the email
   */
  async send(to, subject, html) {
    throw new Error('Method not implemented');
  }

  /**
   * Send a verification email
   * @param {string} to - Recipient email address
   * @param {string} token - Verification token
   * @param {string} username - Username of the recipient
   * @returns {Promise<any>} The result of sending the email
   */
  async sendVerificationEmail(to, token, username) {
    throw new Error('Method not implemented');
  }

  /**
   * Send a password reset email
   * @param {string} to - Recipient email address
   * @param {string} token - Password reset token
   * @returns {Promise<any>} The result of sending the email
   */
  async sendPasswordResetEmail(to, token) {
    throw new Error('Method not implemented');
  }
}

export default BaseMailerAdapter;
