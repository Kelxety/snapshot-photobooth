use anyhow::{anyhow, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CameraInfo {
    pub model: String,
    pub port: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CaptureResult {
    pub image_data: String,  // Base64 encoded
    pub camera_model: String,
}

/// Check if any DSLR support is available (gphoto2 or WPD)
pub fn check_dslr_support_available() -> bool {
    // Try gphoto2 first
    if check_gphoto2_available() {
        return true;
    }
    
    // Fall back to Windows WPD
    #[cfg(windows)]
    {
        if let Ok(cameras) = crate::camera_wpd::list_wpd_cameras() {
            return !cameras.is_empty();
        }
    }
    
    false
}

/// Check if gphoto2 CLI is available
pub fn check_gphoto2_available() -> bool {
    #[cfg(target_os = "windows")]
    {
        // On Windows, ensure msys64 is in PATH
        let msys_bin = "C:\\msys64\\mingw64\\bin";
        if let Ok(current_path) = std::env::var("PATH") {
            if !current_path.contains(msys_bin) {
                let new_path = format!("{};{}", msys_bin, current_path);
                std::env::set_var("PATH", new_path);
            }
        }
    }
    
    Command::new("gphoto2")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// List available DSLR cameras connected via USB (auto-detect, no configuration needed)
pub fn list_cameras() -> Result<Vec<CameraInfo>> {
    // Try gphoto2 first
    match list_gphoto2_cameras() {
        Ok(cameras) if !cameras.is_empty() => return Ok(cameras),
        _ => {}
    }
    
    // Fall back to Windows WPD
    #[cfg(windows)]
    {
        return list_wpd_cameras_as_camerainfo();
    }
    
    #[cfg(not(windows))]
    {
        Err(anyhow!("No cameras detected. Please connect your camera via USB and ensure it's powered on."))
    }
}

#[cfg(windows)]
fn list_wpd_cameras_as_camerainfo() -> Result<Vec<CameraInfo>> {
    let wpd_cameras = crate::camera_wpd::list_wpd_cameras()?;
    
    if wpd_cameras.is_empty() {
        return Err(anyhow!("No cameras detected. Please connect your camera via USB and ensure it's powered on."));
    }
    
    Ok(wpd_cameras
        .into_iter()
        .map(|c| CameraInfo {
            model: format!("{} ({})", c.model, c.manufacturer),
            port: format!("wpd:{}", c.device_id),
        })
        .collect())
}

/// List cameras using gphoto2
fn list_gphoto2_cameras() -> Result<Vec<CameraInfo>> {
    #[cfg(target_os = "windows")]
    {
        // Ensure msys64 is in PATH
        let msys_bin = "C:\\msys64\\mingw64\\bin";
        if let Ok(current_path) = std::env::var("PATH") {
            if !current_path.contains(msys_bin) {
                let new_path = format!("{};{}", msys_bin, current_path);
                std::env::set_var("PATH", new_path);
            }
        }
    }
    
    let output = Command::new("gphoto2")
        .arg("--auto-detect")
        .arg("--debug-logfile=nul")  // Suppress debug output
        .output()
        .map_err(|e| anyhow!("Failed to execute gphoto2: {}. Make sure your camera is connected via USB and powered on.", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Camera detection failed. Please ensure:\n1. Camera is connected via USB\n2. Camera is powered on\n3. Camera is in PC/PTP mode (not mass storage)\n\nError: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut cameras = Vec::new();

    // Parse output like:
    // Model                          Port
    // ----------------------------------------------------------
    // Canon EOS 5D Mark III          usb:001,005
    for line in stdout.lines().skip(2) {  // Skip header lines
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Split by multiple spaces to get model and port
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            // Everything except the last part is the model name
            let model = parts[..parts.len()-1].join(" ");
            let port = parts[parts.len()-1].to_string();
            
            // Only include USB cameras
            if port.starts_with("usb:") {
                cameras.push(CameraInfo { model, port });
            }
        }
    }

    if cameras.is_empty() {
        return Err(anyhow!("No cameras detected. Please connect your camera via USB and ensure it's powered on."));
    }

    Ok(cameras)
}

/// Auto-configure camera for direct USB capture (no manual settings needed)
fn auto_configure_camera(port: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let msys_bin = "C:\\msys64\\mingw64\\bin";
        if let Ok(current_path) = std::env::var("PATH") {
            if !current_path.contains(msys_bin) {
                let new_path = format!("{};{}", msys_bin, current_path);
                std::env::set_var("PATH", new_path);
            }
        }
    }

    // Set camera to PC mode automatically if possible
    let _ = Command::new("gphoto2")
        .arg("--port")
        .arg(port)
        .arg("--set-config")
        .arg("capturetarget=0")  // Save to SDRAM (faster, no card needed)
        .output();

    // Set image format to JPEG for compatibility
    let _ = Command::new("gphoto2")
        .arg("--port")
        .arg(port)
        .arg("--set-config")
        .arg("imageformat=0")  // JPEG
        .output();

    Ok(())
}

/// Capture an image from a DSLR camera (fully automatic, no camera configuration needed)
pub fn capture_image(camera_index: usize) -> Result<CaptureResult> {
    #[cfg(target_os = "windows")]
    {
        // Ensure msys64 is in PATH
        let msys_bin = "C:\\msys64\\mingw64\\bin";
        if let Ok(current_path) = std::env::var("PATH") {
            if !current_path.contains(msys_bin) {
                let new_path = format!("{};{}", msys_bin, current_path);
                std::env::set_var("PATH", new_path);
            }
        }
    }
    
    let cameras = list_cameras()?;
    let camera = cameras
        .get(camera_index)
        .ok_or_else(|| anyhow!("Camera index {} not found", camera_index))?;

    // Check if this is a WPD camera (not supported for direct capture)
    if camera.port.starts_with("wpd:") {
        return Err(anyhow!(
            "Direct DSLR capture is not available.\n\n\
             Your camera is detected but needs to be used in webcam mode:\n\
             1. Install Nikon Webcam Utility or DigiCamControl\n\
             2. Launch the utility software\n\
             3. Your camera will appear as a webcam\n\
             4. Use webcam mode in this app instead\n\n\
             Webcam mode provides the same high-quality images!"
        ));
    }

    // Auto-configure camera for direct USB capture
    let _ = auto_configure_camera(&camera.port);

    // Create a temporary file to store the captured image
    let temp_dir = std::env::temp_dir();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let temp_file = temp_dir.join(format!("dslr_capture_{}.jpg", timestamp));

    // Capture and download the image with force and keep options
    let output = Command::new("gphoto2")
        .arg("--port")
        .arg(&camera.port)
        .arg("--capture-image-and-download")
        .arg("--filename")
        .arg(&temp_file)
        .arg("--force-overwrite")  // Overwrite if exists
        .arg("--skip-existing")    // Skip if already downloaded
        .output()
        .map_err(|e| anyhow!("Failed to capture image: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Check if it's just a warning, not an error
        if temp_file.exists() {
            // Image was captured successfully despite error message
        } else {
            return Err(anyhow!("Capture failed: {} {}", stderr, stdout));
        }
    }

    // Read the captured image
    let image_bytes = std::fs::read(&temp_file)
        .map_err(|e| anyhow!("Failed to read captured image: {}", e))?;

    // Clean up temporary file
    let _ = std::fs::remove_file(&temp_file);

    // Encode to base64
    let image_data = base64::engine::general_purpose::STANDARD.encode(&image_bytes);

    Ok(CaptureResult {
        image_data,
        camera_model: camera.model.clone(),
    })
}

/// Capture an image and save it to a specific path (fully automatic)
pub fn capture_and_save(camera_index: usize, save_path: std::path::PathBuf) -> Result<CaptureResult> {
    #[cfg(target_os = "windows")]
    {
        // Ensure msys64 is in PATH
        let msys_bin = "C:\\msys64\\mingw64\\bin";
        if let Ok(current_path) = std::env::var("PATH") {
            if !current_path.contains(msys_bin) {
                let new_path = format!("{};{}", msys_bin, current_path);
                std::env::set_var("PATH", new_path);
            }
        }
    }
    
    let cameras = list_cameras()?;
    let camera = cameras
        .get(camera_index)
        .ok_or_else(|| anyhow!("Camera index {} not found", camera_index))?;

    // Check if this is a WPD camera (not supported for direct capture)
    if camera.port.starts_with("wpd:") {
        return Err(anyhow!(
            "Direct DSLR capture is not available.\n\n\
             Your camera is detected but needs to be used in webcam mode:\n\
             1. Install Nikon Webcam Utility or DigiCamControl\n\
             2. Launch the utility software\n\
             3. Your camera will appear as a webcam\n\
             4. Use webcam mode in this app instead\n\n\
             Webcam mode provides the same high-quality images!"
        ));
    }

    // Auto-configure camera for direct USB capture
    let _ = auto_configure_camera(&camera.port);

    // Capture and download directly to the desired path
    let output = Command::new("gphoto2")
        .arg("--port")
        .arg(&camera.port)
        .arg("--capture-image-and-download")
        .arg("--filename")
        .arg(&save_path)
        .arg("--force-overwrite")
        .output()
        .map_err(|e| anyhow!("Failed to capture image: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Check if file was created despite error message
        if !save_path.exists() {
            return Err(anyhow!("Capture failed: {} {}", stderr, stdout));
        }
    }

    // Read the captured image for base64 encoding
    let image_bytes = std::fs::read(&save_path)
        .map_err(|e| anyhow!("Failed to read captured image: {}", e))?;

    // Encode to base64
    let image_data = base64::engine::general_purpose::STANDARD.encode(&image_bytes);

    Ok(CaptureResult {
        image_data,
        camera_model: camera.model.clone(),
    })
}
