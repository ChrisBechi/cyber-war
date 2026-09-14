const clampVolume=value=>Math.max(0,Math.min(1,Number.isFinite(value)?value:0));
export class AudioSystem{
 constructor(){this.ctx=null;this.enabled=false;this.volume=.55;this.musicVolume=.65;this.effectsVolume=.85;this.scene='menu';this.level=0;this.nextBeat=0;this.beat=0;this.timer=null;this.voices=0;}
 async unlock(){
  if(!this.ctx){const AC=globalThis.AudioContext||globalThis.webkitAudioContext;if(!AC)return;this.ctx=new AC();const c=this.ctx;
   this.master=c.createGain();this.master.gain.value=0;this.musicBus=c.createGain();this.effectsBus=c.createGain();this.musicBus.connect(this.master);this.effectsBus.connect(this.master);
   this.limiter=c.createDynamicsCompressor();this.limiter.threshold.value=-14;this.limiter.knee.value=12;this.limiter.ratio.value=6;this.limiter.attack.value=.004;this.limiter.release.value=.2;this.master.connect(this.limiter);this.limiter.connect(c.destination);
   this.noise=c.createBuffer(1,c.sampleRate*.6,c.sampleRate);const data=this.noise.getChannelData(0);for(let i=0;i<data.length;i++)data[i]=Math.random()*2-1;this.timer=setInterval(()=>this.schedule(),100);this.mix();
  }
  if(this.ctx.state==='suspended'){
   try{await Promise.race([this.ctx.resume(),new Promise(resolve=>setTimeout(resolve,250))]);}catch{}
  }
  this.nextBeat=this.ctx.currentTime+.03;
 }
 mix(){if(!this.ctx||this.ctx.state==='closed')return;try{const t=this.ctx.currentTime;this.master.gain.setTargetAtTime(this.enabled?this.volume*.48:0,t,.06);this.musicBus.gain.setTargetAtTime(this.musicVolume*(this.scene==='pause'?.28:1),t,.16);this.effectsBus.gain.setTargetAtTime(this.effectsVolume,t,.06);}catch{this.enabled=false;}}
 setEnabled(on){this.enabled=!!on;this.mix();}
 setVolume(value){this.volume=clampVolume(value);this.mix();}
 setMusicVolume(value){this.musicVolume=clampVolume(value);this.mix();}
 setEffectsVolume(value){this.effectsVolume=clampVolume(value);this.mix();}
 setScene(scene,level=0){this.scene=scene;this.level=level;this.beat=0;if(this.ctx)this.nextBeat=this.ctx.currentTime+.05;this.mix();}
 tone(freq,duration=.12,type='sine',gain=.12,delay=0,endFreq=null,bus='effects'){
  if(!this.ctx||this.ctx.state==='closed'||!this.enabled||this.voices>=36)return;try{const c=this.ctx,t=c.currentTime+delay,o=c.createOscillator(),g=c.createGain();this.voices++;o.type=type;o.frequency.setValueAtTime(freq,t);if(endFreq)o.frequency.exponentialRampToValueAtTime(endFreq,t+duration);g.gain.setValueAtTime(.0001,t);g.gain.linearRampToValueAtTime(gain,t+.008);g.gain.exponentialRampToValueAtTime(.0001,t+duration);o.connect(g);g.connect(bus==='music'?this.musicBus:this.effectsBus);o.start(t);o.stop(t+duration+.02);o.onended=()=>{o.disconnect();g.disconnect();this.voices--;};}catch{this.enabled=false;}
 }
 hiss(duration=.12,gain=.1,cutoff=2200,delay=0,bus='effects'){
  if(!this.ctx||this.ctx.state==='closed'||!this.enabled||this.voices>=36)return;try{const c=this.ctx,t=c.currentTime+delay,s=c.createBufferSource(),f=c.createBiquadFilter(),g=c.createGain();this.voices++;s.buffer=this.noise;f.type='lowpass';f.frequency.value=cutoff;g.gain.setValueAtTime(gain,t);g.gain.exponentialRampToValueAtTime(.0001,t+duration);s.connect(f);f.connect(g);g.connect(bus==='music'?this.musicBus:this.effectsBus);s.start(t);s.stop(t+duration);s.onended=()=>{s.disconnect();f.disconnect();g.disconnect();this.voices--;};}catch{this.enabled=false;}
 }
 effect(name,data){if(!this.enabled)return;switch(name){
  case'shot':if(data==='railgun'){this.tone(1200,.22,'sawtooth',.11,0,85);this.hiss(.2,.15,4000);}else if(data==='plasma'){this.tone(640,.16,'sine',.19,0,100);this.tone(840,.1,'triangle',.08);}else if(data==='shotgun'){this.hiss(.2,.24,2400);this.tone(90,.22,'triangle',.2,0,30);}else{this.tone(165,.075,'sawtooth',.1,0,50);this.hiss(.055,.09,5200);}break;
  case'jump':this.tone(190,.13,'sine',.09,0,390);break;
  case'dash':this.hiss(.21,.14,1100);this.tone(100,.18,'sine',.13,0,350);break;
  case'pulse':this.tone(75,.45,'sine',.3,0,450);this.hiss(.27,.16,2800);break;
  case'hurt':this.tone(85,.2,'sawtooth',.17,0,40);break;
  case'kill':this.hiss(data?.boss ? .5 : .14,data?.boss ? .3 : .12,1800);this.tone(95,.15,'sine',.2,0,32);break;
  case'pickup':case'checkpoint':this.tone(660,.2,'sine',.15);this.tone(990,.2,'sine',.1,.08);break;
  case'secret':case'purchase':[440,554,660,880].forEach((f,i)=>this.tone(f,.45,'sine',.14,i*.09));break;
  case'reload':this.hiss(.055,.07,700);this.tone(230,.055,'triangle',.09,.05);break;
  case'reloaded':this.tone(800,.045,'triangle',.08);break;
  case'boss':this.tone(55,.8,'sawtooth',.12);this.tone(58,.8,'sawtooth',.09);this.hiss(.45,.16,800);break;
  case'bossShot':this.tone(54,.22,'sawtooth',.13,0,35);break;
  case'dead':[220,185,146,110].forEach((f,i)=>this.tone(f,.5,'triangle',.13,i*.16));break;
  case'clear':[293.66,369.99,440,587.33].forEach((f,i)=>this.tone(f,.9,'triangle',.15,i*.16));break;
  case'ui':this.tone(680,.06,'sine',.065);break;
 }}
 schedule(){
  if(!this.ctx||!this.enabled||this.ctx.state!=='running')return;const c=this.ctx,playing=this.scene==='play',tempo=playing?112+this.level*12:78,step=60/tempo/2;
  if(this.nextBeat<c.currentTime-.4)this.nextBeat=c.currentTime+.04;
  while(this.nextBeat<c.currentTime+.22){const delay=Math.max(0,this.nextBeat-c.currentTime),beat=this.beat,notes=[[73.42,87.31,65.41,98],[65.41,77.78,58.27,87.31],[73.42,82.41,65.41,110]][this.level%3],root=notes[Math.floor(beat/8)%4];
   const note=(freq,duration,type,gain,end=null)=>this.tone(freq,duration,type,gain,delay,end,'music');
   if(playing){
    if(beat%2===0)note(120,.17,'sine',.25,36);
    if(beat%4===2){this.hiss(.1,.08,3500,delay,'music');note(175,.08,'triangle',.05);}
    this.hiss(beat%4===3?.06:.025,.022,7200,delay,'music');
    note(root*(beat%4===3?2:1),.23,'triangle',.15);
    if(beat%2===1)note(root*[4,6,5,8,4,5,6,3][Math.floor(beat/2)%8],.23,'sine',.065);
    if(beat%8===0){note(root*2,1.7,'sine',.055);note(root*3,1.6,'sine',.035);}
    if(this.level===2&&beat%8===7)note(root*8,.3,'triangle',.04);
   }else{
    if(beat%4===0){note(root,1.5,'triangle',.085);note(root*2,1.6,'sine',.065);note(root*3,1.4,'sine',.035);}
    if(beat%3===0)note(root*4,.8,'sine',.045);
   }
   this.beat++;this.nextBeat+=step;
  }
 }
}
