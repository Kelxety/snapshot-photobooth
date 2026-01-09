#!/usr/bin/env node

const https = require('https');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const GPHOTO2_VERSION = '2.5.31';
const RESOURCES_DIR = path.join(__dirname, 'resources');
const DLL_DIR = path.join(RESOURCES_DIR, 'gphoto2');

// Create directories
if (!fs.existsSync(RESOURCES_DIR)) {
  fs.mkdirSync(RESOURCES_DIR, { recursive: true });
}
if (!fs.existsSync(DLL_DIR)) {
  fs.mkdirSync(DLL_DIR, { recursive: true });
}

console.log('📦 Bundling gphoto2 dependencies...');
console.log(`📁 Target directory: ${DLL_DIR}`);

// For Windows, we'll use vcpkg to get pre-built binaries
async function downloadGphoto2Windows() {
  console.log('🪟 Detecting Windows - Setting up vcpkg...');
  
  try {
    // Check if vcpkg is installed
    try {
      execSync('vcpkg version', { stdio: 'ignore' });
      console.log('✅ vcpkg found');
    } catch {
      console.log('⚠️  vcpkg not found. Installing...');
      console.log('Please install vcpkg manually from: https://vcpkg.io/');
      console.log('Or run: git clone https://github.com/microsoft/vcpkg && cd vcpkg && bootstrap-vcpkg.bat');
      console.log('');
      console.log('⚠️  Skipping gphoto2 bundling. App will use system-installed gphoto2 if available.');
      console.log('✨ The app will work in webcam-only mode without gphoto2.');
      return;
    }

    // Install libgphoto2 via vcpkg
    console.log('📥 Installing libgphoto2 via vcpkg...');
    execSync('vcpkg install libgphoto2:x64-windows', { stdio: 'inherit' });
    
    // Copy DLLs from vcpkg
    const vcpkgRoot = execSync('vcpkg integrate install', { encoding: 'utf8' })
      .match(/([A-Z]:\\.*vcpkg)/)?.[1];
    
    if (vcpkgRoot) {
      const vcpkgBinDir = path.join(vcpkgRoot, 'installed', 'x64-windows', 'bin');
      const dllsToCopy = [
        'gphoto2.dll',
        'gphoto2_port.dll',
        'libusb-1.0.dll',
        'libxml2.dll'
      ];

      console.log('📋 Copying DLLs...');
      for (const dll of dllsToCopy) {
        const src = path.join(vcpkgBinDir, dll);
        const dest = path.join(DLL_DIR, dll);
        
        if (fs.existsSync(src)) {
          fs.copyFileSync(src, dest);
          console.log(`  ✅ ${dll}`);
        } else {
          console.log(`  ⚠️  ${dll} not found`);
        }
      }
    }
    
    console.log('✅ gphoto2 dependencies bundled successfully!');
  } catch (error) {
    console.error('❌ Error bundling gphoto2:', error.message);
    console.log('⚠️  The app will work in webcam-only mode.');
  }
}

// For simpler approach: Create placeholder and instructions
function createInstructions() {
  const readmePath = path.join(DLL_DIR, 'README.txt');
  const content = `GPHOTO2 DLL FILES
=================

To enable DSLR camera support, place the following DLL files in this directory:

Required Files:
- gphoto2.dll (or libgphoto2-6.dll)
- gphoto2_port.dll (or libgphoto2_port-12.dll)
- libusb-1.0.dll
- libxml2.dll

Where to get them:
1. Install MSYS2: https://www.msys2.org/
2. Run: pacman -S mingw-w64-x86_64-libgphoto2
3. Copy DLLs from: C:\\msys64\\mingw64\\bin

The app will automatically detect and use these DLLs if present.
If not present, the app will work in webcam-only mode.
`;
  
  fs.writeFileSync(readmePath, content);
  console.log('📝 Created instructions in resources/gphoto2/README.txt');
}

// Main
if (process.platform === 'win32') {
  createInstructions();
  console.log('');
  console.log('ℹ️  Note: gphoto2 is optional. The app will work without it using webcam mode.');
  console.log('ℹ️  For DSLR support, users can install Nikon Webcam Utility instead.');
  console.log('');
  console.log('To bundle gphoto2 DLLs automatically:');
  console.log('1. Install MSYS2 from https://www.msys2.org/');
  console.log('2. Run: pacman -S mingw-w64-x86_64-libgphoto2');
  console.log('3. Copy DLLs from C:\\msys64\\mingw64\\bin to src-tauri/resources/gphoto2/');
  console.log('');
} else {
  console.log('ℹ️  Platform not Windows - skipping Windows-specific bundling');
  console.log('ℹ️  Linux/Mac users typically have gphoto2 in system packages');
}

console.log('✨ Setup complete!');
