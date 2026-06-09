const assert = require('node:assert/strict');
const path = require('node:path');
const test = require('node:test');
const { cacheRoot, detectPlatform } = require('../lib/platform');
const { parseChecksum } = require('../lib/download');
const { translateArgs } = require('../lib/runner');

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

test('translates explain to Rust analyze-change command', () => {
  assert.deepEqual(translateArgs(['explain', 'jwt to passport', '--json']), [
    'analyze-change',
    'jwt to passport',
    '--json'
  ]);
});
