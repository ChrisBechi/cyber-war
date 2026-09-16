import test from 'node:test';
import assert from 'node:assert/strict';
import { GameWorld, WEAPONS } from '../../../public/games/vigilia/engine.js';
import { ATLAS } from '../../../public/games/vigilia/animation-atlas.js';
import { CHARACTERS } from '../../../public/games/vigilia/characters.js';
import { POSES, updatePlayerAnimation } from '../../../public/games/vigilia/animation.js';
import { Renderer } from '../../../public/games/vigilia/render.js';
import { readPng } from '../scripts/prepare-animation-atlas.mjs';
const step = (w, n, input = {}) => {
  for (let i = 0; i < n; i++) w.update(1 / 60, input);
};
const world = () => {
  const w = new GameWorld();
  w.enemies = [];
  w.crates = [];
  step(w, 40);
  return w;
};

test('Running cycles through all seven poses and immediately returns to idle on release', () => {
  const w = world(),
    seen = new Set();
  for (let i = 0; i < 90; i++) {
    w.update(1 / 60, { right: true });
    seen.add(w.player.animation.frame);
  }
  assert.deepEqual([...seen].sort(), [0, 1, 2, 3, 4, 5, 6]);
  w.update(1 / 60, {});
  assert.equal(w.player.animation.state, 'idle');
  assert.equal(w.player.animation.frame, POSES.idle);
  assert.equal(w.player.vx, 0);
  const p = { onGround: true, vx: 252, animation: { state: 'run', phase: 0.999, time: 1 } };
  updatePlayerAnimation(p, 1 / 60, true);
  assert.equal(p.animation.frame, 0);
});
test('Jump uses rise, apex, fall and landing poses before returning to standing', () => {
  const w = world(),
    seen = new Set();
  w.update(1 / 60, { jump: true });
  assert.equal(w.player.animation.state, 'rise');
  for (let i = 0; i < 80; i++) {
    w.update(1 / 60);
    seen.add(w.player.animation.state);
  }
  for (const s of ['rise', 'apex', 'fall', 'land', 'idle']) assert.ok(seen.has(s), s);
});
test('Crouching holds the feet still, lowers the hitbox and weapon, and cannot stand into a ceiling', () => {
  const w = world(),
    p = w.player,
    feet = p.y + p.h,
    standing = w.muzzlePoint(0);
  w.update(1 / 60, { crouch: true });
  assert.equal(p.animation.state, 'crouch');
  assert.equal(p.h, 29);
  assert.equal(p.y + p.h, feet);
  assert.ok(w.muzzlePoint(0).y > standing.y + 10);
  w.platforms.push({ x: p.x - 10, y: feet - 48, w: 70, h: 10 });
  w.update(1 / 60, {});
  assert.ok(p.crouching);
  assert.equal(p.h, 29);
  w.update(1 / 60, { jump: true });
  assert.ok(p.onGround);
  w.platforms.pop();
  w.update(1 / 60);
  assert.equal(p.h, 47);
  assert.equal(p.y + p.h, feet);
  assert.equal(p.animation.state, 'idle');
});
test('Muzzle follows the rendered sprite registration for every character, pose and direction', () => {
  for (const c of CHARACTERS) {
    const w = world();
    w.profile.character = c.id;
    const p = w.player;
    for (let pose = 0; pose < 14; pose++)
      for (const facing of [-1, 1]) {
        p.animation = { frame: pose };
        p.facing = facing;
        p.aim = facing === 1 ? 0 : Math.PI;
        p.shotCd = 0;
        p.ammo = WEAPONS[0].mag;
        w.bullets = [];
        w.shoot({});
        const bullet = w.bullets[0];
        let tx, ty, flip, draw;
        const ctx = {
          save() {},
          restore() {},
          translate(x, y) {
            tx = x;
            ty = y;
          },
          scale(x) {
            flip = x;
          },
          drawImage(...args) {
            draw = args;
          },
        };
        Renderer.prototype.drawAgent.call(
          {
            agentImage() {
              return {};
            },
          },
          ctx,
          c.id,
          'ivory',
          pose,
          p.x,
          p.y + p.h - 67,
          p.w,
          67,
          facing === -1,
        );
        const f = ATLAS[c.id].frames[pose],
          sx = draw[1],
          sy = draw[2],
          scale = draw[7] / draw[3];
        const visual = {
          x: tx + flip * (draw[5] + (f.muzzle[0] - sx) * scale),
          y: ty + draw[6] + (f.muzzle[1] - sy) * scale,
        };
        assert.ok(Math.abs(bullet.x - visual.x) < 1e-7);
        assert.ok(Math.abs(bullet.y - visual.y) < 1e-7);
      }
  }
});
test('Packaged atlases have real alpha and distinct nonempty frames inside their images', () => {
  for (const [id, atlas] of Object.entries(ATLAS)) {
    const png = readPng(new URL('../../../public/games/vigilia/' + atlas.file, import.meta.url)),
      { w, h, pixels } = png;
    let transparent = 0;
    for (let i = 3; i < pixels.length; i += 4) if (pixels[i] === 0) transparent++;
    assert.ok(transparent > w * h * 0.35, id + ' transparent background');
    assert.equal(pixels[3], 0);
    assert.equal(atlas.frames.length, ['drone', 'boss'].includes(id) ? 7 : 14);
    const cells = new Set();
    for (const f of atlas.frames) {
      const [x, y, width, height] = f.rect;
      assert.ok(x >= 0 && y >= 0 && x + width <= w && y + height <= h);
      assert.ok(width > 20 && height > 20);
      cells.add(f.rect.join(','));
    }
    assert.equal(cells.size, atlas.frames.length);
  }
});
