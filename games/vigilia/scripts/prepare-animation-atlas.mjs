import fs from 'node:fs';
import zlib from 'node:zlib';
import path from 'node:path';

// Decode generated PNGs, then register sprite cells. No resampling of the artwork.
export function readPng(file) {
  const b = fs.readFileSync(file),
    parts = [];
  let w, h, type, depth, interlace;
  for (let off = 8; off < b.length;) {
    const n = b.readUInt32BE(off),
      kind = b.toString('ascii', off + 4, off + 8),
      d = b.subarray(off + 8, off + 8 + n);
    off += n + 12;
    if (kind === 'IHDR') {
      w = d.readUInt32BE(0);
      h = d.readUInt32BE(4);
      depth = d[8];
      type = d[9];
      interlace = d[12];
    }
    if (kind === 'IDAT') parts.push(d);
  }
  if (depth !== 8 || ![2, 6].includes(type) || interlace)
    throw Error('Expected non-interlaced RGB/RGBA PNG');
  const channels = type === 6 ? 4 : 3,
    stride = w * channels,
    raw = zlib.inflateSync(Buffer.concat(parts)),
    pixels = new Uint8Array(w * h * 4);
  let pos = 0,
    prev = new Uint8Array(stride);
  for (let y = 0; y < h; y++) {
    const filter = raw[pos++],
      row = new Uint8Array(stride);
    for (let x = 0; x < stride; x++) {
      const a = x >= channels ? row[x - channels] : 0,
        c = x >= channels ? prev[x - channels] : 0,
        v = prev[x];
      let pred = 0;
      if (filter === 1) pred = a;
      if (filter === 2) pred = v;
      if (filter === 3) pred = Math.floor((a + v) / 2);
      if (filter === 4) {
        const p = a + v - c,
          pa = Math.abs(p - a),
          pb = Math.abs(p - v),
          pc = Math.abs(p - c);
        pred = pa <= pb && pa <= pc ? a : pb <= pc ? v : c;
      }
      row[x] = (raw[pos++] + pred) & 255;
    }
    for (let x = 0; x < w; x++) {
      const i = (y * w + x) * 4,
        j = x * channels;
      pixels.set([row[j], row[j + 1], row[j + 2], channels === 4 ? row[j + 3] : 255], i);
    }
    prev = row;
  }
  return { w, h, pixels };
}
const crcTable = Uint32Array.from({ length: 256 }, (_, n) => {
  for (let i = 0; i < 8; i++) n = n & 1 ? 0xedb88320 ^ (n >>> 1) : n >>> 1;
  return n >>> 0;
});
function chunk(kind, data) {
  const name = Buffer.from(kind),
    payload = Buffer.concat([name, data]);
  let crc = 0xffffffff;
  for (const n of payload) crc = crcTable[(crc ^ n) & 255] ^ (crc >>> 8);
  const out = Buffer.alloc(data.length + 12);
  out.writeUInt32BE(data.length);
  payload.copy(out, 4);
  out.writeUInt32BE((crc ^ 0xffffffff) >>> 0, out.length - 4);
  return out;
}
export function writePng(file, { w, h, pixels }) {
  const head = Buffer.alloc(13);
  head.writeUInt32BE(w);
  head.writeUInt32BE(h, 4);
  head[8] = 8;
  head[9] = 6;
  const raw = Buffer.alloc((w * 4 + 1) * h);
  for (let y = 0; y < h; y++)
    raw.set(pixels.subarray(y * w * 4, (y + 1) * w * 4), y * (w * 4 + 1) + 1);
  fs.writeFileSync(
    file,
    Buffer.concat([
      Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
      chunk('IHDR', head),
      chunk('IDAT', zlib.deflateSync(raw, { level: 9 })),
      chunk('IEND', Buffer.alloc(0)),
    ]),
  );
}

