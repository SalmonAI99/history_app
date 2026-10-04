// ========================================================================
// 3D Ancient Middle East & Canaan Multi-Agent Simulation Sandbox
// Interactive World Model with Dynamic Landmarks, Weather/Day-Night Cycle,
// Follow Camera, Historical Events & Living Tactical Minimap Radar
// ========================================================================


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