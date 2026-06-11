const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
const { cacheRoot, detectPlatform } = require('../lib/platform');
const { isTrustedDownloadUrl, parseChecksum, verifyChecksum } = require('../lib/download');
const { runBinary, translateArgs } = require('../lib/runner');

test('detects supported platforms', () => {
  assert.equal(detectPlatform('linux', 'x64').assetName, 'sbe-core-linux-x64');
  assert.equal(detectPlatform('darwin', 'x64').assetName, 'sbe-core-macos-x64');
  assert.equal(detectPlatform('darwin', 'arm64').assetName, 'sbe-core-macos-arm64');
  assert.equal(detectPlatform('win32', 'x64').assetName, 'sbe-core-windows-x64.exe');
});

test('rejects unsupported platforms', () => {
  assert.throws(() => detectPlatform('linux', 'arm64'), /unsupported platform/);
});

test('builds cache root under home directory', () => {
  assert.equal(cacheRoot('1.2.3', '/home/dev'), path.join('/home/dev', '.sbe', 'bin', '1.2.3'));
});

test('parses checksums', () => {
  const checksums = 'abc123  sbe-core-linux-x64\nffff  other';
  assert.equal(parseChecksum('sbe-core-linux-x64', checksums), 'abc123');
});

test('allows GitHub release asset redirect host', () => {
  assert.equal(isTrustedDownloadUrl('https://github.com/owner/repo/releases/download/v1.0.0/tool.exe'), true);
  assert.equal(isTrustedDownloadUrl('https://objects.githubusercontent.com/github-production-release-asset/file'), true);
  assert.equal(isTrustedDownloadUrl('https://release-assets.githubusercontent.com/github-production-release-asset/file'), true);
  assert.equal(isTrustedDownloadUrl('https://example.com/tool.exe'), false);
  assert.equal(isTrustedDownloadUrl('http://github.com/owner/repo/releases/download/v1.0.0/tool.exe'), false);
});

test('verifies checksum for downloaded temp file', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'sbe-npm-'));
  const file = path.join(dir, 'sbe-core.tmp');
  fs.writeFileSync(file, 'native-binary');
  try {
    verifyChecksum(file, 'sbe-core-windows-x64.exe', [
      '9ec4c62cbabe2558224228ab3254a4e20e24cdf57a2cf3be50f37111723595e5  sbe-core-windows-x64.exe'
    ].join('\n'));
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test('translates explain to Rust analyze-change command', () => {
  assert.deepEqual(translateArgs(['explain', 'jwt to passport', '--json']), [
    'analyze-change',
    'jwt to passport',
    '--json'
  ]);
});

test('runs configured native binary wrapper', async () => {
  const fixture = process.platform === 'win32' ? 'mock-sbe.cmd' : 'mock-sbe.sh';
  const code = await runBinary(path.join(__dirname, 'fixtures', fixture), ['scan', '.']);
  assert.equal(code, 0);
});
