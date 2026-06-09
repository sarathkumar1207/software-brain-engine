#!/usr/bin/env node

const { main } = require('../lib/runner');

main(process.argv.slice(2)).catch((error) => {
  console.error(`sbe: ${error.message}`);
  process.exitCode = 1;
});
