import { deflateSync } from 'zlib';
import { writeFileSync, mkdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

function distToSegment(px, py, x1, y1, x2, y2) {
  const dx = x2 - x1, dy = y2 - y1;
  const lenSq = dx * dx + dy * dy;
  if (lenSq === 0) return Math.hypot(px - x1, py - y1);
  const t = Math.max(0, Math.min(1, ((px - x1) * dx + (py - y1) * dy) / lenSq));
  return Math.hypot(px - (x1 + t * dx), py - (y1 + t * dy));
}

function generateTriangleIcon(size) {
  const padding = Math.max(3, Math.floor(size * 0.12));
  const base = size - 2 * padding;
  const triH = base * Math.sqrt(3) / 2;

  const cx = size / 2, cy = size / 2;

  // Vertices: top, bottom-left, bottom-right
  const ax = cx,            ay = cy - triH / 2;
  const bx = cx - base / 2, by = cy + triH / 2;
  const dx = cx + base / 2, dy = cy + triH / 2;

  const lineHalf = Math.max(0.6, size * 0.016);

  const pixels = Buffer.alloc(size * size * 4, 0);
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const px = x + 0.5, py = y + 0.5;
      const minDist = Math.min(
        distToSegment(px, py, ax, ay, bx, by),
        distToSegment(px, py, bx, by, dx, dy),
        distToSegment(px, py, dx, dy, ax, ay),
      );
      let alpha = 0;
      if (minDist <= lineHalf - 0.5) alpha = 255;
      else if (minDist <= lineHalf + 0.5) alpha = Math.round(255 * (lineHalf + 0.5 - minDist));
      if (alpha > 0) {
        const i = (y * size + x) * 4;
        pixels[i] = pixels[i + 1] = pixels[i + 2] = 255;
        pixels[i + 3] = alpha;
      }
    }
  }
  return pixels;
}

// --- PNG encoder ---

function crc32(buf) {
  let c = 0xFFFFFFFF;
  for (let i = 0; i < buf.length; i++) {
    c ^= buf[i];
    for (let j = 0; j < 8; j++) c = (c & 1) ? ((c >>> 1) ^ 0xEDB88320) : (c >>> 1);
  }
  return (c ^ 0xFFFFFFFF) >>> 0;
}

function chunk(type, data) {
  const t = Buffer.from(type, 'ascii');
  const len = Buffer.alloc(4); len.writeUInt32BE(data.length);
  const crc = Buffer.alloc(4); crc.writeUInt32BE(crc32(Buffer.concat([t, data])));
  return Buffer.concat([len, t, data, crc]);
}

function encodePNG(pixels, size) {
  const sig = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(size, 0); ihdr.writeUInt32BE(size, 4);
  ihdr[8] = 8; ihdr[9] = 6; // 8-bit RGBA

  const stride = size * 4 + 1;
  const raw = Buffer.alloc(size * stride);
  for (let y = 0; y < size; y++) {
    raw[y * stride] = 0; // filter: none
    pixels.copy(raw, y * stride + 1, y * size * 4, (y + 1) * size * 4);
  }

  return Buffer.concat([sig, chunk('IHDR', ihdr), chunk('IDAT', deflateSync(raw)), chunk('IEND', Buffer.alloc(0))]);
}

// --- ICO encoder ---

function encodeICO(entries) {
  const hdr = Buffer.alloc(6);
  hdr.writeUInt16LE(1, 2); hdr.writeUInt16LE(entries.length, 4);
  let offset = 6 + entries.length * 16;
  const dirs = entries.map(({ size, data }) => {
    const e = Buffer.alloc(16);
    e[0] = size < 256 ? size : 0; e[1] = e[0];
    e.writeUInt16LE(1, 4); e.writeUInt16LE(32, 6);
    e.writeUInt32LE(data.length, 8); e.writeUInt32LE(offset, 12);
    offset += data.length;
    return e;
  });
  return Buffer.concat([hdr, ...dirs, ...entries.map(e => e.data)]);
}

// --- Generate ---

const __dirname = dirname(fileURLToPath(import.meta.url));
const dir = join(__dirname, 'icons');
mkdirSync(dir, { recursive: true });

const pngSpecs = [
  { size: 32,  file: '32x32.png' },
  { size: 128, file: '128x128.png' },
  { size: 256, file: '128x128@2x.png' },
  { size: 256, file: '256x256.png' },
];

for (const { size, file } of pngSpecs) {
  const png = encodePNG(generateTriangleIcon(size), size);
  writeFileSync(join(dir, file), png);
  console.log(`${file}: ${png.length} bytes`);
}

const icoEntries = [16, 24, 32, 48, 64, 128, 256].map(size => ({
  size,
  data: encodePNG(generateTriangleIcon(size), size),
}));
const ico = encodeICO(icoEntries);
writeFileSync(join(dir, 'icon.ico'), ico);
console.log(`icon.ico: ${ico.length} bytes (${icoEntries.length} sizes)`);