function extractAlpha(img, rows) {
  const { w, h, pixels: p } = img;
  let transparent = 0;
  for (let i = 3; i < p.length; i += 4) if (p[i] === 0) transparent++;
  if (transparent > w * h * 0.3) return;
  // Flood only the neutral bright matte, preserving the dark outlines and ivory fabric.
  const visited = new Uint8Array(w * h),
    queue = new Int32Array(w * h);
  let end = 0;
  const candidate = (n) => {
    const i = n * 4,
      r = p[i],
      g = p[i + 1],
      b = p[i + 2];
    return Math.min(r, g, b) > 130 && Math.max(r, g, b) - Math.min(r, g, b) < 28;
  };
  const add = (n) => {
    if (n >= 0 && n < w * h && !visited[n] && candidate(n)) {
      visited[n] = 1;
      queue[end++] = n;
    }
  };
  for (let y = 0; y < h; y++) {
    add(y * w);
    add(y * w + w - 1);
  }
  for (let x = 0; x < w; x++) {
    add(x);
    add((h - 1) * w + x);
  }
  for (let row = 1; row < rows; row++) {
    const y = Math.round((row * h) / rows);
    for (let x = 0; x < w; x++) add(y * w + x);
  }
  for (let start = 0; start < end; start++) {
    const n = queue[start],
      x = n % w;
    add(n - w);
    add(n + w);
    if (x > 0) add(n - 1);
    if (x < w - 1) add(n + 1);
  }
  // Enclosed checkerboard gaps are neutral regions with alternating light/dark tiles.
  for (let n = 0; n < w * h; n++) {
    if (visited[n] || !candidate(n)) continue;
    let tail = 0,
      head = 0,
      min = 255,
      max = 0;
    queue[tail++] = n;
    visited[n] = 2;
    while (head < tail) {
      const k = queue[head++],
        x = k % w;
      min = Math.min(min, p[k * 4]);
      max = Math.max(max, p[k * 4]);
      for (const next of [k - w, k + w, x > 0 ? k - 1 : -1, x < w - 1 ? k + 1 : -1])
        if (next >= 0 && next < w * h && !visited[next] && candidate(next)) {
          visited[next] = 2;
          queue[tail++] = next;
        }
    }
    if (tail > 30 && max - min > 34) for (let i = 0; i < tail; i++) visited[queue[i]] = 1;
  }
  for (let n = 0; n < w * h; n++)
    if (visited[n] === 1) {
      p[n * 4] = 0;
      p[n * 4 + 1] = 0;
      p[n * 4 + 2] = 0;
      p[n * 4 + 3] = 0;
    }
}

function packSprites(img, rows) {
  const { w, h, pixels: p } = img,
    labels = new Int32Array(w * h),
    queue = new Int32Array(w * h),
    parts = [];
  for (let n = 0; n < w * h; n++) {
    if (labels[n] || p[n * 4 + 3] < 24) continue;
    const label = parts.length + 1;
    let head = 0,
      tail = 1,
      left = w,
      right = 0,
      top = h,
      bottom = 0;
    queue[0] = n;
    labels[n] = label;
    while (head < tail) {
      const k = queue[head++],
        x = k % w,
        y = Math.floor(k / w);
      left = Math.min(left, x);
      right = Math.max(right, x);
      top = Math.min(top, y);
      bottom = Math.max(bottom, y);
      for (let dy = -1; dy <= 1; dy++)
        for (let dx = -1; dx <= 1; dx++) {
          const xx = x + dx,
            yy = y + dy;
          if (xx < 0 || xx >= w || yy < 0 || yy >= h) continue;
          if (rows === 4 && y > h * 0.76 && Math.floor((xx * 7) / w) !== Math.floor((x * 7) / w))
            continue;
          const next = yy * w + xx;
          if (!labels[next] && p[next * 4 + 3] >= 24) {
            labels[next] = label;
            queue[tail++] = next;
          }
        }
    }
    parts.push({
      label,
      area: tail,
      left,
      right,
      top,
      bottom,
      cx: (left + right) / 2,
      cy: (top + bottom) / 2,
    });
  }
  const major = [...parts].sort((a, b) => b.area - a.area).slice(0, rows * 7);
  if (major.length !== rows * 7 || major.at(-1).area < 1000)
    throw Error('Could not isolate all animation poses');
  major.sort((a, b) => a.cy - b.cy);
  const ordered = [];
  for (let row = 0; row < rows; row++)
    ordered.push(...major.slice(row * 7, row * 7 + 7).sort((a, b) => a.cx - b.cx));
  const owners = new Map(ordered.map((part, i) => [part.label, i]));
  for (const part of parts) {
    if (owners.has(part.label) || part.area < 5) continue;
    let best = 0,
      distance = Infinity;
    for (let i = 0; i < ordered.length; i++) {
      const m = ordered[i],
        dx = Math.max(m.left - part.right, part.left - m.right, 0),
        dy = Math.max(m.top - part.bottom, part.top - m.bottom, 0),
        d = dx * dx + dy * dy + 0.01 * ((m.cx - part.cx) ** 2 + (m.cy - part.cy) ** 2);
      if (d < distance) {
        distance = d;
        best = i;
      }
    }
    if (distance < 1600) owners.set(part.label, best);
  }
  const bounds = ordered.map((p) => ({ ...p }));
  for (const p of parts) {
    const i = owners.get(p.label);
    if (i === undefined) continue;
    const b = bounds[i];
    b.left = Math.min(b.left, p.left);
    b.right = Math.max(b.right, p.right);
    b.top = Math.min(b.top, p.top);
    b.bottom = Math.max(b.bottom, p.bottom);
  }
  const cw = Math.max(...bounds.map((b) => b.right - b.left + 1)) + 24,
    ch = Math.max(...bounds.map((b) => b.bottom - b.top + 1)) + 24,
    out = { w: cw * 7, h: ch * rows, pixels: new Uint8Array(cw * 7 * ch * rows * 4) };
  for (let n = 0; n < w * h; n++) {
    const index = owners.get(labels[n]);
    if (index === undefined) continue;
    const b = bounds[index],
      x = (n % w) - b.left + 12 + (index % 7) * cw,
      y = Math.floor(n / w) - b.top + 12 + Math.floor(index / 7) * ch;
    out.pixels.set(p.subarray(n * 4, n * 4 + 4), (y * out.w + x) * 4);
  }
  return out;
}

