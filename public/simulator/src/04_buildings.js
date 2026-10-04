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