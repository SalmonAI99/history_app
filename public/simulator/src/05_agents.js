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