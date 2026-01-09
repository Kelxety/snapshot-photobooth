# Test DSLR Connection Script
Write-Host "Testing DSLR Connection..." -ForegroundColor Cyan
Write-Host ""

# Add msys64 to PATH
$env:PATH = "C:\msys64\mingw64\bin;$env:PATH"

# Check if gphoto2 exists
if (-not (Get-Command gphoto2 -ErrorAction SilentlyContinue)) {
    Write-Host "gphoto2 is NOT installed!" -ForegroundColor Red
    Write-Host "Install MSYS2 and run: pacman -S mingw-w64-x86_64-libgphoto2" -ForegroundColor Yellow
    exit 1
}

Write-Host "gphoto2 is installed" -ForegroundColor Green
Write-Host ""

# Check for cameras
Write-Host "Detecting DSLR cameras..." -ForegroundColor Cyan
gphoto2 --auto-detect

Write-Host ""
Write-Host "If you see your camera listed above, you are ready to go!" -ForegroundColor Green
Write-Host "If NOT, you need to install the WinUSB driver using Zadig:" -ForegroundColor Yellow
Write-Host "  1. Download Zadig from https://zadig.akeo.ie/" -ForegroundColor Yellow
Write-Host "  2. Run as Administrator" -ForegroundColor Yellow
Write-Host "  3. Options -> List All Devices" -ForegroundColor Yellow
Write-Host "  4. Select your Nikon D5100" -ForegroundColor Yellow
Write-Host "  5. Choose WinUSB driver" -ForegroundColor Yellow
Write-Host "  6. Click Replace Driver" -ForegroundColor Yellow
