import type { MotionArtifact, MotionSpec, RuntimeComponent } from "../motionArtifacts";

export function css3dDieSpec(): MotionSpec {
  return {
    id: "css3d-die-roll",
    name: "CSS 3D Die Roll",
    renderer: "dom-css3d",
    recipe: "dom-css3d.die.roll",
    states: ["idle", "roll", "face_1", "face_2", "face_3", "face_4", "face_5", "face_6"],
    inputs: { size: 200, durationMs: 1500, finalFace: "controlled" },
  };
}

export function createCss3dDieComponent(): RuntimeComponent {
  return {
    id: "runtime-css3d-die",
    name: "Runtime CSS 3D Die",
    recipeId: "dom-css3d.die.roll",
    previewWidth: 960,
    previewHeight: 540,
    states: ["idle", "roll", "face_1", "face_2", "face_3", "face_4", "face_5", "face_6"],
    inputs: [{ name: "roll", kind: "trigger" }],
    assets: [],
    html: `<div class="strut-css3d-die" data-state="idle">
  <div class="scene">
    <div class="shadow" aria-hidden="true"></div>
    <button class="cube" type="button" aria-label="Roll die">
      <span class="face front"><i class="pip center"></i></span>
      <span class="face back"><i class="pip tl"></i><i class="pip tr"></i><i class="pip ml"></i><i class="pip mr"></i><i class="pip bl"></i><i class="pip br"></i></span>
      <span class="face right"><i class="pip tr"></i><i class="pip bl"></i></span>
      <span class="face left"><i class="pip tl"></i><i class="pip tr"></i><i class="pip bl"></i><i class="pip br"></i></span>
      <span class="face top"><i class="pip tl"></i><i class="pip tr"></i><i class="pip center"></i><i class="pip bl"></i><i class="pip br"></i></span>
      <span class="face bottom"><i class="pip tl"></i><i class="pip center"></i><i class="pip br"></i></span>
    </button>
  </div>
</div>`,
    css: `.strut-css3d-die{width:100%;height:100%;display:grid;place-items:center;background:radial-gradient(circle at center,#20264f 0%,#101326 48%,#070814 100%);overflow:hidden}.strut-css3d-die .scene{width:220px;height:220px;position:relative;perspective:850px;perspective-origin:50% 42%;cursor:pointer}.strut-css3d-die .shadow{position:absolute;left:50%;top:235px;width:230px;height:42px;transform:translateX(-50%);border-radius:50%;background:rgba(0,0,0,.45);filter:blur(8px);transition:transform 1.5s cubic-bezier(.175,.885,.32,1.275),opacity 1.5s}.strut-css3d-die .cube{width:200px;height:200px;position:relative;transform-style:preserve-3d;transform:rotateX(-18deg) rotateY(28deg);transition:transform 1.5s cubic-bezier(.175,.885,.32,1.275);border:0;background:transparent;padding:0;cursor:pointer}.strut-css3d-die .face{position:absolute;inset:0;border-radius:24px;border:3px solid rgba(255,255,255,.35);background:radial-gradient(circle at 28% 22%,rgba(255,255,255,.95),transparent 24%),linear-gradient(145deg,#fff 0%,#eef1fa 48%,#c8d0e4 100%);box-shadow:inset -20px -22px 36px rgba(25,30,48,.20),inset 12px 12px 22px rgba(255,255,255,.78),0 18px 32px rgba(0,0,0,.24);backface-visibility:hidden}.strut-css3d-die .front{transform:translateZ(100px)}.strut-css3d-die .back{transform:rotateY(180deg) translateZ(100px)}.strut-css3d-die .right{transform:rotateY(90deg) translateZ(100px)}.strut-css3d-die .left{transform:rotateY(-90deg) translateZ(100px)}.strut-css3d-die .top{transform:rotateX(90deg) translateZ(100px)}.strut-css3d-die .bottom{transform:rotateX(-90deg) translateZ(100px)}.strut-css3d-die .pip{position:absolute;width:30px;height:30px;border-radius:50%;background:radial-gradient(circle at 38% 35%,#3b3b44 0 22%,#080910 70%,#000 100%);box-shadow:inset 1px 2px 4px rgba(255,255,255,.22),inset -2px -3px 5px rgba(0,0,0,.5),0 4px 8px rgba(0,0,0,.42)}.strut-css3d-die .tl{left:32px;top:32px}.strut-css3d-die .tr{right:32px;top:32px}.strut-css3d-die .ml{left:32px;top:85px}.strut-css3d-die .mr{right:32px;top:85px}.strut-css3d-die .bl{left:32px;bottom:32px}.strut-css3d-die .br{right:32px;bottom:32px}.strut-css3d-die .center{left:85px;top:85px}`,
    js: `(() => { const root = document.querySelector('.strut-css3d-die'); const cube = root?.querySelector('.cube'); if (!root || !cube) return; const rotations = {1:[0,0,0],2:[0,-90,0],3:[90,0,0],4:[0,90,0],5:[-90,0,0],6:[0,180,0]}; let rolling = false; function roll(face = Math.floor(Math.random()*6)+1){ if(rolling) return; rolling = true; const target = rotations[face] || rotations[1]; cube.style.transform = 'rotateX(' + (target[0] + 720) + 'deg) rotateY(' + (target[1] + 720) + 'deg) rotateZ(360deg)'; root.dataset.state = 'roll'; setTimeout(() => { cube.style.transition = 'none'; cube.style.transform = 'rotateX(' + target[0] + 'deg) rotateY(' + target[1] + 'deg) rotateZ(' + target[2] + 'deg)'; void cube.offsetWidth; cube.style.transition = 'transform 1.5s cubic-bezier(.175,.885,.32,1.275)'; root.dataset.state = 'face_' + face; rolling = false; }, 1520); } cube.addEventListener('click', () => roll()); root.strut = { roll }; })();`,
  };
}

export function createCss3dDieArtifact(): MotionArtifact {
  const spec = css3dDieSpec();
  return {
    kind: "runtime_component",
    renderer: "dom-css3d",
    spec,
    component: createCss3dDieComponent(),
    activeState: "idle",
  };
}
