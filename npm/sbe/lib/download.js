const crypto = require('node:crypto');
const fs = require('node:fs');
const fsp = require('node:fs/promises');
const https = require('node:https');
const os = require('node:os');
const path = require('node:path');
const { cacheRoot, detectPlatform } = require('./platform');

const OWNER = 'sarathkumar1207';
const REPO = 'software-brain-engine';

function packageVersion() {
  return require('../package.json').version;
}

function releaseBaseUrl(version = packageVersion()) {
  return process.env.SBE_BINARY_BASE_URL || `https://github.com/${OWNER}/${REPO}/releases/download/v${version}`;
}

function binaryPath(version = packageVersion(), platformInfo = detectPlatform()) {
  return path.join(cacheRoot(version), platformInfo.executableName);
}

async function ensureBinary(options = {}) {
  if (process.env.SBE_CORE_PATH) {
    return process.env.SBE_CORE_PATH;
  }

  const version = options.version || packageVersion();
  const platformInfo = options.platformInfo || detectPlatform();
  const destination = binaryPath(version, platformInfo);

  if (fs.existsSync(destination)) {
    return destination;
  }

  await fsp.mkdir(path.dirname(destination), { recursive: true });
  await downloadBinary({ version, platformInfo, destination });
  return destination;
}

async function downloadBinary({ version, platformInfo, destination }) {
  const baseUrl = releaseBaseUrl(version);
  const binaryUrl = `${baseUrl}/${platformInfo.assetName}`;
  const checksumUrl = `${baseUrl}/checksums.txt`;
  const tempPath = `${destination}.${process.pid}.${Date.now()}.tmp`;

  await downloadToFile(binaryUrl, tempPath);

  try {
    const checksums = await fetchText(checksumUrl);
    verifyChecksum(tempPath, platformInfo.assetName, checksums);
  } catch (error) {
    await fsp.rm(tempPath, { force: true });
    throw new Error(`downloaded ${platformInfo.assetName}, but checksum validation failed: ${error.message}`);
  }

  if (os.platform() !== 'win32') {
    await fsp.chmod(tempPath, 0o755);
  }

  await fsp.rename(tempPath, destination);
}

function verifyChecksum(filePath, assetName, checksumsText) {
  const expected = parseChecksum(assetName, checksumsText);
  const actual = crypto.createHash('sha256').update(fs.readFileSync(filePath)).digest('hex');
  if (actual !== expected) {
    throw new Error(`sha256 mismatch for ${assetName}. expected ${expected}, got ${actual}`);
  }
}

function parseChecksum(assetName, checksumsText) {
  for (const line of checksumsText.split(/\r?\n/)) {
    const parts = line.trim().split(/\s+/);
    if (parts.length >= 2 && parts[1].replace(/^\*/, '') === assetName) {
      return parts[0].toLowerCase();
    }
  }
  throw new Error(`missing checksum for ${assetName}`);
}

function isTrustedDownloadUrl(url) {
  try {
    const { hostname, protocol } = new URL(url);
    return (
      protocol === 'https:' &&
      [
        'github.com',
        'objects.githubusercontent.com',
        'release-assets.githubusercontent.com'
      ].includes(hostname)
    );
  } catch {
    return false;
  }
}

function downloadToFile(url, destination, redirects = 0) {
  return new Promise((resolve, reject) => {
    if (!isTrustedDownloadUrl(url)) {
      reject(new Error(`refusing non-GitHub download URL: ${url}`));
      return;
    }

    const request = https.get(url, { headers: { 'User-Agent': 'sbe-npm-cli' } }, (response) => {
      if ([301, 302, 303, 307, 308].includes(response.statusCode)) {
        if (redirects >= 5) {
          response.resume();
          reject(new Error('too many redirects'));
          return;
        }
        response.resume();
        downloadToFile(response.headers.location, destination, redirects + 1).then(resolve, reject);
        return;
      }

      if (response.statusCode !== 200) {
        response.resume();
        reject(new Error(`download failed with HTTP ${response.statusCode}: ${url}`));
        return;
      }

      const file = fs.createWriteStream(destination, { mode: 0o755 });
      response.pipe(file);
      file.on('finish', () => {
        file.close((error) => {
          if (error) {
            reject(error);
            return;
          }
          resolve();
        });
      });
      file.on('error', (error) => {
        response.destroy();
        fs.rm(destination, { force: true }, () => {});
        reject(error);
      });
    });

    request.setTimeout(30000, () => {
      request.destroy(new Error('download timed out'));
    });
    request.on('error', (error) => {
      fs.rm(destination, { force: true }, () => {});
      reject(error);
    });
  });
}

function fetchText(url, redirects = 0) {
  return new Promise((resolve, reject) => {
    if (!isTrustedDownloadUrl(url)) {
      reject(new Error(`refusing non-GitHub checksum URL: ${url}`));
      return;
    }

    https.get(url, { headers: { 'User-Agent': 'sbe-npm-cli' } }, (response) => {
      if ([301, 302, 303, 307, 308].includes(response.statusCode)) {
        if (redirects >= 5) {
          reject(new Error('too many redirects'));
          return;
        }
        fetchText(response.headers.location, redirects + 1).then(resolve, reject);
        return;
      }

      if (response.statusCode !== 200) {
        reject(new Error(`checksum download failed with HTTP ${response.statusCode}`));
        return;
      }

      let body = '';
      response.setEncoding('utf8');
      response.on('data', (chunk) => {
        body += chunk;
      });
      response.on('end', () => resolve(body));
    }).on('error', reject);
  });
}

module.exports = {
  binaryPath,
  ensureBinary,
  isTrustedDownloadUrl,
  parseChecksum,
  releaseBaseUrl,
  verifyChecksum
};
