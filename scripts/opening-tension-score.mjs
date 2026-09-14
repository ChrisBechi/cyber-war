// Original industrial thriller score: a minor-second ostinato, distorted sub pulse,
// metallic percussion and edit-synchronized impacts. No sustained major-chord pads.
export const tensionBpm = 166;
export function tensionScore(rate, duration, shots) {
  const frames = Math.round(rate * duration);
  const output = new Float32Array(frames * 2);
  const echo = new Float32Array(frames);
  const beat = 60 / tensionBpm;
  const final = shots.at(-1).at;
  const interruption = shots.find((shot) => shot.id === 'null-got-you');
  const hits = shots
    .slice(1, -1)
    .filter((shot) => shot.transition !== 'cut' || shot.accent)
    .map((shot) => shot.at);
  const notes = [0, 0, 1, 0, 7, 0, 6, 1, 0, 12, 1, 0, 6, 7, 1, 0];
  const sin = (hz, t) => Math.sin(2 * Math.PI * hz * t);
  const grit = (hz, t) =>
    Math.tanh((sin(hz, t) + 0.5 * sin(hz * 2, t) + 0.28 * sin(hz * 3, t)) * 2.2);
  let seed = 831274,
    previousNoise = 0;
  for (let i = 0; i < frames; i++) {
    const t = i / rate;
    const song = Math.max(0, t - 4.5);
    const quarter = Math.floor(song / beat);
    const phase = song % beat;
    const tick = song % (beat / 2);
    const step = Math.floor(song / (beat / 2));
    const build = Math.min(1, 0.2 + song / 38);
    const intro = t < 4.5;
    const nullMoment = t >= interruption.at && t < interruption.at + interruption.duration;
    const typing = t >= final && t < final + 2.5;
    const end = t > final + 17;
    seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
    const random = seed / 2147483648 - 1;
    const high = (random - previousNoise) * 0.5;
    previousNoise = random;
    const hz = 41.2034 * 2 ** (notes[step % notes.length] / 12);
    const gate = Math.exp(-tick * 19) * Math.min(1, tick * 700);
    const bass = (grit(hz, song) * 0.23 + sin(hz / 2, song) * 0.1) * gate;
    const kick =
      Math.sin(2 * Math.PI * (43 * phase + 3.1 * (1 - Math.exp(-phase * 52)))) *
        Math.exp(-phase * 18) *
        0.74 +
      high * Math.exp(-phase * 270) * 0.07;
    const snare =
      quarter % 2 === 1 ? (high * 0.5 + sin(178, phase) * 0.19) * Math.exp(-phase * 28) : 0;
    const hatPhase = song % (beat / (build > 0.6 ? 4 : 2));
    const hat = high * Math.exp(-hatPhase * 170) * (0.11 + build * 0.12);
    const tom =
      quarter % 8 >= 6
        ? sin(82 - 24 * (tick / beat), tick) * Math.exp(-tick * 22) * build * 0.18
        : 0;
    const tone = hz * (step % 4 === 3 ? 8 : 4);
    const fm = Math.sin(
      2 * Math.PI * tone * song + 2.6 * Math.exp(-tick * 18) * sin(tone * 1.498, song),
    );
    const stab = fm * Math.exp(-tick * 30) * (0.07 + build * 0.12);
    echo[i] = stab;
    const delayed = i > rate * beat * 0.75 ? echo[i - Math.round(rate * beat * 0.75)] * 0.35 : 0;
    // Short dissonant bow/noise accents create unease without a tranquil chord bed.
    const strain =
      (sin(164.81, t) + sin(174.61, t + 0.007)) * 0.013 * Math.max(0, Math.sin(song * 0.7));
    let impacts = 0,
      riser = 0;
    for (const at of hits) {
      const age = t - at;
      if (age >= 0 && age < 0.48)
        impacts += (sin(48, age) * 0.3 + high * 0.16) * Math.exp(-age * 12);
      if (age > -0.3 && age < 0) riser += high * 0.11 * ((age + 0.3) / 0.3) ** 2;
    }
    const heartbeatPhase = t % 0.72;
    const heartbeat = sin(49, heartbeatPhase) * Math.exp(-heartbeatPhase * 24) * 0.24;
    let drums = kick + snare + hat + tom;
    let gain = 0.76 + build * 0.2;
    if (intro) {
      drums = heartbeat;
      gain = 0.75;
      riser += high * (t / 4.5) ** 5 * 0.16;
    }
    if (nullMoment) {
      drums = heartbeat * 0.45;
      gain = 0.3;
    }
    if (typing) {
      drums = 0;
      gain = 0.15;
    }
    if (end) {
      drums *= Math.max(0, 1 - (t - final - 17) / 1.4);
      gain *= 0.65;
    }
    const duck = 0.4 + 0.6 * (1 - Math.exp(-phase * 23));
    const mono = drums + bass * duck + impacts + riser;
    const fade = Math.min(1, t / 0.08, Math.max(0, (duration - 0.12 - t) / 1.7));
    output[i * 2] = Math.tanh((mono + stab * duck + delayed * 0.4 + strain) * 1.6) * gain * fade;
    output[i * 2 + 1] =
      Math.tanh((mono + stab * duck * 0.72 + delayed + strain * 0.72) * 1.6) * gain * fade;
  }
  return output;
}