function framesFor(img, rows, firstRow, rowCount, player) {
  const { w, h, pixels: p } = img,
    frames = [];
  for (let row = firstRow; row < firstRow + rowCount; row++)
    for (let col = 0; col < 7; col++) {
      const x0 = Math.round((col * w) / 7),
        x1 = Math.round(((col + 1) * w) / 7),
        y0 = Math.round((row * h) / rows),
        y1 = Math.round(((row + 1) * h) / rows);
      let left = x1,
        top = y1,
        right = x0,
        bottom = y0;
      const glow = [];
      for (let y = y0; y < y1; y++)
        for (let x = x0; x < x1; x++) {
          const i = (y * w + x) * 4;
          if (p[i + 3] < 40) continue;
          left = Math.min(left, x);
          right = Math.max(right, x);
          top = Math.min(top, y);
          bottom = Math.max(bottom, y);
          if (player && p[i + 1] > 80 && p[i + 1] > p[i] * 1.4 && p[i + 2] > p[i] * 1.3)
            glow.push([x, y]);
        }
      if (right <= left || bottom <= top) throw Error('Empty animation cell ' + row + ':' + col);
      let mx = player ? right : left,
        my = top + (bottom - top) * 0.4;
      if (glow.length) {
        const tip = Math.max(...glow.map((a) => a[0])),
          ys = glow
            .filter((a) => a[0] >= tip - 4)
            .map((a) => a[1])
            .sort((a, b) => a - b);
        mx = tip + 3;
        my = ys[Math.floor(ys.length / 2)];
      }
      frames.push({
        rect: [left, top, right - left + 1, bottom - top + 1],
        anchor: [(left + right) / 2, bottom + 1],
        muzzle: [mx, my],
      });
    }
  const idle = frames[player ? 7 : 0],
    referenceHeight = idle.rect[3];
  if (player) {
    const reach = (referenceHeight * 24) / 67;
    for (let i = 0; i < frames.length; i++) {
      const f = frames[i];
      f.anchor[0] = f.muzzle[0] - reach;
      if (i >= 8 && i <= 10) f.anchor[1] = f.rect[1] + referenceHeight;
    }
  }
  return { referenceHeight, frames };
}

if (process.argv[2]) {
  const [id, source] = process.argv.slice(2),
    rows = id === 'enemies' ? 4 : 2;
  let img = readPng(source);
  extractAlpha(img, rows);
  img = packSprites(img, rows);
  const file = `assets/${id}-animation-v2.png`,
    out = path.resolve('dist', file);
  writePng(out, img);
  const entries =
    id === 'enemies'
      ? {
          trooper: { file, ...framesFor(img, 4, 0, 2, false) },
          drone: { file, ...framesFor(img, 4, 2, 1, false) },
          boss: { file, ...framesFor(img, 4, 3, 1, false) },
        }
      : { [id]: { file, ...framesFor(img, 2, 0, 2, true) } };
  fs.mkdirSync('art-source', { recursive: true });
  fs.writeFileSync(`art-source/${id}-frames.json`, JSON.stringify(entries, null, 2));
  const atlas = {};
  for (const name of ['dante', 'kaia', 'ravi', 'nika', 'enemies']) {
    const f = `art-source/${name}-frames.json`;
    if (fs.existsSync(f)) Object.assign(atlas, JSON.parse(fs.readFileSync(f, 'utf8')));
  }
  fs.writeFileSync(
    'dist/animation-atlas.js',
    '// Generated by scripts/prepare-animation-atlas.mjs. Sprite-space anchors and muzzle coordinates.\nexport const ATLAS = ' +
      JSON.stringify(atlas, null, 2) +
      ';\n',
  );
  let zeros = 0;
  for (let i = 3; i < img.pixels.length; i += 4) if (img.pixels[i] === 0) zeros++;
  console.log(
    JSON.stringify({
      id,
      file,
      width: img.w,
      height: img.h,
      transparentPercent: Math.round((zeros / (img.w * img.h)) * 100),
      entries: Object.fromEntries(Object.entries(entries).map(([k, v]) => [k, v.frames.length])),
    }),
  );
}
