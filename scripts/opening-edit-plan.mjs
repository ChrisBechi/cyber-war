import { readFileSync } from 'node:fs';
export const openingRevision = 4;
export const openingSize = { width: 1600, height: 900 };
const scenes = JSON.parse(
  readFileSync('src/features/opening/scenarios/montage-scenes.json', 'utf8'),
);
// Each take appears once. Simultaneous actions are separately laid-out live applications.
const shots = [
  {
    id: 'login',
    frames: 135,
    source: 'bookends/gameplay-silent.mp4',
    from: 0,
    span: 4.5,
    transition: 'cut',
  },
];
let sourceAt = 0;
for (const [index, scene] of scenes.entries()) {
  shots.push({
    id: scene.id === 'null' ? 'null-got-you' : scene.id,
    scene: scene.id,
    frames: scene.frames,
    source: 'montage/gameplay-silent.mp4',
    from: sourceAt + 0.3,
    span: scene.duration - 0.6,
    transition: scene.transition,
    accent: [3, 7, 11, 15, 19].includes(index),
  });
  sourceAt += scene.duration;
}
// Locked approved finale: original motion, characters, shape and timing; add only the cursor.
shots.push({
  id: 'code-rain-title',
  frames: 660,
  source: 'approved-v3-final.mp4',
  from: 0,
  span: 22,
  transition: 'cut',
  cursor: true,
});
let at = 0;
export const openingEdit = shots.map((shot) => {
  const result = { ...shot, at, duration: shot.frames / 30 };
  at += result.duration;
  return result;
});
export const openingDuration = shots.reduce((sum, shot) => sum + shot.frames, 0) / 30;
const sceneIds = openingEdit.filter((shot) => shot.scene).map((shot) => shot.scene);
if (new Set(sceneIds).size !== sceneIds.length) throw new Error('A montage take was reused.');
if (openingDuration !== 68.1) throw new Error('Keep the approved finale timing unchanged.');
