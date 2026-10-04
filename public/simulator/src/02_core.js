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
    if (window.SandboxNetwork) window.SandboxNetwork.updateInterpolation(delta);
    renderer.render(scene, camera);

    // Render Tactical Minimap
    drawMinimap();
  }

  if (window.SandboxNetwork) window.SandboxNetwork.init();
  animate(performance.now());
};

// ------------------------------------------------------------------------