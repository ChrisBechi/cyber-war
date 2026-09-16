import { clamp, WEAPONS } from './engine.js?v=13';
import { getCharacter } from './characters.js?v=13';
import { ATLAS } from './animation-atlas.js?v=13';
import { POSES, frameGeometry, enemyFrame, bossFrame, PLAYER_HEIGHT } from './animation.js?v=13';
export const CHARACTER_FRAMES = Object.fromEntries(
  ['dante', 'kaia', 'ravi', 'nika'].map((id) => [id, ATLAS[id].frames.map((f) => f.rect)]),
);
export const SPRITES = {
  hero: [
    [85, 43, 185, 263],
    [397, 43, 203, 263],
    [706, 44, 221, 262],
    [1042, 45, 184, 261],
  ],
  drone: [
    [50, 378, 212, 189],
    [373, 369, 202, 198],
    [683, 378, 207, 189],
    [1001, 369, 201, 198],
  ],
  robot: [
    [48, 641, 212, 273],
    [363, 640, 224, 274],
    [678, 641, 213, 273],
    [997, 640, 209, 274],
  ],
  boss: [[3, 937, 347, 254]],
  floor: [[361, 956, 225, 225]],
  crate: [[692, 1006, 188, 147]],
  crystal: [[1039, 989, 117, 175]],
};
export const INVENTORY = {
  ivory: [45, 80, 304, 259],
  ember: [428, 80, 305, 259],
  violet: [811, 80, 305, 259],
  scarf: [1241, 97, 249, 261],
  visor: [55, 473, 310, 170],
  drone: [460, 423, 250, 249],
  medkit: [822, 432, 281, 221],
  shield: [1215, 441, 266, 213],
  carbine: [33, 772, 337, 168],
  shotgun: [404, 787, 342, 137],
  plasma: [786, 780, 350, 143],
  railgun: [1162, 790, 352, 121],
};
export async function loadAssets() {
  const out = {},
    files = {
      key: 'assets/key-art.png',
      env: 'assets/environments.png',
      sprites: 'assets/sprites.png',
      inventory: 'assets/inventory.png',
      ...Object.fromEntries(Object.entries(ATLAS).map(([id, a]) => [id, a.file])),
    },
    cache = new Map();
  await Promise.all(
    Object.entries(files).map(async ([key, url]) => {
      if (!cache.has(url))
        cache.set(
          url,
          new Promise((resolve, reject) => {
            const image = new Image();
            image.onload = () => resolve(image);
            image.onerror = () => reject(new Error('Não foi possível carregar ' + url));
            image.src = url + '?v=9';
          }),
        );
      out[key] = await cache.get(url);
    }),
  );
  return out;
}
export class Renderer {
  constructor(canvas, assets) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d', { alpha: false });
    this.assets = assets;
    this.width = 960;
    this.height = 540;
    this.reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;
    this.heroVariants = {};
    this.makeVariants();
    this.resize();
  }
  resize() {
    this.width = innerWidth < 600 && innerHeight > innerWidth ? 600 : 960;
    this.canvas.width = this.width;
    this.canvas.height = 540;
    this.canvas.parentElement.style.aspectRatio = this.width + '/540';
    this.canvas.parentElement.style.width = 'min(100vw, ' + (100 * this.width) / 540 + 'svh)';
    this.ctx.imageSmoothingEnabled = false;
  }
  makeVariants() {
    this.heroVariants = {};
  }
  agentImage(character, outfit) {
    const id = getCharacter(character).id,
      original = this.assets[id];
    if (!['ember', 'violet'].includes(outfit)) return original;
    const key = id + ':' + outfit;
    if (this.heroVariants[key]) return this.heroVariants[key];
    const c = document.createElement('canvas');
    c.width = original.width;
    c.height = original.height;
    const x = c.getContext('2d');
    x.drawImage(original, 0, 0);
    const pixels = x.getImageData(0, 0, c.width, c.height),
      d = pixels.data,
      tint = outfit === 'ember' ? [226, 152, 72] : [171, 134, 223];
    for (let i = 0; i < d.length; i += 4) {
      const r = d[i],
        g = d[i + 1],
        b = d[i + 2];
      if (
        d[i + 3] > 0 &&
        r > 90 &&
        g > 85 &&
        b > 70 &&
        Math.max(r, g, b) - Math.min(r, g, b) < 58 &&
        r >= b * 0.98
      ) {
        const lum = (r + g + b) / 570;
        d[i] = clamp(tint[0] * lum, 0, 255);
        d[i + 1] = clamp(tint[1] * lum, 0, 255);
        d[i + 2] = clamp(tint[2] * lum, 0, 255);
      }
    }
    x.putImageData(pixels, 0, 0);
    return (this.heroVariants[key] = c);
  }
  drawAgent(c, character, outfit, frame, x, y, w, h, flip = false, alpha = 1) {
    const { atlas, frame: pose } = frameGeometry(character, frame),
      scale = h / atlas.referenceHeight,
      [sx, sy, sw, sh] = pose.rect;
    c.save();
    c.globalAlpha = alpha;
    c.translate(x + w / 2, y + h);
    c.scale(flip ? -1 : 1, 1);
    c.drawImage(
      this.agentImage(character, outfit),
      sx,
      sy,
      sw,
      sh,
      (sx - pose.anchor[0]) * scale,
      (sy - pose.anchor[1]) * scale,
      sw * scale,
      sh * scale,
    );
    c.restore();
  }
  actor(kind, frame, x, y, h, flip = false) {
    const atlas = ATLAS[kind],
      frames = atlas?.frames,
      image = this.assets?.[kind];
    if (!frames?.length || !image) return;
    const count = frames.length,
      raw = Number.isFinite(frame) ? frame : 0,
      value = ((raw % count) + count) % count,
      first = Math.floor(value),
      blend = value - first,
      draw = (index, alpha) => {
        const f = frames[index],
          scale = h / atlas.referenceHeight,
          [sx, sy, sw, sh] = f.rect,
          c = this.ctx;
        c.save();
        c.globalAlpha = alpha;
        c.translate(x, y);
        c.scale(flip ? -1 : 1, 1);
        c.drawImage(
          image,
          sx,
          sy,
          sw,
          sh,
          (sx - f.anchor[0]) * scale,
          (sy - f.anchor[1]) * scale,
          sw * scale,
          sh * scale,
        );
        c.restore();
      };
    draw(first, 1 - blend);
    if (blend > 0.001) draw((first + 1) % count, blend);
  }

  sprite(kind, frame, x, y, w, h, flip = false, alpha = 1, variant = null) {
    const c = this.ctx,
      frames = SPRITES[kind];
    if (!frames?.length) return;
    const rect = frames[((frame % frames.length) + frames.length) % frames.length];
    c.save();
    c.globalAlpha = alpha;
    c.translate(Math.round(x + w / 2), Math.round(y + h));
    c.scale(flip ? -1 : 1, 1);
    c.drawImage(variant || this.assets.sprites, ...rect, -w / 2, -h, w, h);
    c.restore();
  }
  inventory(id, x, y, w, h, flip = false) {
    const rect = INVENTORY[id];
    if (!rect) return;
    const c = this.ctx;
    c.save();
    c.translate(x + w / 2, y + h / 2);
    c.scale(flip ? -1 : 1, 1);
    c.drawImage(this.assets.inventory, ...rect, -w / 2, -h / 2, w, h);
    c.restore();
  }
  text(text, x, y, size = 12, color = '#c6e5e0', align = 'left') {
    const c = this.ctx;
    c.font = `${size}px Consolas, monospace`;
    c.fillStyle = color;
    c.textAlign = align;
    c.fillText(text, x, y);
  }
  draw(world) {
    const c = this.ctx,
      p = world.player,
      W = this.width,
      H = 540,
      cam = world.cameraX,
      t = world.time,
      color = world.level.color;
    world.viewWidth = W;
    c.imageSmoothingEnabled = false;
    c.fillStyle = '#0b1421';
    c.fillRect(0, 0, W, H);
    if (world.levelIndex === 0) {
      const par = (cam / (world.level.length - W)) * 150;
      c.drawImage(this.assets.key, 0, 235, 975, 680, -par - 25, -20, W + 210, 580);
    } else {
      const sw = 1536,
        sh = 512,
        bw = (H * sw) / sh,
        bx = -(cam / (world.level.length - W)) * (bw - W);
      c.drawImage(this.assets.env, 0, world.levelIndex === 1 ? 0 : 512, sw, sh, bx, 0, bw, H);
    }
    c.fillStyle = world.levelIndex === 1 ? '#17101835' : '#06112045';
    c.fillRect(0, 0, W, H);
    let grad = c.createLinearGradient(0, 0, 0, H);
    grad.addColorStop(0, '#03081010');
    grad.addColorStop(0.6, '#050c1810');
    grad.addColorStop(1, '#030912cc');
    c.fillStyle = grad;
    c.fillRect(0, 0, W, H);
    // Fine weather and atmospheric layers drift at different depths.
    if (!this.reduced) {
      c.strokeStyle = world.levelIndex === 0 ? '#8bbfc226' : '#dcaa7233';
      c.lineWidth = 1;
      for (let i = 0; i < 65; i++) {
        let x =
          ((((i * 137.2 - cam * 0.2 + t * (world.levelIndex ? 8 : -58)) % (W + 100)) + W + 100) %
            (W + 100)) -
          50;
        let y = (i * 91 + t * (world.levelIndex ? 22 : 440)) % H;
        if (world.levelIndex === 0) {
          c.beginPath();
          c.moveTo(x, y);
          c.lineTo(x - 4, y + 19);
          c.stroke();
        } else {
          c.fillStyle = i % 3 ? '#e7b26b44' : color + '77';
          c.fillRect(x, y, 1.4, 1.4);
        }
      }
    }
    const shake = this.reduced ? 0 : world.shake;
    c.save();
    c.translate(-Math.round(cam) + (Math.random() - 0.5) * shake, (Math.random() - 0.5) * shake);
    // In-world signals, maintenance signage, checkpoints and extraction gate.
    for (let x = 280; x < world.level.length; x += 710) {
      if (x < cam - 100 || x > cam + W + 100) continue;
      c.fillStyle = '#071422b8';
      c.fillRect(x, 376, 83, 31);
      c.fillStyle = color + '66';
      c.fillRect(x, 376, 2, 31);
      this.text(
        world.levelIndex === 0
          ? 'HELIX // 09'
          : world.levelIndex === 1
            ? 'CAUTION / Δ'
            : 'NEXUS / NET',
        x + 9,
        389,
        8,
        '#9fb7bc',
      );
      this.text(('000' + Math.round(x / 10)).slice(-4), x + 9, 400, 7, color);
      c.fillStyle = '#213643';
      c.fillRect(x + 38, 407, 5, 53);
    }
    for (const pl of world.platforms) {
      if (pl.x + pl.w < cam || pl.x > cam + W) continue;
      const start = Math.max(pl.x, Math.floor(cam / 48) * 48);
      for (let x = start; x < Math.min(pl.x + pl.w, cam + W + 48); x += 48) {
        const width = Math.min(48, pl.x + pl.w - x);
        c.save();
        c.beginPath();
        c.rect(x, pl.y, width, pl.h);
        c.clip();
        this.sprite('floor', 0, x, pl.y, 48, 48);
        if (pl.h > 48) {
          this.sprite('floor', 0, x, pl.y + 48, 48, 48);
          c.fillStyle = '#03071266';
          c.fillRect(x, pl.y + 35, 48, pl.h);
        }
        c.restore();
      }
      c.fillStyle = pl.y === 460 ? '#82a7a1' : '#b2d8d2';
      c.fillRect(pl.x, pl.y, pl.w, 2);
      c.fillStyle = color + '70';
      c.fillRect(pl.x, pl.y + 3, pl.w, 1);
      if (pl.y !== 460) {
        c.fillStyle = '#02050bb5';
        c.fillRect(pl.x, pl.y + 14, pl.w, 8);
        for (let x = pl.x + 15; x < pl.x + pl.w; x += 56) {
          c.fillStyle = color + '99';
          c.fillRect(x, pl.y + 7, 14, 2);
        }
      }
    }
    for (const hazard of world.level.hazards) {
      if (hazard.x + hazard.w < cam || hazard.x > cam + W) continue;
      c.fillStyle = '#180e20';
      c.fillRect(hazard.x, 455, hazard.w, 8);
      for (let x = hazard.x; x < hazard.x + hazard.w; x += 16) {
        c.fillStyle = hazard.active ? '#ff6d89' : hazard.warning ? '#ffce77' : '#422d43';
        c.fillRect(x, 456, 8, 4);
      }
      if (hazard.active) {
        c.fillStyle = world.levelIndex === 1 ? '#ff79363a' : '#bd64ff40';
        c.fillRect(hazard.x, 426, hazard.w, 34);
        for (let x = hazard.x; x < hazard.x + hazard.w; x += 15) {
          c.strokeStyle = world.levelIndex === 1 ? '#ffcb8599' : '#d4aaff99';
          c.beginPath();
          c.moveTo(x, 460);
          c.lineTo(x + Math.sin(t * 28 + x) * 6, 428 + Math.sin(t * 20 + x) * 9);
          c.stroke();
        }
      }
      if (hazard.warning) this.text('⚠', hazard.x + hazard.w / 2, 442, 15, '#ffbc6c', 'center');
    }
    for (const x of world.level.checkpoints) {
      if (x < cam - 80 || x > cam + W + 80) continue;
      const active = world.checkpointSet.has(x);
      c.fillStyle = '#172a38';
      c.fillRect(x, 407, 18, 53);
      c.fillStyle = active ? color : '#527480';
      c.fillRect(x + 4, 412, 10, 26);
      c.fillStyle = active ? color + '16' : '#17394722';
      c.fillRect(x - 12, 400, 42, 60);
      this.text(active ? 'SALVO' : 'LINK', x + 9, 398, 8, active ? color : '#748c99', 'center');
    }
    for (const crate of world.crates)
      if (crate.alive && crate.x > cam - 80 && crate.x < cam + W + 80)
        this.sprite('crate', 0, crate.x, crate.y, crate.w, crate.h);
    for (const pickup of world.pickups)
      if (pickup.alive && pickup.x > cam - 40 && pickup.x < cam + W + 40) {
        const bob = Math.sin(t * 3 + pickup.x) * 4;
        c.save();
        c.globalAlpha = 0.14;
        c.fillStyle = '#6ef5d0';
        c.beginPath();
        c.ellipse(pickup.x + 8, pickup.y + 22, 16, 5, 0, 0, Math.PI * 2);
        c.fill();
        c.restore();
        this.sprite('crystal', 0, pickup.x, pickup.y - 10 + bob, 18, 27);
      }
    for (const secret of world.level.secrets)
      if (!world.secretSet.has(secret.id) && secret.x > cam - 40 && secret.x < cam + W + 40) {
        c.save();
        c.filter = 'hue-rotate(170deg) saturate(1.5)';
        this.sprite('crystal', 0, secret.x - 10, secret.y - 18 + Math.sin(t * 2) * 5, 20, 30);
        c.restore();
        c.globalAlpha = 0.6 + 0.4 * Math.sin(t * 3);
        this.text('?', secret.x, secret.y - 25, 11, '#ffe3a0', 'center');
        c.globalAlpha = 1;
      }
    const gateX = world.level.length - 85;
    c.fillStyle = '#111d32';
    c.fillRect(gateX - 21, 329, 51, 132);
    c.strokeStyle = world.bossDefeated ? color : '#693549';
    c.lineWidth = 3;
    c.strokeRect(gateX - 19, 331, 47, 128);
    c.fillStyle = world.bossDefeated ? color + '33' : '#58253b22';
    c.fillRect(gateX - 12, 339, 33, 114);
    this.text(
      world.bossDefeated ? 'EXTRAÇÃO' : 'BLOQUEADO',
      gateX + 4,
      315,
      9,
      world.bossDefeated ? color : '#e5879d',
      'center',
    );
    if (world.bossDefeated) {
      for (let y = 341; y < 450; y += 10) {
        c.globalAlpha = 0.35 + 0.2 * Math.sin(t * 4 + y);
        c.fillStyle = color;
        c.fillRect(gateX - 11, y, 31, 2);
      }
      c.globalAlpha = 1;
    }
    for (const g of world.ghosts) {
      const life = clamp(g.life / 0.48, 0, 1);
      if (g.pulse) {
        c.strokeStyle =
          '#7dece9' +
          Math.round(life * 130)
            .toString(16)
            .padStart(2, '0');
        c.lineWidth = 3;
        c.beginPath();
        c.arc(g.x, g.y, (1 - life) * 235, 0, Math.PI * 2);
        c.stroke();
      } else
        this.drawAgent(
          c,
          world.profile.character,
          world.profile.loadout?.outfit,
          g.frame ?? POSES.apex,
          g.x,
          g.y + (g.h ?? 47) - PLAYER_HEIGHT,
          25,
          PLAYER_HEIGHT,
          g.facing === -1,
          g.life * 0.8,
        );
    }
    for (const e of world.enemies) {
      if (!e.alive || e.x < cam - 90 || e.x > cam + W + 90) continue;
      c.save();
      if (e.hurt > 0) c.filter = 'brightness(2.8)';
      if (e.type === 'drone')
        this.actor('drone', (e.anim * 7) / 0.95, e.x + e.w / 2, e.y + e.h + 6, 50, e.facing === 1);
      else this.actor('trooper', enemyFrame(e), e.x + e.w / 2, e.y + e.h, 63, e.facing === 1);
      c.restore();
      if (e.hp < e.maxHp) {
        c.fillStyle = '#162632';
        c.fillRect(e.x - 3, e.y - 22, e.w + 6, 3);
        c.fillStyle = '#ff7296';
        c.fillRect(e.x - 3, e.y - 22, ((e.w + 6) * e.hp) / e.maxHp, 3);
      }
      if (e.shootCd < 0.28) {
        c.fillStyle = '#ffac93';
        c.beginPath();
        c.arc(e.x + e.w / 2, e.y + 15, Math.max(0, 2 + e.shootCd * 4), 0, Math.PI * 2);
        c.fill();
      }
    }
    const boss = world.boss;
    if (boss?.alive) {
      c.save();
      if (boss.hurt > 0) c.filter = 'brightness(2)';
      if (world.levelIndex === 2)
        c.filter = (boss.hurt > 0 ? 'brightness(2) ' : '') + 'hue-rotate(285deg)';
      this.actor('boss', bossFrame(boss), boss.x + boss.w / 2, boss.y + boss.h, 138);
      c.restore();
      const cx = boss.x + boss.w / 2,
        cy = boss.y + boss.h * 0.4;
      let g = c.createRadialGradient(cx, cy, 3, cx, cy, 30 + boss.phase * 5);
      g.addColorStop(0, '#96f6ff77');
      g.addColorStop(1, '#96f6ff00');
      c.fillStyle = g;
      c.fillRect(cx - 50, cy - 50, 100, 100);
      if (boss.telegraph > 0) {
        c.save();
        c.setLineDash([9, 8]);
        c.strokeStyle = `rgba(255,108,147,${boss.telegraph * 0.65})`;
        c.lineWidth = 2;
        c.beginPath();
        c.moveTo(cx, cy);
        c.lineTo(p.x + 12, p.y + 20);
        c.stroke();
        c.restore();
        this.text('CARREGANDO', cx, boss.y - 25, 10, '#ffb2c7', 'center');
      }
    }
    // Frame animation preserves a fixed foot anchor while the agent moves.
    if (p.hp > 0) {
      const frame = p.animation?.frame ?? POSES.idle;
      let alpha = p.invuln > 0.1 && Math.floor(t * 18) % 2 === 0 ? 0.48 : 1;
      c.save();
      if (p.hurt > 0) c.filter = 'brightness(2)';
      this.drawAgent(
        c,
        world.profile.character,
        world.profile.loadout?.outfit,
        frame,
        p.x,
        p.y + p.h - PLAYER_HEIGHT,
        p.w,
        PLAYER_HEIGHT,
        p.facing === -1,
        alpha,
      );
      c.restore();
      if (world.profile.loadout?.accessory === 'scarf') {
        c.save();
        c.translate(p.x + 12, p.y + 7);
        c.rotate(Math.sin(t * 11) * 0.15 - p.vx * 0.001);
        this.inventory('scarf', p.facing === 1 ? -21 : 5, 0, 16, 24, p.facing === -1);
        c.restore();
      }
      if (world.hasCompanion)
        this.inventory('drone', p.x - 38, p.y - 44 + Math.sin(t * 4) * 5, 27, 27);
      if (world.profile.loadout?.equipment === 'shield' && p.invuln > 0) {
        c.strokeStyle = '#77dfff66';
        c.lineWidth = 1.5;
        c.beginPath();
        c.ellipse(p.x + 12, p.y + 21, 29, 40, 0, 0, Math.PI * 2);
        c.stroke();
      }
      if (p.flashTime > 0) {
        const { x: mx, y: my } = world.muzzlePoint(p.aim, p);
        c.save();
        c.translate(mx, my);
        c.rotate(p.aim);
        c.fillStyle = WEAPONS[p.weapon].color;
        c.beginPath();
        c.moveTo(14, 0);
        c.lineTo(-3, -5);
        c.lineTo(0, 0);
        c.lineTo(-3, 5);
        c.closePath();
        c.fill();
        c.restore();
      }
    }
    for (const b of world.bullets) {
      c.strokeStyle = b.color;
      c.lineWidth = b.r > 3 ? 4 : 2;
      c.beginPath();
      c.moveTo(b.x, b.y);
      c.lineTo(b.x - b.vx * 0.015, b.y - b.vy * 0.015);
      c.stroke();
      c.fillStyle = '#edffff';
      c.fillRect(b.x - 1, b.y - 1, 3, 3);
      if (b.splash) {
        c.fillStyle = b.color + '33';
        c.beginPath();
        c.arc(b.x, b.y, 10, 0, Math.PI * 2);
        c.fill();
      }
    }
    for (const b of world.enemyBullets) {
      const radius = Math.max(0, Number.isFinite(b.r) ? b.r : 0);
      c.fillStyle = b.color + '25';
      c.beginPath();
      c.arc(b.x, b.y, radius * 2.5, 0, Math.PI * 2);
      c.fill();
      c.fillStyle = b.color;
      c.beginPath();
      c.arc(b.x, b.y, radius, 0, Math.PI * 2);
      c.fill();
      c.fillStyle = '#ffe5e4';
      c.fillRect(b.x - 1, b.y - 1, 2, 2);
    }
    for (const a of world.particles) {
      c.globalAlpha = clamp(a.life / 0.35, 0, 1);
      c.fillStyle = a.color;
      c.fillRect(Math.round(a.x), Math.round(a.y), a.size, a.size);
    }
    c.globalAlpha = 1;
    for (const d of world.damageNumbers) {
      c.globalAlpha = clamp(d.life / 0.25, 0, 1);
      this.text(d.text, d.x, d.y, d.boss ? 14 : 11, d.boss ? '#ffcc89' : '#9affdd', 'center');
    }
    c.globalAlpha = 1;
    c.restore();
    grad = c.createRadialGradient(W / 2, H / 2, H * 0.27, W / 2, H / 2, W * 0.7);
    grad.addColorStop(0, 'transparent');
    grad.addColorStop(1, '#04081270');
    c.fillStyle = grad;
    c.fillRect(0, 0, W, H);
  }
}
