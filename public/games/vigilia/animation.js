import { ATLAS } from './animation-atlas.js?v=13';

export const RUN_FRAMES = 7;
export const RUN_CYCLE_SECONDS = 1.05;
export const POSES = Object.freeze({ idle: 7, rise: 8, apex: 9, fall: 10, land: 11, crouch: 12, crouchFire: 13 });
export const PLAYER_HEIGHT = 67;
export const STAND_HITBOX = 47;
export const CROUCH_HITBOX = 29;
const ENEMY_FIRE_SECONDS = .36;

const smoothstep = value => {
  const t = Math.max(0, Math.min(1, value));
  return t * t * (3 - 2 * t);
};

export function updatePlayerAnimation(player, dt, moving) {
  const previous = player.animation || { state: 'idle', phase: 0, time: 0 };
  let state;
  if (player.dashTime > 0) state = 'dash';
  else if (player.crouching) state = player.firing ? 'crouchFire' : 'crouch';
  else if (!player.onGround) state = player.vy < -85 ? 'rise' : player.vy > 85 ? 'fall' : 'apex';
  else if (moving && Math.abs(player.vx) > 8) state = 'run';
  else if (player.land > 0) state = 'land';
  else state = 'idle';
  const changed = state !== previous.state;
  const runSpeed = Math.max(.35, Math.min(1, Math.abs(player.vx) / 252));
  const phase = state === 'run'
    ? ((changed ? 0 : previous.phase) + dt * runSpeed / RUN_CYCLE_SECONDS) % 1
    : 0;
  player.animation = {
    state, phase, time: changed ? 0 : previous.time + dt,
    frame: state === 'run' ? Math.floor(phase * RUN_FRAMES) : state === 'dash' ? POSES.apex : POSES[state]
  };
  return player.animation;
}

export function characterAtlas(id) { return ATLAS[id] || ATLAS.kaia; }
export function frameGeometry(id, frame) {
  const atlas = characterAtlas(id);
  return { atlas, frame: atlas.frames[frame] || atlas.frames[POSES.idle] };
}

// The renderer and projectile simulation share the exact same sprite registration.
export function playerMuzzle(player, character, facing = player.facing) {
  const { atlas, frame } = frameGeometry(character, player.animation?.frame ?? POSES.idle);
  const scale = PLAYER_HEIGHT / atlas.referenceHeight;
  return {
    x: player.x + player.w / 2 + (frame.muzzle[0] - frame.anchor[0]) * scale * facing,
    y: player.y + player.h + (frame.muzzle[1] - frame.anchor[1]) * scale
  };
}

export function enemyFrame(enemy) {
  if (enemy.fireTime > 0) {
    const progress = 1 - Math.max(0, Math.min(1, enemy.fireTime / ENEMY_FIRE_SECONDS));
    return 10 + smoothstep(progress) * 3;
  }
  if (enemy.active && enemy.shootCd >= 0 && enemy.shootCd < .36) {
    return 7 + smoothstep(1 - enemy.shootCd / .36) * 2;
  }
  if (enemy.type !== 'turret' && enemy.onGround === false && Number.isFinite(enemy.vy)) {
    if (enemy.vy < -110) return 2.25;
    if (enemy.vy > 110) return 5.25;
    return 3.6;
  }
  if (enemy.type === 'turret' || Math.abs(enemy.vx || 0) < 5) return 7;
  return ((enemy.walkPhase || 0) * RUN_FRAMES) % RUN_FRAMES;
}

export function bossFrame(boss) {
  if (boss.fireTime > 0) {
    const progress = 1 - Math.max(0, Math.min(1, boss.fireTime / ENEMY_FIRE_SECONDS));
    return 3 + smoothstep(progress) * 3;
  }
  if (boss.telegraph > 0) return smoothstep(boss.telegraph) * 3;
  const cycle = ((boss.anim || 0) / 1.15) % 2;
  return (cycle <= 1 ? cycle : 2 - cycle) * 2;
}
