import nodemailer from 'nodemailer';
import BaseMailerAdapter from './BaseMailerAdapter.js';
import emailTemplateManager from '../../../libs/templates/email/EmailTemplateManager.js';

/**
 * SMTP adapter for sending emails using nodemailer
 */
class SMTPAdapter extends BaseMailerAdapter {
  constructor() {
    super();
    this.transporter = nodemailer.createTransport({
      host: process.env.STMP_HOST || 'smtp.mailpace.com',
      port: process.env.STMP_PORT || 587,
      secure: process.env.STMP_SECURE || true,
      tls: {
        rejectUnauthorized: false
      },
      auth: {
        user: process.env.STMP_USER || process.env.MAILPACE_API_KEY,
        pass: process.env.STMP_PASSWORD || process.env.MAILPACE_API_KEY
      }
    });
  }

  /**
   * Send an email using SMTP
   * @param {string} to - Recipient email address
   * @param {string} subject - Email subject
   * @param {string} html - HTML content of the email
   * @returns {Promise<any>} The result of sending the email
   */
  async send(to, subject, html) {
    return this.transporter.sendMail({
      from: `noreply@${process.env.SITE_DOMAIN}`,
      to: to,
      subject: subject,
      html: html,
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

export default SMTPAdapter; 