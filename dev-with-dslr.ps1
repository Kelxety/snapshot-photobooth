# Development script with DSLR support
# This script sets up the environment and runs the Tauri dev server

Write-Host "Setting up DSLR support environment..." -ForegroundColor Green

# Set environment variables for build
$env:PKG_CONFIG_PATH = "C:\msys64\mingw64\lib\pkgconfig"
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"

# Add MSYS2 mingw64 bin to PATH for runtime DLL loading
$env:PATH = "C:\msys64\mingw64\bin;$env:PATH"

Write-Host "Environment configured:" -ForegroundColor Cyan
Write-Host "  PKG_CONFIG_PATH: $env:PKG_CONFIG_PATH" -ForegroundColor Gray
Write-Host "  LIBCLANG_PATH: $env:LIBCLANG_PATH" -ForegroundColor Gray
Write-Host "  MSYS2 in PATH: C:\msys64\mingw64\bin" -ForegroundColor Gray

# Check if gphoto2 DLLs are accessible
$gphoto2Dll = "C:\msys64\mingw64\bin\libgphoto2-6.dll"
if (Test-Path $gphoto2Dll) {
    Write-Host ""
    Write-Host "Check mark libgphoto2-6.dll found" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "X libgphoto2-6.dll NOT found at $gphoto2Dll" -ForegroundColor Red
    Write-Host "  Please install libgphoto2 via MSYS2:" -ForegroundColor Yellow
    Write-Host "  pacman -S mingw-w64-x86_64-libgphoto2" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "Starting Tauri dev server..." -ForegroundColor Green
Write-Host "The app will have direct USB access to DSLR cameras via gphoto2." -ForegroundColor Cyan
Write-Host ""

# Run the dev server
pnpm run tauri dev
