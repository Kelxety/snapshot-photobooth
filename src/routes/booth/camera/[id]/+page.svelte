<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { onMount, onDestroy } from 'svelte';
  import { getEvent, type Event } from '$lib/database';
  import { invoke } from '@tauri-apps/api/core';

  interface DSLRCamera {
    name: string;
    model: string;
    port: string;
  }

  interface CaptureResult {
    success: boolean;
    image_data?: string;
    file_path?: string;
    error?: string;
  }

  let eventId = $derived($page.params.id);
  let event = $state<Event | null>(null);
  let videoElement = $state<HTMLVideoElement>();
  let canvasElement = $state<HTMLCanvasElement>();
  let stream: MediaStream | null = null;
  let captureMode = $state<'photo' | 'gif' | 'boomerang' | 'video'>('photo');
  let isCapturing = $state(false);
  let countdown = $state(0);
  let capturedImage = $state<string | null>(null);
  let availableDevices = $state<MediaDeviceInfo[]>([]);
  let selectedDeviceId = $state<string>('');
  let showDeviceSelector = $state(false);
  let cameraLoading = $state(true);
  let cameraError = $state<string | null>(null);
  
  // DSLR support
  let useDSLR = $state(false);
  let dslrSupported = $state(false);
  let dslrCameras = $state<DSLRCamera[]>([]);
  let selectedDSLRIndex = $state(0);
  let showCameraTypeSelector = $state(false);

  onMount(async () => {
    try {
      if (!eventId) {
        throw new Error('Event ID is required');
      }
      const eventData = await getEvent(parseInt(eventId));
      event = eventData;
      
      // Check if DSLR support is available
      await checkDSLRSupport();
      
      // Try to initialize camera
      await initCamera();
    } catch (error) {
      console.error('Error loading event:', error);
    }
  });

  onDestroy(() => {
    stopCamera();
  });

  async function checkDSLRSupport() {
    try {
      dslrSupported = await invoke<boolean>('check_dslr_support');
      if (dslrSupported) {
        dslrCameras = await invoke<DSLRCamera[]>('list_dslr_cameras');
        console.log('DSLR cameras found:', dslrCameras);
        
        // If DSLR cameras are found, show camera type selector
        if (dslrCameras.length > 0) {
          showCameraTypeSelector = true;
        }
      }
    } catch (error) {
      // Silently fail - user can still use webcam mode
      console.log('DSLR detection not available, using webcam mode');
      dslrSupported = false;
    }
  }

  async function switchToDSLR() {
    useDSLR = true;
    showCameraTypeSelector = false;
    stopCamera(); // Stop webcam if running
    
    if (dslrCameras.length === 0) {
      alert('No DSLR cameras detected. Please connect your camera via USB and ensure it\'s turned on.');
    }
  }

  async function switchToWebcam() {
    useDSLR = false;
    showCameraTypeSelector = false;
    await initCamera();
  }

  async function loadAvailableDevices() {
    try {
      // Request permission first with a longer timeout
      const stream = await navigator.mediaDevices.getUserMedia({ video: true, audio: false });
      // Immediately close it after getting permission
      stream.getTracks().forEach(track => track.stop());
      
      // Small delay to ensure devices are ready
      await new Promise(resolve => setTimeout(resolve, 500));
      
      // Now enumerate devices (labels will be available after permission)
      const devices = await navigator.mediaDevices.enumerateDevices();
      const videoDevices = devices.filter(device => device.kind === 'videoinput');
      availableDevices = videoDevices;
      console.log('Available video devices:', videoDevices);
      
      return videoDevices;
    } catch (error) {
      console.error('Error enumerating devices:', error);
      return [];
    }
  }

  async function initCamera(deviceId?: string) {
    cameraLoading = true;
    cameraError = null;
    
    try {
      // Check if mediaDevices is available
      if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
        throw new Error('Camera API not supported in this browser/webview');
      }

      // Load devices if not already loaded
      if (availableDevices.length === 0) {
        const devices = await loadAvailableDevices();
        if (devices.length === 0) {
          showDeviceSelector = true;
          throw new Error('No camera devices found.\n\nFor DigiCamControl:\n1. Make sure DigiCamControl is running\n2. Click "Live View" button in DigiCamControl\n3. Wait a few seconds for camera to initialize\n4. Refresh this page\n\nFor Nikon Webcam Utility:\n1. Install and launch Nikon Webcam Utility\n2. Make sure your camera is connected and on\n3. Refresh this page');
        }
      }
      
      // Stop any existing stream
      stopCamera();

      // Use provided deviceId or selected one
      const targetDeviceId = deviceId || selectedDeviceId || availableDevices[0]?.deviceId;

      console.log('Attempting to start camera:', targetDeviceId);

      // Try with flexible constraints first
      const constraints: MediaStreamConstraints = {
        video: targetDeviceId ? {
          deviceId: { exact: targetDeviceId },
          width: { ideal: 1280 },
          height: { ideal: 720 },
          frameRate: { ideal: 30 }
        } : true,
        audio: false
      };

      stream = await navigator.mediaDevices.getUserMedia(constraints);
      console.log('Camera stream obtained:', stream);
      
      if (videoElement) {
        videoElement.srcObject = stream;
        // Wait for video to be ready and play
        await new Promise((resolve, reject) => {
          if (videoElement) {
            videoElement.onloadedmetadata = () => {
              console.log('Video metadata loaded');
              videoElement?.play()
                .then(() => {
                  console.log('Video playing');
                  resolve(true);
                })
                .catch(reject);
            };
            // Timeout after 5 seconds
            setTimeout(() => reject(new Error('Timeout loading video')), 5000);
          }
        });
      }

      // Update selected device
      if (!selectedDeviceId && stream) {
        const videoTrack = stream.getVideoTracks()[0];
        selectedDeviceId = videoTrack.getSettings().deviceId || '';
      }
      
      cameraLoading = false;
      console.log('Camera initialized successfully');
    } catch (error: any) {
      console.error('Error accessing camera:', error);
      cameraLoading = false;
      
      let errorMessage = 'Could not access camera. ';
      
      if (error.name === 'NotAllowedError' || error.name === 'PermissionDeniedError') {
        errorMessage += 'Camera permission was denied. Please allow camera access when prompted.';
      } else if (error.name === 'NotFoundError' || error.name === 'DevicesNotFoundError') {
        errorMessage += 'No camera found. Please connect a camera and try again.';
      } else if (error.name === 'NotReadableError' || error.name === 'TrackStartError') {
        errorMessage += 'Camera is already in use by another application.\n\nFor Nikon Webcam Utility:\n1. Close DigiCamControl if it\'s running\n2. Make sure Nikon Webcam Utility is running\n3. Try switching cameras using the camera selector';
      } else if (error.message && error.message.includes('Timeout')) {
        errorMessage += 'Timeout starting video source.\n\nFor DSLR cameras:\n1. Make sure Nikon Webcam Utility is running\n2. Close any other camera apps (DigiCamControl, etc.)\n3. Try unplugging and reconnecting your camera\n4. Restart Nikon Webcam Utility';
      } else {
        errorMessage += error.message || 'Unknown error occurred.';
      }
      
      cameraError = errorMessage;
      alert(errorMessage);
    }
  }

  function stopCamera() {
    if (stream) {
      stream.getTracks().forEach(track => track.stop());
      stream = null;
    }
  }

  async function switchDevice(deviceId: string) {
    selectedDeviceId = deviceId;
    showDeviceSelector = false;
    await initCamera(deviceId);
  }

  async function startCapture() {
    if (isCapturing) return;
    
    isCapturing = true;
    countdown = 3;

    const timer = setInterval(() => {
      countdown--;
      if (countdown <= 0) {
        clearInterval(timer);
        if (useDSLR) {
          captureDSLRPhoto();
        } else {
          capturePhoto();
        }
      }
    }, 1000);
  }

  function capturePhoto() {
    if (!videoElement || !canvasElement) return;

    const context = canvasElement.getContext('2d');
    if (!context) return;

    canvasElement.width = videoElement.videoWidth;
    canvasElement.height = videoElement.videoHeight;
    context.drawImage(videoElement, 0, 0);

    capturedImage = canvasElement.toDataURL('image/jpeg');
    isCapturing = false;
  }

  async function captureDSLRPhoto() {
    try {
      const result = await invoke<CaptureResult>('capture_from_dslr', {
        cameraIndex: selectedDSLRIndex
      });
      
      if (result.success && result.image_data) {
        capturedImage = result.image_data;
      } else {
        alert(result.error || 'Failed to capture image from DSLR');
      }
    } catch (error: any) {
      console.error('DSLR capture error:', error);
      alert(`Failed to capture from DSLR: ${error.message || error}`);
    } finally {
      isCapturing = false;
    }
  }

  function retake() {
    capturedImage = null;
    countdown = 0;
    // Restart camera if in webcam mode
    if (!useDSLR) {
      initCamera();
    }
  }

  function handleSaveAndShare() {
    // TODO: Implement save and share functionality
    console.log('Save and share photo');
    retake();
  }

  function handleBack() {
    stopCamera();
    goto('/booth');
  }
