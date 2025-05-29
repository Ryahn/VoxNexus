require('dotenv').config();
const fs = require('fs').promises;
const path = require('path');

const UPLOADS_DIR = path.join(__dirname, '..', 'uploads');
const MAX_FILE_AGE = 24 * 60 * 60 * 1000; // 24 hours in milliseconds

async function cleanupOldFiles() {
  try {
    // Ensure uploads directory exists
    await fs.mkdir(UPLOADS_DIR, { recursive: true });

    // Read all files in the uploads directory
    const files = await fs.readdir(UPLOADS_DIR);

    const now = Date.now();
    let cleanedCount = 0;

    // Process each file
    for (const file of files) {
      const filePath = path.join(UPLOADS_DIR, file);
      
      try {
        const stats = await fs.stat(filePath);
        const fileAge = now - stats.mtime.getTime();

        // Delete files older than MAX_FILE_AGE
        if (fileAge > MAX_FILE_AGE) {
          await fs.unlink(filePath);
          cleanedCount++;
          console.log(`Deleted old file: ${file}`);
        }
      } catch (error) {
        console.error(`Error processing file ${file}:`, error);
      }
    }

    console.log(`Cleanup completed. Deleted ${cleanedCount} old files.`);
  } catch (error) {
    console.error('Cleanup error:', error);
    process.exit(1);
  }
}

// Run cleanup
cleanupOldFiles();
