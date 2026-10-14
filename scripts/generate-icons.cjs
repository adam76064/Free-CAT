/**
 * generate-icons.js
 * Creates minimal placeholder PNG/ICO icon files for Tauri.
 * Run once: node scripts/generate-icons.js
 *
 * In production, replace with proper branding icons.
 * Tauri needs: icons/32x32.png, 128x128.png, 128x128@2x.png, icon.ico, icon.icns
 */

const fs = require('fs');
const path = require('path');

const iconsDir = path.join(__dirname, '..', 'src-tauri', 'icons');
fs.mkdirSync(iconsDir, { recursive: true });

// Minimal valid 1x1 PNG (44 bytes) — we'll tile it
// This is the smallest valid PNG: 1x1 pixel, purple (#6172f3)
const MIN_PNG_1x1 = Buffer.from([
  0x89,0x50,0x4e,0x47,0x0d,0x0a,0x1a,0x0a, // PNG signature
  0x00,0x00,0x00,0x0d,0x49,0x48,0x44,0x52, // IHDR length + type
  0x00,0x00,0x00,0x01,0x00,0x00,0x00,0x01, // width=1, height=1
  0x08,0x02,0x00,0x00,0x00,0x90,0x77,0x53, // bit depth=8, colortype=2 (RGB), CRC
  0xde,0x00,0x00,0x00,0x0c,0x49,0x44,0x41, // IDAT length + type
  0x54,0x08,0xd7,0x63,0x60,0xd2,0xf3,0x62, // IDAT compressed data (1 purple pixel)
  0xb4,0x00,0x00,0x00,0x76,0x00,0x01,0xf5, // IDAT data + CRC
  0x27,0xac,0x66,0x00,0x00,0x00,0x00,0x49, // IEND
  0x45,0x4e,0x44,0xae,0x42,0x60,0x82       // IEND CRC
]);

// Write placeholder PNGs (same 1x1 image — enough for cargo check to pass)
const pngFiles = ['32x32.png', '128x128.png', '128x128@2x.png', 'icon.png'];
for (const f of pngFiles) {
  fs.writeFileSync(path.join(iconsDir, f), MIN_PNG_1x1);
  console.log('Created:', f);
}

// Minimal valid ICO file (1x1 pixel, 32bpp)
// ICO header + directory entry + BMP data
const icoData = Buffer.alloc(40 + 4 + 6 + 16);
let off = 0;

// ICO Header
icoData.writeUInt16LE(0, off); off += 2;  // reserved
icoData.writeUInt16LE(1, off); off += 2;  // type=1 (ICO)
icoData.writeUInt16LE(1, off); off += 2;  // count=1

// Directory entry
icoData.writeUInt8(1, off); off++;    // width=1
icoData.writeUInt8(1, off); off++;    // height=1
icoData.writeUInt8(0, off); off++;    // color count
icoData.writeUInt8(0, off); off++;    // reserved
icoData.writeUInt16LE(1, off); off += 2;  // planes
icoData.writeUInt16LE(32, off); off += 2; // bit count
icoData.writeUInt32LE(40 + 4, off); off += 4; // size of image data
icoData.writeUInt32LE(22, off); off += 4; // offset to image data (6+16)

// BITMAPINFOHEADER (40 bytes)
const bmpOff = 22;
icoData.writeUInt32LE(40, bmpOff);           // header size
icoData.writeInt32LE(1, bmpOff + 4);         // width
icoData.writeInt32LE(2, bmpOff + 8);         // height (×2 for ICO)
icoData.writeUInt16LE(1, bmpOff + 12);       // planes
icoData.writeUInt16LE(32, bmpOff + 14);      // bit count
// rest is 0 (no compression, size calculated, etc.)

// Pixel data: 1 pixel BGRA = purple (#6172f3)
icoData.writeUInt8(0xf3, bmpOff + 40);  // B
icoData.writeUInt8(0x72, bmpOff + 41);  // G
icoData.writeUInt8(0x61, bmpOff + 42);  // R
icoData.writeUInt8(0xff, bmpOff + 43);  // A

const icoPath = path.join(iconsDir, 'icon.ico');
fs.writeFileSync(icoPath, icoData);
console.log('Created: icon.ico');

// macOS placeholder (just a copy of the 1x1 PNG renamed)
const icnsPath = path.join(iconsDir, 'icon.icns');
fs.writeFileSync(icnsPath, MIN_PNG_1x1);
console.log('Created: icon.icns (placeholder)');

console.log('\nDone! Replace these with proper branding icons before release.');
console.log('Use: npm run tauri icon path/to/1024x1024.png  (generates all sizes)');