</script>

<div class="camera-screen">
  {#if !capturedImage}
    <!-- Camera View -->
    <div class="camera-container">
      {#if !useDSLR}
        {#if cameraLoading}
          <div class="camera-loading">
            <div class="loading-spinner"></div>
            <p>Starting camera...</p>
          </div>
        {/if}
        <video
          bind:this={videoElement}
          autoplay
          playsinline
          muted
          class="camera-feed"
          class:hidden={cameraLoading}
        ></video>
      {:else}
        <!-- DSLR Mode - Show placeholder -->
        <div class="dslr-placeholder">
          <div class="dslr-info">
            <h2>📷 DSLR Mode</h2>
            <p>{dslrCameras[selectedDSLRIndex]?.model || 'Camera Ready'}</p>
            <p class="hint">Press capture to take a photo</p>
          </div>
        </div>
      {/if}
      <canvas bind:this={canvasElement} class="hidden-canvas"></canvas>

      <!-- Camera Type Selector Modal -->
      {#if showCameraTypeSelector}
        <div class="device-selector-modal">
          <div class="modal-content">
            <h3>Select Camera Type</h3>
            <div class="device-list">
              {#if dslrCameras.length > 0}
                <button class="device-option" class:selected={useDSLR} onclick={switchToDSLR}>
                  📷 DSLR Camera
                  <span class="subtitle">{dslrCameras.length} camera(s) detected</span>
                </button>
              {/if}
              {#if availableDevices.length > 0}
                <button class="device-option" class:selected={!useDSLR} onclick={switchToWebcam}>
                  🎥 Webcam
                  <span class="subtitle">{availableDevices.length} device(s) available</span>
                </button>
              {/if}
            </div>
            <button class="close-btn" onclick={() => showCameraTypeSelector = false}>Close</button>
          </div>
        </div>
      {/if}

      {#if countdown > 0}
        <div class="countdown">{countdown}</div>
      {/if}

      <!-- Event Info Overlay -->
      <div class="event-overlay">
        <div class="event-badge">
          <span class="icon">🎉</span>
          {event?.name || 'Event'}
        </div>
      </div>

      <!-- Camera Controls Top Right -->
      <div class="top-right-controls">
        {#if dslrSupported && (dslrCameras.length > 0 || availableDevices.length > 0)}
          <button class="camera-switch-btn" onclick={() => showCameraTypeSelector = !showCameraTypeSelector}>
            {useDSLR ? '📷 DSLR' : '🎥 Webcam'}
          </button>
        {/if}

        {#if !useDSLR && availableDevices.length > 1}
          <button class="camera-switch-btn" onclick={() => showDeviceSelector = !showDeviceSelector}>
            🎥 Change Webcam
          </button>
        {/if}
      </div>

      <!-- Controls -->
      <div class="controls">
        <button class="back-btn" onclick={handleBack}>
          ← Back
        </button>

        <div class="capture-controls">
          <div class="mode-selector">
            <button
              class="mode-option"
              class:active={captureMode === 'photo'}
              onclick={() => captureMode = 'photo'}
            >
              📷 Photo
            </button>
            <button
              class="mode-option"
              class:active={captureMode === 'gif'}
              onclick={() => captureMode = 'gif'}
            >
              🎬 GIF
            </button>
            <button
              class="mode-option"
              class:active={captureMode === 'boomerang'}
              onclick={() => captureMode = 'boomerang'}
            >
              🔄 Boomerang
            </button>
            <button
              class="mode-option"
              class:active={captureMode === 'video'}
              onclick={() => captureMode = 'video'}
            >
              📹 Video
            </button>
          </div>

          <button
            class="capture-btn"
            onclick={startCapture}
            disabled={isCapturing}
          >
            {#if isCapturing}
              <span class="pulse"></span>
            {:else}
              <span class="capture-circle"></span>
            {/if}
          </button>
        </div>

        <div class="spacer"></div>
      </div>

      <!-- Device Selector Modal -->
      {#if showDeviceSelector && availableDevices.length > 0}
        <div class="device-selector-modal">
          <div class="modal-content">
            <h3>Select Camera</h3>
            <div class="device-list">
              {#each availableDevices as device}
                <button
                  class="device-option"
                  class:selected={device.deviceId === selectedDeviceId}
                  onclick={() => switchDevice(device.deviceId)}
                >
                  📷 {device.label || `Camera ${device.deviceId.slice(0, 8)}...`}
                  {#if device.deviceId === selectedDeviceId}
                    <span class="check">✓</span>
                  {/if}
                </button>
              {/each}
            </div>
            <button class="close-btn" onclick={() => showDeviceSelector = false}>Close</button>
          </div>
        </div>
      {/if}
    </div>
  {:else}
    <!-- Preview Screen -->
    <div class="preview-container">
      <img src={capturedImage} alt="Captured" class="preview-image" />

      <div class="preview-controls">
        <button class="retake-btn" onclick={retake}>
          🔄 Retake
        </button>
        <button class="save-btn" onclick={handleSaveAndShare}>
          ✓ Save & Share
        </button>
      </div>

      <button class="back-btn-preview" onclick={handleBack}>
        ← Back to Booth
      </button>
    </div>
  {/if}
</div>

<style>
.camera-screen {
  width: 100vw;
  height: 100vh;
  background-color: #000;
  overflow: hidden;
  position: relative;
}

.camera-container {
  width: 100%;
  height: 100%;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.camera-feed {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
  background: #000;
}

.camera-feed.hidden {
  opacity: 0;
  pointer-events: none;
}

.camera-loading {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  text-align: center;
  color: white;
  z-index: 10;
}

.loading-spinner {
  width: 60px;
  height: 60px;
  border: 4px solid rgba(255, 255, 255, 0.2);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 20px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.camera-loading p {
  font-size: 18px;
  opacity: 0.9;
}

.hidden-canvas {
  display: none;
}

.dslr-placeholder {
  width: 100%;
  height: 100%;
  background: linear-gradient(135deg, #1e3a8a 0%, #3b82f6 100%);
  display: flex;
  align-items: center;
  justify-content: center;
}

.dslr-info {
  text-align: center;
  color: white;
}

.dslr-info h2 {
  font-size: 48px;
  margin: 0 0 20px 0;
}

.dslr-info p {
  font-size: 24px;
  margin: 10px 0;
  opacity: 0.9;
}

.dslr-info .hint {
  font-size: 18px;
  opacity: 0.7;
  margin-top: 20px;
}


.countdown {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 120px;
  font-weight: bold;
  color: white;
  text-shadow: 0 0 20px rgba(0, 0, 0, 0.8);
  animation: pulse 0.5s ease-in-out;
}

@keyframes pulse {
  0%, 100% {
    transform: translate(-50%, -50%) scale(1);
    opacity: 1;
  }
  50% {
    transform: translate(-50%, -50%) scale(1.2);
    opacity: 0.8;
  }
}

.event-overlay {
  position: absolute;
  top: 20px;
  left: 20px;
}

.event-badge {
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(10px);
  color: white;
  padding: 12px 20px;
  border-radius: 12px;
  font-size: 18px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 10px;
}

.top-right-controls {
  position: absolute;
  top: 20px;
  right: 20px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  z-index: 10;
}

.controls {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 40px;
  background: linear-gradient(to top, rgba(0, 0, 0, 0.8), transparent);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.back-btn {
  background: rgba(255, 255, 255, 0.2);
  backdrop-filter: blur(10px);
  border: none;
  color: white;
  padding: 12px 24px;
  border-radius: 8px;
  font-size: 16px;
  cursor: pointer;
  transition: background 0.2s;
}

.back-btn:hover {
  background: rgba(255, 255, 255, 0.3);
}

.capture-controls {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
}

.mode-selector {
  display: flex;
  gap: 12px;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(10px);
  padding: 8px;
  border-radius: 12px;
}

.mode-option {
  background: transparent;
  border: 2px solid transparent;
  color: white;
  padding: 10px 20px;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.mode-option:hover {
  background: rgba(255, 255, 255, 0.1);
}

.mode-option.active {
  background: rgba(59, 130, 246, 0.5);
  border-color: #3b82f6;
}

.capture-btn {
  width: 80px;
  height: 80px;
  border-radius: 50%;
  border: 4px solid white;
  background: rgba(255, 255, 255, 0.2);
  backdrop-filter: blur(10px);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.2s;
}

.capture-btn:hover:not(:disabled) {
  transform: scale(1.1);
}

.capture-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.capture-circle {
  width: 60px;
  height: 60px;
  border-radius: 50%;
  background: white;
}

.pulse {
  width: 60px;
  height: 60px;
  border-radius: 50%;
  background: #ef4444;
  animation: pulse-animation 1s infinite;
}

@keyframes pulse-animation {
  0%, 100% {
    transform: scale(1);
    opacity: 1;
  }
  50% {
    transform: scale(1.2);
    opacity: 0.7;
  }
}

.spacer {
  width: 120px;
}

/* Preview Screen */
.preview-container {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: #000;
  position: relative;
}

.preview-image {
  max-width: 90%;
  max-height: 70vh;
  object-fit: contain;
  border-radius: 12px;
  box-shadow: 0 10px 50px rgba(0, 0, 0, 0.5);
}

.preview-controls {
  display: flex;
  gap: 20px;
  margin-top: 40px;
}

.retake-btn,
.save-btn {
  padding: 16px 40px;
  font-size: 18px;
  font-weight: 600;
  border: none;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.retake-btn {
  background: rgba(255, 255, 255, 0.2);
  backdrop-filter: blur(10px);
  color: white;
}

.retake-btn:hover {
  background: rgba(255, 255, 255, 0.3);
}

.save-btn {
  background: #3b82f6;
  color: white;
}

.save-btn:hover {
  background: #2563eb;
  transform: translateY(-2px);
}

.back-btn-preview {
  position: absolute;
  top: 20px;
  left: 20px;
  background: rgba(255, 255, 255, 0.2);
  backdrop-filter: blur(10px);
  border: none;
  color: white;
  padding: 12px 24px;
  border-radius: 8px;
  font-size: 16px;
  cursor: pointer;
  transition: background 0.2s;
}

.back-btn-preview:hover {
  background: rgba(255, 255, 255, 0.3);
}

/* Camera Switch Button */
.camera-switch-btn {
  background: rgba(255, 255, 255, 0.2);
  backdrop-filter: blur(10px);
  border: none;
  color: white;
  padding: 12px 24px;
  border-radius: 8px;
  font-size: 16px;
  cursor: pointer;
  transition: background 0.2s;
}

.camera-switch-btn:hover {
  background: rgba(255, 255, 255, 0.3);
}

/* Device Selector Modal */
.device-selector-modal {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(10px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: rgba(30, 30, 30, 0.95);
  border-radius: 16px;
  padding: 32px;
  max-width: 500px;
  width: 90%;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
}

.modal-content h3 {
  color: white;
  font-size: 24px;
  margin: 0 0 20px 0;
  text-align: center;
}

.device-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 20px;
}

.device-option {
  background: rgba(255, 255, 255, 0.1);
  border: 2px solid transparent;
  color: white;
  padding: 16px 20px;
  border-radius: 12px;
  font-size: 16px;
  cursor: pointer;
  transition: all 0.2s;
  text-align: left;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.device-option:hover {
  background: rgba(255, 255, 255, 0.2);
  border-color: rgba(59, 130, 246, 0.5);
}

.device-option.selected {
  background: rgba(59, 130, 246, 0.3);
  border-color: #3b82f6;
}

.device-option .check {
  color: #3b82f6;
  font-size: 20px;
  font-weight: bold;
}

.device-option .subtitle {
  display: block;
  font-size: 14px;
  opacity: 0.7;
  margin-top: 4px;
}


.close-btn {
  width: 100%;
  background: rgba(255, 255, 255, 0.1);
  border: none;
  color: white;
  padding: 14px;
  border-radius: 12px;
  font-size: 16px;
  cursor: pointer;
  transition: background 0.2s;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

@media (max-width: 768px) {
  .controls {
    padding: 20px;
  }

  .mode-selector {
    flex-wrap: wrap;
    justify-content: center;
  }

  .mode-option {
    padding: 8px 16px;
    font-size: 12px;
  }

  .capture-btn {
    width: 70px;
    height: 70px;
  }

  .capture-circle,
  .pulse {
    width: 50px;
    height: 50px;
  }

  .countdown {
    font-size: 80px;
  }

  .preview-controls {
    flex-direction: column;
    gap: 12px;
  }
}
</style>
