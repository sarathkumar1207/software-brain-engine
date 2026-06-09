const os = require('node:os');
const path = require('node:path');

function detectPlatform(platform = process.platform, arch = process.arch) {
  if (platform === 'linux' && arch === 'x64') {
    return {
      target: 'linux-x64',
      assetName: 'sbe-core-linux-x64',
      executableName: 'sbe-core'
    };
  }

  if (platform === 'darwin' && arch === 'x64') {
    return {
      target: 'macos-x64',
      assetName: 'sbe-core-macos-x64',
      executableName: 'sbe-core'
    };
  }

  if (platform === 'darwin' && arch === 'arm64') {
    return {
      target: 'macos-arm64',
      assetName: 'sbe-core-macos-arm64',
      executableName: 'sbe-core'
    };
  }

  if (platform === 'win32' && arch === 'x64') {
    return {
      target: 'windows-x64',
      assetName: 'sbe-core-windows-x64.exe',
      executableName: 'sbe-core.exe'
    };
  }

  throw new Error(`unsupported platform ${platform}-${arch}. Supported: linux-x64, macos-x64, macos-arm64, windows-x64.`);
}

function cacheRoot(version, homeDir = os.homedir()) {
  return path.join(homeDir, '.sbe', 'bin', version);
}

module.exports = {
  cacheRoot,
  detectPlatform
};
