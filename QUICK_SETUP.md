# Quick DSLR Setup for Windows

## Step 1: Install MSYS2

1. Download MSYS2 from: https://www.msys2.org/
2. Run the installer
3. Open **MSYS2 MINGW64** terminal (not MSYS2 MSYS)

## Step 2: Install libgphoto2

In the MSYS2 MINGW64 terminal, run:

```bash
pacman -Syu
pacman -S mingw-w64-x86_64-libgphoto2 mingw-w64-x86_64-pkg-config
```

## Step 3: Set Environment Variables

Add these to your system PATH:
- `C:\msys64\mingw64\bin`
- `C:\msys64\mingw64\lib`

Set PKG_CONFIG_PATH:
```powershell
$env:PKG_CONFIG_PATH = "C:\msys64\mingw64\lib\pkgconfig"
```

## Step 4: Install USB Driver (Zadig)

1. Download Zadig: https://zadig.akeo.ie/
2. Connect your Nikon camera via USB and turn it ON
3. Open Zadig
4. Go to Options → List All Devices
5. Select your camera from the dropdown
6. Select "libusb-win32" or "WinUSB" driver
7. Click "Replace Driver" or "Install Driver"

⚠️ **Warning**: This replaces the Nikon driver. To revert, uninstall in Device Manager.

## Step 5: Build and Run

```bash
pnpm install
pnpm run tauri dev
```

## Troubleshooting

### "pkg-config not found" error:
```powershell
$env:PKG_CONFIG_PATH = "C:\msys64\mingw64\lib\pkgconfig"
```

### "libgphoto2 not found" error:
Make sure `C:\msys64\mingw64\bin` is in your PATH

### Camera not detected:
- Check USB cable connection
- Ensure camera is turned ON
- Check camera is in PTP mode (not Mass Storage)
- Try different USB port
- Verify driver is installed with Zadig

### Check if setup is working:
Open MSYS2 terminal and run:
```bash
gphoto2 --auto-detect
```

If your camera is listed, setup is complete!

## Your Camera is Ready!

Once setup is complete, your app will:
- ✅ Detect DSLR cameras automatically
- ✅ Allow switching between webcam and DSLR modes  
- ✅ Capture high-res images directly from camera
- ✅ Download images instantly

The setup is one-time only. After this, just connect your camera and go!
