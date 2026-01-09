# Snap Booth - Easy Distribution Guide

## For App Distribution (No Dependencies Required!)

Your app now works **out of the box** for most users without any setup!

### How it works:

1. **Webcam Mode (Default)**: Works immediately with any webcam
2. **DSLR Mode (Optional)**: Automatically enabled if user has:
   - Nikon Webcam Utility installed, OR
   - OBS Virtual Camera running, OR
   - Any camera configured as a webcam device

### Building for Distribution:

```bash
# Build the installer (no dependencies needed!)
pnpm run tauri:build
```

This creates an installer in `src-tauri/target/release/bundle/` that users can install with one click.

## User Experience:

### Simple Setup (No DSLR):
1. Install SnapBooth.exe
2. Run the app
3. Use with any webcam - works immediately!

### DSLR Setup (Optional):
Users who want DSLR support have 3 easy options:

#### Option 1: Nikon Webcam Utility (Easiest)
1. Download from Nikon website
2. Install and connect camera
3. Camera appears as webcam in your app automatically!

#### Option 2: OBS Virtual Camera
1. Install OBS Studio
2. Add camera as video source
3. Start Virtual Camera
4. Select "OBS Virtual Camera" in your app

#### Option 3: Advanced (Direct USB Control)
For power users who want native gphoto2 support:
- Follow DSLR_SETUP.md for advanced configuration
- This is optional and only for specific use cases

## Advantages of This Approach:

✅ **Zero setup for basic users** - just install and run
✅ **No dependency hell** - works on any Windows PC
✅ **Flexible** - supports both webcams and DSLRs
✅ **Simple distribution** - one installer for everyone
✅ **Professional** - users can choose Nikon Webcam Utility (official)
✅ **No driver conflicts** - uses standard webcam APIs

## Distribution Checklist:

- [x] App works without any dependencies
- [x] Webcam support built-in
- [x] DSLR support via webcam utilities (Nikon/OBS)
- [x] Single installer for all users
- [x] No manual DLL copying needed
- [x] No driver installation required

## What Gets Installed:

When users run your installer:
- ✅ SnapBooth application
- ✅ All necessary runtime files
- ✅ App works immediately with webcams
- ❌ NO external dependencies
- ❌ NO gphoto2 setup needed
- ❌ NO driver installation

## Recommended User Instructions:

**Include this in your documentation:**

```markdown
# SnapBooth Installation

1. Download and run SnapBooth-Installer.exe
2. Follow the installation wizard
3. Launch SnapBooth

**For Webcam Users:**
- Just connect your webcam and go!

**For DSLR Users (Nikon):**
- Install Nikon Webcam Utility from Nikon's website
- Connect your camera via USB
- Turn on the camera
- Your DSLR will appear as a webcam in SnapBooth!
```

This approach gives users a **professional, polished experience** without technical setup!
