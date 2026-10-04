// Lighting: Sunlight, Ambient, Rim & Night Torches
// ------------------------------------------------------------------------
function setupLighting() {
  ambientLight = new THREE.AmbientLight(0xffedd5, 0.7);
  scene.add(ambientLight);

  sunLight = new THREE.DirectionalLight(0xfef3c7, 0.9);
  sunLight.position.set(40, 60, 30);
  sunLight.castShadow = true;
  sunLight.shadow.mapSize.width = 1024;
  sunLight.shadow.mapSize.height = 1024;
  sunLight.shadow.camera.near = 10;
  sunLight.shadow.camera.far = 150;
  const d = 45;
  sunLight.shadow.camera.left = -d;
  sunLight.shadow.camera.right = d;
  sunLight.shadow.camera.top = d;
  sunLight.shadow.camera.bottom = -d;
  sunLight.shadow.bias = -0.001;
  scene.add(sunLight);

  rimLight = new THREE.DirectionalLight(0x38bdf8, 0.35);
  rimLight.position.set(-30, 25, -30);
  scene.add(rimLight);

  // Warm Torches around the Citadel for Night Mode
  nightTorchesGroup = new THREE.Group();
  const torchPositions = [
    [0, 3.2, -1],     // Temple Altar
    [-3.8, 5.5, -6.5], // Jachin
    [3.8, 5.5, -6.5],  // Boaz
    [-22, 6.0, -11],  // Palace Balcony
    [-6.0, 5.0, 32],  // West Gate bastion
    [6.0, 5.0, 32],   // East Gate bastion
    [0, 3.0, 6],      // Bazaar Plaza
    [22, 3.0, 10],    // Gihon Cistern
  ];

  torchPositions.forEach(([tx, ty, tz]) => {
    const pl = new THREE.PointLight(0xf59e0b, 0, 16);
    pl.position.set(tx, ty, tz);
    nightTorchesGroup.add(pl);
  });

  scene.add(nightTorchesGroup);
}

// ------------------------------------------------------------------------
// Ground: Desert Sand & Grid
// ------------------------------------------------------------------------
function setupGround() {
  const groundGeo = new THREE.PlaneGeometry(120, 120);
  const groundMat = new THREE.MeshStandardMaterial({
    color: 0x451a03,
    roughness: 0.95,
    metalness: 0.05,
  });
  groundMesh = new THREE.Mesh(groundGeo, groundMat);
  groundMesh.rotation.x = -Math.PI / 2;
  groundMesh.position.y = -0.05;
  groundMesh.receiveShadow = true;
  scene.add(groundMesh);

  const gridHelper = new THREE.GridHelper(100, 50, 0x9a3412, 0x78350f);
  gridHelper.position.y = 0.01;
  scene.add(gridHelper);
}

// ------------------------------------------------------------------------
// Time of Day & Weather Switcher
// ------------------------------------------------------------------------
window.setTimeOfDay = function (time) {
  state.timeOfDay = time;
  if (!scene || !ambientLight || !sunLight || !rimLight) return;

  if (time === 'dawn') {
    scene.background.setHex(0x382824);
    scene.fog.color.setHex(0x382824);
    ambientLight.color.setHex(0xfecdd3);
    ambientLight.intensity = 0.65;
    sunLight.color.setHex(0xfde68a);
    sunLight.intensity = 0.75;
    sunLight.position.set(50, 25, 40);
    rimLight.color.setHex(0x38bdf8);
    rimLight.intensity = 0.25;
    setNightTorches(false);
    showFloatToast('🌅 破晓晨曦：旭日初升，城邦苏醒');
  } else if (time === 'noon') {
    scene.background.setHex(0x1e3a8a);
    scene.fog.color.setHex(0x1e3a8a);
    ambientLight.color.setHex(0xffffff);
    ambientLight.intensity = 0.85;
    sunLight.color.setHex(0xffedd5);
    sunLight.intensity = 1.15;
    sunLight.position.set(10, 75, 10);
    rimLight.color.setHex(0x93c5fd);
    rimLight.intensity = 0.3;
    setNightTorches(false);
    showFloatToast('☀️ 正午艳阳：万物生辉，商贾云集');
  } else if (time === 'dusk') {
    scene.background.setHex(0x1a140f);
    scene.fog.color.setHex(0x1a140f);
    ambientLight.color.setHex(0xffedd5);
    ambientLight.intensity = 0.7;
    sunLight.color.setHex(0xfef3c7);
    sunLight.intensity = 0.9;
    sunLight.position.set(40, 60, 30);
    rimLight.color.setHex(0x38bdf8);
    rimLight.intensity = 0.35;
    setNightTorches(false);
    showFloatToast('🌇 晚霞暮霭：金光普照，炊烟袅袅');
  } else if (time === 'night') {
    scene.background.setHex(0x050814);
    scene.fog.color.setHex(0x050814);
    ambientLight.color.setHex(0x1e293b);
    ambientLight.intensity = 0.35;
    sunLight.color.setHex(0x93c5fd);
    sunLight.intensity = 0.35;
    sunLight.position.set(-30, 45, -30);
    rimLight.color.setHex(0x60a5fa);
    rimLight.intensity = 0.5;
    setNightTorches(true);
    showFloatToast('🌙 星夜降临：万家灯火，圣火长明');
  }

  if (state.dayNightCycle) {
    state.dayNightCycle.timer = state.dayNightCycle.durationPerPhase;
  }
  notifyCitizensPhaseChange(time);
};

function notifyCitizensPhaseChange(newPhase) {
  if (!agents || !ROUTINE_SCHEDULE) return;
  agents.forEach((c) => {
    if (c.type !== 'ancient_citizen') return;
    c.currentPhase = newPhase;
    const sched = ROUTINE_SCHEDULE[c.role] && ROUTINE_SCHEDULE[c.role][newPhase];
    if (sched) {
      c.subState = 'COMMUTING';
      c.task = `🚶 赶往: ${sched.task.split(' ')[1] || sched.task}`;
      c.dialogue = sched.dialogue;
      const tx = sched.target[0] + (Math.random() - 0.5) * (sched.spread || 4.0);
      const tz = sched.target[1] + (Math.random() - 0.5) * (sched.spread || 4.0);
      c.target.set(tx, 0, tz);
      c.stateTimer = 0;
    }
  });
}

function setNightTorches(enabled) {
  if (!nightTorchesGroup) return;
  nightTorchesGroup.children.forEach((torch) => {
    torch.intensity = enabled ? 2.2 : 0;
  });
}

// ------------------------------------------------------------------------
// Mode Switcher
// ------------------------------------------------------------------------
window.setSimulationMode = function () {
  state.mode = 'temple_city';
  state.selectedAgent = null;
  state.selectedBuilding = null;
  followAgent = null;
  if (controlledAgent && window.exitControlMode) window.exitControlMode();
  clearBuildingHighlight();

  if (simGroup) scene.remove(simGroup);
  simGroup = new THREE.Group();
  scene.add(simGroup);

  agents = [];
  knightAgents = [];
  activeMixers = [];
  interactiveBuildings = [];
  smokeParticles = [];
  pressWheel = null;

  initTempleCitySimulation();

  updateStatsHUD();
  updateControlsUI();
  updateInspectorHUD();
};

// ========================================================================