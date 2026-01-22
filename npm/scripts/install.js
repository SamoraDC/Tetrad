#!/usr/bin/env node

/**
 * Tetrad MCP - Binary installer
 *
 * Downloads the correct pre-compiled binary for the current platform
 * from GitHub Releases.
 */

const https = require('https');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const zlib = require('zlib');

const PACKAGE_VERSION = require('../package.json').version;
const GITHUB_REPO = 'SamoraDC/Tetrad';
const BINARY_NAME = process.platform === 'win32' ? 'tetrad.exe' : 'tetrad';

// Map Node.js platform/arch to Rust target triples
const PLATFORM_MAP = {
  'darwin-x64': 'x86_64-apple-darwin',
  'darwin-arm64': 'aarch64-apple-darwin',
  'linux-x64': 'x86_64-unknown-linux-gnu',
  'linux-arm64': 'aarch64-unknown-linux-gnu',
  'win32-x64': 'x86_64-pc-windows-msvc',
};

function getPlatformKey() {
  return `${process.platform}-${process.arch}`;
}

function getTargetTriple() {
  const key = getPlatformKey();
  const triple = PLATFORM_MAP[key];

  if (!triple) {
    console.error(`Unsupported platform: ${key}`);
    console.error(`Supported platforms: ${Object.keys(PLATFORM_MAP).join(', ')}`);
    process.exit(1);
  }

  return triple;
}

function getDownloadUrl(version, target) {
  const ext = process.platform === 'win32' ? 'zip' : 'tar.gz';
  return `https://github.com/${GITHUB_REPO}/releases/download/v${version}/tetrad-${target}.${ext}`;
}

function downloadFile(url) {
  return new Promise((resolve, reject) => {
    const followRedirects = (url, redirectCount = 0) => {
      if (redirectCount > 5) {
        reject(new Error('Too many redirects'));
        return;
      }

      https.get(url, (response) => {
        if (response.statusCode >= 300 && response.statusCode < 400 && response.headers.location) {
          followRedirects(response.headers.location, redirectCount + 1);
          return;
        }

        if (response.statusCode !== 200) {
          reject(new Error(`Failed to download: HTTP ${response.statusCode}`));
          return;
        }

        const chunks = [];
        response.on('data', (chunk) => chunks.push(chunk));
        response.on('end', () => resolve(Buffer.concat(chunks)));
        response.on('error', reject);
      }).on('error', reject);
    };

    followRedirects(url);
  });
}

async function extractTarGz(buffer, destDir) {
  const tar = require('tar');
  const tmpFile = path.join(destDir, 'temp.tar.gz');

  // Write buffer to temp file
  fs.writeFileSync(tmpFile, buffer);

  // Extract using tar
  await tar.extract({
    file: tmpFile,
    cwd: destDir,
  });

  // Clean up temp file
  fs.unlinkSync(tmpFile);
}

async function extractZip(buffer, destDir) {
  const AdmZip = require('adm-zip');
  const zip = new AdmZip(buffer);
  zip.extractAllTo(destDir, true);
}

async function installBinaryFromCargo() {
  console.log('Attempting to install via cargo...');
  try {
    execSync('cargo install tetrad', { stdio: 'inherit' });

    // Find the installed binary
    const cargoHome = process.env.CARGO_HOME || path.join(require('os').homedir(), '.cargo');
    const cargoBin = path.join(cargoHome, 'bin', BINARY_NAME);

    if (fs.existsSync(cargoBin)) {
      const destPath = path.join(__dirname, '..', 'bin', BINARY_NAME);
      fs.copyFileSync(cargoBin, destPath);
      fs.chmodSync(destPath, 0o755);
      console.log('Successfully installed tetrad via cargo');
      return true;
    }
  } catch (err) {
    console.log('Cargo installation failed, will try GitHub releases...');
  }
  return false;
}

async function main() {
  const binDir = path.join(__dirname, '..', 'bin');
  const binaryPath = path.join(binDir, BINARY_NAME);

  // Skip if binary already exists
  if (fs.existsSync(binaryPath)) {
    console.log('Tetrad binary already installed');
    return;
  }

  // Ensure bin directory exists
  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }

  const target = getTargetTriple();
  const url = getDownloadUrl(PACKAGE_VERSION, target);

  console.log(`Installing Tetrad v${PACKAGE_VERSION} for ${target}...`);
  console.log(`Downloading from: ${url}`);

  try {
    const buffer = await downloadFile(url);

    if (process.platform === 'win32') {
      await extractZip(buffer, binDir);
    } else {
      await extractTarGz(buffer, binDir);
    }

    // Make binary executable
    if (process.platform !== 'win32') {
      fs.chmodSync(binaryPath, 0o755);
    }

    console.log('Tetrad installed successfully!');

  } catch (err) {
    console.error(`Failed to download pre-built binary: ${err.message}`);
    console.log('');
    console.log('Trying fallback installation via cargo...');

    const cargoInstalled = await installBinaryFromCargo();

    if (!cargoInstalled) {
      console.error('');
      console.error('Could not install Tetrad automatically.');
      console.error('');
      console.error('Please install manually:');
      console.error('  1. Install Rust: https://rustup.rs/');
      console.error('  2. Run: cargo install tetrad');
      console.error('  3. Copy binary: sudo cp ~/.cargo/bin/tetrad /usr/local/bin/');
      console.error('');
      process.exit(1);
    }
  }
}

main().catch((err) => {
  console.error('Installation failed:', err);
  process.exit(1);
});
