import express from 'express';
import path from 'path';
import { fileURLToPath } from 'url';
import { authenticate } from '../middleware/auth.js';
import AttachmentService from '../services/attachment/AttachmentService.js';

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
            '.gif': 'image/gif',
            '.pdf': 'application/pdf',
            '.webp': 'image/webp',
            '.mp4': 'video/mp4',
            '.mp3': 'audio/mpeg',
            '.wav': 'audio/wav',
            '.ogg': 'audio/ogg',
            '.zip': 'application/zip',
            '.rar': 'application/rar',
            '.txt': 'text/plain',
            '.csv': 'text/csv',
            '.7z': 'application/x-7z-compressed',
            '.tar': 'application/x-tar',
            '.gz': 'application/gzip',
            '.bz2': 'application/x-bzip2',
            '.doc': 'application/msword',
            '.docx': 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
            '.xls': 'application/vnd.ms-excel',
            '.xlsx': 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
            '.js': 'application/javascript',
            '.css': 'text/css',
            '.html': 'text/html',
            '.json': 'application/json',
            '.xml': 'application/xml',
            '.svg': 'image/svg+xml',
            '.ico': 'image/x-icon',
            '.toml': 'text/toml',
            '.yaml': 'text/yaml',
            '.yml': 'text/yaml',
            '.sh': 'text/x-shellscript',
            '.bash': 'text/x-shellscript',
            '.zsh': 'text/x-shellscript',
            '.fish': 'text/x-shellscript',
            '.php': 'application/x-httpd-php',
            '.py': 'text/x-python',
            '.rb': 'text/x-ruby',
            '.java': 'text/x-java',
            '.c': 'text/x-csrc',
            '.cpp': 'text/x-c++src',
            '.h': 'text/x-c++hdr',
            '.md': 'text/markdown',
            '.apk': 'application/vnd.android.package-archive',
            '.ipa': 'application/x-itunes-ipa',
            '.deb': 'application/x-debian-package',
            '.rpm': 'application/x-redhat-package-manager',
            '.exe': 'application/x-msdownload',
            '.msi': 'application/x-msdownload',
            '.cab': 'application/vnd.ms-cab-compressed',
            '.torrent': 'application/x-bittorrent',
            '.sql': 'text/x-sql',
            '.sqlite': 'application/x-sqlite3',
            '.sqlite3': 'application/x-sqlite3',
            '.sqlite3': 'application/x-sqlite3',
            '.avif': 'image/avif',
            '.heic': 'image/heic',
            '.heif': 'image/heif',
            '.heif': 'image/heif',
            '.webm': 'video/webm',
        }[ext] || 'application/octet-stream';
        
        res.setHeader('Content-Type', contentType);
        res.send(fileBuffer);
    } catch (error) {
        console.error('Error serving file:', error);
        res.status(404).json({ error: 'File not found' });
    }
});

export default router; 