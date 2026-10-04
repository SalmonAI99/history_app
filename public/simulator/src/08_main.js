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