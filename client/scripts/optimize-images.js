import imagemin from 'imagemin';
import imageminMozjpeg from 'imagemin-mozjpeg';
import imageminPngquant from 'imagemin-pngquant';
import imageminGifsicle from 'imagemin-gifsicle';
import imageminSvgo from 'imagemin-svgo';
import path from 'path';
import { fileURLToPath } from 'url';
import fs from 'fs';

// Get the equivalent of __dirname in ES modules
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Ensure the optimized directory exists
const optimizedDir = path.join(__dirname, '../src/assets/optimized');
if (!fs.existsSync(optimizedDir)) {
    fs.mkdirSync(optimizedDir, { recursive: true });
}

// Optimize images
(async () => {
    try {
        const files = await imagemin(['src/assets/*.{jpg,jpeg,png,gif,svg}'], {
            destination: 'src/assets/optimized',
            plugins: [
                imageminMozjpeg({
                    quality: 65,
                    progressive: true
                }),
                imageminPngquant({
                    quality: [0.65, 0.90],
                    speed: 4
                }),
                imageminGifsicle({
                    interlaced: false
                }),
                imageminSvgo({
                    plugins: [
                        { name: 'removeViewBox', active: false },
                        { name: 'cleanupIDs', active: false }
                    ]
                })
            ]
        });

        console.log('Images optimized successfully!');
        console.log('Optimized files:', files.map(file => path.basename(file.destinationPath)));
    } catch (error) {
        console.error('Error optimizing images:', error);
        process.exit(1);
    }
})(); 