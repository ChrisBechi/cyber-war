import { updatePlayerAnimation, playerMuzzle, STAND_HITBOX, CROUCH_HITBOX } from './animation.js?v=13';
export const WEAPONS=[
 {id:'carbine',name:'R-09 / CARABINA',mag:24,rate:.125,reload:1.1,damage:16,speed:910,pellets:1,spread:.018,color:'#8cffe0'},
 {id:'shotgun',name:'VÓRTEX / ESPINGARDA',mag:8,rate:.48,reload:1.45,damage:13,speed:760,pellets:5,spread:.19,color:'#ffc786'},
 {id:'plasma',name:'ARCO / PLASMA',mag:18,rate:.19,reload:1.3,damage:29,speed:690,pellets:1,spread:.005,color:'#c3a1ff',splash:42},
 {id:'railgun',name:'HORIZONTE / RAILGUN',mag:5,rate:.72,reload:1.7,damage:85,speed:1450,pellets:1,spread:0,color:'#f2fcff',pierce:true}
];
const platform=(x,y,w,kind='metal')=>({x,y,w,h:y===460?100:22,kind});
export const LEVELS=[
 {name:'DISTRITO CHUVA',chapter:'INFILTRAÇÃO',length:4700,color:'#6ef5d0',boss:'ARACNE / VIGIA DO DISTRITO',bossHp:430,gravity:1450,
 objective:'Alcance a torre de transmissão.',brief:'As ruas ainda escutam. Abra caminho até a torre.',clear:'SINAL RECUPERADO.',story:'A torre voltou a transmitir. O comando das máquinas vem de dentro da Usina Helix.',
 ground:[[0,1100],[1260,2420],[2535,4700]],platforms:[[440,367,170],[810,306,150],[1050,390,255],[1510,363,170],[1710,268,165],[1960,348,190],[2350,376,250],[2860,340,200],[3300,366,180],[3660,300,180],[4210,350,200]],
 enemies:[[650,'trooper'],[920,'drone'],[1460,'trooper'],[1740,'drone'],[2160,'trooper'],[2610,'trooper'],[2820,'drone'],[3160,'trooper'],[3540,'drone'],[3840,'trooper']],
 hazards:[],checkpoints:[1400,2780,3930],secrets:[{x:1775,y:234,id:'enigma',name:'O ENIGMA',text:'Acredite no coração… dos circuitos.'},{x:3430,y:332,id:'rain',name:'LÁGRIMAS NA CHUVA',text:'Todos estes momentos ficaram salvos. Backup concluído.'}]},
 {name:'USINA HELIX',chapter:'SABOTAGEM',length:5400,color:'#ffba72',boss:'CÉRBERO / FORNALHA AUTÔNOMA',bossHp:660,gravity:1450,
 objective:'Desative a fornalha central.',brief:'O calor mantém a rede viva. Interrompa a alimentação.',clear:'FORNALHA SILENCIADA.',story:'Sem energia, Helix perdeu suas defesas externas. O Núcleo Zero está exposto. Esta é a nossa janela.',
 ground:[[0,940],[1060,2000],[2130,3220],[3380,5400]],platforms:[[400,350,240],[850,385,280],[1500,310,210],[1840,366,350],[2450,357,210],[2940,390,340],[3180,320,270],[3600,340,200],[4160,350,220],[4850,348,160]],
 enemies:[[560,'trooper'],[850,'drone'],[1270,'turret'],[1570,'drone'],[1840,'trooper'],[2300,'trooper'],[2580,'drone'],[2890,'turret'],[3470,'trooper'],[3780,'drone'],[4160,'trooper'],[4440,'turret']],
 hazards:[{x:1280,w:145,period:4.6,on:1.5},{x:2650,w:160,period:5.2,on:1.7},{x:3780,w:170,period:5,on:1.7}],checkpoints:[1180,2400,4040],secrets:[{x:1560,y:275,id:'answer',name:'A RESPOSTA É 42',text:'A pergunta continua classificada.'},{x:3250,y:285,id:'cake',name:'O BOLO É REAL',text:'Só não passe da porta de testes.'}]},
 {name:'NÚCLEO ZERO',chapter:'ÚLTIMA TRANSMISSÃO',length:6100,color:'#c9a0ff',boss:'NEXUS / CONSCIÊNCIA CENTRAL',bossHp:1000,gravity:1200,
 objective:'Encontre e derrube a consciência central.',brief:'O núcleo está aprendendo. Não dê tempo para ele evoluir.',clear:'PROTOCOLO ZERO ENCERRADO.',story:'',
 ground:[[0,1150],[1280,2610],[2760,3900],[4020,6100]],platforms:[[380,355,200],[800,286,190],[1040,378,300],[1510,346,170],[1920,290,200],[2370,371,280],[2590,303,250],[3100,343,180],[3510,284,200],[3800,374,300],[4310,330,230],[4650,285,190],[5120,357,170],[5590,333,170]],
 enemies:[[530,'trooper'],[840,'drone'],[1370,'turret'],[1670,'drone'],[2040,'trooper'],[2350,'drone'],[2840,'trooper'],[3130,'turret'],[3420,'drone'],[3740,'trooper'],[4190,'drone'],[4510,'turret'],[4770,'drone'],[5090,'trooper']],
 hazards:[{x:1590,w:170,period:5,on:1.4},{x:3260,w:160,period:4.4,on:1.5},{x:4480,w:160,period:5.1,on:1.7}],checkpoints:[1390,2910,4590,5360],secrets:[{x:1980,y:255,id:'cat',name:'GATO DE SCHRÖDINGER',text:'Ele estava aqui. E não estava.'},{x:4700,y:250,id:'magnate',name:'O ÚLTIMO MAGNATA',text:'Alguns impérios começam com um único pixel.'}]}
];
export const clamp=(v,a,b)=>Math.max(a,Math.min(b,v));
export const overlaps=(a,b)=>a.x<b.x+b.w&&a.x+a.w>b.x&&a.y<b.y+b.h&&a.y+a.h>b.y;
export function rng(seed=9){return()=>{seed=(seed*1664525+1013904223)>>>0;return seed/4294967296;};}

