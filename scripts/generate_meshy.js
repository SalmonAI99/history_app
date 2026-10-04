const https = require('https');
const fs = require('fs');
const path = require('path');

const MESHY_API_KEY = process.env.MESHY_API_KEY;

if (!MESHY_API_KEY) {
  console.error("Error: MESHY_API_KEY environment variable is not set.");
  console.error("Please set it using: export MESHY_API_KEY='your_api_key_here'");
  process.exit(1);
}

const args = process.argv.slice(2);
if (args.length < 2) {
  console.error("Usage: node generate_meshy.js <output_filename_without_ext> \"<prompt>\"");
  process.exit(1);
}

const [filename, prompt] = args;
const outDir = path.join(__dirname, '..', 'public', 'models');

if (!fs.existsSync(outDir)) {
  fs.mkdirSync(outDir, { recursive: true });
}

// 1. Initiate Generation
function startGeneration() {
  return new Promise((resolve, reject) => {
    const data = JSON.stringify({
      mode: "preview",
      prompt: prompt,
      art_style: "realistic",
      negative_prompt: "low quality, low resolution, ugly, blurry"
    });

    const req = https.request({
      hostname: 'api.meshy.ai',
      path: '/openapi/v2/text-to-3d',
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${MESHY_API_KEY}`,
        'Content-Type': 'application/json',
        'Content-Length': data.length
      }
    }, (res) => {
      let body = '';
      res.on('data', chunk => body += chunk);
      res.on('end', () => {
        if (res.statusCode >= 200 && res.statusCode < 300) {
          const json = JSON.parse(body);
          resolve(json.result); // Task ID
        } else {
          reject(`API Error ${res.statusCode}: ${body}`);
        }
      });
    });

    req.on('error', reject);
    req.write(data);
    req.end();
  });
}

// 2. Poll for Completion
function pollTask(taskId) {
  return new Promise((resolve, reject) => {
    const check = () => {
      const req = https.request({
        hostname: 'api.meshy.ai',
        path: `/openapi/v2/text-to-3d/${taskId}`,
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${MESHY_API_KEY}`
        }
      }, (res) => {
        let body = '';
        res.on('data', chunk => body += chunk);
        res.on('end', () => {
          if (res.statusCode >= 200 && res.statusCode < 300) {
            const json = JSON.parse(body);
            console.log(`Task Status: ${json.status} (Progress: ${json.progress}%)`);
            
            if (json.status === 'SUCCEEDED') {
              resolve(json.model_urls.glb);
            } else if (json.status === 'FAILED' || json.status === 'EXPIRED') {
              reject(`Task failed with status: ${json.status}`);
            } else {
              setTimeout(check, 5000); // Check again in 5 seconds
            }
          } else {
            reject(`API Error ${res.statusCode}: ${body}`);
          }
        });
      });
      req.on('error', reject);
      req.end();
    };
    check();
  });
}

// 3. Download GLB
function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https.get(url, (res) => {
      if (res.statusCode !== 200) {
        reject(`Failed to download: ${res.statusCode}`);
        return;
      }
      res.pipe(file);
      file.on('finish', () => {
        file.close(resolve);
      });
    }).on('error', (err) => {
      fs.unlink(dest, () => reject(err));
    });
  });
}

async function main() {
  try {
    console.log(`Starting generation for: ${filename}`);
    console.log(`Prompt: ${prompt}`);
    
    const taskId = await startGeneration();
    console.log(`Task ID: ${taskId}`);
    
    console.log(`Polling for completion...`);
    const glbUrl = await pollTask(taskId);
    console.log(`Model generated successfully! URL: ${glbUrl}`);
    
    const dest = path.join(outDir, `${filename}.glb`);
    console.log(`Downloading to ${dest}...`);
    await downloadFile(glbUrl, dest);
    
    console.log(`Done! Downloaded ${filename}.glb`);
  } catch (err) {
    console.error("Error:", err);
    process.exit(1);
  }
}

main();
