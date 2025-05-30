import { DomainClient } from '@mailpace/mailpace.js';
import BaseMailerAdapter from './BaseMailerAdapter.js';
import emailTemplateManager from '../../../libs/templates/email/EmailTemplateManager.js';

/**
 * MailPace adapter for sending emails using the MailPace API
 */
class MailPaceAdapter extends BaseMailerAdapter {
  constructor() {
    super();
    this.mailer = new DomainClient(process.env.MAILPACE_API_KEY);
  }

  /**
   * Send an email using MailPace API
   * @param {string} to - Recipient email address
   * @param {string} subject - Email subject
   * @param {string} html - HTML content of the email
   * @returns {Promise<any>} The result of sending the email
   */
  async send(to, subject, html) {
    return this.mailer.sendEmail({
      from: `noreply@${process.env.SITE_DOMAIN}`,
      to: to,
      subject: subject,
      htmlbody: html,
    });
  }

  /**
   * Send a verification email
   * @param {string} to - Recipient email address
   * @param {string} token - Verification token
   * @param {string} username - Username of the recipient
   * @returns {Promise<any>} The result of sending the email
   */
  async sendVerificationEmail(to, token, username) {
    const verificationLink = `${process.env.SITE_DOMAIN}/verify?token=${token}`;
    const html = emailTemplateManager.renderTemplate('email-verification', {
      username,
      verificationLink,
      currentYear: new Date().getFullYear()
    });
    return this.send(to, 'VoxNexus - Verify your email', html);
  }

  /**
   * Send a password reset email
   * @param {string} to - Recipient email address
   * @param {string} token - Password reset token
   * @returns {Promise<any>} The result of sending the email
   */
  async sendPasswordResetEmail(to, token) {
    const resetLink = `${process.env.SITE_DOMAIN}/reset-password?token=${token}`;
    const html = emailTemplateManager.renderTemplate('password-reset', {
      resetLink,
      currentYear: new Date().getFullYear()
    });
    return this.send(to, 'VoxNexus - Reset your password', html);
  }
}

export default MailPaceAdapter;
