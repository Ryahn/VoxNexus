import { DomainClient } from '@mailpace/mailpace.js';
import nodemailer from 'nodemailer';
import emailTemplateManager from './templates/email/EmailTemplateManager.js';

class Mailer {
    constructor(type = 'API') {
        this.type = type;
        this.mailer = new DomainClient(process.env.MAILPACE_API_KEY);
        
        this.transporter = nodemailer.createTransport({
            host: 'smtp.mailpace.com',
            port: 587,
            secure: true,
            tls: {
                rejectUnauthorized: false
            },
            auth: {
                user: process.env.MAILPACE_API_KEY,
                pass: process.env.MAILPACE_API_KEY
            }
        });

        // Load email templates
        emailTemplateManager.loadTemplates().catch(error => {
            console.error('Failed to load email templates:', error);
        });
    }

    async send(to, subject, html) {
        if (this.type === 'API') {
            return this.mailer.sendEmail({
                from: `noreply@${process.env.SITE_DOMAIN}`,
                to: to,
                subject: subject,
                htmlbody: html,
            });
        } else {
            return this.transporter.sendMail({
                from: `noreply@${process.env.SITE_DOMAIN}`,
                to: to,
                subject: subject,
                html: html,
            });
        }
    }

    async sendVerificationEmail(to, token, username) {
        const verificationLink = `${process.env.SITE_DOMAIN}/verify?token=${token}`;
        const html = emailTemplateManager.renderTemplate('email-verification', {
            username,
            verificationLink,
            currentYear: new Date().getFullYear()
        });
        return this.send(to, 'VoxNexus - Verify your email', html);
    }

    async sendPasswordResetEmail(to, token) {
        const resetLink = `${process.env.SITE_DOMAIN}/reset-password?token=${token}`;
        const html = emailTemplateManager.renderTemplate('password-reset', {
            resetLink,
            currentYear: new Date().getFullYear()
        });
        return this.send(to, 'VoxNexus - Reset your password', html);
    }
}

// Create and export a default instance
const mailer = new Mailer(process.env.MAIL_TYPE || 'API');
export default mailer;

// Also export the class for custom instances
export { Mailer };

