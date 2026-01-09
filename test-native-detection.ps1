# Test Native Camera Detection
Write-Host "Testing native Windows camera detection..." -ForegroundColor Cyan
Write-Host ""

# Test PowerShell detection directly
Write-Host "1. Testing PowerShell USB device query..." -ForegroundColor Yellow
$devices = Get-PnpDevice -PresentOnly | Where-Object { ($_.FriendlyName -like '*Nikon*' -or $_.FriendlyName -like '*Canon*' -or $_.FriendlyName -like '*DSC*') -and $_.Status -eq 'OK' }

if ($devices) {
    Write-Host "Found camera devices:" -ForegroundColor Green
    $devices | ForEach-Object {
        Write-Host "  - $($_.FriendlyName) [$($_.Class)]" -ForegroundColor White
        Write-Host "    ID: $($_.InstanceId)" -ForegroundColor Gray
    }
} else {
    Write-Host "No camera devices found via PowerShell" -ForegroundColor Red
}

Write-Host ""
Write-Host "2. Now test your app with: pnpm tauri dev" -ForegroundColor Cyan
Write-Host "   Your D5100 should appear in the camera list!" -ForegroundColor Green
