#!/usr/bin/env node

const { ensureBinary } = require('../lib/download');

if (process.env.SBE_SKIP_DOWNLOAD === '1' || process.env.SBE_SKIP_DOWNLOAD === 'true') {
  console.log('sbe: skipping native binary download because SBE_SKIP_DOWNLOAD is set');
  process.exit(0);
}

ensureBinary()
  .then((binary) => {
    console.log(`sbe: native binary ready at ${binary}`);
  })
  .catch((error) => {
    console.warn(`sbe: native binary download skipped: ${error.message}`);
    console.warn('sbe: the CLI will retry the download on first run');
  });
