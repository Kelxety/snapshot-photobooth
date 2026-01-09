# DSLR Camera Setup Guide

**Note:** For most users, we recommend using **Nikon Webcam Utility** or **OBS Virtual Camera** instead of this advanced setup. See [DISTRIBUTION.md](DISTRIBUTION.md) for the recommended approach.

This guide is for developers and advanced users who want **native USB control** via gphoto2.

---

## Recommended Approach (Easy)

Instead of the complex gphoto2 setup, users should:

1. **Install Nikon Webcam Utility** (official from Nikon)
2. **Or use OBS Studio Virtual Camera**
3. Your camera will appear as a standard webcam - no configuration needed!

✅ This works immediately with the app
✅ No driver conflicts
✅ Official Nikon software
✅ Simple for end users

---

## Advanced: Native gphoto2 Support

This section is only needed if you want to build with native gphoto2 support for direct camera control.

**⚠️ This is optional and not recommended for distribution!**

## Windows Setup

### 1. Install libgphoto2 for Windows

You have two options:

#### Option A: Using MSYS2 (Recommended)

1. **Install MSYS2**: Download from https://www.msys2.org/
2. **Open MSYS2 MINGW64 terminal** and run:
   ```bash
   pacman -S mingw-w64-x86_64-libgphoto2
   ```
3. **Add to PATH**: Add `C:\msys64\mingw64\bin` to your system PATH
4. **Copy DLLs**: Copy these files from `C:\msys64\mingw64\bin` to your project's `src-tauri\target\debug`:
   - `libgphoto2-6.dll`
   - `libgphoto2_port-12.dll`
   - All dependency DLLs (libusb, libxml2, etc.)

#### Option B: Pre-built Binaries

1. Download libgphoto2 Windows binaries from: http://www.gphoto.org/proj/libgphoto2/support.php
2. Extract and add to system PATH
3. Copy DLL files to your project directory

### 2. Install USB Drivers

For Windows to communicate with your camera:

1. **Download Zadig**: https://zadig.akeo.ie/
2. **Connect your Nikon camera** via USB and turn it on
3. **Open Zadig** and select your camera from the device list
4. **Install libusb-win32 or WinUSB driver** for your camera

⚠️ **Important**: This will replace the default Nikon driver. Your camera won't work with Nikon software while using libusb driver. To revert, uninstall the driver in Device Manager and let Windows reinstall the default one.

### 3. Build and Run

```bash
pnpm install
pnpm tauri dev
```

## Supported Cameras

The app supports most Nikon DSLR and mirrorless cameras:
- Nikon D series (D3500, D5600, D7500, D850, etc.)
- Nikon Z series (Z5, Z6, Z7, Z8, Z9, etc.)
- Many Canon, Sony, and other brand cameras

Check full compatibility: http://www.gphoto.org/proj/libgphoto2/support.php

## Usage

1. **Connect your camera** via USB cable
2. **Turn on the camera**
3. **Open the app** and navigate to the camera page
4. **Click "Switch"** button to select between Webcam and DSLR modes
5. **Select DSLR Camera** if multiple cameras are connected
6. **Press Capture** - The app will trigger your camera shutter and download the image automatically!

## Troubleshooting

### "No DSLR cameras detected"
- Ensure camera is turned ON
- Check USB cable connection
- Verify libusb driver is installed (use Zadig)
- Try different USB port
- Check camera is in PTP/PC mode (not MTP/Mass Storage)

### "Failed to capture from DSLR"
- Camera might be in wrong mode (set to P, A, S, or M mode)
- Memory card might be full
- Battery might be low
- Try manually taking a photo first to ensure camera is working

### "gphoto2 not available"
- libgphoto2 DLLs not found in PATH
- Copy DLLs to `src-tauri\target\debug` folder
- Restart the app after copying DLLs

### Driver conflicts
If you need to use both this app and Nikon software:
1. Use Device Manager to switch drivers
2. Or keep both: Use WinUSB for this app, and reinstall Nikon driver when needed
3. Or use Nikon Webcam Utility and select webcam mode in the app

## Alternative: Webcam Mode

If you don't want to deal with drivers, you can still use:
- **Nikon Webcam Utility**: Makes your camera appear as a webcam
- **OBS Virtual Camera**: Stream your camera through OBS
- **Built-in webcam**: Any standard webcam device

The app supports both native DSLR control and standard webcam APIs!

## Development Notes

The DSLR functionality is implemented in:
- `src-tauri/src/camera.rs` - Rust module for gphoto2 integration
- `src-tauri/src/lib.rs` - Tauri commands
- `src/routes/booth/camera/[id]/+page.svelte` - Frontend UI

gphoto2 provides direct camera control including:
- ✅ Trigger shutter remotely
- ✅ Download images instantly  
- ✅ High-resolution capture
- ⚠️ Live view preview (not yet implemented)
- ⚠️ Camera settings control (not yet implemented)
