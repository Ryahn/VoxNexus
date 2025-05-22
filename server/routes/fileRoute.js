import express from 'express';
import path from 'path';
import { fileURLToPath } from 'url';
import { authenticate } from '../middleware/auth.js';
import AttachmentService from '../services/AttachmentService.js';

const router = express.Router();
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Serve files from storage directory
router.use('/storage', express.static(path.join(process.cwd(), 'storage')));

// Get file by path
router.get('/:type/:id/:filename', authenticate, async (req, res) => {
    try {
        const { type, id, filename } = req.params;
        const filePath = `${type}/${id}/${filename}`;
        
        const fileBuffer = await AttachmentService.getAttachment(filePath);
        
        // Set appropriate content type based on file extension
        const ext = path.extname(filename).toLowerCase();
        const contentType = {
            '.jpg': 'image/jpeg',
            '.jpeg': 'image/jpeg',
            '.png': 'image/png',
            '.gif': 'image/gif'
        }[ext] || 'application/octet-stream';
        
        res.setHeader('Content-Type', contentType);
        res.send(fileBuffer);
    } catch (error) {
        console.error('Error serving file:', error);
        res.status(404).json({ error: 'File not found' });
    }
});

export default router; 