export class GameWorld{
 constructor(levelIndex=0,config={}){
  this.levelIndex=levelIndex;this.level=LEVELS[levelIndex];this.time=0;this.status='playing';this.random=rng(923+levelIndex);this.events=[];this.bullets=[];this.enemyBullets=[];this.particles=[];this.pickups=[];this.ghosts=[];this.damageNumbers=[];this.score=config.score||0;this.kills=config.kills||0;this.stageKills=0;this.totalTime=config.totalTime||0;this.combo=0;this.comboTimer=0;this.cameraX=0;this.viewWidth=960;this.boss=null;this.bossDefeated=false;this.bossStarted=false;this.shake=0;this.hitstop=0;this.lastCheckpoint=80;this.secretsFound=[];this.profile=config.profile||{};
  this.applyLoadout(this.profile,true);
  this.player={x:80,y:395,w:25,h:47,vx:0,vy:0,hp:this.maxHp,maxHp:this.maxHp,facing:1,onGround:false,coyote:0,jumps:0,invuln:1.4,dashTime:0,dashCd:0,pulseCd:0,shotCd:0,reloading:0,reloadTotal:0,weapon:0,ammo:24,aim:0,anim:0,hurt:0,land:0,companionCd:0};
  this.weaponUnlocked=[true,levelIndex>=1,levelIndex>=2,false];
  for(let i=0;i<WEAPONS.length;i++)if(this.profile.owned?.includes(WEAPONS[i].id))this.weaponUnlocked[i]=true;
  const preferred=WEAPONS.findIndex(w=>w.id===this.profile.loadout?.weapon);
  this.equipWeapon(preferred>=0&&this.weaponUnlocked[preferred]?preferred:levelIndex,false);
  this.platforms=[...this.level.ground.map(([x,end])=>platform(x,460,end-x)),...this.level.platforms.map(p=>platform(...p))];
  this.enemies=this.level.enemies.map(([x,type],i)=>({id:i,x,y:type==='drone'?260+(i%3)*35:414,w:type==='drone'?38:28,h:type==='drone'?34:46,vx:0,vy:0,hp:type==='turret'?65:type==='drone'?35:45,maxHp:type==='turret'?65:type==='drone'?35:45,type,phase:i*.9,baseY:type==='drone'?260+(i%3)*35:414,shootCd:1+this.random()*1.5,hurt:0,facing:-1,alive:true,active:false,anim:0,walkPhase:(i*.17)%1,jumpCooldown:.35+(i%3)*.1,onGround:false}));
  this.crates=[350,1390,2260,3030].filter(x=>x<this.level.length-1000).map((x,i)=>({x,y:426,w:44,h:34,hp:28,alive:true,id:i}));
  this.level.platforms.forEach(([x,y,w],i)=>{if(i%3===0)this.pickups.push({x:x+w/2,y:y-18,w:18,h:24,type:'health',alive:true});});
  this.checkpointSet=new Set();this.secretSet=new Set();this.event('radio',this.level.brief);
 }
 applyLoadout(profile,initial=false){
  this.profile=profile||{};const gear=profile.loadout?.equipment;const acc=profile.loadout?.accessory;
  this.maxHp=gear==='shield'?130:100;this.moveSpeed=gear==='boots'?292:252;this.damageMult=acc==='visor'?1.18:1;this.pulseMax=gear==='capacitor'?4.8:7;this.hasCompanion=acc==='drone';
  if(!initial&&this.player){this.player.maxHp=this.maxHp;this.player.hp=Math.min(this.player.hp,this.maxHp);for(let i=0;i<WEAPONS.length;i++)if(profile.owned?.includes(WEAPONS[i].id))this.weaponUnlocked[i]=true;const preferred=WEAPONS.findIndex(w=>w.id===profile.loadout?.weapon);if(preferred>=0)this.equipWeapon(preferred);}
 }
 event(type,data={}){this.events.push({type,data});}
 takeEvents(){const e=this.events;this.events=[];return e;}
 equipWeapon(index,notify=true){if(!this.weaponUnlocked[index])return false;const p=this.player;if(p.weapon===index&&notify)return true;p.weapon=index;p.ammo=WEAPONS[index].mag;p.reloading=0;p.shotCd=.1;if(notify)this.event('weapon',WEAPONS[index].name);return true;}
 reload(){const p=this.player,w=WEAPONS[p.weapon];if(p.reloading>0||p.ammo===w.mag)return;p.reloading=w.reload;p.reloadTotal=w.reload;this.event('reload');}
 burst(x,y,color,count=12,power=130){for(let i=0;i<count;i++){const a=this.random()*Math.PI*2,s=(.3+this.random())*power;this.particles.push({x,y,vx:Math.cos(a)*s,vy:Math.sin(a)*s,life:.3+this.random()*.4,maxLife:.7,size:1+this.random()*3,color});}}
  damagePlayer(amount,sourceX){const p=this.player;if(this.status!=='playing'||this.trainerGodMode||p.invuln>0||p.dashTime>0)return false;p.hp=Math.max(0,p.hp-amount);p.invuln=.85;p.hurt=.3;p.vx=(p.x>sourceX?1:-1)*170;p.vy=-120;this.shake=7;this.combo=0;this.event('hurt',amount);this.burst(p.x+12,p.y+20,'#f86f94',9,130);if(p.hp<=0){this.status='dead';this.burst(p.x+12,p.y+20,'#f2d1b1',28,210);this.event('dead');}return true;}
 damageEnemy(e,amount){if(!e.alive)return;e.hp-=amount;e.hurt=.11;this.damageNumbers.push({x:e.x+e.w/2,y:e.y,life:.65,text:Math.round(amount),boss:!!e.boss});this.burst(e.x+e.w/2,e.y+e.h/2,e.boss?'#ffb67a':'#72f6d4',4,75);if(e.hp<=0){e.alive=false;this.kills++;this.stageKills++;this.combo=this.comboTimer>0?this.combo+1:1;this.comboTimer=4;const points=(e.boss?1500:100)*Math.min(4,1+Math.floor(this.combo/4));this.score+=points;this.shake=e.boss?12:3;this.event('kill',{boss:!!e.boss,credits:e.boss?200:25});this.burst(e.x+e.w/2,e.y+e.h/2,e.boss?'#ffb571':'#fb7999',e.boss?55:20,e.boss?300:150);if(e.boss){this.bossDefeated=true;this.enemyBullets=[];this.event('bossDown');}else if(this.random()<.23)this.pickups.push({x:e.x,y:e.y,w:18,h:22,type:'health',alive:true,vy:0});}}
 muzzlePoint(angle=this.player.aim,player=this.player){return playerMuzzle(player,this.profile.character,Math.cos(angle)>=0?1:-1);}
 setCrouching(wanted){const p=this.player,next=wanted?CROUCH_HITBOX:STAND_HITBOX;if(p.h===next){p.crouching=wanted;return true;}const y=p.y+p.h-next;if(!wanted&&this.platforms.some(pl=>overlaps({x:p.x,y,w:p.w,h:next},pl)))return false;p.y=y;p.h=next;p.crouching=wanted;return true;}
 shoot(input){const p=this.player,w=WEAPONS[p.weapon];if(p.shotCd>0||p.reloading>0)return;if(p.ammo<=0){this.reload();return;}p.ammo--;p.shotCd=w.rate;p.flashTime=.045;let angle=p.aim;if(input.autoAim){const targets=[...this.enemies,...(this.boss?[this.boss]:[])].filter(e=>e.alive&&Math.abs(e.x-p.x)<620);targets.sort((a,b)=>Math.hypot(a.x-p.x,a.y-p.y)-Math.hypot(b.x-p.x,b.y-p.y));if(targets[0])angle=Math.atan2(targets[0].y+targets[0].h*.5-(p.y+20),targets[0].x+targets[0].w*.5-(p.x+12));else angle=p.facing===1?0:Math.PI;p.aim=angle;}
  p.facing=Math.cos(angle)>=0?1:-1;const{x,y}=this.muzzlePoint(angle);
  for(let i=0;i<w.pellets;i++){const a=angle+(w.pellets>1?(i-(w.pellets-1)/2)*w.spread/2:(this.random()-.5)*w.spread);this.bullets.push({x,y,px:x,py:y,vx:Math.cos(a)*w.speed,vy:Math.sin(a)*w.speed,life:w.id==='shotgun'?.7:1.25,damage:w.damage*this.damageMult,color:w.color,r:w.splash?5:2.5,splash:w.splash||0,pierce:!!w.pierce,hits:new Set()});}this.event('shot',w.id);this.burst(x,y,w.color,3,65);if(p.ammo<=0)this.event('empty');
 }
 enemyShot(e,angle,speed=205,r=4,color='#ff718d'){e.fireTime=.36;this.enemyBullets.push({x:e.x+e.w/2,y:e.y+e.h*.45,vx:Math.cos(angle)*speed,vy:Math.sin(angle)*speed,life:4,r,damage:e.boss?13:10,color});}
 moveBody(body,dt){body.x+=body.vx*dt;for(const pl of this.platforms)if(overlaps(body,pl)){if(body.vx>0)body.x=pl.x-body.w;else if(body.vx<0)body.x=pl.x+pl.w;body.vx=0;}const wasY=body.y;body.y+=body.vy*dt;body.onGround=false;for(const pl of this.platforms)if(overlaps(body,pl)){if(body.vy>=0&&wasY+body.h<=pl.y+4){body.y=pl.y-body.h;body.vy=0;body.onGround=true;body.ground=pl;}else if(body.vy<0&&wasY>=pl.y+pl.h-3){body.y=pl.y+pl.h;body.vy=0;}}}
 update(dt,input={}){
  if(this.status!=='playing')return;dt=Math.min(dt,.04);this.time+=dt;this.totalTime+=dt;this.comboTimer=Math.max(0,this.comboTimer-dt);if(this.comboTimer===0)this.combo=0;this.shake=Math.max(0,this.shake-dt*20);
  const p=this.player;p.anim+=dt;p.flashTime=Math.max(0,(p.flashTime||0)-dt);p.hurt=Math.max(0,p.hurt-dt);p.land=Math.max(0,p.land-dt);p.invuln=Math.max(0,p.invuln-dt);p.dashCd=Math.max(0,p.dashCd-dt);p.pulseCd=Math.max(0,p.pulseCd-dt);p.shotCd=Math.max(0,p.shotCd-dt);p.companionCd=Math.max(0,p.companionCd-dt);
  if(input.weapon!==undefined)this.equipWeapon(input.weapon);if(input.reload)this.reload();
  if(p.reloading>0){p.reloading-=dt;if(p.reloading<=0){p.reloading=0;p.ammo=WEAPONS[p.weapon].mag;this.event('reloaded');}}
  let dir=(input.right?1:0)-(input.left?1:0);if(dir)p.facing=dir;
  this.setCrouching(!!input.crouch&&p.onGround&&p.dashTime<=0&&!input.jump);p.firing=!!input.shoot;
  if(input.aim){p.aim=Math.atan2(input.aim.y-(p.y+19),input.aim.x-(p.x+12));if(input.shoot)p.facing=Math.cos(p.aim)>=0?1:-1;}else p.aim=p.facing===1?0:Math.PI;
  if(p.onGround){p.coyote=.12;p.jumps=0;}else p.coyote=Math.max(0,p.coyote-dt);
  if(input.jump&&(p.coyote>0||p.jumps<2)&&this.setCrouching(false)){p.vy=-(p.jumps===0?560:505);p.jumps++;p.coyote=0;p.onGround=false;this.event('jump');this.burst(p.x+12,p.y+p.h,'#89dcd3',7,90);}
  if(input.dash&&p.dashCd<=0){p.dashTime=.18;p.dashCd=1.55;p.vy=0;this.event('dash');}
  if(p.dashTime>0){p.dashTime-=dt;p.vx=p.facing*750;p.vy=0;this.ghosts.push({x:p.x,y:p.y,h:p.h,frame:p.animation?.frame??9,life:.22,facing:p.facing});}else{const speed=p.crouching?0:this.moveSpeed;p.vx+=((dir*speed)-p.vx)*Math.min(1,dt*(p.onGround?22:9));if(p.onGround&&(!dir||p.crouching))p.vx=0;p.vy=Math.min(820,p.vy+this.level.gravity*dt);}
  const grounded=p.onGround;this.moveBody(p,dt);p.x=clamp(p.x,4,this.level.length-p.w-8);if(!grounded&&p.onGround){p.land=.12;this.burst(p.x+12,p.y+p.h,'#547680',5,50);}
  if(p.y>600){this.setCrouching(false);p.x=this.lastCheckpoint;p.y=340;p.vy=0;p.invuln=0;this.damagePlayer(18,p.x-1);p.invuln=1.8;this.event('radio','Queda detectada. Reposicionando no último ponto seguro.');}
  updatePlayerAnimation(p,dt,!!dir);if(input.shoot)this.shoot(input);
  if(input.pulse&&p.pulseCd<=0){p.pulseCd=this.pulseMax;this.event('pulse');this.shake=5;this.burst(p.x+12,p.y+20,'#9bf5ff',40,300);for(const e of [...this.enemies,...(this.boss?[this.boss]:[])])if(e.alive&&Math.hypot(e.x-p.x,e.y-p.y)<210)this.damageEnemy(e,70);this.enemyBullets=this.enemyBullets.filter(b=>Math.hypot(b.x-p.x,b.y-p.y)>230);this.ghosts.push({pulse:true,x:p.x+12,y:p.y+20,life:.48});}
  if(this.hasCompanion&&p.companionCd<=0){const target=this.enemies.find(e=>e.alive&&Math.abs(e.x-p.x)<450)||(this.boss?.alive&&Math.abs(this.boss.x-p.x)<650?this.boss:null);if(target){const x=p.x-20,y=p.y-30,a=Math.atan2(target.y+target.h/2-y,target.x+target.w/2-x);this.bullets.push({x,y,px:x,py:y,vx:Math.cos(a)*650,vy:Math.sin(a)*650,life:1,damage:10,color:'#aaf8fe',r:2,hits:new Set()});p.companionCd=.85;}}
  for(const e of this.enemies){if(!e.alive)continue;e.fireTime=Math.max(0,(e.fireTime||0)-dt);e.jumpCooldown=Math.max(0,(e.jumpCooldown||0)-dt);e.active=Math.abs(e.x-p.x)<780;if(!e.active)continue;e.anim+=dt;e.hurt=Math.max(0,e.hurt-dt);e.facing=p.x>e.x?1:-1;e.shootCd=Math.max(0,e.shootCd-dt);
   if(e.type==='drone'){e.x+=Math.sign(p.x-e.x)*Math.max(0,Math.abs(p.x-e.x)-180)*dt*.25;e.y=e.baseY+Math.sin(this.time*2+e.phase)*22;}
   else{e.vy=Math.min(800,(e.vy||0)+1450*dt);const close=Math.abs(p.x-e.x);e.vx=e.type==='turret'?0:close<420&&close>170?e.facing*(50+this.levelIndex*10):Math.sin(this.time+e.phase)*18;const footX=e.x+e.w/2+e.facing*24;const support=this.platforms.some(pl=>footX>pl.x&&footX<pl.x+pl.w&&Math.abs(pl.y-(e.y+e.h))<20);if(e.onGround&&!support){const next=this.platforms.find(pl=>pl.x>e.x+e.w&&pl.x-e.x<=240&&pl.y<e.y+e.h+120);const ceiling=this.platforms.some(pl=>pl.x<e.x+e.w+50&&pl.x+pl.w>e.x-50&&pl.y<e.y&&pl.y+pl.h>e.y-110);if(next&&e.facing===1&&!ceiling&&e.jumpCooldown<=0){e.vx=50+this.levelIndex*10;e.vy=-470;e.onGround=false;e.jumpCooldown=2.4;}else e.vx=0;}this.moveBody(e,dt);if(e.onGround&&Math.abs(e.vx)>5){const cadence=.82+Math.min(.28,Math.abs(e.vx)/220);e.walkPhase=((e.walkPhase||0)+dt*cadence)%1;}else if(!e.onGround){e.walkPhase=((e.walkPhase||0)+dt*.18)%1;}if(e.y>570){e.alive=false;continue;}}
   if(e.shootCd<=0&&Math.abs(p.x-e.x)<580){const a=Math.atan2(p.y+20-(e.y+e.h*.45),p.x+12-(e.x+e.w/2));this.enemyShot(e,a,185+this.levelIndex*25);if(e.type==='turret'){this.enemyShot(e,a-.11,195);this.enemyShot(e,a+.11,195);}e.shootCd=(e.type==='turret'?1.9:1.6)-this.levelIndex*.14+this.random()*.7;}
   if(overlaps(p,e))this.damagePlayer(12,e.x);
  }
  if(!this.bossStarted&&p.x>this.level.length-810){this.bossStarted=true;this.boss={boss:true,id:'boss',type:'boss',x:this.level.length-350,y:325,w:124,h:128,hp:this.level.bossHp,maxHp:this.level.bossHp,alive:true,hurt:0,phase:1,shootCd:1.8,attackTime:0,telegraph:0,attack:0,anim:0};this.event('boss',this.level.boss);this.event('radio',this.levelIndex===2?'É ele. O NEXUS. Observe o núcleo: ele muda de fase quando perde integridade.':'Contato pesado. Espere o disparo, use o impulso e ataque pelos intervalos.');}
  const boss=this.boss;if(boss?.alive){boss.fireTime=Math.max(0,(boss.fireTime||0)-dt);boss.anim+=dt;boss.hurt=Math.max(0,boss.hurt-dt);boss.phase=boss.hp<boss.maxHp*.3?3:boss.hp<boss.maxHp*.65?2:1;boss.attackTime+=dt;boss.y=324+Math.sin(this.time*1.4)*4;boss.x=this.level.length-380+Math.sin(this.time*.5)*80;boss.shootCd-=dt;boss.telegraph=Math.max(0,boss.shootCd<.6?1-boss.shootCd/.6:0);if(boss.shootCd<=0){const a=Math.atan2(p.y+20-(boss.y+boss.h*.45),p.x+12-(boss.x+boss.w/2));const spread=2+boss.phase+this.levelIndex;for(let i=0;i<spread;i++)this.enemyShot(boss,a+(i-(spread-1)/2)*.16,185+boss.phase*18,5);boss.attack++;if(boss.phase>=2&&boss.attack%2===0){for(let i=0;i<9;i++)this.enemyShot(boss,Math.PI+i*Math.PI/8,155,4,'#d6a2ff');}boss.shootCd=2.1-boss.phase*.25;this.event('bossShot');}if(overlaps(p,boss))this.damagePlayer(18,boss.x);if(p.x>this.level.length-155)p.x=this.level.length-155;}
  for(const b of this.bullets){if(b.life<=0)continue;b.px=b.x;b.py=b.y;b.x+=b.vx*dt;b.y+=b.vy*dt;b.life-=dt;for(const e of [...this.enemies,...(boss?[boss]:[])])if(e.alive&&!b.hits.has(e.id)&&overlaps({x:b.x-b.r,y:b.y-b.r,w:b.r*2,h:b.r*2},e)){this.damageEnemy(e,b.damage);b.hits.add(e.id);if(b.splash)for(const other of this.enemies)if(other!==e&&other.alive&&Math.hypot(other.x-e.x,other.y-e.y)<b.splash)this.damageEnemy(other,b.damage*.5);if(!b.pierce){b.life=0;break;}}
   if(b.life>0)for(const c of this.crates)if(c.alive&&overlaps({x:b.x-2,y:b.y-2,w:4,h:4},c)){c.hp-=b.damage;b.life=0;this.burst(b.x,b.y,'#ffc080',5,80);if(c.hp<=0){c.alive=false;this.event('crate',{credits:15});this.pickups.push({x:c.x+12,y:c.y-10,w:18,h:24,type:'health',alive:true});}break;}
   if(b.life>0&&this.platforms.some(pl=>b.y>pl.y&&b.y<pl.y+pl.h&&b.x>pl.x&&b.x<pl.x+pl.w)){b.life=0;this.burst(b.x,b.y,b.color,2,40);}}
  this.bullets=this.bullets.filter(b=>b.life>0);
  for(const b of this.enemyBullets){b.x+=b.vx*dt;b.y+=b.vy*dt;b.life-=dt;if(overlaps({x:b.x-b.r,y:b.y-b.r,w:b.r*2,h:b.r*2},p)){this.damagePlayer(b.damage,b.x);b.life=0;}if(this.platforms.some(pl=>b.y>pl.y&&b.y<pl.y+pl.h&&b.x>pl.x&&b.x<pl.x+pl.w))b.life=0;}
  this.enemyBullets=this.enemyBullets.filter(b=>b.life>0);
  for(const hazard of this.level.hazards){const phase=(this.time+hazard.x*.003)%hazard.period;hazard.active=phase<hazard.on;hazard.warning=phase>hazard.period-.7;if(hazard.active&&p.x+p.w>hazard.x&&p.x<hazard.x+hazard.w&&p.y+p.h>438)this.damagePlayer(14,hazard.x);}
  for(const pickup of this.pickups){if(!pickup.alive)continue;if(overlaps(p,{x:pickup.x-7,y:pickup.y-7,w:32,h:38})){pickup.alive=false;p.hp=Math.min(p.maxHp,p.hp+(this.profile.loadout?.equipment==='medkit'?28:18));this.score+=25;this.event('pickup');this.burst(pickup.x,pickup.y,'#6effd7',12,100);}}
  for(const secret of this.level.secrets)if(!this.secretSet.has(secret.id)&&Math.hypot(p.x+12-secret.x,p.y+20-secret.y)<35){this.secretSet.add(secret.id);this.secretsFound.push(secret.id);this.score+=750;this.event('secret',{...secret,credits:125});this.burst(secret.x,secret.y,'#ffdb8a',30,200);}
  for(const x of this.level.checkpoints)if(p.x>x&&!this.checkpointSet.has(x)){this.checkpointSet.add(x);this.lastCheckpoint=x;this.event('checkpoint');}
  if(this.bossDefeated&&p.x>this.level.length-110){this.status='clear';this.score+=1000+Math.round(p.hp*5);this.event('clear',{credits:250});}
  const camTarget=clamp(p.x-this.viewWidth*.33,0,this.level.length-this.viewWidth);this.cameraX+=(camTarget-this.cameraX)*Math.min(1,dt*8);
  for(const a of this.particles){a.x+=a.vx*dt;a.y+=a.vy*dt;a.vy+=330*dt;a.life-=dt;}this.particles=this.particles.filter(a=>a.life>0).slice(-320);
  for(const g of this.ghosts)g.life-=dt;this.ghosts=this.ghosts.filter(g=>g.life>0).slice(-35);for(const d of this.damageNumbers){d.y-=dt*40;d.life-=dt;}this.damageNumbers=this.damageNumbers.filter(d=>d.life>0).slice(-30);
 }
}
