// Windows Portable Devices (WPD) camera support
// This provides native Windows support for PTP camera DETECTION only

#[cfg(windows)]
use anyhow::Result;
#[cfg(windows)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WpdCameraInfo {
    pub model: String,
    pub device_id: String,
    pub manufacturer: String,
}

/// List all portable devices (cameras) by checking USB devices
#[cfg(windows)]
pub fn list_wpd_cameras() -> Result<Vec<WpdCameraInfo>> {
    use std::process::Command;
    
    // Use PowerShell to query USB devices for cameras
    let output = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg("Get-PnpDevice -PresentOnly | Where-Object { ($_.FriendlyName -like '*Nikon*' -or $_.FriendlyName -like '*Canon*' -or $_.FriendlyName -like '*DSC*' -or $_.FriendlyName -like '*DSLR*') -and $_.Status -eq 'OK' } | ForEach-Object { \"$($_.FriendlyName)|$($_.InstanceId)\" }")
        .output();
    
    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut cameras = Vec::new();
            
            for line in stdout.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 2 {
                    let name = parts[0].trim().to_string();
                    let device_id = parts[1].trim().to_string();
                    
                    // Extract manufacturer from name
                    let manufacturer = if name.to_lowercase().contains("nikon") {
                        "Nikon".to_string()
                    } else if name.to_lowercase().contains("canon") {
                        "Canon".to_string()
                    } else {
                        "Unknown".to_string()
                    };
                    
                    cameras.push(WpdCameraInfo {
                        model: name,
                        device_id,
                        manufacturer,
                    });
                }
            }
            
            Ok(cameras)
        }
        _ => {
            // If PowerShell fails, return empty list (not an error)
            // This allows the app to fall back to webcam mode gracefully
            Ok(Vec::new())
        }
    }
}

// Non-Windows stub
#[cfg(not(windows))]
pub fn list_wpd_cameras() -> Result<Vec<WpdCameraInfo>> {
    Ok(Vec::new())
}
