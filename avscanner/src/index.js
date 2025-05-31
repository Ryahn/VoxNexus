require('dotenv').config();
const express = require('express');
const multer = require('multer');
const { exec } = require('child_process');
const { promisify } = require('util');
const execAsync = promisify(exec);
const fs = require('fs').promises;
const path = require('path');

const app = express();
const port = process.env.PORT || 3000;
const maxFileSize = parseInt(process.env.MAX_FILE_SIZE) || 104857600; // 100MB default

// Configure multer for file uploads
const storage = multer.diskStorage({
  destination: (req, file, cb) => {
    cb(null, 'uploads/');
  },
  filename: (req, file, cb) => {
    cb(null, `${Date.now()}-${file.originalname}`);
  }
});

const upload = multer({
  storage,
  limits: {
    fileSize: maxFileSize
  }
});

// Middleware to check API key
const checkApiKey = (req, res, next) => {
  const apiKey = req.headers['x-api-key'];
  if (!apiKey || apiKey !== process.env.API_KEY) {
    return res.status(401).json({ error: 'Invalid API key' });
  }
  next();
};

// Create uploads directory if it doesn't exist
async function ensureUploadsDir() {
  try {
    await fs.mkdir('uploads', { recursive: true });
  } catch (error) {
    console.error('Error creating uploads directory:', error);
  }
}

// Scan file with ClamAV
async function scanWithClamAV(filePath) {
  try {
    const { stdout } = await execAsync(`clamscan ${filePath}`);
    return {
      isInfected: stdout.includes('FOUND'),
      details: stdout
    };
  } catch (error) {
    console.error('ClamAV scan error:', error);
    throw new Error('ClamAV scan failed');
  }
}

// Scan file with Linux Malware Detect
async function scanWithLMD(filePath) {
  try {
    const { stdout } = await execAsync(`maldet --scan-all ${filePath}`);
    // Check for "malware hits" followed by a number greater than 0
    const isInfected = /malware hits (\d+)/.test(stdout) && parseInt(stdout.match(/malware hits (\d+)/)[1]) > 0;
    return {
      isInfected,
      details: stdout
    };
  } catch (error) {
    console.error('LMD scan error:', error);
    throw new Error('LMD scan failed');
  }
}

// API endpoint for file scanning
app.post('/scan', checkApiKey, upload.single('file'), async (req, res) => {
  try {
    if (!req.file) {
      return res.status(400).json({ error: 'No file uploaded' });
    }

    const filePath = `/app/${req.file.path}`;
    const results = {
      filename: req.file.originalname,
      size: req.file.size,
      scans: {}
    };

    // Run both scans in parallel
    const [clamavResult, lmdResult] = await Promise.all([
      scanWithClamAV(filePath),
      scanWithLMD(filePath)
    ]);

    results.scans.clamav = clamavResult;
    results.scans.lmd = lmdResult;

    // Clean up the uploaded file
    await fs.unlink(filePath);

    res.json(results);
  } catch (error) {
    console.error('Scan error:', error);
    res.status(500).json({ error: 'Scan failed' });
  }
});

// Health check endpoint
app.get('/health', (req, res) => {
  res.json({ status: 'ok' });
});

// Start server
ensureUploadsDir().then(() => {
  app.listen(port, () => {
    console.log(`AV Scanner API listening on port ${port}`);
  });
}); 