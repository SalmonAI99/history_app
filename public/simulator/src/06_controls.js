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

  if (window.SandboxNetwork) {
    let mType = isKnight ? "knight" : "citizen";
    window.SandboxNetwork.sendMove(mesh.position.x, mesh.position.y, mesh.position.z, mesh.rotation.y, controlledAgent.currentAnim || "idle", mType);
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