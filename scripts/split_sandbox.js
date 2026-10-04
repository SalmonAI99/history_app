const fs = require('fs');
const path = require('path');

const sandboxPath = path.join(__dirname, '../public/simulator/sandbox.js');
const srcDir = path.join(__dirname, '../public/simulator/src');

if (!fs.existsSync(srcDir)) {
  fs.mkdirSync(srcDir, { recursive: true });
}

let content = fs.readFileSync(sandboxPath, 'utf8');
const originalLength = content.split('\n').length;

content = content.replace(/^\(function \(\) \{\n  'use strict';\n/, '');
content = content.replace(/\n\}\)\(\);\s*$/, '');

let lines = content.split('\n');

const files = {
  '00_globals.js': [],
  '01_audio.js': [],
  '02_core.js': [],
  '03_world.js': [],
  '04_buildings.js': [],
  '05_agents.js': [],
  '06_controls.js': [],
  '07_ui.js': [],
  '08_main.js': []
};

const headerMapping = [
  { match: 'Web Audio Synthesizer', file: '01_audio.js' },
  { match: 'Toast Notification', file: '07_ui.js' },
  { match: 'Initialize Three.js Engine', file: '02_core.js' },
  { match: 'Lighting: Sunlight, Ambient, Rim', file: '03_world.js' },
  { match: 'Ground: Desert Sand', file: '03_world.js' },
  { match: 'Time of Day & Weather', file: '03_world.js' },
  { match: 'Mode Switcher', file: '03_world.js' },
  { match: 'CAMEL MESH GENERATOR', file: '04_buildings.js' },
  { match: 'SIMULATION 2: REALISTIC', file: '04_buildings.js' },
  { match: 'Knight 3D Character', file: '05_agents.js' },
  { match: 'Ancient Citizen Daily Schedule', file: '05_agents.js' },
  { match: 'Raycaster & Selection', file: '06_controls.js' },
  { match: 'Interactive Landmark Fast-Travel', file: '06_controls.js' },
  { match: 'Follow Camera Toggle', file: '06_controls.js' },
  { match: 'Third-Person Character Possession', file: '06_controls.js' },
  { match: 'Spawn Tool Toggle', file: '06_controls.js' },
  { match: 'Historical Events Trigger', file: '08_main.js' },
  { match: 'Minimap / Tactical Radar', file: '07_ui.js' },
  { match: 'Telemetry HUD & Inspector', file: '07_ui.js' },
  { match: 'Interactive Building Action', file: '04_buildings.js' },
  { match: 'External Controls API', file: '08_main.js' }
];

let currentFile = '00_globals.js';
let currentLineIdx = 0;

while (currentLineIdx < lines.length) {
  let line = lines[currentLineIdx];
  
  // Check if this line is a section header (e.g., // Web Audio Synthesizer)
  if (line.startsWith('  // ') || line.startsWith('// ')) {
    for (const mapping of headerMapping) {
      if (line.includes(mapping.match)) {
        currentFile = mapping.file;
        break;
      }
    }
  }
  
  // Dedent by 2 spaces since we removed the IIFE
  if (line.startsWith('  ')) {
    line = line.substring(2);
  }
  
  files[currentFile].push(line);
  currentLineIdx++;
}

let totalLines = 0;
for (const [filename, linesArray] of Object.entries(files)) {
  const outPath = path.join(srcDir, filename);
  fs.writeFileSync(outPath, linesArray.join('\n'), 'utf8');
  console.log(`Wrote ${filename} (${linesArray.length} lines)`);
  totalLines += linesArray.length;
}

console.log(`\nOriginal lines: ${originalLength}`);
console.log(`Total split lines: ${totalLines}`);
