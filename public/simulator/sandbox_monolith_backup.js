// ========================================================================
// 3D Ancient Middle East & Canaan Multi-Agent Simulation Sandbox
// Interactive World Model with Dynamic Landmarks, Weather/Day-Night Cycle,
// Follow Camera, Historical Events & Living Tactical Minimap Radar
// ========================================================================

(function () {
  'use strict';

  let scene, camera, renderer, controls;
  let animationFrameId = null;
  let container = null;
  let canvasMount = null;

  // Global Sandbox State
  const state = {
    mode: 'temple_city',
    isPlaying: true,
    speedMultiplier: 1.0,
    selectedAgent: null,
    selectedBuilding: null,
    timeOfDay: 'dusk', // 'dawn', 'noon', 'dusk', 'night'
    dayNightCycle: {
      autoProgress: true,
      durationPerPhase: 45.0,
      timer: 45.0,
    },
    spawnToolActive: false,

    // Temple City & Ancient Economy (Realistic City-State ABM)
    templeCity: {
      tradeProsperity: 1.2,
      tithesRate: 0.10,
      citizenCount: 42,
      grainStorage: 480,
      waterSupply: 92,
      templeTreasury: 1350,
      sacredAroma: 95,
      citizenMorale: 96,
      marketTurnover: 340,
    },

    // Metrics
    metrics: {
      fps: 60,
      agentCount: 42,
      harmonyScore: '96%',
    }
  };

  let simGroup = null;
  let groundMesh = null;
  let gateObjects = [];
  let agents = [];
  let graphNodes = [];
  let graphEdges = [];
  let buildingTiles = [];
  let interactiveBuildings = [];
  let smokeParticles = [];
  let pressWheel = null;
  let selectionIndicator = null;
  let audioCtx = null;
  let cachedBushModel = null;
  let bushInstances = [];
  let cachedKnightModel = null;
  let knightClips = { walk: null, run: null };
  let activeMixers = [];
  let knightAgents = [];

  // Interactive Camera & Environment Globals
  let ambientLight = null;
  let sunLight = null;
  let rimLight = null;
  let nightTorchesGroup = null;
  let rainGroup = null;
  let rainActive = false;
  let rainTimer = 0;
  let followAgent = null;
  let targetLookAt = new THREE.Vector3(0, 0, 0);
  let isTransitioningCamera = false;

  // Third-Person Character Control Globals
  let orthoCamera = null;
  let perspCamera = null;
  let controlledAgent = null;
  let activePrompt = null;
  let isMouseDownOnControl = false;
  let prevMouseX = 0;
  const inputKeys = {
    forward: false,
    backward: false,
    left: false,
    right: false,
    sprint: false,
  };

  // ------------------------------------------------------------------------
  // Web Audio Synthesizer for Actions & Events
  // ------------------------------------------------------------------------
  function triggerCitySound(type) {
    try {
      if (!audioCtx) {
        audioCtx = new (window.AudioContext || window.webkitAudioContext)();
      }
      if (audioCtx.state === 'suspended') {
        audioCtx.resume();
      }
      const now = audioCtx.currentTime;
      if (type === 'chime') {
        [523.25, 659.25, 783.99, 1046.50].forEach((freq, i) => {
          const osc = audioCtx.createOscillator();
          const gain = audioCtx.createGain();
          osc.type = 'sine';
          osc.frequency.setValueAtTime(freq, now + i * 0.08);
          gain.gain.setValueAtTime(0.12, now + i * 0.08);
          gain.gain.exponentialRampToValueAtTime(0.001, now + i * 0.08 + 1.2);
          osc.connect(gain);
          gain.connect(audioCtx.destination);
          osc.start(now + i * 0.08);
          osc.stop(now + i * 0.08 + 1.2);
        });
      } else if (type === 'horn') {
        [392.0, 523.25, 659.25].forEach((freq, i) => {
          const osc = audioCtx.createOscillator();
          const gain = audioCtx.createGain();
          osc.type = 'sawtooth';
          osc.frequency.setValueAtTime(freq, now + i * 0.12);
          gain.gain.setValueAtTime(0.09, now + i * 0.12);
          gain.gain.exponentialRampToValueAtTime(0.001, now + i * 0.12 + 0.8);
          osc.connect(gain);
          gain.connect(audioCtx.destination);
          osc.start(now + i * 0.12);
          osc.stop(now + i * 0.12 + 0.8);
        });
      } else if (type === 'water') {
        const osc = audioCtx.createOscillator();
        const gain = audioCtx.createGain();
        osc.type = 'sine';
        osc.frequency.setValueAtTime(560, now);
        osc.frequency.exponentialRampToValueAtTime(260, now + 0.5);
        gain.gain.setValueAtTime(0.16, now);
        gain.gain.exponentialRampToValueAtTime(0.01, now + 0.5);
        osc.connect(gain);
        gain.connect(audioCtx.destination);
        osc.start(now);
        osc.stop(now + 0.5);
      } else if (type === 'anvil') {
        const osc = audioCtx.createOscillator();
        const gain = audioCtx.createGain();
        osc.type = 'triangle';
        osc.frequency.setValueAtTime(1300, now);
        gain.gain.setValueAtTime(0.16, now);
        gain.gain.exponentialRampToValueAtTime(0.001, now + 0.35);
        osc.connect(gain);
        gain.connect(audioCtx.destination);
        osc.start(now);
        osc.stop(now + 0.35);
      } else if (type === 'coins') {
        [1760, 2200, 2637].forEach((freq, i) => {
          const osc = audioCtx.createOscillator();
          const gain = audioCtx.createGain();
          osc.type = 'sine';
          osc.frequency.setValueAtTime(freq, now + i * 0.05);
          gain.gain.setValueAtTime(0.1, now + i * 0.05);
          gain.gain.exponentialRampToValueAtTime(0.001, now + i * 0.05 + 0.35);
          osc.connect(gain);
          gain.connect(audioCtx.destination);
          osc.start(now + i * 0.05);
          osc.stop(now + i * 0.05 + 0.35);
        });
      }
    } catch (e) {
      console.warn('Audio feedback error:', e);
    }
  }

  // ------------------------------------------------------------------------
  // Toast Notification
  // ------------------------------------------------------------------------
  function showFloatToast(msg) {
    let toast = document.getElementById('sb-toast');
    if (!toast) {
      toast = document.createElement('div');
      toast.id = 'sb-toast';
      toast.style.cssText = `
        position: absolute;
        top: 20px;
        left: 50%;
        transform: translateX(-50%);
        background: rgba(15, 23, 42, 0.95);
        color: #fef08a;
        border: 1px solid #d97706;
        padding: 8px 18px;
        border-radius: 20px;
        font-size: 13px;
        font-weight: bold;
        box-shadow: 0 10px 25px rgba(0,0,0,0.7);
        pointer-events: none;
        z-index: 100;
        transition: opacity 0.3s ease, transform 0.3s ease;
        opacity: 0;
      `;
      const target = canvasMount || container;
      if (target) target.appendChild(toast);
    }
    toast.textContent = msg;
    toast.style.opacity = '1';
    toast.style.transform = 'translateX(-50%) translateY(0)';
    clearTimeout(toast._timer);
    toast._timer = setTimeout(() => {
      toast.style.opacity = '0';
      toast.style.transform = 'translateX(-50%) translateY(-10px)';
    }, 2800);
  }

  // ------------------------------------------------------------------------
  // Initialize Three.js Engine
  // ------------------------------------------------------------------------
  window.initSandbox3D = function (domContainer) {
    if (renderer) {
      if (animationFrameId) cancelAnimationFrame(animationFrameId);
      if (renderer.domElement && renderer.domElement.parentNode) {
        renderer.domElement.parentNode.removeChild(renderer.domElement);
      }
    }

    container = domContainer;
    canvasMount = domContainer.querySelector('#sandbox-canvas-container') || domContainer;
    const width = container.clientWidth || 840;
    const height = container.clientHeight || 520;

    // 1. Scene & Palette
    scene = new THREE.Scene();
    scene.background = new THREE.Color(0x1a140f);
    scene.fog = new THREE.FogExp2(0x1a140f, 0.007);

    // 2. Dual Camera Setup: Orthographic (Isometric God-View) & Perspective (Third-Person Character Control)
    const aspect = width / height;
    const d = 36;
    orthoCamera = new THREE.OrthographicCamera(-d * aspect, d * aspect, d, -d, 1, 1000);
    orthoCamera.position.set(45, 45, 45);
    orthoCamera.lookAt(0, 0, 0);

    perspCamera = new THREE.PerspectiveCamera(58, aspect, 0.1, 1000);
    perspCamera.position.set(0, 5, 10);

    camera = orthoCamera;

    // 3. Renderer with Soft Shadows
    renderer = new THREE.WebGLRenderer({ antialias: true, alpha: false });
    renderer.setSize(width, height);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.shadowMap.enabled = true;
    renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    canvasMount.appendChild(renderer.domElement);

    // 4. Orbit Controls
    if (window.THREE && window.THREE.OrbitControls) {
      controls = new THREE.OrbitControls(camera, renderer.domElement);
      controls.enableDamping = true;
      controls.dampingFactor = 0.08;
      controls.maxPolarAngle = Math.PI / 2.15;
      controls.minZoom = 0.5;
      controls.maxZoom = 2.5;
    }

    // 4b. Character Keyboard & Mouse Controls
    setupCharacterKeyControls();

    // 5. Lighting Setup
    setupLighting();

    // 6. Ground Base
    setupGround();

    // 7. Raycaster for clicking buildings & agents
    setupRaycaster();

    // 8. Load Initial Simulation Mode
    setSimulationMode(state.mode);

    // 9. Resize Listener
    window.addEventListener('resize', onWindowResize);

    // 10. Start Animation Loop
    let lastTime = performance.now();
    let frameCount = 0;
    let lastFpsUpdate = performance.now();

    function animate(currentTime) {
      animationFrameId = requestAnimationFrame(animate);

      const delta = Math.min((currentTime - lastTime) / 1000, 0.1);
      lastTime = currentTime;

      frameCount++;
      if (currentTime - lastFpsUpdate > 500) {
        state.metrics.fps = Math.round((frameCount * 1000) / (currentTime - lastFpsUpdate));
        frameCount = 0;
        lastFpsUpdate = currentTime;
        updateStatsHUD();
      }

      // Character Third-Person Camera vs God-View Isometric Camera
      if (controlledAgent && controlledAgent.mesh) {
        if (controls) controls.enabled = false;
        updateControlledAgent(delta);

        // Elevated third-person chase camera (RPG over-the-shoulder perspective)
        const charPos = controlledAgent.mesh.position;
        const rotY = controlledAgent.mesh.rotation.y;
        const camDist = 7.2;
        const camHeight = 4.6;
        const targetCamX = charPos.x - Math.sin(rotY) * camDist;
        const targetCamZ = charPos.z - Math.cos(rotY) * camDist;
        const targetCamY = charPos.y + camHeight;

        perspCamera.position.lerp(new THREE.Vector3(targetCamX, targetCamY, targetCamZ), 0.16);

        const lookDist = 3.5;
        const targetLookX = charPos.x + Math.sin(rotY) * lookDist;
        const targetLookZ = charPos.z + Math.cos(rotY) * lookDist;
        const targetLookY = charPos.y + 1.35;
        perspCamera.lookAt(targetLookX, targetLookY, targetLookZ);

        camera = perspCamera;
      } else {
        if (controls) {
          controls.enabled = true;
          controls.object = orthoCamera;
        }
        camera = orthoCamera;

        // Smooth Camera Transitions & Agent Follow (God-View Mode)
        if (followAgent && followAgent.mesh) {
          controls.target.lerp(followAgent.mesh.position, 0.08);
        } else if (isTransitioningCamera) {
          controls.target.lerp(targetLookAt, 0.08);
          if (controls.target.distanceTo(targetLookAt) < 0.15) {
            controls.target.copy(targetLookAt);
            isTransitioningCamera = false;
          }
        }
      }

      // Night Torches Ambient Flicker
      if (state.timeOfDay === 'night' && nightTorchesGroup) {
        nightTorchesGroup.children.forEach((torch, idx) => {
          if (torch.isPointLight) {
            torch.intensity = 2.0 + Math.sin(currentTime * 0.007 + idx * 1.5) * 0.4;
          }
        });
      }

      // Rain Particles Simulation
      if (rainActive && rainGroup) {
        rainTimer -= delta;
        const positions = rainGroup.geometry.attributes.position.array;
        for (let i = 1; i < positions.length; i += 3) {
          positions[i] -= 42.0 * delta;
          if (positions[i] < 0) positions[i] = 38.0;
        }
        rainGroup.geometry.attributes.position.needsUpdate = true;
        if (rainTimer <= 0) {
          rainActive = false;
          scene.remove(rainGroup);
        }
      }

      // Day-Night natural cycle flow
      if (state.dayNightCycle && state.dayNightCycle.autoProgress && state.isPlaying) {
        state.dayNightCycle.timer -= delta * state.speedMultiplier;
        if (state.dayNightCycle.timer <= 0) {
          state.dayNightCycle.timer = state.dayNightCycle.durationPerPhase;
          const phases = ['dawn', 'noon', 'dusk', 'night'];
          const nextIdx = (phases.indexOf(state.timeOfDay) + 1) % phases.length;
          window.setTimeOfDay(phases[nextIdx]);
        }
      }

      if (state.isPlaying) {
        updateSimulation(delta * state.speedMultiplier);
      }

      // Skeletal Animation Mixers update (Knight Walking/Running)
      if (activeMixers.length > 0) {
        const animDelta = delta * (state.isPlaying ? state.speedMultiplier : (controlledAgent ? 1.0 : 0));
        for (let i = 0; i < activeMixers.length; i++) {
          activeMixers[i].update(animDelta);
        }
      }

      if (!controlledAgent && controls) controls.update();
      renderer.render(scene, camera);

      // Render Tactical Minimap
      drawMinimap();
    }

    animate(performance.now());
  };

  // ------------------------------------------------------------------------
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
  // CAMEL MESH GENERATOR (FOR CARAVANS ENTERING THE CITY)
  // ========================================================================


  function buildCamelMesh(cargo) {
    const group = new THREE.Group();
    const camelMat = new THREE.MeshStandardMaterial({ color: 0xc2854e, roughness: 0.8 });
    const cargoMat = new THREE.MeshStandardMaterial({ color: 0xb45309, roughness: 0.6 });

    const body = new THREE.Mesh(new THREE.BoxGeometry(2.4, 1.2, 1.0), camelMat);
    body.position.y = 1.3;
    body.castShadow = true;
    group.add(body);

    const hump = new THREE.Mesh(new THREE.BoxGeometry(0.9, 0.7, 0.8), camelMat);
    hump.position.set(-0.2, 2.05, 0);
    group.add(hump);

    const neck = new THREE.Mesh(new THREE.BoxGeometry(0.5, 1.2, 0.5), camelMat);
    neck.position.set(1.1, 2.0, 0);
    neck.rotation.z = -0.3;
    group.add(neck);

    const head = new THREE.Mesh(new THREE.BoxGeometry(0.8, 0.5, 0.5), camelMat);
    head.position.set(1.4, 2.5, 0);
    group.add(head);

    [-0.7, 0.7].forEach((lx) => {
      [-0.35, 0.35].forEach((lz) => {
        const leg = new THREE.Mesh(new THREE.BoxGeometry(0.25, 1.3, 0.25), camelMat);
        leg.position.set(lx, 0.65, lz);
        leg.castShadow = true;
        group.add(leg);
      });
    });

    [-0.55, 0.55].forEach((sideZ) => {
      const pack = new THREE.Mesh(new THREE.BoxGeometry(1.2, 0.7, 0.4), cargoMat);
      pack.position.set(-0.2, 1.3, sideZ);
      pack.castShadow = true;
      group.add(pack);
    });

    return group;
  }

  // ========================================================================
  // SIMULATION 2: REALISTIC ANCIENT CITY-STATE (10 LANDMARKS & ABM)
  // ========================================================================

  function registerBuilding(data, group, radius = 9.0) {
    data.mesh = group;
    data.radius = radius;
    group.userData.building = data;
    group.traverse((child) => {
      if (child.isMesh) {
        child.userData.building = data;
      }
    });
    interactiveBuildings.push(data);
    simGroup.add(group);
  }

  function loadBushAsset(onLoaded) {
    if (cachedBushModel) {
      if (onLoaded) onLoaded(cachedBushModel);
      return;
    }

    function doLoad() {
      if (!window.THREE || !window.THREE.GLTFLoader) {
        console.warn('THREE.GLTFLoader is not ready yet');
        return;
      }
      const loader = new THREE.GLTFLoader();
      const modelPath = '/models/Bush03.glb';
      loader.load(
        modelPath,
        (gltf) => {
          cachedBushModel = gltf.scene;
          cachedBushModel.traverse((child) => {
            if (child.isMesh) {
              child.castShadow = true;
              child.receiveShadow = true;
              if (child.material) {
                if (child.material.transparent) {
                  child.material.alphaTest = 0.45;
                  child.material.depthWrite = true;
                  child.material.side = THREE.DoubleSide;
                }
                if (child.name.includes('Fruit') || (child.material.name && child.material.name.includes('Fruit'))) {
                  child.material.emissive = new THREE.Color(0xf43f5e);
                  child.material.emissiveIntensity = 0.8;
                }
              }
            }
          });
          if (onLoaded) onLoaded(cachedBushModel);
        },
        undefined,
        (err) => {
          console.warn('Fallback: loading Bush03.glb from /storage/Bush03.glb', err);
          loader.load('/storage/Bush03.glb', (gltf2) => {
            cachedBushModel = gltf2.scene;
            cachedBushModel.traverse((child) => {
              if (child.isMesh) {
                child.castShadow = true;
                child.receiveShadow = true;
                if (child.material && child.material.transparent) {
                  child.material.alphaTest = 0.45;
                  child.material.depthWrite = true;
                  child.material.side = THREE.DoubleSide;
                }
              }
            });
            if (onLoaded) onLoaded(cachedBushModel);
          });
        }
      );
    }

    if (!window.THREE || !window.THREE.GLTFLoader) {
      const s = document.createElement('script');
      s.src = '/GLTFLoader.js';
      s.onload = doLoad;
      document.head.appendChild(s);
    } else {
      doLoad();
    }
  }

  function createBushInstance(scale = 1.0) {
    if (!cachedBushModel) return null;
    const instance = cachedBushModel.clone(true);
    instance.scale.set(scale, scale, scale);
    return instance;
  }

  // ------------------------------------------------------------------------
  // Knight 3D Character Asset Loader & Animated Agent System
  // ------------------------------------------------------------------------
  let isKnightLoading = false;
  let knightLoadCallbacks = [];

  function loadKnightAsset(onLoaded) {
    if (cachedKnightModel && knightClips.walk && knightClips.run) {
      if (onLoaded) onLoaded(cachedKnightModel);
      return;
    }

    if (onLoaded) knightLoadCallbacks.push(onLoaded);
    if (isKnightLoading) return;
    isKnightLoading = true;

    function doLoad() {
      if (!window.THREE || !window.THREE.GLTFLoader) {
        console.warn('THREE.GLTFLoader is not ready yet');
        return;
      }
      const loader = new THREE.GLTFLoader();
      const walkingPath = '/models/knight/knight_walking.glb';
      const runningPath = '/models/knight/knight_running.glb';

      loader.load(
        walkingPath,
        (gltfWalk) => {
          cachedKnightModel = gltfWalk.scene;
          knightClips.walk = gltfWalk.animations && gltfWalk.animations[0];

          cachedKnightModel.traverse((child) => {
            if (child.isMesh || child.isSkinnedMesh) {
              child.castShadow = true;
              child.receiveShadow = true;
              if (child.material) {
                child.material.roughness = 0.35;
                child.material.metalness = 0.82;
              }
            }
          });

          // Also load running animation
          loader.load(
            runningPath,
            (gltfRun) => {
              knightClips.run = gltfRun.animations && gltfRun.animations[0];
              isKnightLoading = false;
              const cbs = knightLoadCallbacks.slice();
              knightLoadCallbacks = [];
              cbs.forEach((cb) => {
                try { cb(cachedKnightModel); } catch (e) { console.error(e); }
              });
            },
            undefined,
            (err) => {
              console.warn('Fallback: could not load knight running anim', err);
              isKnightLoading = false;
              const cbs = knightLoadCallbacks.slice();
              knightLoadCallbacks = [];
              cbs.forEach((cb) => {
                try { cb(cachedKnightModel); } catch (e) { console.error(e); }
              });
            }
          );
        },
        undefined,
        (err) => {
          console.warn('Error loading knight walking model:', err);
          isKnightLoading = false;
        }
      );
    }

    if (!window.THREE || !window.THREE.GLTFLoader) {
      const s = document.createElement('script');
      s.src = '/GLTFLoader.js';
      s.onload = doLoad;
      document.head.appendChild(s);
    } else {
      doLoad();
    }
  }

  function createKnightAgent(x, z, role = '圣殿重甲骑士', isHero = false) {
    if (!cachedKnightModel || !simGroup) return null;

    let knightMesh = null;
    if (window.THREE && window.THREE.SkeletonUtils && typeof window.THREE.SkeletonUtils.clone === 'function') {
      knightMesh = THREE.SkeletonUtils.clone(cachedKnightModel);
    } else {
      knightMesh = cachedKnightModel.clone(true);
    }

    const scale = 1.05;
    knightMesh.scale.set(scale, scale, scale);
    knightMesh.position.set(x, 0, z);

    let mixer = null;
    let actions = {};
    if (window.THREE && window.THREE.AnimationMixer) {
      mixer = new THREE.AnimationMixer(knightMesh);
      if (knightClips.walk) {
        actions.walk = mixer.clipAction(knightClips.walk);
        actions.walk.setLoop(THREE.LoopRepeat);
      }
      if (knightClips.run) {
        actions.run = mixer.clipAction(knightClips.run);
        actions.run.setLoop(THREE.LoopRepeat);
      }
      activeMixers.push(mixer);
    }

    const agentObj = {
      id: isHero ? '圣殿大骑士-统帅' : `圣殿骑士-${knightAgents.length + 1}`,
      type: 'ancient_knight',
      role: role,
      color: 0x94a3b8,
      mesh: knightMesh,
      mixer: mixer,
      actions: actions,
      currentAnim: null,
      shekels: 150,
      devotion: 100,
      currentPhase: state.timeOfDay || 'dusk',
      subState: 'PATROLLING',
      task: '⚔️ 披挂铠甲执剑守望城防',
      dialogue: '“圣殿骑士守望耶路撒冷，愿公义与和平护佑此城！”',
      stateTimer: 6.0,
      patrolPoints: [
        [0, 24],   // South Gate
        [0, 5],    // Central Bazaar
        [0, -6],   // Altar
        [-14, -10] // Palace
      ],
      currentPatrolIdx: Math.floor(Math.random() * 4),
      target: new THREE.Vector3(x, 0, z),
      setAnimation: function (animName, fadeTime = 0.22) {
        if (!this.actions || this.currentAnim === animName) return;
        const nextAction = animName ? this.actions[animName] : null;
        const prevAction = this.currentAnim ? this.actions[this.currentAnim] : null;

        if (prevAction) {
          if (nextAction) {
            prevAction.crossFadeTo(nextAction, fadeTime, true);
            nextAction.reset().play();
          } else {
            prevAction.fadeOut(fadeTime);
          }
        } else if (nextAction) {
          nextAction.reset().fadeIn(fadeTime).play();
        }
        this.currentAnim = animName;
      }
    };

    knightMesh.userData.agent = agentObj;
    knightMesh.traverse((child) => {
      if (child.isMesh || child.isSkinnedMesh) {
        child.userData.agent = agentObj;
      }
    });

    simGroup.add(knightMesh);
    agents.push(agentObj);
    knightAgents.push(agentObj);

    return agentObj;
  }

  function spawnCityKnights() {
    loadKnightAsset(() => {
      if (!simGroup) return;
      // 1. South Citadel Gate Commander Knight
      createKnightAgent(0, 25, '圣殿六室门卫队长');
      // 2. Temple Sanctuary Paladin Knight
      createKnightAgent(0, -6, '圣所铜祭坛守护骑士');
      // 3. Lebanon Palace Royal Guard Knight
      createKnightAgent(-15, -10, '林宫王廷禁卫骑士');
    });
  }

  function buildCanaanBushes() {
    loadBushAsset((model) => {
      if (!simGroup) return;

      // 1. Primary Interactive Landmark: The Sacred Canaan Bush & Flora
      // Located at (10, 0, -8) south of Solomon's Temple Complex
      const mainBushGroup = new THREE.Group();
      mainBushGroup.position.set(10, 0, -8);

      const mainMesh = createBushInstance(1.35);
      if (mainMesh) {
        mainBushGroup.add(mainMesh);
      }

      // Decorative limestone planter/pedestal base
      const planterMat = new THREE.MeshStandardMaterial({ color: 0xfef3c7, roughness: 0.6 });
      const planter = new THREE.Mesh(new THREE.CylinderGeometry(1.6, 1.8, 0.45, 16), planterMat);
      planter.position.y = 0.22;
      planter.receiveShadow = true;
      mainBushGroup.add(planter);

      // Gold rim accent
      const goldTrim = new THREE.Mesh(new THREE.TorusGeometry(1.65, 0.08, 8, 24), new THREE.MeshStandardMaterial({ color: 0xfde047, metalness: 0.7, roughness: 0.3 }));
      goldTrim.rotation.x = Math.PI / 2;
      goldTrim.position.y = 0.45;
      mainBushGroup.add(goldTrim);

      registerBuilding({
        id: 'bush_flora',
        name: '🌿 迦南美地圣木与佳果灌木 (Sacred Bush & Flora of Canaan)',
        district: '摩利亚山圣苑与丰产佳果植被区',
        lectureNote: '《申命记》第八章称颂迦南应许美地出产小麦、大麦、葡萄、无花果、石榴与橄榄。此丛灌木（Bush03 3D高精模型）象征旷野荆棘圣火的显圣记忆，也是古代近东祭司典礼中献初熟圣果与制作圣膏香油的重要自然馈赠。',
        getLiveStats: () => `灌木长势: 丰盛繁茂 | 枝头石榴与佳果: 累累缀枝 | 圣地甘霖润泽度: 100%`,
        actionName: '🌿 采摘初熟圣地佳果 (+民心与圣香香气)'
      }, mainBushGroup, 6.0);
      bushInstances.push(mainBushGroup);

      // 2. Oasis Bush Clusters near Gihon Cistern / Pool
      const oasisPositions = [
        [16, 14, 1.1, 0.4],
        [28, 14, 1.0, 1.2],
        [25, 4, 0.9, -0.6],
        [18, 5, 0.85, 2.1],
      ];
      oasisPositions.forEach(([bx, bz, bscale, brot]) => {
        const b = createBushInstance(bscale);
        if (b) {
          b.position.set(bx, 0, bz);
          b.rotation.y = brot;
          simGroup.add(b);
          bushInstances.push(b);
        }
      });

      // 3. Agricultural Bush Grove near Olive Press & Terraces
      const orchardPositions = [
        [-16, 22, 1.15, 0.8],
        [-27, 21, 0.95, -1.3],
        [-17, 13, 0.85, 1.7],
      ];
      orchardPositions.forEach(([bx, bz, bscale, brot]) => {
        const b = createBushInstance(bscale);
        if (b) {
          b.position.set(bx, 0, bz);
          b.rotation.y = brot;
          simGroup.add(b);
          bushInstances.push(b);
        }
      });

      // 4. Courtyard Greens near Four-Room Residential Houses & Palace
      const domesticPositions = [
        [16, -4, 0.9, 0.5],
        [28, -8, 0.8, -0.9],
        [-12, -10, 1.05, 1.0],
      ];
      domesticPositions.forEach(([bx, bz, bscale, brot]) => {
        const b = createBushInstance(bscale);
        if (b) {
          b.position.set(bx, 0, bz);
          b.rotation.y = brot;
          simGroup.add(b);
          bushInstances.push(b);
        }
      });
    });
  }

  function initTempleCitySimulation() {
    interactiveBuildings = [];
    smokeParticles = [];
    bushInstances = [];

    // 10 Archaeological Landmarks + Canaan Bush Flora Landmark
    buildSolomonTempleComplex(0, -20);
    buildPalaceOfLebanon(-22, -18);
    buildRoyalGranary(22, -18);
    buildGihonCistern(22, 10);
    buildBronzeFoundry(-22, 4);
    buildOlivePress(-22, 18);
    buildCentralBazaar(0, 6);
    buildFourRoomHouses(22, -4);
    buildSixChamberGate(0, 32);
    buildPerimeterWalls();
    buildCanaanBushes();

    spawnAncientCitizens();
    spawnCityKnights();
    state.metrics.agentCount = state.templeCity.citizenCount + 3;
  }

  function buildSolomonTempleComplex(x, z) {
    const group = new THREE.Group();
    group.position.set(x, 0, z);

    const goldMat = new THREE.MeshStandardMaterial({ color: 0xfde047, metalness: 0.7, roughness: 0.2 });
    const limestoneMat = new THREE.MeshStandardMaterial({ color: 0xfef3c7, roughness: 0.55 });
    const bronzeMat = new THREE.MeshStandardMaterial({ color: 0xc2854e, metalness: 0.65, roughness: 0.35 });

    const terrace = new THREE.Mesh(new THREE.BoxGeometry(26, 1.2, 32), limestoneMat);
    terrace.position.y = 0.6;
    terrace.receiveShadow = true;
    group.add(terrace);

    const temple = new THREE.Mesh(new THREE.BoxGeometry(15, 9.5, 20), limestoneMat);
    temple.position.set(0, 5.95, -3);
    temple.castShadow = true;
    temple.receiveShadow = true;
    group.add(temple);

    const cornice = new THREE.Mesh(new THREE.BoxGeometry(16, 0.9, 21), goldMat);
    cornice.position.set(0, 11.1, -3);
    group.add(cornice);

    const porch = new THREE.Mesh(new THREE.BoxGeometry(15, 8.0, 5.5), limestoneMat);
    porch.position.set(0, 5.2, 9.75);
    porch.castShadow = true;
    group.add(porch);

    const porchRoof = new THREE.Mesh(new THREE.BoxGeometry(15.6, 0.7, 5.8), goldMat);
    porchRoof.position.set(0, 9.5, 9.75);
    group.add(porchRoof);

    // Jachin & Boaz
    [-3.8, 3.8].forEach((px) => {
      const pillar = new THREE.Mesh(new THREE.CylinderGeometry(0.5, 0.55, 10, 16), bronzeMat);
      pillar.position.set(px, 5.5, 13.5);
      pillar.castShadow = true;
      group.add(pillar);

      const capital = new THREE.Mesh(new THREE.CylinderGeometry(0.9, 0.55, 1.5, 16), goldMat);
      capital.position.set(px, 11.2, 13.5);
      group.add(capital);
    });

    // Great Bronze Altar & Horns
    const altar = new THREE.Mesh(new THREE.BoxGeometry(5.0, 2.0, 5.0), bronzeMat);
    altar.position.set(0, 2.2, 19.5);
    altar.castShadow = true;
    group.add(altar);

    [-2.2, 2.2].forEach((hx) => {
      [-2.2, 2.2].forEach((hz) => {
        const horn = new THREE.Mesh(new THREE.BoxGeometry(0.5, 0.6, 0.5), goldMat);
        horn.position.set(hx, 3.4, 19.5 + hz);
        group.add(horn);
      });
    });

    // Altar Smoke Particles
    for (let i = 0; i < 14; i++) {
      const p = new THREE.Mesh(
        new THREE.SphereGeometry(0.35 + Math.random() * 0.35, 8, 8),
        new THREE.MeshBasicMaterial({ color: 0xffedd5, transparent: true, opacity: 0.6 })
      );
      p.startX = x;
      p.startY = 3.5;
      p.startZ = z + 19.5;
      p.maxDist = 12.0;
      p.speedY = 1.8 + Math.random() * 1.5;
      p.position.set(p.startX + (Math.random() - 0.5) * 1.2, p.startY + Math.random() * 6, p.startZ + (Math.random() - 0.5) * 1.2);
      simGroup.add(p);
      smokeParticles.push(p);
    }

    // Molten Sea
    const basin = new THREE.Mesh(new THREE.CylinderGeometry(2.4, 1.9, 1.6, 16), bronzeMat);
    basin.position.set(8.5, 2.0, 16);
    basin.castShadow = true;
    group.add(basin);

    const water = new THREE.Mesh(new THREE.CircleGeometry(2.2, 16), new THREE.MeshBasicMaterial({ color: 0x38bdf8 }));
    water.rotation.x = -Math.PI / 2;
    water.position.set(8.5, 2.8, 16);
    group.add(water);

    registerBuilding({
      id: 'temple_solomon',
      name: '🏛️ 所罗门第一圣殿 (The First Temple)',
      district: '圣殿山圣所核心区 (Acropolis)',
      lectureNote: '建于公元前10世纪摩利亚山，历时7年由推罗与希伯来工匠共建。殿内以黎巴嫩雪松木贴墙并包上纯金，至圣所内安放神圣约柜。门前立有雅斤（祂必建立）与波阿斯（力量在祂）两根青铜立柱，是古代以色列民族信仰与政治凝聚的核心。',
      getLiveStats: () => `圣殿金库: <strong>${state.templeCity.templeTreasury}</strong> 舍客勒 | 圣香奉纳: <strong>${state.templeCity.sacredAroma.toFixed(0)}%</strong> | 每日行晨昏两番燔祭`,
      actionName: '🔥 举行晨昏燔祭 (+敬虔 & 金库)'
    }, group, 14.0);
  }

  function buildPalaceOfLebanon(x, z) {
    const group = new THREE.Group();
    group.position.set(x, 0, z);

    const cedarMat = new THREE.MeshStandardMaterial({ color: 0x78350f, roughness: 0.7 });
    const stoneMat = new THREE.MeshStandardMaterial({ color: 0xfef3c7, roughness: 0.6 });
    const royalPurple = new THREE.MeshStandardMaterial({ color: 0x6b21a8, roughness: 0.5 });

    const hall = new THREE.Mesh(new THREE.BoxGeometry(16, 7.0, 14), stoneMat);
    hall.position.y = 3.5;
    hall.castShadow = true;
    hall.receiveShadow = true;
    group.add(hall);

    for (let c = -6; c <= 6; c += 2.4) {
      const col = new THREE.Mesh(new THREE.CylinderGeometry(0.35, 0.35, 7.0, 12), cedarMat);
      col.position.set(c, 3.5, 7.2);
      col.castShadow = true;
      group.add(col);
    }

    const upper = new THREE.Mesh(new THREE.BoxGeometry(12, 3.8, 10), cedarMat);
    upper.position.set(0, 8.9, 0);
    upper.castShadow = true;
    group.add(upper);

    [-4, 4].forEach((bx) => {
      const banner = new THREE.Mesh(new THREE.BoxGeometry(1.4, 4.0, 0.1), royalPurple);
      banner.position.set(bx, 5.0, 7.3);
      group.add(banner);
    });

    registerBuilding({
      id: 'palace_lebanon',
      name: '👑 利巴嫩林宫与王廷大殿 (Palace of the Forest of Lebanon)',
      district: '王家行政与官邸区',
      lectureNote: '所罗门王历时十三年建成的宏伟行政宫殿。因以45根黎巴嫩高山雪松为梁柱如同一片森林而得名。内设著名的象牙金宝座，是国王行使司法裁决与接见各国使臣（如示巴女王）的行政枢纽。',
      getLiveStats: () => `行政运转: 完好 | 皇家卫队: 50 勇士巡逻 | 示巴女王与各国岁贡通达`,
      actionName: '📜 颁布王室律令 (+贸易繁荣)'
    }, group, 11.0);
  }

  function buildRoyalGranary(x, z) {
    const group = new THREE.Group();
    group.position.set(x, 0, z);

    const stoneMat = new THREE.MeshStandardMaterial({ color: 0xd6cbba, roughness: 0.8 });
    const mudMat = new THREE.MeshStandardMaterial({ color: 0xd97706, roughness: 0.9 });
    const sackMat = new THREE.MeshStandardMaterial({ color: 0xb45309, roughness: 0.7 });

    [
      { ox: -3.8, oz: 0, r: 2.3, h: 6.8 },
      { ox: 1.2, oz: -1.2, r: 2.6, h: 7.6 },
      { ox: 5.4, oz: 1.8, r: 2.2, h: 6.2 },
    ].forEach((s) => {
      const silo = new THREE.Mesh(new THREE.CylinderGeometry(s.r, s.r * 1.08, s.h, 16), stoneMat);
      silo.position.set(s.ox, s.h / 2, s.oz);
      silo.castShadow = true;
      group.add(silo);

      const cone = new THREE.Mesh(new THREE.ConeGeometry(s.r * 1.15, 2.2, 16), mudMat);
      cone.position.set(s.ox, s.h + 1.1, s.oz);
      cone.castShadow = true;
      group.add(cone);
    });

    const depot = new THREE.Mesh(new THREE.BoxGeometry(13, 3.8, 6.5), mudMat);
    depot.position.set(0, 1.9, 6.0);
    depot.castShadow = true;
    group.add(depot);

    [-2, 0, 2].forEach((sx) => {
      const sack = new THREE.Mesh(new THREE.BoxGeometry(0.9, 0.6, 0.7), sackMat);
      sack.position.set(sx, 0.4, 9.6);
      group.add(sack);
    });

    registerBuilding({
      id: 'royal_granary',
      name: '🌾 皇家战略粮仓与谷物筒仓 (Royal Granary & Silos)',
      district: '王家仓廪储备区',
      lectureNote: '古代中东应对极端旱灾、蝗灾与异族围城的生命线。集中储备来自耶斯列谷地的大麦、小麦与豆类，支撑常备军与利未祭司供养，并在荒年开仓救济平民。',
      getLiveStats: () => `战略储麦: <strong>${state.templeCity.grainStorage}</strong> 公吨 | 防灾缓冲期: 10 个月 | 干燥度: 优`,
      actionName: '🌾 开仓赈济分粮 (+民心满意)'
    }, group, 10.0);
  }

  function buildGihonCistern(x, z) {
    const group = new THREE.Group();
    group.position.set(x, 0, z);

    const stoneMat = new THREE.MeshStandardMaterial({ color: 0xd6cbba, roughness: 0.75 });
    const woodMat = new THREE.MeshStandardMaterial({ color: 0x78350f, roughness: 0.7 });
    const waterMat = new THREE.MeshStandardMaterial({ color: 0x0284c7, roughness: 0.1, transparent: true, opacity: 0.85 });

    const basinBorder = new THREE.Mesh(new THREE.BoxGeometry(14, 0.9, 12), stoneMat);
    basinBorder.position.y = 0.45;
    basinBorder.castShadow = true;
    group.add(basinBorder);

    const pool = new THREE.Mesh(new THREE.PlaneGeometry(12.4, 10.4), waterMat);
    pool.rotation.x = -Math.PI / 2;
    pool.position.y = 0.55;
    group.add(pool);

    for (let s = 0; s < 4; s++) {
      const step = new THREE.Mesh(new THREE.BoxGeometry(3.5, 0.2, 0.6), stoneMat);
      step.position.set(-4.5, 0.7 - s * 0.15, -4.5 + s * 0.6);
      group.add(step);
    }

    const p1 = new THREE.Mesh(new THREE.CylinderGeometry(0.18, 0.18, 3.8), woodMat);
    p1.position.set(3, 1.9, 0);
    group.add(p1);
    const p2 = p1.clone();
    p2.position.set(-3, 1.9, 0);
    group.add(p2);
    const beam = new THREE.Mesh(new THREE.BoxGeometry(6.4, 0.25, 0.25), woodMat);
    beam.position.set(0, 3.8, 0);
    group.add(beam);

    registerBuilding({
      id: 'gihon_cistern',
      name: '💧 基训泉与中央地下蓄水工程 (Gihon Spring Cistern)',
      district: '城邦水利工程生命线',
      lectureNote: '迦南夏季干旱无雨，基训泉是耶路撒冷唯一的常年自然水源。古代开凿地下暗渠与竖井（西罗亚工程雏形），将泉水隐秘引至城内蓄水池，攻城敌军无法切断水源，是城邦生存与繁荣的关键保障。',
      getLiveStats: () => `水池水位: <strong>${state.templeCity.waterSupply.toFixed(0)}%</strong> | 暗渠流量: 恒定涌出 | 每日汲水: 1200 陶瓮`,
      actionName: '💧 开启地下暗渠注水 (+蓄水至100%)'
    }, group, 9.5);
  }

  function buildBronzeFoundry(x, z) {
    const group = new THREE.Group();
    group.position.set(x, 0, z);

    const stoneMat = new THREE.MeshStandardMaterial({ color: 0x475569, roughness: 0.85 });
    const copperMat = new THREE.MeshStandardMaterial({ color: 0xea580c, metalness: 0.6, roughness: 0.35 });
    const glowMat = new THREE.MeshBasicMaterial({ color: 0xf97316 });

    const workshop = new THREE.Mesh(new THREE.BoxGeometry(11, 4.8, 9), stoneMat);
    workshop.position.y = 2.4;
    workshop.castShadow = true;
    group.add(workshop);

    const chimney = new THREE.Mesh(new THREE.CylinderGeometry(0.7, 0.9, 6.8, 12), stoneMat);
    chimney.position.set(-3.5, 5.2, -2.5);
    group.add(chimney);

    const forge = new THREE.Mesh(new THREE.BoxGeometry(2.4, 1.8, 0.4), glowMat);
    forge.position.set(0, 1.5, 4.6);
    group.add(forge);

    const anvil = new THREE.Mesh(new THREE.BoxGeometry(0.8, 0.9, 0.6), copperMat);
    anvil.position.set(2.4, 0.45, 5.4);
    group.add(anvil);

    [-1, 0, 1].forEach((bx) => {
      const bar = new THREE.Mesh(new THREE.BoxGeometry(0.8, 0.25, 0.4), copperMat);
      bar.position.set(bx * 0.9 - 2.5, 0.2, 5.4);
      group.add(bar);
    });

    registerBuilding({
      id: 'bronze_foundry',
      name: '⚒️ 推罗青铜工匠冶炼坊 (Phoenician Bronze Foundry)',
      district: '手工业与金属工坊区',
      lectureNote: '推罗王希兰派遣精通铜铁冶炼的腓尼基大师户兰（Hiram）在此督工。圣殿所有的洗礼盆、铲子、肉叉、灯台等青铜圣器皆由推罗工匠在疏割与撒拉但之间的泥土中铸造。',
      getLiveStats: () => `熔炉温度: 1080℃ | 铸造项目: 圣所金灯台与铜盆 | 工匠组: 腓尼基与希伯来大师`,
      actionName: '⚒️ 锻造精炼青铜礼器 (+工商业)'
    }, group, 8.5);
  }

  function buildOlivePress(x, z) {
    const group = new THREE.Group();
    group.position.set(x, 0, z);

    const stoneMat = new THREE.MeshStandardMaterial({ color: 0xd6cbba, roughness: 0.7 });
    const woodMat = new THREE.MeshStandardMaterial({ color: 0x78350f, roughness: 0.7 });
    const jarMat = new THREE.MeshStandardMaterial({ color: 0xc2410c, roughness: 0.8 });

    const roof = new THREE.Mesh(new THREE.BoxGeometry(11, 0.4, 9), woodMat);
    roof.position.y = 4.2;
    group.add(roof);

    [-4.5, 4.5].forEach((px) => {
      [-3.5, 3.5].forEach((pz) => {
        const post = new THREE.Mesh(new THREE.CylinderGeometry(0.2, 0.2, 4.2), woodMat);
        post.position.set(px, 2.1, pz);
        group.add(post);
      });
    });

    const basin = new THREE.Mesh(new THREE.CylinderGeometry(2.4, 2.6, 0.9, 16), stoneMat);
    basin.position.y = 0.45;
    basin.castShadow = true;
    group.add(basin);

    pressWheel = new THREE.Mesh(new THREE.CylinderGeometry(1.2, 1.2, 0.5, 16), stoneMat);
    pressWheel.rotation.x = Math.PI / 2;
    pressWheel.position.set(0, 1.25, 0);
    pressWheel.castShadow = true;
    group.add(pressWheel);

    [-3.0, -1.8, 1.8, 3.0].forEach((jx) => {
      const jar = new THREE.Mesh(new THREE.CylinderGeometry(0.4, 0.25, 1.1, 10), jarMat);
      jar.position.set(jx, 0.55, 3.5);
      jar.castShadow = true;
      group.add(jar);
    });

    registerBuilding({
      id: 'olive_press',
      name: '🫒 古迦南橄榄石磨压榨坊 (Olive Press / Beit Habad)',
      district: '手工业与农业深加工区',
      lectureNote: '迦南土地三大丰产标志之一（五谷、新酒、纯油）。首道冷压清油专门供给圣殿金灯台长明不灭与祭司君王受膏；随后的重石压榨油用于民间食用与制皂，残渣作燃料。',
      getLiveStats: () => `初熟圣油产量: 35 瓮 | 压榨石轮运转中 | 纯油纯度: 100%`,
      actionName: '🫒 研磨冷榨特级纯油 (+圣香香气)'
    }, group, 8.5);
  }

  function buildCentralBazaar(x, z) {
    const group = new THREE.Group();
    group.position.set(x, 0, z);

    const woodMat = new THREE.MeshStandardMaterial({ color: 0x78350f, roughness: 0.7 });
    const colors = [0x0284c7, 0xd97706, 0x9333ea, 0xe11d48, 0x16a34a, 0xf59e0b];

    const stalls = [
      { ox: -4.5, oz: -3.0, c: colors[0] },
      { ox: 4.5, oz: -3.0, c: colors[1] },
      { ox: -4.5, oz: 3.0, c: colors[2] },
      { ox: 4.5, oz: 3.0, c: colors[3] },
    ];

    stalls.forEach((s) => {
      const counter = new THREE.Mesh(new THREE.BoxGeometry(3.6, 0.9, 1.8), woodMat);
      counter.position.set(s.ox, 0.45, s.oz);
      counter.castShadow = true;
      group.add(counter);

      const awningMat = new THREE.MeshStandardMaterial({ color: s.c, roughness: 0.6 });
      const awning = new THREE.Mesh(new THREE.BoxGeometry(4.0, 0.1, 2.6), awningMat);
      awning.position.set(s.ox, 2.5, s.oz);
      awning.rotation.x = 0.15;
      awning.castShadow = true;
      group.add(awning);

      [-1.8, 1.8].forEach((ax) => {
        [-1.0, 1.0].forEach((az) => {
          const post = new THREE.Mesh(new THREE.CylinderGeometry(0.08, 0.08, 2.4), woodMat);
          post.position.set(s.ox + ax, 1.2, s.oz + az);
          group.add(post);
        });
      });
    });

    registerBuilding({
      id: 'central_bazaar',
      name: '🏺 迦南城邦中央彩篷大集市 (Central Bazaar & Shuk)',
      district: '商贸流转与集市区',
      lectureNote: '地中海与阿拉伯半岛商贸交汇中心。聚集了骆驼商队带来的阿拉伯乳香、没药、推罗紫布、埃及麻布与亚喀巴湾金砂，使用标准银舍客勒天平称重进行公平交易。',
      getLiveStats: () => `贸易繁荣度: <strong>${state.templeCity.tradeProsperity.toFixed(1)}x</strong> | 今日流水: <strong>${state.templeCity.marketTurnover}</strong> 舍客勒 | 摊位活跃: 100%`,
      actionName: '💰 举办列国贸易大集 (+交易流水)'
    }, group, 10.0);
  }

  function buildFourRoomHouses(x, z) {
    const group = new THREE.Group();
    group.position.set(x, 0, z);

    const mudbrickMat = new THREE.MeshStandardMaterial({ color: 0xd97706, roughness: 0.9 });
    const cedarMat = new THREE.MeshStandardMaterial({ color: 0x78350f, roughness: 0.7 });
    const stoneMat = new THREE.MeshStandardMaterial({ color: 0xd6cbba, roughness: 0.8 });

    const h1 = new THREE.Mesh(new THREE.BoxGeometry(13, 5.2, 10), mudbrickMat);
    h1.position.set(0, 2.6, -1);
    h1.castShadow = true;
    h1.receiveShadow = true;
    group.add(h1);

    const roof1 = new THREE.Mesh(new THREE.BoxGeometry(13.4, 0.4, 10.4), cedarMat);
    roof1.position.set(0, 5.4, -1);
    group.add(roof1);

    const parapet = new THREE.Mesh(new THREE.BoxGeometry(13.6, 0.6, 0.3), stoneMat);
    parapet.position.set(0, 5.8, 4.1);
    group.add(parapet);

    [-3, 0, 3].forEach((px) => {
      const pillar = new THREE.Mesh(new THREE.BoxGeometry(0.6, 4.8, 0.6), stoneMat);
      pillar.position.set(px, 2.4, 4.0);
      group.add(pillar);
    });

    const h2 = new THREE.Mesh(new THREE.BoxGeometry(10, 4.2, 8), mudbrickMat);
    h2.position.set(2, 2.1, 9.5);
    h2.castShadow = true;
    group.add(h2);

    registerBuilding({
      id: 'four_room_house',
      name: '🏡 经典铁器时代四室泥砖宅邸 (Israelite Four-Room House)',
      district: '城民居住生活街区',
      lectureNote: '圣经考古学铁器时代最典型民居形制。底层中央为露天庭院与牲畜圈，两侧长室储粮与手工作坊；二层为全家卧室；平顶露台（依据律法修筑护栏）可晾晒大麦、葡萄干并用于夜间纳凉祈祷。',
      getLiveStats: () => `居住世系: 约瑟支派工匠家族 (8口人) | 家内储物: 陶罐面粉充足 | 民心指数: <strong>${state.templeCity.citizenMorale}%</strong>`,
      actionName: '🕊️ 视察民间生活 (+城民满意度)'
    }, group, 11.0);
  }

  function buildSixChamberGate(x, z) {
    const group = new THREE.Group();
    group.position.set(x, 0, z);

    const stoneMat = new THREE.MeshStandardMaterial({ color: 0xd6cbba, roughness: 0.75 });
    const woodMat = new THREE.MeshStandardMaterial({ color: 0x78350f, roughness: 0.7 });

    [-6.2, 6.2].forEach((bx) => {
      const block = new THREE.Mesh(new THREE.BoxGeometry(8.5, 7.8, 9.5), stoneMat);
      block.position.set(bx, 3.9, 0);
      block.castShadow = true;
      block.receiveShadow = true;
      group.add(block);

      const parapet = new THREE.Mesh(new THREE.BoxGeometry(8.7, 0.9, 9.7), woodMat);
      parapet.position.set(bx, 8.25, 0);
      group.add(parapet);
    });

    const arch = new THREE.Mesh(new THREE.BoxGeometry(5.0, 1.6, 9.5), stoneMat);
    arch.position.set(0, 7.0, 0);
    arch.castShadow = true;
    group.add(arch);

    const elderBench = new THREE.Mesh(new THREE.BoxGeometry(6.0, 0.7, 1.2), stoneMat);
    elderBench.position.set(0, 0.35, -6.5);
    group.add(elderBench);

    [-1.8, 0, 1.8].forEach((ex) => {
      const elder = new THREE.Mesh(new THREE.CylinderGeometry(0.3, 0.4, 1.1), new THREE.MeshStandardMaterial({ color: 0xfef08a }));
      elder.position.set(ex, 1.1, -6.5);
      group.add(elder);
    });

    registerBuilding({
      id: 'city_gate_elders',
      name: '⚖️ 六室防御城门与长老审判座 (Six-Chamber City Gate & Judgement Seat)',
      district: '防务关隘与市政司法区',
      lectureNote: '在夏琐、米吉多与基色发掘出的所罗门标志性六室城门。古代中东城门口是军事第一防线，更是城邦最重要的公共法庭与契约见证地（如《路得记》波阿斯在城门办理赎买手续）。',
      getLiveStats: () => `城防守卫: 24 守备精兵 | 关卡排队: 畅通 | 今日长老调解: 2 桩契约完备`,
      actionName: '🛡️ 检阅城防卫队 (+民心安宁)'
    }, group, 12.0);
  }

  function buildPerimeterWalls() {
    const wallMat = new THREE.MeshStandardMaterial({ color: 0xd6cbba, roughness: 0.8 });

    const nWall = new THREE.Mesh(new THREE.BoxGeometry(64, 4.6, 2.0), wallMat);
    nWall.position.set(0, 2.3, -33);
    nWall.castShadow = true;
    simGroup.add(nWall);

    const wWall = new THREE.Mesh(new THREE.BoxGeometry(2.0, 4.6, 66), wallMat);
    wWall.position.set(-33, 2.3, 0);
    wWall.castShadow = true;
    simGroup.add(wWall);

    const eWall = new THREE.Mesh(new THREE.BoxGeometry(2.0, 4.6, 66), wallMat);
    eWall.position.set(33, 2.3, 0);
    eWall.castShadow = true;
    simGroup.add(eWall);

    [-22, 22].forEach((sx) => {
      const sWall = new THREE.Mesh(new THREE.BoxGeometry(20, 4.6, 2.0), wallMat);
      sWall.position.set(sx, 2.3, 33);
      sWall.castShadow = true;
      simGroup.add(sWall);
    });

    [-33, 33].forEach((tx) => {
      [-33, 33].forEach((tz) => {
        const tower = new THREE.Mesh(new THREE.BoxGeometry(4.2, 7.5, 4.2), wallMat);
        tower.position.set(tx, 3.75, tz);
        tower.castShadow = true;
        simGroup.add(tower);
      });
    });
  }

  // ------------------------------------------------------------------------
  // Ancient Citizen Daily Schedule Table (Circadian FSM System)
  // ------------------------------------------------------------------------
  const ROUTINE_SCHEDULE = {
    '利未祭司': {
      dawn: {
        target: [0, -1],
        spread: 3.0,
        task: '🔥 圣殿燔祭坛主礼晨祷',
        state: 'PRAYING',
        duration: 9.0,
        dialogue: '“诸天述说荣耀，晨光照亮摩利亚山。愿晨香如馨香之气上升。”',
        effect: { aroma: 0.5, devotion: 1.0 }
      },
      noon: {
        target: [22, -18],
        spread: 4.0,
        task: '🌾 战略粮仓清点分配十一奉献',
        state: 'WORKING',
        duration: 8.0,
        dialogue: '“库房大麦与清油充足，利未人当按班次分给孤儿与寡妇。”',
        effect: { treasury: 2, morale: 0.5 }
      },
      dusk: {
        target: [10, -8],
        spread: 2.5,
        task: '🌿 圣苑灌木果林采摘初熟圣果',
        state: 'WORKING',
        duration: 8.0,
        dialogue: '“初熟的石榴与无花果，当先献于圣所，感谢上天赐福美地。”',
        effect: { aroma: 0.4, devotion: 0.8 }
      },
      night: {
        target: [0, -22],
        spread: 2.0,
        task: '🕯️ 圣所内殿守夜诵读经卷',
        state: 'RESTING',
        duration: 12.0,
        dialogue: '“夜间在圣所举手赞美，金灯台长明不灭。”',
        effect: { devotion: 1.5, morale: 0.3 }
      }
    },
    '推罗工匠': {
      dawn: {
        target: [-22, 4],
        spread: 3.5,
        task: '⚒️ 青铜冶炼坊添炭开炉',
        state: 'WORKING',
        duration: 8.0,
        dialogue: '“推罗大师户兰传授的泥型浇铸术，今日当熔铸铜海的莲花纹边缘。”',
        effect: { prosperity: 0.02 }
      },
      noon: {
        target: [-20, 2],
        spread: 2.5,
        task: '🔨 锻造精打圣所青铜器',
        state: 'WORKING',
        duration: 9.0,
        dialogue: '“火候已足！锤声铿锵，让每件青铜圣器都能经受数百年风霜。”',
        effect: { prosperity: 0.04, turnover: 1.5 }
      },
      dusk: {
        target: [0, 6],
        spread: 4.0,
        task: '💰 中央集市交付铜器结算',
        state: 'TRADING',
        duration: 7.0,
        dialogue: '“这批洗礼铜盆与火铲完工，共计换取六十舍客勒银币。”',
        effect: { turnover: 3.0, shekels: 3 }
      },
      night: {
        target: [22, -4],
        spread: 3.0,
        task: '🏡 四室民居二楼宴饮就寝',
        state: 'SLEEPING',
        duration: 12.0,
        dialogue: '“劳碌一天，在泥砖宅邸中饮一杯新酒，安然入眠。”',
        effect: { morale: 0.5 }
      }
    },
    '橄榄农夫': {
      dawn: {
        target: [10, -8],
        spread: 3.0,
        task: '🌿 圣苑灌木果林除草剪枝',
        state: 'WORKING',
        duration: 8.0,
        dialogue: '“晨露滋润枝条，今年迦南雨水丰沛，橄榄定能丰收。”',
        effect: { aroma: 0.3, grain: 0.5 }
      },
      noon: {
        target: [-22, 18],
        spread: 3.0,
        task: '🫒 推动石轮冷榨特级纯油',
        state: 'WORKING',
        duration: 9.0,
        dialogue: '“重石压榨下流出的是头道纯油，色泽如琥珀，点灯无烟。”',
        effect: { aroma: 0.5, water: -0.2 }
      },
      dusk: {
        target: [22, 10],
        spread: 3.5,
        task: '💧 基训泉蓄水池汲水洗瓮',
        state: 'WORKING',
        duration: 7.0,
        dialogue: '“基训暗渠水质甘冽，把陶瓮洗净备好，明日装运新油。”',
        effect: { water: -0.3, morale: 0.4 }
      },
      night: {
        target: [22, -4],
        spread: 4.0,
        task: '🏡 庭院收纳农具闭户安睡',
        state: 'SLEEPING',
        duration: 12.0,
        dialogue: '“牛羊入圈，大麦归仓，全家安歇。”',
        effect: { morale: 0.6 }
      }
    },
    '希伯来学者': {
      dawn: {
        target: [0, -18],
        spread: 3.0,
        task: '📜 圣殿台阶晨间默想妥拉',
        state: 'PRAYING',
        duration: 8.0,
        dialogue: '“智慧从敬虔而来。文字与律法是我们跨越千年的传家至宝。”',
        effect: { devotion: 1.0, morale: 0.5 }
      },
      noon: {
        target: [-22, -18],
        spread: 3.5,
        task: '👑 林宫王廷起草公文律令',
        state: 'WORKING',
        duration: 9.0,
        dialogue: '“记录所罗门王裁决示巴使臣进贡香料与黄金的详册。”',
        effect: { prosperity: 0.03, treasury: 2 }
      },
      dusk: {
        target: [0, 32],
        spread: 4.0,
        task: '⚖️ 城门口见证长老契约仲裁',
        state: 'WORKING',
        duration: 7.0,
        dialogue: '“在此城门见证田契交割与立约，秉公行义，不可偏护有钱人。”',
        effect: { morale: 0.8 }
      },
      night: {
        target: [22, -4],
        spread: 2.5,
        task: '🕯️ 书斋研墨秉烛抄录羊皮卷',
        state: 'WORKING',
        duration: 12.0,
        dialogue: '“蜡烛未灭，必须保证羊皮纸上一字不差，代代相传。”',
        effect: { devotion: 0.8, morale: 0.4 }
      }
    },
    '远洋商贾': {
      dawn: {
        target: [0, 32],
        spread: 4.0,
        task: '🐫 六室城门卫所检验通关',
        state: 'COMMUTING',
        duration: 7.0,
        dialogue: '“自推罗港历经七日跋涉，携紫袍与乳香来到耶路撒冷圣城。”',
        effect: { turnover: 2.0 }
      },
      noon: {
        target: [0, 6],
        spread: 3.5,
        task: '🏺 中央彩篷大集摆摊称银',
        state: 'TRADING',
        duration: 9.0,
        dialogue: '“来自推罗的高级紫布，请用天平称出纯正舍客勒银币！”',
        effect: { turnover: 4.5, prosperity: 0.05, shekels: 5 }
      },
      dusk: {
        target: [22, -18],
        spread: 3.0,
        task: '🌾 战略粮仓批量采购大麦',
        state: 'TRADING',
        duration: 7.0,
        dialogue: '“订购四十驮袋优质耶斯列谷麦，装船运往腓尼基沿海。”',
        effect: { treasury: 5, turnover: 2.5 }
      },
      night: {
        target: [-22, -18],
        spread: 3.0,
        task: '🍷 王家商馆驿站喂驼歇宿',
        state: 'SLEEPING',
        duration: 12.0,
        dialogue: '“骆驼饮饱清泉，核算今日流水，明日启程回程。”',
        effect: { morale: 0.4 }
      }
    },
    '城邦平民': {
      dawn: {
        target: [22, 10],
        spread: 3.5,
        task: '💧 基训泉排队挑水备晨炊',
        state: 'WORKING',
        duration: 7.0,
        dialogue: '“清晨基训泉水最清凉，挑两瓮回家揉面烤无酵饼。”',
        effect: { water: -0.2, morale: 0.4 }
      },
      noon: {
        target: [0, 6],
        spread: 4.0,
        task: '🧺 中央大集采买时令干果',
        state: 'TRADING',
        duration: 8.0,
        dialogue: '“买一串无花果饼与半袋大麦，今日集市真热闹！”',
        effect: { turnover: 1.5, morale: 0.5 }
      },
      dusk: {
        target: [22, -4],
        spread: 3.5,
        task: '🏡 平顶露台晾晒大麦闲谈',
        state: 'SOCIALIZING',
        duration: 7.0,
        dialogue: '“夕阳西下，邻里在平顶露台纳凉，晚风吹过令人舒畅。”',
        effect: { morale: 0.8 }
      },
      night: {
        target: [22, -4],
        spread: 2.5,
        task: '💤 宅内阖门熄灯就寝安睡',
        state: 'SLEEPING',
        duration: 12.0,
        dialogue: '“一家老小同榻而眠，守夜人的更声令人安心。”',
        effect: { morale: 0.7 }
      }
    }
  };

  function spawnAncientCitizens() {
    const roles = [
      { name: '利未祭司', color: 0xfef08a },
      { name: '推罗工匠', color: 0x38bdf8 },
      { name: '橄榄农夫', color: 0x22c55e },
      { name: '希伯来学者', color: 0xf97316 },
      { name: '城邦平民', color: 0xfed7aa },
      { name: '远洋商贾', color: 0xa855f7 },
    ];

    const count = state.templeCity.citizenCount;
    const curPhase = state.timeOfDay || 'dusk';

    for (let i = 0; i < count; i++) {
      const r = roles[i % roles.length];
      const citizenGroup = new THREE.Group();

      const robe = new THREE.Mesh(new THREE.CylinderGeometry(0.35, 0.45, 1.4), new THREE.MeshStandardMaterial({ color: r.color }));
      robe.position.y = 0.7;
      robe.castShadow = true;
      citizenGroup.add(robe);

      const head = new THREE.Mesh(new THREE.SphereGeometry(0.28, 12, 12), new THREE.MeshStandardMaterial({ color: 0xfed7aa }));
      head.position.y = 1.6;
      citizenGroup.add(head);

      const sched = (ROUTINE_SCHEDULE[r.name] && ROUTINE_SCHEDULE[r.name][curPhase]) || {
        target: [0, 0],
        spread: 6.0,
        task: '日常作息',
        dialogue: '“平安归于这城。”',
        state: 'WORKING',
        duration: 8.0,
        effect: {}
      };

      const cx = sched.target[0] + (Math.random() - 0.5) * (sched.spread || 5.0);
      const cz = sched.target[1] + (Math.random() - 0.5) * (sched.spread || 5.0);
      citizenGroup.position.set(cx, 0, cz);
      simGroup.add(citizenGroup);

      const agentObj = {
        id: `城邦居民-${i + 1}`,
        type: 'ancient_citizen',
        role: r.name,
        color: r.color,
        mesh: citizenGroup,
        shekels: Math.round(15 + Math.random() * 25),
        devotion: 92,
        currentPhase: curPhase,
        subState: sched.state || 'WORKING',
        task: sched.task,
        dialogue: sched.dialogue,
        stateTimer: sched.duration * (0.6 + Math.random() * 0.8),
        stepCycle: Math.random() * Math.PI * 2,
        socialCooldown: 5.0 + Math.random() * 8.0,
        socialTimer: 0,
        target: new THREE.Vector3(cx, 0, cz),
      };

      citizenGroup.traverse((c) => {
        if (c.isMesh) c.userData.agent = agentObj;
      });

      agents.push(agentObj);
    }
  }

  function updateTempleCitySimulation(dt) {
    state.templeCity.sacredAroma = Math.max(30, state.templeCity.sacredAroma - 0.25 * dt);
    state.templeCity.waterSupply = Math.max(20, state.templeCity.waterSupply - 0.18 * dt);
    state.templeCity.grainStorage = Math.max(50, state.templeCity.grainStorage - 0.15 * dt);

    if (pressWheel) {
      pressWheel.rotation.z += 0.6 * dt;
    }

    smokeParticles.forEach((p) => {
      p.position.y += p.speedY * dt;
      p.position.x += Math.sin(p.position.y * 1.5) * 0.12 * dt;
      const progress = (p.position.y - p.startY) / p.maxDist;
      p.material.opacity = Math.max(0, 0.6 * (1.0 - progress));
      if (progress >= 1.0) {
        p.position.y = p.startY;
        p.position.x = p.startX + (Math.random() - 0.5) * 1.2;
        p.position.z = p.startZ + (Math.random() - 0.5) * 1.2;
      }
    });

    const timeNow = performance.now() * 0.001;

    agents.forEach((c, idx) => {
      if (c.type === 'caravan') {
        if (c.mesh) {
          c.mesh.position.addScaledVector(c.dir, c.speed * dt);
          if (c.mesh.position.z < -4) {
            c.mesh.position.z = 38;
          }
        }
        return;
      }

      if (c.type === 'ancient_knight') {
        if (c === controlledAgent) return;

        const dist = c.mesh.position.distanceTo(c.target);
        if (dist < 1.6) {
          if (typeof c.setAnimation === 'function') c.setAnimation(null, 0.35);
          c.stateTimer -= dt;
          c.task = '🛡️ 披甲屹立警惕岗哨';
          if (c.stateTimer <= 0) {
            c.currentPatrolIdx = (c.currentPatrolIdx + 1) % c.patrolPoints.length;
            const pt = c.patrolPoints[c.currentPatrolIdx];
            c.target.set(pt[0] + (Math.random() - 0.5) * 3.5, 0, pt[1] + (Math.random() - 0.5) * 3.5);
            c.stateTimer = 9.0 + Math.random() * 6.0;
          }
        } else {
          if (typeof c.setAnimation === 'function') c.setAnimation('walk', 0.25);
          c.task = '⚔️ 执剑踏步巡查要道';
          const dir = new THREE.Vector3().subVectors(c.target, c.mesh.position).normalize();
          c.mesh.position.addScaledVector(dir, 3.2 * dt);
          c.mesh.rotation.y = Math.atan2(dir.x, dir.z);
        }
        return;
      }

      if (c.type !== 'ancient_citizen') return;

      if (c === controlledAgent) {
        // Controlled directly by player WASD inputs
        return;
      }

      const sched = (ROUTINE_SCHEDULE[c.role] && ROUTINE_SCHEDULE[c.role][state.timeOfDay]) || null;

      // Phase sync
      if (c.currentPhase !== state.timeOfDay && sched) {
        c.currentPhase = state.timeOfDay;
        c.subState = 'COMMUTING';
        c.task = `🚶 赶往: ${sched.task.split(' ')[1] || sched.task}`;
        c.dialogue = sched.dialogue;
        const tx = sched.target[0] + (Math.random() - 0.5) * (sched.spread || 4.0);
        const tz = sched.target[1] + (Math.random() - 0.5) * (sched.spread || 4.0);
        c.target.set(tx, 0, tz);
      }

      // Social greetings when crossing paths
      if (c.socialCooldown > 0) {
        c.socialCooldown -= dt;
      } else if (c.subState === 'COMMUTING' || c.subState === 'WORKING') {
        for (let j = idx + 1; j < agents.length; j++) {
          const other = agents[j];
          if (other.type === 'ancient_citizen' && other.socialCooldown <= 0) {
            const distToOther = c.mesh.position.distanceTo(other.mesh.position);
            if (distToOther < 2.5) {
              c.subState = 'SOCIALIZING';
              other.subState = 'SOCIALIZING';
              c.socialTimer = 2.4;
              other.socialTimer = 2.4;
              c.socialCooldown = 14.0;
              other.socialCooldown = 14.0;
              c.task = `💬 与【${other.role}】互致平安 (Shalom)`;
              other.task = `💬 与【${c.role}】互致平安 (Shalom)`;
              state.templeCity.citizenMorale = Math.min(100, state.templeCity.citizenMorale + 0.2);
              break;
            }
          }
        }
      }

      // 1. STATE: SOCIALIZING
      if (c.subState === 'SOCIALIZING') {
        c.socialTimer -= dt;
        c.mesh.position.y = 0;
        c.mesh.rotation.y += Math.sin(timeNow * 5.0 + idx) * 0.02;
        if (c.socialTimer <= 0) {
          c.subState = 'COMMUTING';
          if (sched) c.task = sched.task;
        }
        return;
      }

      // 2. STATE: COMMUTING (Walking with natural bounce)
      if (c.subState === 'COMMUTING') {
        const dist = c.mesh.position.distanceTo(c.target);
        if (dist < 1.4) {
          if (sched) {
            c.subState = sched.state || 'WORKING';
            c.task = sched.task;
            c.dialogue = sched.dialogue;
            c.stateTimer = sched.duration * (0.8 + Math.random() * 0.4);

            // Arrival benefits to the city state
            if (sched.effect.aroma) state.templeCity.sacredAroma = Math.min(100, state.templeCity.sacredAroma + sched.effect.aroma);
            if (sched.effect.morale) state.templeCity.citizenMorale = Math.min(100, state.templeCity.citizenMorale + sched.effect.morale);
            if (sched.effect.turnover) state.templeCity.marketTurnover += sched.effect.turnover;
            if (sched.effect.treasury) state.templeCity.templeTreasury += sched.effect.treasury;
            if (sched.effect.prosperity) state.templeCity.tradeProsperity = Math.min(3.0, state.templeCity.tradeProsperity + sched.effect.prosperity);
            if (sched.effect.devotion) c.devotion = Math.min(100, c.devotion + sched.effect.devotion);
            if (sched.effect.shekels) c.shekels += sched.effect.shekels;
          } else {
            c.subState = 'WORKING';
            c.stateTimer = 5.0;
          }
          c.mesh.position.y = 0;
        } else {
          const dir = new THREE.Vector3().subVectors(c.target, c.mesh.position).normalize();
          c.mesh.position.addScaledVector(dir, 3.8 * dt);
          c.mesh.rotation.y = Math.atan2(dir.x, dir.z);

          // Step bounce animation while walking
          c.stepCycle = (c.stepCycle || 0) + dt * 9;
          c.mesh.position.y = Math.abs(Math.sin(c.stepCycle)) * 0.12;
        }
        return;
      }

      // 3. STATE: WORKING / PRAYING / TRADING
      if (c.subState === 'WORKING' || c.subState === 'TRADING' || c.subState === 'PRAYING') {
        c.stateTimer -= dt;

        if (c.subState === 'PRAYING') {
          // Reverent bowing motion
          c.mesh.rotation.x = Math.sin(timeNow * 2.2 + idx) * 0.16;
          c.mesh.position.y = -Math.abs(Math.sin(timeNow * 2.2 + idx)) * 0.08;
        } else if (c.subState === 'WORKING') {
          // Rhythmic labor animation
          c.mesh.position.y = Math.sin(timeNow * 4.0 + idx * 2) * 0.05;
          c.mesh.rotation.y += Math.sin(timeNow * 1.5 + idx) * 0.01;
        } else if (c.subState === 'TRADING') {
          // Bargaining / talking motion
          c.mesh.rotation.y += Math.sin(timeNow * 3.0 + idx) * 0.03;
          c.mesh.position.y = Math.abs(Math.sin(timeNow * 3.0 + idx)) * 0.03;
        }

        if (c.stateTimer <= 0 && sched) {
          // Wander slightly around current workplace/station
          c.target.set(
            sched.target[0] + (Math.random() - 0.5) * (sched.spread || 4.0),
            0,
            sched.target[1] + (Math.random() - 0.5) * (sched.spread || 4.0)
          );
          c.subState = 'COMMUTING';
        }
        return;
      }

      // 4. STATE: SLEEPING / NIGHT REST
      if (c.subState === 'SLEEPING' || c.subState === 'RESTING') {
        c.mesh.position.y = -0.32; // Resting pose
        c.mesh.rotation.x = 0.22;  // Slight recline
        c.mesh.rotation.z = Math.sin(timeNow * 0.7 + idx) * 0.02; // Gentle breathing
      }
    });
  }

  function updateSimulation(dt) {
    updateTempleCitySimulation(dt);
  }

  // ------------------------------------------------------------------------
  // Raycaster & Selection Highlights & Spawn Tool
  // ------------------------------------------------------------------------
  function highlightBuilding(building) {
    if (!selectionIndicator) {
      const geo = new THREE.RingGeometry(0.85, 1.0, 32);
      const mat = new THREE.MeshBasicMaterial({ color: 0xfbbf24, side: THREE.DoubleSide });
      selectionIndicator = new THREE.Mesh(geo, mat);
      selectionIndicator.rotation.x = -Math.PI / 2;
      scene.add(selectionIndicator);
    }
    selectionIndicator.visible = true;
    if (building.mesh) {
      selectionIndicator.position.set(building.mesh.position.x, 0.08, building.mesh.position.z);
      const scale = building.radius || 9.0;
      selectionIndicator.scale.set(scale, scale, scale);
    }
  }

  function clearBuildingHighlight() {
    if (selectionIndicator) selectionIndicator.visible = false;
  }

  function setupRaycaster() {
    const raycaster = new THREE.Raycaster();
    const mouse = new THREE.Vector2();

    renderer.domElement.addEventListener('mousemove', (event) => {
      if (controlledAgent) return;
      const rect = renderer.domElement.getBoundingClientRect();
      mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
      mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;

      raycaster.setFromCamera(mouse, camera);
      const intersects = raycaster.intersectObjects(simGroup.children, true);

      let isInteractive = state.spawnToolActive;
      for (let i = 0; i < intersects.length; i++) {
        let obj = intersects[i].object;
        while (obj && obj !== simGroup) {
          if (obj.userData && (obj.userData.building || obj.userData.agent)) {
            isInteractive = true;
            break;
          }
          obj = obj.parent;
        }
        if (isInteractive) break;
      }
      renderer.domElement.style.cursor = isInteractive ? 'pointer' : 'default';
    });

    renderer.domElement.addEventListener('click', (event) => {
      if (controlledAgent) return;
      const rect = renderer.domElement.getBoundingClientRect();
      mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
      mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;

      raycaster.setFromCamera(mouse, camera);

      // Handle Spawn Tool: click on ground spawns citizen
      if (state.spawnToolActive && groundMesh) {
        const groundHits = raycaster.intersectObject(groundMesh);
        if (groundHits.length > 0) {
          const pt = groundHits[0].point;
          spawnSingleCitizenAt(pt.x, pt.z);
          state.spawnToolActive = false;
          showFloatToast(`👥 新城民已在坐标 (${pt.x.toFixed(0)}, ${pt.z.toFixed(0)}) 安置入城！`);
          triggerCitySound('coins');
          return;
        }
      }

      const intersects = raycaster.intersectObjects(simGroup.children, true);

      if (intersects.length > 0) {
        let hitBuilding = null;
        let hitAgent = null;

        for (let i = 0; i < intersects.length; i++) {
          let obj = intersects[i].object;
          while (obj && obj !== simGroup) {
            if (obj.userData && obj.userData.building) {
              hitBuilding = obj.userData.building;
              break;
            }
            if (obj.userData && obj.userData.agent) {
              hitAgent = obj.userData.agent;
              break;
            }
            obj = obj.parent;
          }
          if (hitBuilding || hitAgent) break;
        }

        if (hitBuilding) {
          state.selectedBuilding = hitBuilding;
          state.selectedAgent = null;
          followAgent = null;
          highlightBuilding(hitBuilding);
          updateInspectorHUD();
          return;
        }

        if (hitAgent) {
          state.selectedAgent = hitAgent;
          state.selectedBuilding = null;
          clearBuildingHighlight();
          updateInspectorHUD();
          return;
        }

        let hit = intersects[0].object;
        let found = agents.find((a) => a.mesh === hit || (a.mesh.children && a.mesh.children.includes(hit)));
        if (!found) found = graphNodes.find((n) => n.mesh === hit);

        if (found) {
          state.selectedAgent = found;
          state.selectedBuilding = null;
          clearBuildingHighlight();
          updateInspectorHUD();
          return;
        }
      }

      state.selectedBuilding = null;
      state.selectedAgent = null;
      followAgent = null;
      clearBuildingHighlight();
      updateInspectorHUD();
    });
  }

  function spawnSingleCitizenAt(x, z) {
    const roles = ['利未祭司', '推罗工匠', '橄榄农夫', '希伯来学者', '城邦平民', '远洋商贾'];
    const colors = [0xfef08a, 0x38bdf8, 0x22c55e, 0xf97316, 0xfed7aa, 0xa855f7];
    const roleIdx = agents.length % roles.length;
    const rRole = roles[roleIdx];
    const curPhase = state.timeOfDay || 'dusk';

    const citizenGroup = new THREE.Group();
    const robe = new THREE.Mesh(new THREE.CylinderGeometry(0.35, 0.45, 1.4), new THREE.MeshStandardMaterial({ color: colors[roleIdx] }));
    robe.position.y = 0.7;
    robe.castShadow = true;
    citizenGroup.add(robe);

    const head = new THREE.Mesh(new THREE.SphereGeometry(0.28, 12, 12), new THREE.MeshStandardMaterial({ color: 0xfed7aa }));
    head.position.y = 1.6;
    citizenGroup.add(head);

    citizenGroup.position.set(x, 0, z);
    simGroup.add(citizenGroup);

    const sched = (ROUTINE_SCHEDULE[rRole] && ROUTINE_SCHEDULE[rRole][curPhase]) || null;

    const agentObj = {
      id: `招募城民-${agents.length + 1}`,
      type: 'ancient_citizen',
      role: rRole,
      color: colors[roleIdx],
      mesh: citizenGroup,
      shekels: 20,
      devotion: 95,
      currentPhase: curPhase,
      subState: sched ? 'COMMUTING' : 'WORKING',
      task: sched ? sched.task : '新入城邦定居',
      dialogue: sched ? sched.dialogue : '“愿平安临到这城。”',
      stateTimer: sched ? sched.duration : 6.0,
      stepCycle: Math.random() * Math.PI * 2,
      socialCooldown: 6.0,
      socialTimer: 0,
      target: sched ? new THREE.Vector3(sched.target[0], 0, sched.target[1]) : new THREE.Vector3(x, 0, z),
    };

    citizenGroup.traverse((c) => {
      if (c.isMesh) c.userData.agent = agentObj;
    });

    agents.push(agentObj);
    state.templeCity.citizenCount++;
    state.metrics.agentCount++;
    updateStatsHUD();
  }

  // ------------------------------------------------------------------------
  // Interactive Landmark Fast-Travel & Tour
  // ------------------------------------------------------------------------
  window.focusLandmark = function (id) {
    const b = interactiveBuildings.find((item) => item.id === id);
    if (!b || !b.mesh) return;

    followAgent = null;
    targetLookAt.set(b.mesh.position.x, 2.0, b.mesh.position.z);
    isTransitioningCamera = true;
    state.selectedBuilding = b;
    state.selectedAgent = null;
    highlightBuilding(b);
    updateInspectorHUD();
    triggerCitySound('chime');
    showFloatToast(`📍 镜头已聚焦: ${b.name}`);
  };

  // ------------------------------------------------------------------------
  // Follow Camera Toggle
  // ------------------------------------------------------------------------
  window.toggleAgentFollow = function () {
    if (followAgent) {
      followAgent = null;
      showFloatToast('⏹️ 已停止视角锁定跟随');
    } else if (state.selectedAgent && state.selectedAgent.mesh) {
      followAgent = state.selectedAgent;
      showFloatToast(`🎥 正在锁定追踪个体: ${followAgent.id}`);
      triggerCitySound('horn');
    }
    updateInspectorHUD();
  };

  // ------------------------------------------------------------------------
  // Third-Person Character Possession & WASD Control System
  // ------------------------------------------------------------------------
  let keyControlsInitialized = false;

  function setupCharacterKeyControls() {
    if (keyControlsInitialized) return;
    keyControlsInitialized = true;

    window.addEventListener('keydown', (e) => {
      if (e.target && (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA')) return;

      if (e.code === 'KeyW' || e.code === 'ArrowUp') {
        inputKeys.forward = true;
        if (controlledAgent) e.preventDefault();
      } else if (e.code === 'KeyS' || e.code === 'ArrowDown') {
        inputKeys.backward = true;
        if (controlledAgent) e.preventDefault();
      } else if (e.code === 'KeyA' || e.code === 'ArrowLeft') {
        inputKeys.left = true;
        if (controlledAgent) e.preventDefault();
      } else if (e.code === 'KeyD' || e.code === 'ArrowRight') {
        inputKeys.right = true;
        if (controlledAgent) e.preventDefault();
      } else if (e.code === 'ShiftLeft' || e.code === 'ShiftRight') {
        inputKeys.sprint = true;
      } else if (e.code === 'Space') {
        if (controlledAgent) {
          e.preventDefault();
          triggerControlledAction();
        }
      } else if (e.code === 'Escape') {
        if (controlledAgent) {
          e.preventDefault();
          window.exitControlMode();
        }
      }
    });

    window.addEventListener('keyup', (e) => {
      if (e.code === 'KeyW' || e.code === 'ArrowUp') inputKeys.forward = false;
      else if (e.code === 'KeyS' || e.code === 'ArrowDown') inputKeys.backward = false;
      else if (e.code === 'KeyA' || e.code === 'ArrowLeft') inputKeys.left = false;
      else if (e.code === 'KeyD' || e.code === 'ArrowRight') inputKeys.right = false;
      else if (e.code === 'ShiftLeft' || e.code === 'ShiftRight') inputKeys.sprint = false;
    });

    if (renderer && renderer.domElement) {
      renderer.domElement.addEventListener('mousedown', (e) => {
        if (controlledAgent && e.button === 0) {
          isMouseDownOnControl = true;
          prevMouseX = e.clientX;
        }
      });
    }

    window.addEventListener('mouseup', () => {
      isMouseDownOnControl = false;
    });

    window.addEventListener('mousemove', (e) => {
      if (controlledAgent && isMouseDownOnControl && controlledAgent.mesh) {
        const dx = e.clientX - prevMouseX;
        prevMouseX = e.clientX;
        controlledAgent.mesh.rotation.y -= dx * 0.0055;
      }
    });
  }

  function updateControlledAgent(delta) {
    if (!controlledAgent || !controlledAgent.mesh) return;

    const mesh = controlledAgent.mesh;
    const isKnight = controlledAgent.type === 'ancient_knight';
    const turnSpeed = 3.2;

    if (inputKeys.left) {
      mesh.rotation.y += turnSpeed * delta;
    }
    if (inputKeys.right) {
      mesh.rotation.y -= turnSpeed * delta;
    }

    const moveSpeed = inputKeys.sprint ? (isKnight ? 9.6 : 9.2) : (isKnight ? 4.8 : 5.0);
    let forwardDist = 0;
    if (inputKeys.forward) forwardDist += moveSpeed * delta;
    if (inputKeys.backward) forwardDist -= (moveSpeed * 0.55) * delta;

    if (forwardDist !== 0) {
      if (isKnight && typeof controlledAgent.setAnimation === 'function') {
        controlledAgent.setAnimation(inputKeys.sprint ? 'run' : 'walk', 0.2);
      }

      const rotY = mesh.rotation.y;
      const dx = Math.sin(rotY) * forwardDist;
      const dz = Math.cos(rotY) * forwardDist;

      let nextX = mesh.position.x + dx;
      let nextZ = mesh.position.z + dz;

      // City boundaries:
      // Inside city walls: x in [-31.5, 31.5], z in [-31.5, 31.5]
      // Six-Chamber Gate at south allows exiting to caravan valley (z up to 46 for |x| < 4.5)
      const inGateCorridor = Math.abs(nextX) < 4.5 && nextZ > 28 && nextZ < 46;
      if (!inGateCorridor) {
        nextX = Math.max(-31.5, Math.min(31.5, nextX));
        nextZ = Math.max(-31.5, Math.min(31.5, nextZ));
      } else {
        nextZ = Math.min(46, nextZ);
      }

      mesh.position.x = nextX;
      mesh.position.z = nextZ;
      controlledAgent.target.set(nextX, 0, nextZ);

      if (!isKnight) {
        // Walking bounce & rock animation for procedural citizens
        controlledAgent.walkCycle = (controlledAgent.walkCycle || 0) + delta * (inputKeys.sprint ? 14 : 9);
        mesh.position.y = 0.5 + Math.abs(Math.sin(controlledAgent.walkCycle)) * 0.16;
        mesh.rotation.z = Math.sin(controlledAgent.walkCycle) * 0.05;
      } else {
        mesh.position.y = 0;
        mesh.rotation.z = 0;
      }
    } else {
      if (isKnight && typeof controlledAgent.setAnimation === 'function') {
        controlledAgent.setAnimation(null, 0.35); // idle standing pose
        mesh.position.y = 0;
        mesh.rotation.z = 0;
      } else {
        mesh.position.y = 0.5;
        mesh.rotation.z = 0;
      }
    }

    // Update Proximity & Action HUD
    updateControlProximityPrompt();
  }

  function updateControlProximityPrompt() {
    if (!controlledAgent || !controlledAgent.mesh) return;

    const charPos = controlledAgent.mesh.position;
    let closestItem = null;
    let minDistance = 5.5;

    // Check buildings
    for (const b of interactiveBuildings) {
      if (!b.mesh) continue;
      const d = charPos.distanceTo(b.mesh.position);
      if (d < (b.radius || 9.0) * 0.75 && d < minDistance) {
        minDistance = d;
        closestItem = { type: 'building', data: b };
      }
    }

    // Check sacred bushes
    for (const bush of bushInstances) {
      const d = charPos.distanceTo(bush.position);
      if (d < 4.5 && d < minDistance) {
        minDistance = d;
        closestItem = {
          type: 'bush',
          data: { name: '🌿 迦南古橄榄圣木', prompt: '触摸圣木领受祝福与芳香' }
        };
      }
    }

    // Check other citizens
    for (const a of agents) {
      if (a === controlledAgent || !a.mesh) continue;
      const d = charPos.distanceTo(a.mesh.position);
      if (d < 3.2 && d < minDistance) {
        minDistance = d;
        closestItem = {
          type: 'citizen',
          data: a
        };
      }
    }

    activePrompt = closestItem;
    renderControlPromptHUD(closestItem);
  }

  function triggerControlledAction() {
    if (!activePrompt) {
      showFloatToast('✨ 你在此城邦举目观望，赞美这流奶与蜜之地！');
      triggerCitySound('chime');
      return;
    }

    if (activePrompt.type === 'bush') {
      triggerCitySound('chime');
      showFloatToast('🌿 你轻抚了圣木郁葱的枝叶，芳香四溢！心智敬虔度提升，城邦香膏 +10');
      state.templeCity.sacredAroma = Math.min(100, state.templeCity.sacredAroma + 10);
      if (controlledAgent) controlledAgent.devotion = Math.min(100, controlledAgent.devotion + 5);
      return;
    }

    if (activePrompt.type === 'citizen') {
      const other = activePrompt.data;
      triggerCitySound('chime');
      showFloatToast(`💬 你向【${other.role}】问安：“愿平安归于你！” 他回道：“${other.dialogue || '愿行公义者得享繁盛。'}”`);
      state.templeCity.citizenMorale = Math.min(100, state.templeCity.citizenMorale + 1.5);
      return;
    }

    if (activePrompt.type === 'building') {
      const b = activePrompt.data;
      if (typeof window.triggerBuildingAction === 'function') {
        window.triggerBuildingAction(b.id);
      } else {
        showFloatToast(`🏛️ 你在【${b.name}】前行礼参拜！`);
      }
    }
  }

  function showControlHUD(agent) {
    let hud = document.getElementById('sb-control-hud');
    if (!hud) {
      hud = document.createElement('div');
      hud.id = 'sb-control-hud';
      hud.style.cssText = `
        position: absolute;
        bottom: 16px;
        left: 50%;
        transform: translateX(-50%);
        z-index: 90;
        display: flex;
        align-items: center;
        gap: 14px;
        background: rgba(15, 23, 42, 0.94);
        backdrop-filter: blur(12px);
        border: 1px solid #38bdf8;
        border-radius: 12px;
        padding: 8px 16px;
        box-shadow: 0 12px 30px rgba(0,0,0,0.7);
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        pointer-events: auto;
      `;
      const target = canvasMount || container;
      if (target) target.appendChild(hud);
    }

    hud.innerHTML = `
      <div style="display: flex; align-items: center; gap: 8px;">
        <span style="background: linear-gradient(135deg, #1e3a8a, #0369a1); color: #bae6fd; font-weight: 800; font-size: 12px; padding: 4px 10px; border-radius: 6px; border: 1px solid #38bdf8;">
          👤 ${agent.role || '城邦公民'}
        </span>
        <span style="color: #cbd5e1; font-size: 11.5px; display: flex; align-items: center; gap: 8px;">
          <span><b style="color: #fde047;">[WASD]</b> 移动</span>
          <span><b style="color: #fde047;">[Shift]</b> 疾跑</span>
          <span><b style="color: #38bdf8;">[Space]</b> 互动</span>
          <span style="color: #94a3b8; font-size: 10.5px;">(按住左键拖拽平滑转向)</span>
        </span>
      </div>
      <button onclick="window.exitControlMode()" style="background: #dc2626; color: #fff; border: none; padding: 4px 10px; border-radius: 6px; font-weight: bold; font-size: 11px; cursor: pointer; transition: all 0.2s; white-space: nowrap; box-shadow: 0 2px 6px rgba(220, 38, 38, 0.4);">
        ✕ 退出 (ESC)
      </button>
    `;
    hud.style.display = 'flex';

    let promptEl = document.getElementById('sb-control-prompt');
    if (!promptEl) {
      promptEl = document.createElement('div');
      promptEl.id = 'sb-control-prompt';
      promptEl.style.cssText = `
        position: absolute;
        bottom: 68px;
        left: 50%;
        transform: translateX(-50%);
        z-index: 90;
        display: none;
        align-items: center;
        gap: 6px;
        background: rgba(6, 78, 59, 0.95);
        backdrop-filter: blur(8px);
        border: 1px solid #34d399;
        color: #ecfdf5;
        font-weight: bold;
        font-size: 12px;
        padding: 5px 14px;
        border-radius: 20px;
        box-shadow: 0 4px 14px rgba(16, 185, 129, 0.4);
        pointer-events: none;
      `;
      const target = canvasMount || container;
      if (target) target.appendChild(promptEl);
    }
  }

  function hideControlHUD() {
    const hud = document.getElementById('sb-control-hud');
    if (hud) hud.style.display = 'none';
    const promptEl = document.getElementById('sb-control-prompt');
    if (promptEl) promptEl.style.display = 'none';
  }

  function renderControlPromptHUD(item) {
    const promptEl = document.getElementById('sb-control-prompt');
    if (!promptEl) return;
    if (!item) {
      promptEl.style.display = 'none';
      return;
    }

    let text = '';
    if (item.type === 'bush') {
      text = `🌿 [按 Space 空格] 采摘圣木枝条，领受大地祝福与芳香`;
    } else if (item.type === 'citizen') {
      text = `💬 [按 Space 空格] 与【${item.data.role}】互致平安 (Shalom)`;
    } else if (item.type === 'building') {
      text = `🏛️ [按 Space 空格] 拜访【${item.data.name}】进行城邦交互`;
    }

    promptEl.innerHTML = text;
    promptEl.style.display = 'flex';
  }

  window.takeControlOfAgent = function (agent) {
    if (!agent) {
      agent = agents.find((a) => a.type === 'ancient_citizen') || agents[0];
    }
    if (!agent || !agent.mesh) {
      showFloatToast('⚠️ 暂无可操控的个体');
      return;
    }

    followAgent = null;
    isTransitioningCamera = false;
    controlledAgent = agent;

    inputKeys.forward = false;
    inputKeys.backward = false;
    inputKeys.left = false;
    inputKeys.right = false;
    inputKeys.sprint = false;

    const charPos = agent.mesh.position;
    // If agent is close to outer walls, orient them towards city center
    if (Math.abs(charPos.x) > 16 || Math.abs(charPos.z) > 16) {
      agent.mesh.rotation.y = Math.atan2(-charPos.x, -charPos.z);
    }
    const rotY = agent.mesh.rotation.y;
    const camDist = 7.2;
    const camHeight = 4.6;
    perspCamera.position.set(
      charPos.x - Math.sin(rotY) * camDist,
      charPos.y + camHeight,
      charPos.z - Math.cos(rotY) * camDist
    );
    perspCamera.lookAt(
      charPos.x + Math.sin(rotY) * 3.5,
      charPos.y + 1.35,
      charPos.z + Math.cos(rotY) * 3.5
    );

    camera = perspCamera;
    if (controls) controls.enabled = false;

    const card = document.getElementById('sb-inspector');
    if (card) card.style.display = 'none';

    showControlHUD(agent);
    triggerCitySound('horn');
    if (agent.type === 'ancient_knight') {
      showFloatToast(`⚔️ 已附身操控：【${agent.role}】！按 WASD 铁甲踏步巡游 (按住 Shift 疾跑冲锋)`);
    } else {
      showFloatToast(`🎮 已附身操控：【${agent.role || '城邦公民'}】！按 WASD 穿梭漫游古代城邦`);
    }
  };

  window.takeControlOfKnight = function () {
    let knight = agents.find((a) => a.type === 'ancient_knight');
    if (knight) {
      window.takeControlOfAgent(knight);
      return;
    }
    showFloatToast('⚔️ 圣殿骑士正在披挂全身板甲入城，请稍候...');
    loadKnightAsset(() => {
      knight = agents.find((a) => a.type === 'ancient_knight');
      if (!knight) {
        knight = createKnightAgent(0, 8, '圣殿重装大骑士', true);
      }
      if (knight) {
        window.takeControlOfAgent(knight);
      }
    });
  };

  window.controlRandomCitizen = function () {
    const citizens = agents.filter((a) => a.type === 'ancient_citizen');
    // Prefer citizens stationed in or near the central districts
    citizens.sort((a, b) => a.mesh.position.length() - b.mesh.position.length());
    const poolSize = Math.max(1, Math.min(citizens.length, 15));
    const pick = citizens.length > 0 ? citizens[Math.floor(Math.random() * poolSize)] : agents[0];
    window.takeControlOfAgent(pick);
  };

  window.exitControlMode = function () {
    if (!controlledAgent) return;
    controlledAgent = null;
    activePrompt = null;

    camera = orthoCamera;
    if (controls) {
      controls.enabled = true;
      controls.object = orthoCamera;
    }

    hideControlHUD();
    showFloatToast('👁️ 已退回上帝俯瞰全景视角');
  };

  // ------------------------------------------------------------------------
  // Spawn Tool Toggle
  // ------------------------------------------------------------------------
  window.toggleSpawnTool = function () {
    state.spawnToolActive = !state.spawnToolActive;
    if (state.spawnToolActive) {
      showFloatToast('👥 招募模式已激活：点击场景中任意空地即可安置新城民');
    } else {
      showFloatToast('已退出招募模式');
    }
  };

  // ------------------------------------------------------------------------
  // Historical Events Trigger
  // ------------------------------------------------------------------------
  window.triggerHistoricalEvent = function (eventType) {
    if (eventType === 'caravan_arrival') {
      const cargo = '推罗特级紫布与没药';
      for (let i = 0; i < 3; i++) {
        const camel = buildCamelMesh(cargo);
        camel.position.set(2.0, 0, 38 + i * 4.5);
        camel.rotation.y = Math.PI;
        simGroup.add(camel);
        const camelObj = {
          id: `岁贡驼队-${agents.length + 1}`,
          type: 'caravan',
          cargo,
          mesh: camel,
          speed: 0.45,
          maxSpeed: 0.6,
          lane: 2,
          dir: new THREE.Vector3(0, 0, -1),
        };
        camel.traverse((c) => {
          if (c.isMesh) c.userData.agent = camelObj;
        });
        agents.push(camelObj);
      }
      state.templeCity.marketTurnover += 85;
      triggerCitySound('horn');
      triggerCitySound('coins');
      showFloatToast('🐫 远方香料商队穿越六室城门入城！集市流水大增');
    } else if (eventType === 'divine_rain') {
      triggerDivineRain();
      state.templeCity.waterSupply = 100.0;
      state.templeCity.grainStorage += 40;
      triggerCitySound('water');
      showFloatToast('🌧️ 旷野天降甘霖！蓄水池已满，田垄润泽');
    } else if (eventType === 'jubilee_feast') {
      state.templeCity.sacredAroma = 100.0;
      state.templeCity.citizenMorale = 100.0;
      agents.forEach((a) => {
        if (a.type === 'ancient_citizen') {
          a.pauseTimer = 6.0;
          a.task = '欢度禧年安息圣节';
        }
      });
      triggerCitySound('chime');
      triggerCitySound('horn');
      showFloatToast('🕊️ 盛大禧年节期降临！全城欢庆赞美');
    } else if (eventType === 'year_of_abundance') {
      state.templeCity.grainStorage += 150;
      state.templeCity.tradeProsperity = Math.min(3.0, state.templeCity.tradeProsperity + 0.4);
      triggerCitySound('coins');
      showFloatToast('🌾 丰收之年！谷物满仓，城邦繁荣倍增');
    } else if (eventType === 'bush_bloom') {
      state.templeCity.sacredAroma = Math.min(100, state.templeCity.sacredAroma + 18);
      state.templeCity.citizenMorale = Math.min(100, state.templeCity.citizenMorale + 15);
      triggerCitySound('chime');
      showFloatToast('🌿 迦南圣木与果林生机盎然！全城圣香与民心倍增');
    }

    updateStatsHUD();
    updateControlsUI();
    updateInspectorHUD();
  };

  function triggerDivineRain() {
    if (!rainGroup) {
      const rainCount = 500;
      const rainGeo = new THREE.BufferGeometry();
      const rainPositions = new Float32Array(rainCount * 3);
      for (let i = 0; i < rainCount; i++) {
        rainPositions[i * 3] = (Math.random() - 0.5) * 80;
        rainPositions[i * 3 + 1] = Math.random() * 40;
        rainPositions[i * 3 + 2] = (Math.random() - 0.5) * 80;
      }
      rainGeo.setAttribute('position', new THREE.BufferAttribute(rainPositions, 3));
      const rainMat = new THREE.PointsMaterial({ color: 0x7dd3fc, size: 0.35, transparent: true, opacity: 0.7 });
      rainGroup = new THREE.Points(rainGeo, rainMat);
    }
    rainActive = true;
    rainTimer = 8.0;
    scene.add(rainGroup);
  }

  // ------------------------------------------------------------------------
  // Minimap / Tactical Radar Renderer
  // ------------------------------------------------------------------------
  function drawMinimap() {
    const canvas = document.getElementById('sb-minimap');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    const w = canvas.width;
    const h = canvas.height;
    ctx.clearRect(0, 0, w, h);

    // Background circle
    ctx.fillStyle = 'rgba(15, 23, 42, 0.9)';
    ctx.beginPath();
    ctx.arc(w / 2, h / 2, w / 2 - 2, 0, Math.PI * 2);
    ctx.fill();
    ctx.strokeStyle = '#d97706';
    ctx.lineWidth = 1.5;
    ctx.stroke();

    // Fortress Walls footprint
    ctx.strokeStyle = 'rgba(214, 203, 186, 0.35)';
    ctx.lineWidth = 1.2;
    ctx.strokeRect(w * 0.16, h * 0.16, w * 0.68, h * 0.68);

    // Buildings footprints
    interactiveBuildings.forEach((b) => {
      if (!b.mesh) return;
      const mx = w / 2 + (b.mesh.position.x / 80) * (w * 0.68);
      const my = h / 2 + (b.mesh.position.z / 80) * (h * 0.68);
      if (b.id === 'bush_flora') {
        ctx.fillStyle = b === state.selectedBuilding ? '#4ade80' : '#22c55e';
      } else {
        ctx.fillStyle = b === state.selectedBuilding ? '#fde047' : '#64748b';
      }
      ctx.fillRect(mx - 2.5, my - 2.5, 5, 5);
    });

    // Agents
    agents.forEach((a) => {
      if (!a.mesh) return;
      const mx = w / 2 + (a.mesh.position.x / 80) * (w * 0.68);
      const my = h / 2 + (a.mesh.position.z / 80) * (h * 0.68);

      let col = '#fed7aa';
      let rad = 1.6;
      if (a.type === 'caravan') { col = '#f59e0b'; rad = 2.2; }
      else if (a.type === 'ancient_knight') { col = '#e2e8f0'; rad = 2.6; }
      else if (a.role === '利未祭司') col = '#fde047';
      else if (a.role === '推罗工匠') col = '#38bdf8';
      else if (a.role === '橄榄农夫') col = '#22c55e';
      else if (a.role === '远洋商贾') col = '#c084fc';

      ctx.fillStyle = col;
      ctx.beginPath();
      ctx.arc(mx, my, a === followAgent ? 3.4 : rad, 0, Math.PI * 2);
      ctx.fill();
    });

    // Controlled Agent Indicator
    if (controlledAgent && controlledAgent.mesh) {
      const mx = w / 2 + (controlledAgent.mesh.position.x / 80) * (w * 0.68);
      const my = h / 2 + (controlledAgent.mesh.position.z / 80) * (h * 0.68);
      ctx.fillStyle = '#10b981';
      ctx.beginPath();
      ctx.arc(mx, my, 4.5, 0, Math.PI * 2);
      ctx.fill();
      ctx.strokeStyle = '#ffffff';
      ctx.lineWidth = 1.6;
      ctx.stroke();

      // Heading pointer line
      const rotY = controlledAgent.mesh.rotation.y;
      ctx.strokeStyle = '#34d399';
      ctx.lineWidth = 2.4;
      ctx.beginPath();
      ctx.moveTo(mx, my);
      ctx.lineTo(mx + Math.sin(rotY) * 9, my + Math.cos(rotY) * 9);
      ctx.stroke();
    }

    // Camera view center box
    if (!controlledAgent && controls) {
      const cx = w / 2 + (controls.target.x / 80) * (w * 0.68);
      const cy = h / 2 + (controls.target.z / 80) * (h * 0.68);
      ctx.strokeStyle = '#38bdf8';
      ctx.lineWidth = 1.2;
      ctx.strokeRect(cx - 3, cy - 3, 6, 6);
    }
  }

  // ------------------------------------------------------------------------
  // Telemetry HUD & Inspector
  // ------------------------------------------------------------------------
  function updateStatsHUD() {
    const fpsEl = document.getElementById('sb-fps');
    const countEl = document.getElementById('sb-agents');
    if (fpsEl) fpsEl.textContent = `${state.metrics.fps} FPS`;
    if (countEl) countEl.textContent = `${state.metrics.agentCount}`;
  }

  window.closeInspector = function () {
    state.selectedBuilding = null;
    state.selectedAgent = null;
    followAgent = null;
    clearBuildingHighlight();
    const card = document.getElementById('sb-inspector');
    if (card) card.style.display = 'none';
  };

  function updateInspectorHUD() {
    const card = document.getElementById('sb-inspector');
    if (!card) return;

    if (!state.selectedBuilding && !state.selectedAgent) {
      card.style.display = 'none';
      return;
    }

    card.style.display = 'block';
    const title = document.getElementById('sb-insp-title');
    const details = document.getElementById('sb-insp-body');

    // Case 1: Building Inspector
    if (state.selectedBuilding) {
      const b = state.selectedBuilding;
      if (title) {
        title.innerHTML = `
          <div style="display: flex; justify-content: space-between; align-items: center; width: 100%;">
            <span style="color: #fde047; font-size: 14.5px; font-weight: 800;">${b.name}</span>
            <button onclick="window.closeInspector()" style="background: transparent; border: none; color: #94a3b8; font-size: 15px; cursor: pointer; padding: 0 4px; line-height: 1;">✕</button>
          </div>
        `;
      }
      if (details) {
        details.innerHTML = `
          <div style="margin-bottom: 8px;">
            <span style="background: #1e3a8a; color: #93c5fd; padding: 2px 8px; border-radius: 4px; font-size: 11px; font-weight: bold;">
              ${b.district}
            </span>
          </div>
          <div style="color: #cbd5e1; font-size: 12px; margin-bottom: 10px; line-height: 1.6; border-left: 3px solid #d97706; padding: 4px 8px; background: rgba(0,0,0,0.25); border-radius: 0 4px 4px 0;">
            ${b.lectureNote}
          </div>
          <div style="background: rgba(15, 23, 42, 0.85); border: 1px solid #334155; border-radius: 6px; padding: 8px 10px; font-size: 12px; color: #38bdf8; margin-bottom: 12px; line-height: 1.6;">
            ${typeof b.getLiveStats === 'function' ? b.getLiveStats() : b.stats}
          </div>
          <button onclick="window.triggerBuildingAction('${b.id}')" style="width: 100%; padding: 8px 12px; background: linear-gradient(135deg, #d97706, #b45309); color: #fff; font-weight: bold; font-size: 12px; border: none; border-radius: 6px; cursor: pointer; transition: all 0.2s; box-shadow: 0 3px 10px rgba(217, 119, 6, 0.4);">
            ${b.actionName}
          </button>
        `;
      }
      return;
    }

    // Case 2: Individual Agent Inspector
    if (state.selectedAgent) {
      const a = state.selectedAgent;
      if (title) {
        title.innerHTML = `
          <div style="display: flex; justify-content: space-between; align-items: center; width: 100%;">
            <span style="color: #38bdf8; font-size: 14px; font-weight: 800;">📜 历史个体: ${a.id}</span>
            <button onclick="window.closeInspector()" style="background: transparent; border: none; color: #94a3b8; font-size: 15px; cursor: pointer; padding: 0 4px; line-height: 1;">✕</button>
          </div>
        `;
      }
      if (details) {
        let agentContent = '';
        if (a.type === 'caravan') {
          agentContent = `
            <strong>类型:</strong> 远洋商贸驼队<br/>
            <strong>所运物资:</strong> ${a.cargo}<br/>
            <strong>行进状态:</strong> 经由六室城门入城交易<br/>
            <strong>商队归属:</strong> 推罗与示巴进贡行商
          `;
        } else if (a.type === 'ancient_knight') {
          agentContent = `
            <div style="margin-bottom: 8px;">
              <span style="background: linear-gradient(135deg, #334155, #0f172a); color: #f8fafc; padding: 2px 8px; border-radius: 4px; font-size: 11px; font-weight: bold; border: 1px solid #94a3b8;">
                ⚔️ 圣殿重装铁甲骑士
              </span>
              <span style="background: #0369a1; color: #bae6fd; padding: 2px 7px; border-radius: 4px; font-size: 11px; font-weight: bold; margin-left: 5px;">
                🛡️ 城防驻守
              </span>
            </div>
            <strong>骑士军阶:</strong> ${a.role}<br/>
            <strong>装备披挂:</strong> 全身精钢锁子板甲、十字战袍、双手巨剑<br/>
            <strong>巡航路线:</strong> 南门六室城门 ↔ 圣殿铜祭坛 ↔ 林宫王廷<br/>
            <strong>誓约忠诚度:</strong> 100% (终身立约守护圣所)<br/>
            <div style="margin-top: 8px; border-left: 3px solid #e2e8f0; padding: 5px 8px; background: rgba(0,0,0,0.3); border-radius: 0 4px 4px 0; font-style: italic; color: #f1f5f9; font-size: 11.5px; line-height: 1.5;">
              ${a.dialogue || '“圣殿骑士守望耶路撒冷，愿公义与和平护佑此城！”'}
            </div>
          `;
        } else {
          let stateTag = '🚶 赶往工位途中';
          if (a.subState === 'WORKING') stateTag = '🔨 专注营生劳作';
          else if (a.subState === 'PRAYING') stateTag = '🔥 圣所虔诚祷告';
          else if (a.subState === 'TRADING') stateTag = '💰 集市称银交易';
          else if (a.subState === 'SOCIALIZING') stateTag = '💬 邻里驻足互致平安';
          else if (a.subState === 'SLEEPING' || a.subState === 'RESTING') stateTag = '💤 宅内熄灯安歇';

          const phaseLabels = { dawn: '🌅 破晓晨曦', noon: '☀️ 正午艳阳', dusk: '🌇 晚霞暮霭', night: '🌙 星夜更深' };

          agentContent = `
            <div style="margin-bottom: 8px;">
              <span style="background: #1e3a8a; color: #93c5fd; padding: 2px 7px; border-radius: 4px; font-size: 11px; font-weight: bold; margin-right: 5px;">
                ${phaseLabels[a.currentPhase] || '☀️ 正午'}
              </span>
              <span style="background: #065f46; color: #6ee7b7; padding: 2px 7px; border-radius: 4px; font-size: 11px; font-weight: bold;">
                ${stateTag}
              </span>
            </div>
            <strong>社会身份:</strong> ${a.role}<br/>
            <strong>当前作息日程:</strong> <span style="color: #fde047; font-weight: bold;">${a.task || '城内营生'}</span><br/>
            <strong>持有财产:</strong> ${a.shekels.toFixed(1)} 舍客勒银币<br/>
            <strong>圣所敬虔度:</strong> ${a.devotion}%<br/>
            <div style="margin-top: 8px; border-left: 3px solid #38bdf8; padding: 5px 8px; background: rgba(0,0,0,0.3); border-radius: 0 4px 4px 0; font-style: italic; color: #cbd5e1; font-size: 11.5px; line-height: 1.5;">
              ${a.dialogue || '“愿和平归于耶路撒冷，愿行公义者得享繁盛。”'}
            </div>
          `;
        }

        details.innerHTML = `
          <div style="font-size: 12px; color: #cbd5e1; line-height: 1.6; margin-bottom: 12px;">
            ${agentContent}
          </div>
          <button onclick="window.toggleAgentFollow()" style="width: 100%; padding: 8px 12px; background: ${followAgent === a ? '#dc2626' : '#0284c7'}; color: #fff; font-weight: bold; font-size: 12px; border: none; border-radius: 6px; cursor: pointer; transition: all 0.2s;">
            ${followAgent === a ? '⏹️ 停止视角锁定跟随' : '🎥 锁定跟随个体巡游'}
          </button>
          <button onclick="window.takeControlOfAgent(state.selectedAgent)" style="width: 100%; margin-top: 8px; padding: 8px 12px; background: ${a.type === 'ancient_knight' ? 'linear-gradient(135deg, #0284c7, #2563eb)' : 'linear-gradient(135deg, #10b981, #059669)'}; color: #fff; font-weight: bold; font-size: 12px; border: none; border-radius: 6px; cursor: pointer; transition: all 0.2s; box-shadow: 0 3px 10px rgba(0,0,0,0.4);">
            ${a.type === 'ancient_knight' ? '⚔️ 附身操控圣殿骑士 (WASD第三人称视角)' : '🎮 附身操控此人 (WASD第三人称视角)'}
          </button>
        `;
      }
    }
  }

  // ------------------------------------------------------------------------
  // Interactive Building Action Dispatcher
  // ------------------------------------------------------------------------
  window.triggerBuildingAction = function (id) {
    if (id === 'temple_solomon') {
      state.templeCity.sacredAroma = Math.min(100, state.templeCity.sacredAroma + 15);
      state.templeCity.templeTreasury += 25;
      triggerCitySound('chime');
      showFloatToast('🔥 晨昏燔祭已献上！全城敬虔度与金库提升');
    } else if (id === 'palace_lebanon') {
      state.templeCity.templeTreasury += 40;
      state.templeCity.tradeProsperity = Math.min(3.0, state.templeCity.tradeProsperity + 0.15);
      triggerCitySound('horn');
      showFloatToast('👑 所罗门王颁布行政律令，远洋岁贡如约而至');
    } else if (id === 'royal_granary') {
      if (state.templeCity.grainStorage > 15) {
        state.templeCity.grainStorage -= 15;
        state.templeCity.citizenMorale = Math.min(100, state.templeCity.citizenMorale + 10);
        triggerCitySound('coins');
        showFloatToast('🌾 皇家粮仓开仓济民！城邦民心安乐');
      }
    } else if (id === 'gihon_cistern') {
      state.templeCity.waterSupply = 100.0;
      triggerCitySound('water');
      showFloatToast('💧 基训地下暗渠清泉涌入，蓄水池已蓄满！');
    } else if (id === 'bronze_foundry') {
      state.templeCity.tradeProsperity = Math.min(3.0, state.templeCity.tradeProsperity + 0.1);
      triggerCitySound('anvil');
      showFloatToast('⚒️ 推罗工匠精炼青铜礼器出炉，工艺卓绝');
    } else if (id === 'olive_press') {
      state.templeCity.sacredAroma = Math.min(100, state.templeCity.sacredAroma + 8);
      triggerCitySound('water');
      showFloatToast('🫒 榨得特级初熟橄榄纯油，供奉圣殿长明灯');
    } else if (id === 'central_bazaar') {
      state.templeCity.marketTurnover += 65;
      state.templeCity.tradeProsperity = Math.min(3.0, state.templeCity.tradeProsperity + 0.2);
      triggerCitySound('coins');
      showFloatToast('💰 诸国商贾齐聚中央大集，贸易繁荣');
    } else if (id === 'four_room_house') {
      state.templeCity.citizenMorale = Math.min(100, state.templeCity.citizenMorale + 5);
      triggerCitySound('chime');
      showFloatToast('🏡 四室民居五谷充盈，琴瑟相和');
    } else if (id === 'city_gate_elders') {
      state.templeCity.citizenMorale = Math.min(100, state.templeCity.citizenMorale + 6);
      triggerCitySound('horn');
      showFloatToast('⚖️ 城门口长老秉公判决，卫队戍卫严密');
    } else if (id === 'bush_flora') {
      state.templeCity.sacredAroma = Math.min(100, state.templeCity.sacredAroma + 12);
      state.templeCity.citizenMorale = Math.min(100, state.templeCity.citizenMorale + 10);
      triggerCitySound('chime');
      showFloatToast('🌿 采得迦南初熟佳果与香草！全城民心与圣所香气提升');
    }

    updateInspectorHUD();
    updateControlsUI();
  };

  function updateControlsUI() {
    const slidersContainer = document.getElementById('sb-sliders');
    if (!slidersContainer) return;

    slidersContainer.innerHTML = `
      <div style="display: flex; gap: 14px; align-items: center; border-right: 1px solid #334155; padding-right: 14px; margin-right: 8px;">
        <span title="战略储粮">🌾 <strong>${state.templeCity.grainStorage.toFixed(0)}t</strong></span>
        <span title="基训泉蓄水池水位">💧 <strong>${state.templeCity.waterSupply.toFixed(0)}%</strong></span>
        <span title="圣殿金库">💰 <strong>${state.templeCity.templeTreasury}</strong> 舍客勒</span>
        <span title="民心满意度">🕊️ <strong>${state.templeCity.citizenMorale}%</strong></span>
      </div>
      <div class="slider-row" style="display: flex; align-items: center; gap: 6px;">
        <label style="font-size: 11px; color: #94a3b8;">商贸繁荣: <span id="val-growth" style="color: #38bdf8; font-weight: bold;">${state.templeCity.tradeProsperity.toFixed(1)}x</span></label>
        <input type="range" min="0.5" max="3.0" step="0.2" value="${state.templeCity.tradeProsperity}" oninput="window.setSandboxParam('templeCity.tradeProsperity', this.value); document.getElementById('val-growth').innerText=this.value+'x';">
      </div>
      <div class="slider-row" style="display: flex; align-items: center; gap: 6px;">
        <label style="font-size: 11px; color: #94a3b8;">什一奉献: <span id="val-tax" style="color: #4ade80; font-weight: bold;">${(state.templeCity.tithesRate * 100).toFixed(0)}%</span></label>
        <input type="range" min="0.05" max="0.25" step="0.01" value="${state.templeCity.tithesRate}" oninput="window.setSandboxParam('templeCity.tithesRate', this.value); document.getElementById('val-tax').innerText=(this.value*100).toFixed(0)+'%';">
      </div>
      <button onclick="window.toggleSpawnTool()" style="margin-left: 8px; padding: 4px 10px; background: ${state.spawnToolActive ? '#dc2626' : '#0369a1'}; color: #fff; font-size: 11px; font-weight: bold; border: 1px solid #38bdf8; border-radius: 6px; cursor: pointer;">
        ${state.spawnToolActive ? '❌ 退出招募' : '➕ 招募新居民'}
      </button>
      <button onclick="if (window.plantBush) window.plantBush((Math.random()-0.5)*36, (Math.random()-0.5)*36, 0.9+Math.random()*0.5)" style="margin-left: 6px; padding: 4px 10px; background: #166534; color: #bbf7d0; font-size: 11px; font-weight: bold; border: 1px solid #22c55e; border-radius: 6px; cursor: pointer;">
        🌱 栽植圣木
      </button>
    `;
  }

  // ------------------------------------------------------------------------
  // External Controls API
  // ------------------------------------------------------------------------
  window.plantBush = function (x, z, scale = 1.0) {
    loadBushAsset(() => {
      const b = createBushInstance(scale);
      if (b && simGroup) {
        b.position.set(x || 0, 0, z || 0);
        b.rotation.y = Math.random() * Math.PI * 2;
        simGroup.add(b);
        bushInstances.push(b);
        showFloatToast(`🌱 圣木/灌木已栽植在坐标 (${(x || 0).toFixed(0)}, ${(z || 0).toFixed(0)})！`);
        triggerCitySound('chime');
      }
    });
  };
  window.setSandboxParam = function (key, value) {
    const val = parseFloat(value);
    if (key === 'templeCity.tradeProsperity') {
      state.templeCity.tradeProsperity = val;
    } else if (key === 'templeCity.tithesRate') {
      state.templeCity.tithesRate = val;
    }
  };

  window.toggleSandboxPlayback = function () {
    state.isPlaying = !state.isPlaying;
    return state.isPlaying;
  };

  window.setSandboxSpeed = function (speed) {
    state.speedMultiplier = parseFloat(speed);
  };

  function onWindowResize() {
    if (!container || !renderer) return;
    const w = container.clientWidth || 840;
    const h = container.clientHeight || 520;
    const aspect = w / h;
    const d = 36;
    if (orthoCamera) {
      orthoCamera.left = -d * aspect;
      orthoCamera.right = d * aspect;
      orthoCamera.top = d;
      orthoCamera.bottom = -d;
      orthoCamera.updateProjectionMatrix();
    }
    if (perspCamera) {
      perspCamera.aspect = aspect;
      perspCamera.updateProjectionMatrix();
    }
    renderer.setSize(w, h);
  }

  window._sandbox = {
    state,
    get controlledAgent() { return controlledAgent; },
    get camera() { return camera; },
    get perspCamera() { return perspCamera; },
    get orthoCamera() { return orthoCamera; },
    get agents() { return agents; },
    get scene() { return scene; },
    get simGroup() { return simGroup; },
    get renderer() { return renderer; },
    get controls() { return controls; },
    get inputKeys() { return inputKeys; },
  };
})();
