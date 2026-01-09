@echo off
echo ============================================
echo DSLR Camera Support - Prerequisites Check
echo ============================================
echo.

REM Check if MSYS2 is installed
if exist "C:\msys64\mingw64\bin\pkg-config.exe" (
    echo [OK] MSYS2 is installed
    goto :check_libs
) else (
    echo [MISSING] MSYS2 is not installed
    echo.
    echo Please follow these steps:
    echo.
    echo 1. Download MSYS2 from: https://www.msys2.org/
    echo 2. Run the installer
    echo 3. Open MSYS2 MINGW64 terminal and run:
    echo    pacman -Syu
    echo    pacman -S mingw-w64-x86_64-libgphoto2 mingw-w64-x86_64-pkg-config
    echo.
    echo 4. Then run this script again
    pause
    exit /b 1
)

:check_libs
REM Check if libgphoto2 is installed
if exist "C:\msys64\mingw64\lib\pkgconfig\libgphoto2.pc" (
    echo [OK] libgphoto2 is installed
) else (
    echo [MISSING] libgphoto2 is not installed
    echo.
    echo Open MSYS2 MINGW64 terminal and run:
    echo pacman -S mingw-w64-x86_64-libgphoto2
    pause
    exit /b 1
)

echo.
echo ============================================
echo All prerequisites are installed!
echo ============================================
echo.
echo Setting up environment for build...
echo.

REM Set environment variables
setx PKG_CONFIG_PATH "C:\msys64\mingw64\lib\pkgconfig"
set PKG_CONFIG_PATH=C:\msys64\mingw64\lib\pkgconfig
set PATH=C:\msys64\mingw64\bin;%PATH%

echo Environment configured!
echo.
echo You can now run: pnpm run tauri dev
echo.
pause
