// ========================================================================
// 3D Multi-Agent Networking (WebSocket Sync)
// ========================================================================

const SandboxNetwork = {
  ws: null,
  otherPlayers: {},
  myId: null,
  
  init: function() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = protocol + '//' + window.location.host + '/ws/simulator';
    this.ws = new WebSocket(wsUrl);
    
    this.ws.onopen = () => {
      console.log("3D Sandbox Connected to Multiplayer Server!");
    };
    
    this.ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        this.handleServerMessage(msg);
      } catch (e) {}
    };
  },
  
  handleServerMessage: function(msg) {
    if (msg.Welcome) {
        this.myId = msg.Welcome.id;
    } else if (msg.StateSync) {
        msg.StateSync.players.forEach(p => this.updateNetworkPlayer(p));
    } else if (msg.PlayerJoined) {
        this.updateNetworkPlayer(msg.PlayerJoined.player);
        if (window.showFloatToast && msg.PlayerJoined.player.id !== this.myId) {
            window.showFloatToast("🌐 另一位探索者进入了沙盘！");
        }
    } else if (msg.PlayerMoved3D) {
        this.updateNetworkPlayer(msg.PlayerMoved3D);
    } else if (msg.PlayerLeft) {
        this.removeNetworkPlayer(msg.PlayerLeft.id);
        if (window.showFloatToast) window.showFloatToast("👋 一位探索者离开了沙盘。");
    }
  },
  
  updateNetworkPlayer: function(p) {
    if (!p.id || !window.simGroup) return;
    if (p.id === this.myId) return;

    let agent = this.otherPlayers[p.id];
    
    if (!agent) {
        if (p.model === 'knight') {
            if (window.createKnightAgent) {
                agent = window.createKnightAgent(p.x, p.z, 'Network Player', false);
                if (agent) {
                    this.otherPlayers[p.id] = agent;
                }
            }
        }
    }
    
    if (agent && agent.mesh) {
        agent.targetX = p.x;
        agent.targetY = p.y;
        agent.targetZ = p.z;
        agent.targetRy = p.ry;
        
        if (agent.setAnimation && p.anim) {
            if (agent.currentAnim !== p.anim) {
                agent.setAnimation(p.anim);
                agent.currentAnim = p.anim;
            }
        }
    }
  },
  
  removeNetworkPlayer: function(id) {
    const agent = this.otherPlayers[id];
    if (agent && agent.mesh) {
        if (window.simGroup) window.simGroup.remove(agent.mesh);
        // Clean up from active agents array if needed
        if (window.agents) {
            const idx = window.agents.indexOf(agent);
            if (idx !== -1) window.agents.splice(idx, 1);
        }
    }
    delete this.otherPlayers[id];
  },
  
  sendMove: function(x, y, z, ry, anim, model) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        this.ws.send(JSON.stringify({
            Move3D: { x, y, z, ry, anim, model }
        }));
    }
  },

  updateInterpolation: function(dt) {
    for (let id in this.otherPlayers) {
        let agent = this.otherPlayers[id];
        if (!agent || !agent.mesh) continue;
        
        if (agent.targetX !== undefined) {
            // Smooth lerp for positions
            agent.mesh.position.x += (agent.targetX - agent.mesh.position.x) * dt * 10.0;
            agent.mesh.position.y += (agent.targetY - agent.mesh.position.y) * dt * 10.0;
            agent.mesh.position.z += (agent.targetZ - agent.mesh.position.z) * dt * 10.0;
            
            // Smooth spherical interpolation for rotation? Or just simple angle lerp
            let diff = agent.targetRy - agent.mesh.rotation.y;
            // Normalize diff to -PI to PI
            diff = Math.atan2(Math.sin(diff), Math.cos(diff));
            agent.mesh.rotation.y += diff * dt * 10.0;
        }
    }
  }
};

window.SandboxNetwork = SandboxNetwork;
