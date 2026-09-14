import fs from 'node:fs';
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {ATLAS} from '../dist/animation-atlas.js';
import {Renderer} from '../dist/render.js';
import {GameWorld} from '../dist/engine.js';
const require=createRequire(process.argv[2]);
const {createCanvas,loadImage}=require('@napi-rs/canvas');
globalThis.matchMedia=()=>({matches:false});globalThis.innerWidth=1280;globalThis.innerHeight=720;
globalThis.document={createElement:()=>createCanvas(1,1)};
const images={key:'assets/key-art.png',env:'assets/environments.png',sprites:'assets/sprites.png',inventory:'assets/inventory.png',...Object.fromEntries(Object.entries(ATLAS).map(([id,a])=>[id,a.file]))};
const assets=Object.fromEntries(await Promise.all(Object.entries(images).map(async([id,file])=>[id,await loadImage('dist/'+file)])));
const canvas=createCanvas(960,540);canvas.parentElement={style:{}};const renderer=new Renderer(canvas,assets);
const sheet=createCanvas(1400,800),ctx=sheet.getContext('2d');ctx.fillStyle='#152b39';ctx.fillRect(0,0,sheet.width,sheet.height);ctx.font='14px Consolas';ctx.imageSmoothingEnabled=false;
const labels=['RUN 1','RUN 2','RUN 3','RUN 4','RUN 5','RUN 6','RUN 7','IDLE','RISE','APEX','FALL','LAND','CROUCH','FIRE'];
for(let row=0;row<4;row++){const id=['dante','kaia','ravi','nika'][row];for(let frame=0;frame<14;frame++){const x=frame*100,y=row*140;ctx.fillStyle='#82b7c0';ctx.fillText(frame===0?id.toUpperCase():labels[frame],x+5,y+18);ctx.fillStyle='#224657';ctx.fillRect(x,y+126,100,1);renderer.drawAgent(ctx,id,row===2?'violet':'ivory',frame,x,y+24,100,102);}}
for(let row=0;row<3;row++){const kind=['trooper','drone','boss'][row],atlas=ATLAS[kind];assert.ok(atlas,kind);renderer.ctx=ctx;for(let f=0;f<7;f++){const x=f*100,y=590+row*65;ctx.fillStyle='#82b7c0';ctx.fillText(kind,x+5,y-3);renderer.actor(kind,f,x+50,y+54,kind==='boss'?47:55);}}
renderer.ctx=canvas.getContext('2d');fs.mkdirSync('art-source/verification',{recursive:true});fs.writeFileSync('art-source/verification/animation-poses.png',sheet.toBuffer('image/png'));
let rendered=0;for(let level=0;level<3;level++)for(const width of [960,600]){
 const world=new GameWorld(level,{profile:{character:['dante','kaia','nika'][level],loadout:{outfit:level===2?'ember':'ivory'}}});renderer.width=width;
 for(let i=0;i<1200;i++){world.player.invuln=10;world.update(1/60,{right:i<900,jump:i%75===0,crouch:i>=900&&i<990,shoot:i%2===0,autoAim:true,pulse:i%180===0});if(i%12===0){renderer.draw(world);rendered++;}}
 world.player.x=world.level.length-700;world.update(1/60);for(let i=0;i<180;i++){world.player.invuln=10;world.update(1/60);if(i%12===0){renderer.draw(world);rendered++;}}
 assert.ok(world.bossStarted);fs.writeFileSync('art-source/verification/level-'+level+'-'+width+'.png',canvas.toBuffer('image/png'));
}
console.log('Rendered '+rendered+' gameplay frames through the real Canvas renderer, including 3 levels, 2 widths, crouch, jumps, bullets and bosses.');
