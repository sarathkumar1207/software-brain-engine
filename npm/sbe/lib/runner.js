const { spawn } = require('node:child_process');
const { ensureBinary } = require('./download');

const HELP = `Software Brain Engine

Usage:
  sbe scan [path] [--json]
  sbe graph <symbol> [path] [--json]
  sbe impact <symbol> [path] [--json]
  sbe explain <flow> [path] [--json]

Examples:
  sbe scan .
  sbe graph createUser --json
  sbe impact saveUser
  sbe explain "jwt to passport" --json

The npm package downloads the native Rust engine on first run and caches it under ~/.sbe/bin.
`;

async function main(args) {
  if (args.includes('--help') || args.includes('-h')) {
    process.stdout.write(HELP);
    return;
  }

  const binary = await ensureBinary();
  const translatedArgs = translateArgs(args);
  const code = await runBinary(binary, translatedArgs);
  process.exitCode = code;
}

function translateArgs(args) {
  if (args[0] === 'explain') {
    return ['analyze-change', ...args.slice(1)];
  }
  return args;
}

function runBinary(binary, args) {
  return new Promise((resolve, reject) => {
    const child = spawn(binary, args, {
      stdio: 'inherit',
      windowsHide: true
    });

    child.on('error', reject);
    child.on('close', (code) => resolve(code || 0));
  });
}

module.exports = {
  HELP,
  main,
  runBinary,
  translateArgs
};
