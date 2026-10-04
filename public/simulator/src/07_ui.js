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
  if (window.SandboxNetwork) {
    for (const id in window.SandboxNetwork.otherPlayers) {
      const a = window.SandboxNetwork.otherPlayers[id];
      if (!a || !a.mesh) continue;
      const mx = w / 2 + (a.mesh.position.x / 80) * (w * 0.68);
      const my = h / 2 + (a.mesh.position.z / 80) * (h * 0.68);
      ctx.fillStyle = "#3b82f6";
      ctx.beginPath();
      ctx.arc(mx, my, 3.2, 0, Math.PI * 2);
      ctx.fill();
      ctx.strokeStyle = "#ffffff";
      ctx.lineWidth = 1.0;
      ctx.stroke();
    }
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