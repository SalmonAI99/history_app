// Historical Map RPG Exploration Simulator Engine
(function () {
  'use strict';

  // Game World State
  const state = {
    player: {
      x: 100,
      y: 330,
      speed: 3.5,
      targetX: null,
      targetY: null,
      direction: 'down', // 'down', 'up', 'left', 'right'
      isMoving: false,
      stepAnim: 0,
    },
    resources: 75,
    knowledge: 60,
    hope: 80,
    soundEnabled: true,
    unlockedArtifacts: new Set(),
    activeNearbyNpc: null,
    dialogueOpen: false,
    visitedLandmarks: new Set(),
  };

  // Sound Synthesizer via Web Audio API (100% offline, zero latency)
  const audioCtx = window.AudioContext || window.webkitAudioContext ? new (window.AudioContext || window.webkitAudioContext)() : null;

  function playSound(type) {
    if (!state.soundEnabled || !audioCtx) return;
    try {
      if (audioCtx.state === 'suspended') audioCtx.resume();
      const osc = audioCtx.createOscillator();
      const gain = audioCtx.createGain();
      osc.connect(gain);
      gain.connect(audioCtx.destination);
      const now = audioCtx.currentTime;

      if (type === 'step') {
        osc.type = 'sine';
        osc.frequency.setValueAtTime(140, now);
        osc.frequency.exponentialRampToValueAtTime(80, now + 0.04);
        gain.gain.setValueAtTime(0.04, now);
        gain.gain.linearRampToValueAtTime(0.001, now + 0.05);
        osc.start(now);
        osc.stop(now + 0.05);
      } else if (type === 'pickup') {
        osc.type = 'triangle';
        osc.frequency.setValueAtTime(587.33, now); // D5
        osc.frequency.setValueAtTime(880, now + 0.08); // A5
        gain.gain.setValueAtTime(0.12, now);
        gain.gain.linearRampToValueAtTime(0.01, now + 0.18);
        osc.start(now);
        osc.stop(now + 0.18);
      } else if (type === 'dialogue') {
        osc.type = 'sine';
        osc.frequency.setValueAtTime(440, now);
        osc.frequency.exponentialRampToValueAtTime(659.25, now + 0.1);
        gain.gain.setValueAtTime(0.1, now);
        gain.gain.linearRampToValueAtTime(0.01, now + 0.15);
        osc.start(now);
        osc.stop(now + 0.15);
      } else if (type === 'victory') {
        osc.type = 'triangle';
        osc.frequency.setValueAtTime(523.25, now);
        osc.frequency.setValueAtTime(659.25, now + 0.1);
        osc.frequency.setValueAtTime(783.99, now + 0.2);
        osc.frequency.setValueAtTime(1046.5, now + 0.3);
        gain.gain.setValueAtTime(0.2, now);
        gain.gain.linearRampToValueAtTime(0.01, now + 0.6);
        osc.start(now);
        osc.stop(now + 0.6);
      }
    } catch (e) {}
  }

  // 5 Collectible Historical Artifacts
  const ARTIFACTS = [
    {
      id: 'torah',
      icon: '📜',
      name: '摩西羊皮卷',
      landmark: '尼罗河与旷野',
      desc: '在荒漠漫游中诞生的古老法典，使族人无论走到哪里都拥有统一的道德与精神指引。'
    },
    {
      id: 'temple',
      icon: '🏛️',
      name: '所罗门雪松木',
      landmark: '所罗门圣殿',
      desc: '第一圣殿的栋梁，象征古代王国空前繁荣与对外通商的黄金鼎盛期。'
    },
    {
      id: 'ledger',
      icon: '🧮',
      name: '中世纪跨国账册',
      landmark: '中世纪欧洲城镇',
      desc: '在被严禁购买农田的时代，通过流动贸易、行医与记账金融开辟生路的历史物证。'
    },
    {
      id: 'wall_note',
      icon: '🧱',
      name: '哭墙祈祷信笺',
      landmark: '哭墙广场',
      desc: '第二圣殿被罗马摧毁后仅存的西墙石缝中，塞满了流散四海后人对安宁的祈盼。'
    },
    {
      id: 'hat',
      icon: '🎩',
      name: '卓别林礼帽徽章',
      landmark: '和平讲坛',
      desc: '1940年电影《大独裁者》反法西斯的象征，向人类发出拒绝仇恨、拥抱和平的呐喊。'
    }
  ];

  // Map NPCs & Historical Landmarks
  const NPCS = [
    {
      id: 'moses',
      name: '先知与向导 · 摩西',
      titleTag: '尼罗河畔营地',
      x: 120,
      y: 310,
      icon: '🧙‍♂️',
      color: '#d97706',
      dialogue: '“法老的追兵在后，红海在前。族人们口渴疲惫，对前途感到迷茫。但只要我们心怀信念、坚持教育与规诫，就一定能穿越荒漠走向应许之地！”',
      choices: [
        {
          title: '📖 优先携带羊皮典籍与饮水器皿',
          desc: '虽然负重增加，但让孩子们沿途继续学习文字与祖先的历史。（对应第13-17页）',
          impact: '文化 +25, 资源 -5, 获得《摩西羊皮卷》',
          rewardArtifact: 'torah',
          effect: { res: -5, know: 25, hope: 15 }
        },
        {
          title: '🏃 轻装急行，紧跟火柱方向昼夜兼程',
          desc: '全族凝聚一心，避开追兵。（对应第17页）',
          impact: '信念 +25, 资源 +10, 获得《摩西羊皮卷》',
          rewardArtifact: 'torah',
          effect: { res: 10, know: 10, hope: 25 }
        }
      ]
    },
    {
      id: 'oasis_elder',
      name: '旷野向导 · 勘探长老',
      titleTag: '西奈旷野绿洲',
      x: 270,
      y: 200,
      icon: '🧔',
      color: '#ca8a04',
      dialogue: '“西奈旷野烈日如火，我们找到了这处清泉绿洲！请在此休整补充淡水，前方就是迦南的门户。”',
      choices: [
        {
          title: '💧 汲取地下清泉并分配给老弱',
          desc: '确保所有人都有足够清水穿越最后一段旱谷。',
          impact: '资源 +30, 信念 +15',
          effect: { res: 30, know: 5, hope: 15 }
        }
      ]
    },
    {
      id: 'solomon',
      name: '贤王 · 所罗门',
      titleTag: '耶路撒冷 · 第一圣殿',
      x: 440,
      y: 110,
      icon: '👑',
      color: '#eab308',
      dialogue: '“大卫王收复迦南、定都耶路撒冷。如今国家和平繁荣，我从黎巴嫩运来雪松木建造圣殿，作为全民族的信仰灯塔与智慧圣地。”（对应第25-26页）',
      choices: [
        {
          title: '🌲 采购黎巴嫩雪松，精工雕琢圣殿石壁',
          desc: '打造千年不倒的精神中心，吸引邻国使节来访交流。',
          impact: '文化 +25, 信念 +25, 获得《所罗门雪松木》',
          rewardArtifact: 'temple',
          effect: { res: -10, know: 25, hope: 25 }
        },
        {
          title: '⚖️ 设立公共讲堂与智慧法庭',
          desc: '用公正法度平息争端，减轻徭役与赋税。',
          impact: '文化 +30, 资源 +15, 获得《所罗门雪松木》',
          rewardArtifact: 'temple',
          effect: { res: 15, know: 30, hope: 15 }
        }
      ]
    },
    {
      id: 'scholar',
      name: '中世纪学者与旅商',
      titleTag: '中世纪欧洲城镇广场',
      x: 690,
      y: 110,
      icon: '🧑‍🏫',
      color: '#2563eb',
      dialogue: '“圣殿被毁后族人流散至西欧。领主下令：禁止我们购买和耕种土地，亦排斥加入工匠行会。大家无法种田，该如何生存？”（对应第32-38页）',
      choices: [
        {
          title: '📚 无论贫富，全家坚持读书与医学传习',
          desc: '“土地会被剥夺，但装在脑子里的知识谁也抢不走！”崇尚读书教育，通晓多国语言。',
          impact: '文化 +35, 信念 +20, 获得《中世纪跨国账册》',
          rewardArtifact: 'ledger',
          effect: { res: 10, know: 35, hope: 20 }
        },
        {
          title: '🧮 联结跨区域商队，开展贸易与信用记账',
          desc: '利用遍布地中海的网络搭建贸易桥梁，从事长途商业。',
          impact: '资源 +35, 文化 +20, 获得《中世纪跨国账册》',
          rewardArtifact: 'ledger',
          effect: { res: 35, know: 20, hope: 10 }
        }
      ]
    },
    {
      id: 'wall_guardian',
      name: '哭墙守望老人',
      titleTag: '耶路撒冷旧城 · 哭墙',
      x: 430,
      y: 330,
      icon: '👴',
      color: '#64748b',
      dialogue: '“第二圣殿被罗马摧毁后，只剩下这面西墙基石。千百年来，流散在外的后人回到这里抚石哀哭，在石缝中塞入对民族与和平的祈求。”（对应第50-54页）',
      choices: [
        {
          title: '🕊️ 将写满“愿和平永驻”的信笺塞入石缝',
          desc: '抚摸古老巨石，传承不灭的希望。',
          impact: '信念 +30, 文化 +15, 获得《哭墙祈祷信笺》',
          rewardArtifact: 'wall_note',
          effect: { res: 5, know: 15, hope: 30 }
        }
      ]
    },
    {
      id: 'chaplin',
      name: '和平大师 · 查理·卓别林',
      titleTag: '近代反思讲坛',
      x: 700,
      y: 320,
      icon: '🎩',
      color: '#0f172a',
      dialogue: '“在二战的浩劫后，我在电影《大独裁者》中呼吁：我们要用仁爱取代暴力，用理智破除偏见！生活本该自由美好，教育与同理心才是和平的归宿！”（对应第88-90页）',
      choices: [
        {
          title: '🤝 倡导用教育、理解与理性取代仇恨',
          desc: '铭记惨痛教训，呼吁两族人民互尊互重、共创未来。',
          impact: '信念 +35, 文化 +30, 获得《卓别林礼帽徽章》',
          rewardArtifact: 'hat',
          effect: { res: 15, know: 30, hope: 35 }
        }
      ]
    }
  ];

  // Collectible items on map
  let mapPickups = [
    { x: 190, y: 260, type: 'water', icon: '💧', name: '甘泉水囊', res: 15, collected: false },
    { x: 360, y: 190, type: 'scroll', icon: '📜', name: '古老字卷', know: 15, collected: false },
    { x: 550, y: 130, type: 'olive', icon: '🌿', name: '橄榄枝', hope: 15, collected: false },
    { x: 560, y: 310, type: 'water', icon: '💧', name: '旅者补给', res: 15, collected: false },
    { x: 330, y: 340, type: 'scroll', icon: '📜', name: '圣殿残卷', know: 15, collected: false }
  ];

  // DOM Elements
  const el = {
    canvas: document.getElementById('game-canvas'),
    canvasViewport: document.getElementById('canvas-viewport'),
    epochBadge: document.getElementById('current-quest-badge'),
    barRes: document.getElementById('bar-res'),
    valRes: document.getElementById('val-res'),
    barKnow: document.getElementById('bar-know'),
    valKnow: document.getElementById('val-know'),
    barHope: document.getElementById('bar-hope'),
    valHope: document.getElementById('val-hope'),
    btnSound: document.getElementById('btn-sound-toggle'),
    btnArtifacts: document.getElementById('btn-artifacts'),
    artifactCount: document.getElementById('artifact-count'),
    proximityPrompt: document.getElementById('proximity-prompt'),
    promptActionText: document.getElementById('prompt-action-text'),
    dialogueModal: document.getElementById('dialogue-modal'),
    dialogueSpeakerName: document.getElementById('dialogue-speaker-name'),
    dialogueAvatar: document.getElementById('dialogue-avatar'),
    dialogueBodyText: document.getElementById('dialogue-body-text'),
    dialogueChoices: document.getElementById('dialogue-choices'),
    dialogueCloseBtn: document.getElementById('dialogue-close-btn'),
    artifactsModal: document.getElementById('artifacts-modal'),
    artifactsClose: document.getElementById('artifacts-close'),
    artifactsGrid: document.getElementById('artifacts-grid'),
    summaryModal: document.getElementById('summary-modal'),
    btnReplay: document.getElementById('btn-replay'),
    dpadActionBtn: document.getElementById('dpad-action-btn'),
  };

  const ctx = el.canvas.getContext('2d');
  const MAP_W = el.canvas.width;
  const MAP_H = el.canvas.height;

  // Keyboard input tracking
  const keysDown = {};
  window.addEventListener('keydown', (e) => {
    keysDown[e.code] = true;

    // Interaction key (Space or E)
    if (['Space', 'KeyE'].includes(e.code)) {
      if (state.activeNearbyNpc && !state.dialogueOpen) {
        e.preventDefault();
        openDialogue(state.activeNearbyNpc);
      }
    }
  });

  window.addEventListener('keyup', (e) => {
    keysDown[e.code] = false;
  });

  // Mouse / Touch click-to-move
  el.canvas.addEventListener('click', (e) => {
    if (state.dialogueOpen) return;
    const rect = el.canvas.getBoundingClientRect();
    const scaleX = MAP_W / rect.width;
    const scaleY = MAP_H / rect.height;
    state.player.targetX = (e.clientX - rect.left) * scaleX;
    state.player.targetY = (e.clientY - rect.top) * scaleY;
  });

  // Mobile Virtual D-Pad
  let dpadInterval = null;
  document.querySelectorAll('.dpad-btn[data-dir]').forEach((btn) => {
    const dir = btn.dataset.dir;
    const startMove = (e) => {
      e.preventDefault();
      clearInterval(dpadInterval);
      dpadInterval = setInterval(() => {
        if (dir === 'up') movePlayer(0, -state.player.speed);
        if (dir === 'down') movePlayer(0, state.player.speed);
        if (dir === 'left') movePlayer(-state.player.speed, 0);
        if (dir === 'right') movePlayer(state.player.speed, 0);
      }, 25);
    };
    const stopMove = () => clearInterval(dpadInterval);

    btn.addEventListener('touchstart', startMove, { passive: false });
    btn.addEventListener('touchend', stopMove);
    btn.addEventListener('mousedown', startMove);
    btn.addEventListener('mouseup', stopMove);
    btn.addEventListener('mouseleave', stopMove);
  });

  if (el.dpadActionBtn) {
    el.dpadActionBtn.onclick = () => {
      if (state.activeNearbyNpc && !state.dialogueOpen) {
        openDialogue(state.activeNearbyNpc);
      }
    };
  }

  // Move Player Function with Bounds & Direction
  function movePlayer(dx, dy) {
    const p = state.player;
    if (dx !== 0 || dy !== 0) {
      p.isMoving = true;
      p.stepAnim += 0.2;
      p.x = Math.max(25, Math.min(MAP_W - 25, p.x + dx));
      p.y = Math.max(35, Math.min(MAP_H - 30, p.y + dy));

      if (Math.abs(dx) > Math.abs(dy)) {
        p.direction = dx > 0 ? 'right' : 'left';
      } else {
        p.direction = dy > 0 ? 'down' : 'up';
      }

      // Check item collisions
      checkItemPickups();
    } else {
      p.isMoving = false;
    }
  }

  // Pickup Collision Check
  function checkItemPickups() {
    const p = state.player;
    mapPickups.forEach((item) => {
      if (!item.collected) {
        const dist = Math.hypot(p.x - item.x, p.y - item.y);
        if (dist < 26) {
          item.collected = true;
          playSound('pickup');
          state.resources += item.res || 0;
          state.knowledge += item.know || 0;
          state.hope += item.hope || 0;
          updateStatusUI();
        }
      }
    });
  }

  // Main Game Loop (60 FPS)
  function gameLoop() {
    updateGameLogic();
    renderGameWorld();
    requestAnimationFrame(gameLoop);
  }

  // Update Logic
  function updateGameLogic() {
    if (state.dialogueOpen) return;

    let dx = 0;
    let dy = 0;

    // Keyboard arrow / WASD movement
    if (keysDown['ArrowUp'] || keysDown['KeyW']) dy -= state.player.speed;
    if (keysDown['ArrowDown'] || keysDown['KeyS']) dy += state.player.speed;
    if (keysDown['ArrowLeft'] || keysDown['KeyA']) dx -= state.player.speed;
    if (keysDown['ArrowRight'] || keysDown['KeyD']) dx += state.player.speed;

    // Click-to-move interpolation
    if (state.player.targetX !== null && state.player.targetY !== null) {
      const dist = Math.hypot(state.player.targetX - state.player.x, state.player.targetY - state.player.y);
      if (dist > 4) {
        const angle = Math.atan2(state.player.targetY - state.player.y, state.player.targetX - state.player.x);
        dx = Math.cos(angle) * state.player.speed;
        dy = Math.sin(angle) * state.player.speed;
      } else {
        state.player.targetX = null;
        state.player.targetY = null;
      }
    }

    movePlayer(dx, dy);

    // Proximity check with NPCs
    let nearby = null;
    NPCS.forEach((npc) => {
      const dist = Math.hypot(state.player.x - npc.x, state.player.y - npc.y);
      if (dist < 55) {
        nearby = npc;
      }
    });

    state.activeNearbyNpc = nearby;
    if (nearby && !state.dialogueOpen) {
      el.proximityPrompt.style.display = 'flex';
      el.promptActionText.textContent = `与 ${nearby.name} 交谈`;
    } else {
      el.proximityPrompt.style.display = 'none';
    }
  }

  // Render 2D Historical World
  function renderGameWorld() {
    ctx.clearRect(0, 0, MAP_W, MAP_H);

    // 1. Terrain & Regions
    drawMapBackground();

    // 2. Landmarks & Roads
    drawRoadsAndLandmarks();

    // 3. Collectibles
    drawPickups();

    // 4. NPCs
    drawNpcs();

    // 5. Player Character
    drawPlayer();
  }

  // Draw Themed Regional Background
  function drawMapBackground() {
    // West: Desert / Egypt (Warm sand)
    ctx.fillStyle = '#fde68a';
    ctx.fillRect(0, 0, 320, MAP_H);

    // Nile river on left edge
    ctx.fillStyle = '#38bdf8';
    ctx.beginPath();
    ctx.moveTo(30, 0);
    ctx.bezierCurveTo(80, 150, 20, 300, 70, MAP_H);
    ctx.lineTo(0, MAP_H);
    ctx.lineTo(0, 0);
    ctx.fill();

    // Middle-East: Hills & Jerusalem (Limestone/Olive Green)
    const midGrad = ctx.createLinearGradient(320, 0, 560, MAP_H);
    midGrad.addColorStop(0, '#e2e8f0');
    midGrad.addColorStop(0.5, '#dcfce7');
    midGrad.addColorStop(1, '#fef3c7');
    ctx.fillStyle = midGrad;
    ctx.fillRect(320, 0, 260, MAP_H);

    // East: Europe & Town (Cobblestone / Forest)
    const eastGrad = ctx.createLinearGradient(580, 0, MAP_W, MAP_H);
    eastGrad.addColorStop(0, '#e2e8f0');
    eastGrad.addColorStop(1, '#cbd5e1');
    ctx.fillStyle = eastGrad;
    ctx.fillRect(580, 0, MAP_W - 580, MAP_H);

    // Region boundary accents
    ctx.strokeStyle = '#d6cbba';
    ctx.lineWidth = 2;
    ctx.setLineDash([6, 6]);
    ctx.beginPath();
    ctx.moveTo(320, 0); ctx.lineTo(320, MAP_H);
    ctx.moveTo(580, 0); ctx.lineTo(580, MAP_H);
    ctx.stroke();
    ctx.setLineDash([]);

    // Region Labels
    ctx.font = 'bold 12px sans-serif';
    ctx.fillStyle = '#b45309';
    ctx.fillText('🇪🇬 尼罗河与西奈旷野', 90, 25);
    ctx.fillStyle = '#15803d';
    ctx.fillText('🏛️ 迦南 · 耶路撒冷圣山', 360, 25);
    ctx.fillStyle = '#1e3a8a';
    ctx.fillText('🏰 中世纪西欧城镇', 640, 25);
  }

  // Draw Roads, Rivers & Landmark Buildings
  function drawRoadsAndLandmarks() {
    // Ancient Trading Road
    ctx.strokeStyle = '#e6ccaa';
    ctx.lineWidth = 14;
    ctx.lineCap = 'round';
    ctx.beginPath();
    ctx.moveTo(110, 310);
    ctx.quadraticCurveTo(240, 270, 280, 210);
    ctx.quadraticCurveTo(340, 150, 440, 120);
    ctx.quadraticCurveTo(560, 110, 680, 110);
    ctx.quadraticCurveTo(600, 250, 440, 330);
    ctx.quadraticCurveTo(550, 340, 690, 320);
    ctx.stroke();

    // 1. Egypt Pyramid / Camp
    drawPyramid(70, 250);

    // 2. Sinai Palm Oasis
    drawOasis(270, 180);

    // 3. Solomon's Temple Landmark
    drawTempleBuilding(440, 75);

    // 4. European Townhouses
    drawTownHouses(680, 75);

    // 5. Western Wall (Limestone Wall)
    drawWallStructure(430, 290);

    // 6. Peace Forum
    drawPeacePodium(690, 285);
  }

  function drawPyramid(x, y) {
    ctx.fillStyle = '#d97706';
    ctx.beginPath();
    ctx.moveTo(x - 28, y + 25);
    ctx.lineTo(x, y - 20);
    ctx.lineTo(x + 28, y + 25);
    ctx.fill();
    ctx.fillStyle = '#b45309';
    ctx.beginPath();
    ctx.moveTo(x, y - 20);
    ctx.lineTo(x + 28, y + 25);
    ctx.lineTo(x + 12, y + 25);
    ctx.fill();
  }

  function drawOasis(x, y) {
    // Water pool
    ctx.fillStyle = '#38bdf8';
    ctx.beginPath();
    ctx.arc(x, y + 10, 24, 0, Math.PI * 2);
    ctx.fill();
    // Palm
    ctx.fillStyle = '#78350f';
    ctx.fillRect(x + 12, y - 25, 4, 30);
    ctx.fillStyle = '#15803d';
    ctx.beginPath();
    ctx.arc(x + 14, y - 28, 16, 0, Math.PI, true);
    ctx.fill();
  }

  function drawTempleBuilding(x, y) {
    ctx.fillStyle = '#fef3c7';
    ctx.fillRect(x - 30, y - 10, 60, 40);
    ctx.fillStyle = '#d97706';
    ctx.fillRect(x - 34, y - 15, 68, 6);
    // Pillars
    ctx.fillStyle = '#b45309';
    ctx.fillRect(x - 22, y - 9, 6, 38);
    ctx.fillRect(x - 6, y - 9, 6, 38);
    ctx.fillRect(x + 10, y - 9, 6, 38);
  }

  function drawTownHouses(x, y) {
    ctx.fillStyle = '#78350f';
    ctx.beginPath();
    ctx.moveTo(x - 30, y);
    ctx.lineTo(x, y - 22);
    ctx.lineTo(x + 30, y);
    ctx.fill();
    ctx.fillStyle = '#fed7aa';
    ctx.fillRect(x - 24, y, 48, 30);
    ctx.fillStyle = '#0284c7';
    ctx.fillRect(x - 14, y + 6, 10, 12);
    ctx.fillRect(x + 4, y + 6, 10, 12);
  }

  function drawWallStructure(x, y) {
    ctx.fillStyle = '#d6cbba';
    ctx.fillRect(x - 40, y - 15, 80, 36);
    ctx.strokeStyle = '#9ca3af';
    ctx.lineWidth = 1;
    ctx.strokeRect(x - 40, y - 15, 80, 36);
    ctx.fillStyle = '#15803d';
    ctx.fillRect(x - 10, y - 5, 8, 4);
    ctx.fillRect(x + 14, y + 6, 10, 4);
  }

  function drawPeacePodium(x, y) {
    ctx.fillStyle = '#f8fafc';
    ctx.beginPath();
    ctx.arc(x, y, 25, 0, Math.PI * 2);
    ctx.fill();
    ctx.strokeStyle = '#3b82f6';
    ctx.lineWidth = 2;
    ctx.stroke();
    ctx.font = '16px sans-serif';
    ctx.fillText('🕊️', x - 8, y + 6);
  }

  // Draw Pickups (Water, Scrolls, Olive branches)
  function drawPickups() {
    mapPickups.forEach((item) => {
      if (!item.collected) {
        // Floating glow
        ctx.fillStyle = 'rgba(255, 255, 255, 0.7)';
        ctx.beginPath();
        ctx.arc(item.x, item.y, 14, 0, Math.PI * 2);
        ctx.fill();

        ctx.font = '16px sans-serif';
        ctx.fillText(item.icon, item.x - 8, item.y + 6);
      }
    });
  }

  // Draw NPCs with Glowing interactive rings
  function drawNpcs() {
    NPCS.forEach((npc) => {
      // Glow ring if visited vs new
      const isVisited = state.visitedLandmarks.has(npc.id);
      ctx.fillStyle = isVisited ? 'rgba(74, 222, 128, 0.25)' : 'rgba(250, 204, 21, 0.35)';
      ctx.beginPath();
      ctx.arc(npc.x, npc.y, 20, 0, Math.PI * 2);
      ctx.fill();

      // NPC Figure
      ctx.fillStyle = npc.color;
      ctx.fillRect(npc.x - 8, npc.y - 12, 16, 24);
      ctx.fillStyle = '#fed7aa';
      ctx.beginPath();
      ctx.arc(npc.x, npc.y - 18, 7, 0, Math.PI * 2);
      ctx.fill();

      // Head icon/hat
      ctx.font = '14px sans-serif';
      ctx.fillText(npc.icon, npc.x - 7, npc.y - 28);

      // Name banner
      ctx.font = 'bold 11px sans-serif';
      ctx.fillStyle = '#0f172a';
      ctx.textAlign = 'center';
      ctx.fillText(npc.name.split('·')[0], npc.x, npc.y + 24);
      ctx.textAlign = 'start';
    });
  }

  // Draw Player Character (Smooth sprite with walking animation)
  function drawPlayer() {
    const p = state.player;

    // Shadow
    ctx.fillStyle = 'rgba(0, 0, 0, 0.25)';
    ctx.beginPath();
    ctx.ellipse(p.x, p.y + 12, 10, 5, 0, 0, Math.PI * 2);
    ctx.fill();

    // Body Robe (Blue Explorer)
    ctx.fillStyle = '#2563eb';
    ctx.fillRect(p.x - 7, p.y - 10, 14, 20);

    // Head
    ctx.fillStyle = '#fed7aa';
    ctx.beginPath();
    ctx.arc(p.x, p.y - 16, 7, 0, Math.PI * 2);
    ctx.fill();

    // Explorer Turban / Cap
    ctx.fillStyle = '#1e3a8a';
    ctx.fillRect(p.x - 8, p.y - 24, 16, 6);

    // Moving Legs
    const legOffset = p.isMoving ? Math.sin(p.stepAnim) * 4 : 0;
    ctx.fillStyle = '#1e293b';
    ctx.fillRect(p.x - 5, p.y + 10, 3, 6 + legOffset);
    ctx.fillRect(p.x + 2, p.y + 10, 3, 6 - legOffset);

    // Directional indicator
    ctx.fillStyle = '#fde047';
    if (p.direction === 'right') {
      ctx.fillRect(p.x + 3, p.y - 17, 3, 3);
    } else if (p.direction === 'left') {
      ctx.fillRect(p.x - 6, p.y - 17, 3, 3);
    } else if (p.direction === 'down') {
      ctx.fillRect(p.x - 3, p.y - 17, 2, 2);
      ctx.fillRect(p.x + 1, p.y - 17, 2, 2);
    }
  }

  // Open Dialogue Modal
  function openDialogue(npc) {
    state.dialogueOpen = true;
    playSound('dialogue');

    el.dialogueSpeakerName.textContent = `${npc.name} (${npc.titleTag})`;
    el.dialogueAvatar.textContent = npc.icon;
    el.dialogueBodyText.textContent = npc.dialogue;

    // Render Choices
    let html = '';
    npc.choices.forEach((choice, idx) => {
      html += `
        <button class="choice-btn" data-choice="${idx}">
          <div class="choice-title">${choice.title}</div>
          <div class="choice-desc">${choice.desc}</div>
          <div class="choice-impact">效果：${choice.impact}</div>
        </button>
      `;
    });
    el.dialogueChoices.innerHTML = html;

    el.dialogueModal.style.display = 'flex';
  }

  // Choice Handler
  el.dialogueChoices.addEventListener('click', (e) => {
    const btn = e.target.closest('.choice-btn');
    if (!btn || !state.activeNearbyNpc) return;

    const idx = parseInt(btn.dataset.choice, 10);
    const npc = state.activeNearbyNpc;
    const choice = npc.choices[idx];

    // Apply choice effects
    if (choice.effect) {
      state.resources += choice.effect.res || 0;
      state.knowledge += choice.effect.know || 0;
      state.hope += choice.effect.hope || 0;
    }
    if (choice.rewardArtifact) {
      state.unlockedArtifacts.add(choice.rewardArtifact);
      playSound('pickup');
    }

    state.visitedLandmarks.add(npc.id);
    updateStatusUI();
    closeDialogue();

    // Check game completion (all 5 core landmarks visited)
    if (state.visitedLandmarks.size >= 5) {
      setTimeout(showCompletion, 800);
    }
  });

  function closeDialogue() {
    state.dialogueOpen = false;
    el.dialogueModal.style.display = 'none';
  }

  el.dialogueCloseBtn.onclick = closeDialogue;

  // Update Status UI & Quest Badge
  function updateStatusUI() {
    state.resources = Math.max(0, Math.min(100, state.resources));
    state.knowledge = Math.max(0, Math.min(100, state.knowledge));
    state.hope = Math.max(0, Math.min(100, state.hope));

    el.barRes.style.width = `${state.resources}%`;
    el.valRes.textContent = state.resources;

    el.barKnow.style.width = `${state.knowledge}%`;
    el.valKnow.textContent = state.knowledge;

    el.barHope.style.width = `${state.hope}%`;
    el.valHope.textContent = state.hope;

    el.artifactCount.textContent = state.unlockedArtifacts.size;

    // Quest badge tracker
    const visitedCount = state.visitedLandmarks.size;
    if (visitedCount === 0) {
      el.epochBadge.textContent = '当前使命：在尼罗河营地寻找向导摩西';
    } else if (visitedCount === 1) {
      el.epochBadge.textContent = '当前使命：穿越旷野绿洲，前往耶路撒冷圣山';
    } else if (visitedCount === 2) {
      el.epochBadge.textContent = '当前使命：与所罗门王共谋圣殿基业';
    } else if (visitedCount === 3) {
      el.epochBadge.textContent = '当前使命：探索中世纪欧洲城镇，探究土地法令与生存智慧';
    } else if (visitedCount === 4) {
      el.epochBadge.textContent = '当前使命：拜访哭墙遗址与卓别林和平讲坛';
    } else {
      el.epochBadge.textContent = '🎉 全图漫游探索已完成！';
    }
  }

  // Artifacts Modal
  el.btnArtifacts.onclick = () => {
    let gridHtml = '';
    ARTIFACTS.forEach((art) => {
      const unlocked = state.unlockedArtifacts.has(art.id);
      gridHtml += `
        <div class="artifact-card ${unlocked ? 'unlocked' : ''}">
          <div class="art-icon">${unlocked ? art.icon : '❓'}</div>
          <div class="art-name">${unlocked ? art.name : '未解锁见证物'}</div>
          <div class="art-desc">${unlocked ? art.desc : `探访【${art.landmark}】做出历史抉择解锁`}</div>
        </div>
      `;
    });
    el.artifactsGrid.innerHTML = gridHtml;
    el.artifactsModal.style.display = 'flex';
  };

  el.artifactsClose.onclick = () => {
    el.artifactsModal.style.display = 'none';
  };

  // Sound Toggle
  el.btnSound.onclick = () => {
    state.soundEnabled = !state.soundEnabled;
    el.btnSound.textContent = state.soundEnabled ? '🔊' : '🔇';
  };

  // Show Completion Summary Modal
  function showCompletion() {
    playSound('victory');
    el.summaryModal.style.display = 'flex';
  }

  el.btnReplay.onclick = () => {
    el.summaryModal.style.display = 'none';
    state.player.x = 100;
    state.player.y = 330;
    state.player.targetX = null;
    state.player.targetY = null;
    state.resources = 75;
    state.knowledge = 60;
    state.hope = 80;
    state.unlockedArtifacts.clear();
    state.visitedLandmarks.clear();
    mapPickups.forEach(p => p.collected = false);
    updateStatusUI();
  };

  // Start Engine
  updateStatusUI();
  gameLoop();
})();
