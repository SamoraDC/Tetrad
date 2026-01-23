#!/usr/bin/env node

/**
 * Tetrad MCP - CLI wrapper
 *
 * This script executes the Tetrad binary with the provided arguments.
 */

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const BINARY_NAME = process.platform === 'win32' ? 'tetrad.exe' : 'tetrad';
const binaryPath = path.join(__dirname, BINARY_NAME);

// Check if binary exists
if (!fs.existsSync(binaryPath)) {
  console.error('Error: Tetrad binary not found.');
  console.error('');
  console.error('The binary should have been installed during npm install.');
  console.error('Try reinstalling: npm install -g @samoradc/tetrad');
  console.error('');
  console.error('Or install manually:');
  console.error('  cargo install tetrad');
  console.error('  sudo cp ~/.cargo/bin/tetrad /usr/local/bin/');
  process.exit(1);
}

// Pass all arguments to the binary
const args = process.argv.slice(2);

const child = spawn(binaryPath, args, {
  stdio: 'inherit',
  env: process.env,
});

child.on('error', (err) => {
  console.error(`Failed to start Tetrad: ${err.message}`);
  process.exit(1);
});

child.on('exit', (code, signal) => {
  if (signal) {
    process.exit(1);
  }
  process.exit(code || 0);
});
