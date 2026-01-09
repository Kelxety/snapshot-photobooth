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
  let selectedFilter = $state<string>('none');
  
  // Multi-photo session - uses custom template
  let sessionPhotos = $state<string[]>([]);
  let currentPhotoIndex = $state(0);
  const MAX_PHOTOS = 10;
  let finalComposite = $state<string | null>(null);
  
  // Photo boxes from template
  interface PhotoBox {
    id: number;
    x: number;
    y: number;
    width: number;
    height: number;
    color: string;
    number: number;
  }
  
  let photoBoxes = $state<PhotoBox[]>([]);
  
  // Get required photos count from photo boxes
  let requiredPhotos = $derived(() => {
    return photoBoxes.length || 1;
  });
  
  // DSLR support
  let useDSLR = $state(false);
  let dslrSupported = $state(false);
  let dslrCameras = $state<DSLRCamera[]>([]);
  let selectedDSLRIndex = $state(0);
  let showCameraTypeSelector = $state(false);

  const filters = [
    { name: 'None', value: 'none', filter: '' },
    { name: 'Vintage', value: 'vintage', filter: 'sepia(50%) contrast(120%) brightness(90%)' },
    { name: 'Black & White', value: 'bw', filter: 'grayscale(100%)' },
    { name: 'Cool', value: 'cool', filter: 'brightness(110%) contrast(110%) hue-rotate(180deg)' },
    { name: 'Warm', value: 'warm', filter: 'brightness(110%) saturate(150%) hue-rotate(-15deg)' },
    { name: 'Dramatic', value: 'dramatic', filter: 'contrast(150%) brightness(90%) saturate(120%)' },
    { name: 'Fade', value: 'fade', filter: 'brightness(110%) contrast(90%) saturate(80%)' },
    { name: 'Vivid', value: 'vivid', filter: 'saturate(180%) contrast(110%)' }
  ];

  const layouts = [
    { name: '1 Photo', value: 'layout-1', boxes: 1, description: 'Single large photo' },
    { name: '2 Photos', value: 'layout-2', boxes: 2, description: 'Two photos side by side' },
    { name: '3 Photos', value: 'layout-3', boxes: 3, description: 'Three photos in a row' },
    { name: '4 Photos', value: 'layout-4', boxes: 4, description: 'Four photos in a grid' },
    { name: '5 Photos', value: 'layout-5', boxes: 5, description: 'Five photos collage' }
  ];

  onMount(async () => {
    try {
      if (!eventId) {
        throw new Error('Event ID is required');
      }
      const eventData = await getEvent(parseInt(eventId));
      event = eventData;
      
      // Parse photo boxes from event
      if (event?.photo_boxes) {
        try {
          photoBoxes = JSON.parse(event.photo_boxes);
          console.log('Loaded photo boxes:', photoBoxes);
        } catch (e) {
          console.error('Failed to parse photo boxes:', e);
          photoBoxes = [];
        }
      }
      
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

    const photoData = canvasElement.toDataURL('image/jpeg');
    sessionPhotos.push(photoData);
    sessionPhotos = sessionPhotos; // Trigger reactivity
    
    capturedImage = photoData;
    isCapturing = false;
  }

  async function captureDSLRPhoto() {
    try {
      const result = await invoke<CaptureResult>('capture_from_dslr', {
        cameraIndex: selectedDSLRIndex
      });
      
      if (result.success && result.image_data) {
        const photoData = result.image_data;
        sessionPhotos.push(photoData);
        sessionPhotos = sessionPhotos; // Trigger reactivity
        
        capturedImage = photoData;
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
    selectedFilter = 'none';
    // Continue session - don't restart camera
  }
  
  function takeAnother() {
    if (sessionPhotos.length >= MAX_PHOTOS) {
      alert(`Maximum ${MAX_PHOTOS} photos reached!`);
      return;
    }
    retake();
  }
  
  function endSession() {
    if (sessionPhotos.length === 0) {
      alert('Please take at least one photo!');
      return;
    }
    const required = requiredPhotos();
    if (sessionPhotos.length < required) {
      alert(`Please take at least ${required} photo(s) for the selected layout!`);
      return;
    }
    // Generate composite immediately with event's layout
    generateComposite();
  }
  
  function generateComposite() {
    if (!canvasElement) {
      console.error('Canvas element not found');
      alert('Error: Canvas not ready');
      return;
    }
    
    if (!event?.template_image) {
      alert('No template image found for this event');
      return;
    }
    
    if (photoBoxes.length === 0) {
      alert('No photo boxes defined in template');
      return;
    }
    
    console.log('Generating composite with template');
    console.log('Session photos:', sessionPhotos.length);
    console.log('Photo boxes:', photoBoxes.length);
    
    const ctx = canvasElement.getContext('2d');
    if (!ctx) {
      console.error('Canvas context not available');
      alert('Error: Cannot create canvas context');
      return;
    }
    
    // Set canvas size based on paper size
    const paperSizes: Record<string, { width: number; height: number }> = {
      '4R': { width: 1200, height: 1800 },
      '5R': { width: 1500, height: 2100 }
    };
    
    const size = paperSizes[event.paper_size || '4R'];
    canvasElement.width = size.width;
    canvasElement.height = size.height;
    
    // White background
    ctx.fillStyle = '#ffffff';
    ctx.fillRect(0, 0, size.width, size.height);
    
    const loadImage = (src: string): Promise<HTMLImageElement> => {
      return new Promise((resolve, reject) => {
        const img = new Image();
        img.onload = () => resolve(img);
        img.onerror = reject;
        img.src = src;
      });
    };
    
    // Load all captured photos
    Promise.all(sessionPhotos.map(loadImage))
      .then(photoImages => {
        console.log('All photos loaded, placing into boxes...');
        
        // Sort boxes by number to ensure correct placement
        const sortedBoxes = [...photoBoxes].sort((a, b) => a.number - b.number);
        
        // Scale factor from 1000px base to actual canvas size
        const scaleX = size.width / 1000;
        const scaleY = size.height / 1000;
        
        // Draw each photo into its box
        sortedBoxes.forEach((box, index) => {
          if (index >= photoImages.length) return;
          
          const photoImg = photoImages[index];
          
          // Scale box coordinates
          const boxX = box.x * scaleX;
          const boxY = box.y * scaleY;
          const boxW = box.width * scaleX;
          const boxH = box.height * scaleY;
          
          // Calculate scale to fit photo in box (cover mode)
          const scaleToFit = Math.max(boxW / photoImg.width, boxH / photoImg.height);
          const scaledW = photoImg.width * scaleToFit;
          const scaledH = photoImg.height * scaleToFit;
          
          // Center the photo in the box
          const offsetX = (boxW - scaledW) / 2;
          const offsetY = (boxH - scaledH) / 2;
          
          // Clip to box bounds and draw
          ctx.save();
          ctx.beginPath();
          ctx.rect(boxX, boxY, boxW, boxH);
          ctx.clip();
          ctx.drawImage(photoImg, boxX + offsetX, boxY + offsetY, scaledW, scaledH);
          ctx.restore();
        });
        
        // Load and overlay template image on top
        if (!event?.template_image) {
          throw new Error('Template image not found');
        }
        return loadImage(event.template_image);
      })
      .then(templateImg => {
        console.log('Template image loaded, overlaying...');
        
        // Draw template on top (PNG with transparency)
        ctx.drawImage(templateImg, 0, 0, size.width, size.height);
        
        if (!canvasElement) {
          console.error('Canvas element lost after drawing');
          return;
        }
        
        console.log('Converting canvas to data URL...');
        finalComposite = canvasElement.toDataURL('image/jpeg', 0.95);
        console.log('Composite generated successfully!');
      })
      .catch(error => {
        console.error('Error generating composite:', error);
        alert('Error creating layout: ' + error.message);
      });
  }
  
  function resetSession() {
    sessionPhotos = [];
    capturedImage = null;
    finalComposite = null;
    selectedFilter = 'none';
    initCamera();
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
        
        <!-- Session Progress -->
        <div class="session-progress">
          <span class="photo-count">{sessionPhotos.length}/{requiredPhotos()} photos</span>
          <span class="layout-info">{event?.paper_size || '4R'} Template</span>
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
      <img 
        src={capturedImage} 
        alt="Captured" 
        class="preview-image" 
        style="filter: {filters.find(f => f.value === selectedFilter)?.filter || ''}"
      />
      
      <!-- Session Photos Gallery -->
      {#if sessionPhotos.length > 0}
        <div class="session-gallery">
          <h3>📸 Session Photos ({sessionPhotos.length}/{requiredPhotos()})</h3>
          <div class="gallery-grid">
            {#each sessionPhotos as photo, index}
              <div class="gallery-item" class:current={index === sessionPhotos.length - 1}>
                <img src={photo} alt="Photo {index + 1}" />
                <span class="photo-number">{index + 1}</span>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Filter Selector -->
      <div class="filter-selector">
        <h3>✨ Filters</h3>
        <div class="filter-options">
          {#each filters as filter}
            <button
              class="filter-option"
              class:active={selectedFilter === filter.value}
              onclick={() => selectedFilter = filter.value}
            >
              <div class="filter-preview" style="filter: {filter.filter}">
                <img src={capturedImage} alt={filter.name} />
              </div>
              <span class="filter-name">{filter.name}</span>
            </button>
          {/each}
        </div>
      </div>

      <div class="preview-controls">
        {#if sessionPhotos.length < requiredPhotos()}
          <button class="take-another-btn" onclick={takeAnother}>
            📷 Take Another ({sessionPhotos.length}/{requiredPhotos()})
          </button>
        {:else}
          <button class="end-session-btn" onclick={endSession}>
            ✓ Generate Photo Layout
          </button>
        {/if}
      </div>

      <button class="back-btn-preview" onclick={handleBack}>
        ← Back to Booth
      </button>
    </div>
  {/if}
  
  <!-- Final Composite View -->
  {#if finalComposite}
    <div class="final-composite-view">
      <div class="composite-container">
        <h2>🎉 Your Photo Layout</h2>
        <img src={finalComposite} alt="Final Layout" class="composite-image" />
        
        <div class="composite-actions">
          <button class="print-btn" onclick={() => window.print()}>
            🖨️ Print
          </button>
          <button class="save-btn" onclick={handleSaveAndShare}>
            💾 Save & Share
          </button>
          <button class="new-session-btn" onclick={resetSession}>
            🔄 New Session
          </button>
        </div>
        
        <button class="back-btn-composite" onclick={handleBack}>
          ← Back to Booth
        </button>
      </div>
    </div>
  {/if}
  
  <!-- Hidden canvas for image processing - always available -->
  <canvas bind:this={canvasElement} class="hidden-canvas"></canvas>
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
  display: flex;
  flex-direction: column;
  gap: 10px;
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

.session-progress {
  background: rgba(59, 130, 246, 0.9);
  backdrop-filter: blur(10px);
  color: white;
  padding: 8px 16px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  text-align: center;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.layout-info {
  font-size: 12px;
  opacity: 0.9;
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
  padding: 40px 20px;
  overflow-y: auto;
}

.photo-with-frame {
  position: relative;
  display: inline-block;
  max-width: 90%;
  margin: 0 auto;
}

.photo-with-frame img {
  display: block;
  max-width: 100%;
  max-height: 40vh;
  object-fit: contain;
  transition: filter 0.3s ease;
}

/* Frame Styles */
.photo-with-frame[data-frame="none"] img {
  border-radius: 12px;
  box-shadow: 0 10px 50px rgba(0, 0, 0, 0.5);
}

.photo-with-frame[data-frame="classic"] img {
  border: 20px solid white;
  box-shadow: 0 0 50px rgba(0, 0, 0, 0.5);
}

.photo-with-frame[data-frame="polaroid"] img {
  border: 30px solid white;
  border-bottom: 80px solid white;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
}

.photo-with-frame[data-frame="gold"] img {
  border: 25px solid #d4af37;
  padding: 10px;
  background: white;
  box-shadow: 
    0 0 0 3px #8b7220,
    0 0 40px rgba(212, 175, 55, 0.5);
}

.photo-with-frame[data-frame="rounded"] img {
  border: 15px solid white;
  border-radius: 30px;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.4);
}

.photo-with-frame[data-frame="neon"] img {
  border: 5px solid #00ffff;
  padding: 20px;
  background: #000;
  box-shadow: 
    0 0 30px #00ffff,
    0 0 60px #00ffff;
}

.photo-with-frame[data-frame="film"] {
  background: #1a1a1a;
  padding: 40px;
  position: relative;
}

.photo-with-frame[data-frame="film"]::before,
.photo-with-frame[data-frame="film"]::after {
  content: '';
  position: absolute;
  left: 10px;
  right: 10px;
  height: 15px;
  background: repeating-linear-gradient(
    90deg,
    transparent,
    transparent 10px,
    white 10px,
    white 15px
  );
}

.photo-with-frame[data-frame="film"]::before {
  top: 10px;
}

.photo-with-frame[data-frame="film"]::after {
  bottom: 10px;
}

.photo-with-frame[data-frame="film"] img {
  border: 15px solid transparent;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.5);
}

.preview-image {
  max-width: 90%;
  max-height: 40vh;
  object-fit: contain;
  border-radius: 12px;
  box-shadow: 0 10px 50px rgba(0, 0, 0, 0.5);
  transition: filter 0.3s ease;
}

.filter-selector {
  margin-top: 30px;
  width: 100%;
  max-width: 800px;
}

.filter-selector h3 {
  color: white;
  font-size: 20px;
  margin-bottom: 15px;
  text-align: center;
}

.filter-options {
  display: flex;
  gap: 12px;
  overflow-x: auto;
  padding: 10px 0;
  scrollbar-width: thin;
  scrollbar-color: rgba(255, 255, 255, 0.3) transparent;
}

.filter-options::-webkit-scrollbar {
  height: 6px;
}

.filter-options::-webkit-scrollbar-track {
  background: transparent;
}

.filter-options::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.3);
  border-radius: 3px;
}

.filter-option {
  flex-shrink: 0;
  background: rgba(255, 255, 255, 0.1);
  border: 2px solid transparent;
  border-radius: 12px;
  padding: 8px;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.filter-option:hover {
  background: rgba(255, 255, 255, 0.2);
  transform: translateY(-2px);
}

.filter-option.active {
  border-color: #3b82f6;
  background: rgba(59, 130, 246, 0.2);
}

.filter-preview {
  width: 80px;
  height: 80px;
  border-radius: 8px;
  overflow: hidden;
  position: relative;
}

.filter-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.filter-name {
  color: white;
  font-size: 12px;
  font-weight: 500;
  text-align: center;
}

/* Frame Selector - similar to filter selector */
.frame-selector {
  margin-top: 30px;
  width: 100%;
  max-width: 800px;
}

.frame-selector h3 {
  color: white;
  font-size: 20px;
  margin-bottom: 15px;
  text-align: center;
}

.frame-options {
  display: flex;
  gap: 12px;
  overflow-x: auto;
  padding: 10px 0;
  scrollbar-width: thin;
  scrollbar-color: rgba(255, 255, 255, 0.3) transparent;
}

.frame-options::-webkit-scrollbar {
  height: 6px;
}

.frame-options::-webkit-scrollbar-track {
  background: transparent;
}

.frame-options::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.3);
  border-radius: 3px;
}

.frame-option {
  flex-shrink: 0;
  background: rgba(255, 255, 255, 0.1);
  border: 2px solid transparent;
  border-radius: 12px;
  padding: 8px;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.frame-option:hover {
  background: rgba(255, 255, 255, 0.2);
  transform: translateY(-2px);
}

.frame-option.active {
  border-color: #f59e0b;
  background: rgba(245, 158, 11, 0.2);
}

.frame-preview {
  width: 80px;
  height: 80px;
  border-radius: 8px;
  overflow: visible;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #1a1a1a;
}

.frame-preview img {
  max-width: 60px;
  max-height: 60px;
  object-fit: cover;
  border-radius: 4px;
}

/* Frame preview styles */
.frame-preview[data-frame="classic"] img {
  border: 3px solid white;
  box-shadow: 0 0 10px rgba(0, 0, 0, 0.5);
}

.frame-preview[data-frame="polaroid"] img {
  border: 5px solid white;
  border-bottom: 12px solid white;
}

.frame-preview[data-frame="gold"] img {
  border: 4px solid #d4af37;
  padding: 2px;
  background: white;
  box-shadow: 0 0 0 1px #8b7220;
}

.frame-preview[data-frame="rounded"] img {
  border: 3px solid white;
  border-radius: 8px;
}

.frame-preview[data-frame="neon"] img {
  border: 2px solid #00ffff;
  padding: 3px;
  background: #000;
  box-shadow: 0 0 10px #00ffff;
}

.frame-preview[data-frame="film"] {
  background: #333;
  padding: 8px;
  position: relative;
}

.frame-preview[data-frame="film"]::before,
.frame-preview[data-frame="film"]::after {
  content: '';
  position: absolute;
  left: 2px;
  right: 2px;
  height: 3px;
  background: repeating-linear-gradient(
    90deg,
    transparent,
    transparent 3px,
    white 3px,
    white 5px
  );
}

.frame-preview[data-frame="film"]::before {
  top: 2px;
}

.frame-preview[data-frame="film"]::after {
  bottom: 2px;
}

.frame-name {
  color: white;
  font-size: 12px;
  font-weight: 500;
  text-align: center;
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

/* Session Gallery */
.session-gallery {
  margin-top: 30px;
  width: 100%;
  max-width: 800px;
}

.session-gallery h3 {
  color: white;
  font-size: 18px;
  margin-bottom: 15px;
  text-align: center;
}

.gallery-grid {
  display: flex;
  gap: 10px;
  overflow-x: auto;
  padding: 10px 0;
  scrollbar-width: thin;
  scrollbar-color: rgba(255, 255, 255, 0.3) transparent;
}

.gallery-grid::-webkit-scrollbar {
  height: 6px;
}

.gallery-grid::-webkit-scrollbar-track {
  background: transparent;
}

.gallery-grid::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.3);
  border-radius: 3px;
}

.gallery-item {
  position: relative;
  flex-shrink: 0;
  width: 80px;
  height: 80px;
  border-radius: 8px;
  overflow: hidden;
  border: 2px solid rgba(255, 255, 255, 0.3);
  transition: all 0.2s;
}

.gallery-item:hover {
  transform: scale(1.05);
  border-color: rgba(255, 255, 255, 0.6);
}

.gallery-item.current {
  border-color: #3b82f6;
  box-shadow: 0 0 10px rgba(59, 130, 246, 0.5);
}

.gallery-item img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.photo-number {
  position: absolute;
  top: 4px;
  right: 4px;
  background: rgba(0, 0, 0, 0.7);
  color: white;
  font-size: 10px;
  padding: 2px 6px;
  border-radius: 4px;
  font-weight: 600;
}

/* New Action Buttons */
.take-another-btn,
.end-session-btn {
  padding: 16px 32px;
  font-size: 18px;
  font-weight: 600;
  border: none;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.take-another-btn {
  background: rgba(255, 255, 255, 0.2);
  backdrop-filter: blur(10px);
  color: white;
}

.take-another-btn:hover {
  background: rgba(255, 255, 255, 0.3);
  transform: translateY(-2px);
}

.end-session-btn {
  background: #10b981;
  color: white;
}

.end-session-btn:hover {
  background: #059669;
  transform: translateY(-2px);
}

/* Layout Selection Modal */
.layout-modal {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.modal-content-large {
  background: #1a1a1a;
  border-radius: 20px;
  padding: 40px;
  max-width: 900px;
  width: 100%;
  max-height: 90vh;
  overflow-y: auto;
}

.modal-content-large h2 {
  color: white;
  font-size: 32px;
  margin-bottom: 10px;
  text-align: center;
}

.modal-subtitle {
  color: rgba(255, 255, 255, 0.7);
  font-size: 16px;
  text-align: center;
  margin-bottom: 30px;
}

.layout-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 20px;
  margin-bottom: 30px;
}

.layout-option {
  background: rgba(255, 255, 255, 0.05);
  border: 3px solid transparent;
  border-radius: 16px;
  padding: 20px;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.layout-option:hover:not(.disabled) {
  background: rgba(255, 255, 255, 0.1);
  transform: translateY(-4px);
}

.layout-option.active {
  border-color: #3b82f6;
  background: rgba(59, 130, 246, 0.1);
}

.layout-option.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.layout-preview {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  padding: 20px;
  aspect-ratio: 3/4;
  display: grid;
  gap: 8px;
}

.layout-preview[data-layout="layout-1"] {
  grid-template-columns: 1fr;
}

.layout-preview[data-layout="layout-2"] {
  grid-template-columns: 1fr 1fr;
}

.layout-preview[data-layout="layout-3"] {
  grid-template-columns: 1fr 1fr 1fr;
}

.layout-preview[data-layout="layout-4"] {
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr;
}

.layout-preview[data-layout="layout-5"] {
  grid-template-columns: 1fr 1fr 1fr;
  grid-template-rows: 1fr 1fr;
}

.layout-preview[data-layout="layout-5"] .layout-box:nth-child(1),
.layout-preview[data-layout="layout-5"] .layout-box:nth-child(2) {
  grid-column: span 2;
}

.layout-preview[data-layout="layout-5"] .layout-box:nth-child(1) {
  grid-column: 1 / 2;
}

.layout-preview[data-layout="layout-5"] .layout-box:nth-child(2) {
  grid-column: 2 / 4;
}

.layout-box {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 4px;
  border: 2px dashed rgba(255, 255, 255, 0.3);
}

.layout-info {
  text-align: center;
}

.layout-info h4 {
  color: white;
  font-size: 18px;
  margin: 0 0 8px 0;
}

.layout-info p {
  color: rgba(255, 255, 255, 0.7);
  font-size: 14px;
  margin: 0;
}

.layout-info .warning {
  display: block;
  color: #fbbf24;
  font-size: 12px;
  margin-top: 8px;
  font-weight: 600;
}

.modal-actions {
  display: flex;
  gap: 15px;
  justify-content: center;
}

.cancel-btn,
.generate-btn {
  padding: 16px 40px;
  font-size: 18px;
  font-weight: 600;
  border: none;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.cancel-btn {
  background: rgba(255, 255, 255, 0.1);
  color: white;
}

.cancel-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.generate-btn {
  background: #3b82f6;
  color: white;
}

.generate-btn:hover {
  background: #2563eb;
  transform: translateY(-2px);
}

/* Final Composite View */
.final-composite-view {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: #000;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 40px;
}

.composite-container {
  text-align: center;
  max-width: 90%;
}

.composite-container h2 {
  color: white;
  font-size: 36px;
  margin-bottom: 30px;
}

.composite-image {
  max-width: 100%;
  max-height: 60vh;
  border-radius: 12px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.8);
}

.composite-actions {
  display: flex;
  gap: 20px;
  justify-content: center;
  margin-top: 30px;
}

.print-btn,
.new-session-btn {
  padding: 16px 32px;
  font-size: 18px;
  font-weight: 600;
  border: none;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.print-btn {
  background: #8b5cf6;
  color: white;
}

.print-btn:hover {
  background: #7c3aed;
  transform: translateY(-2px);
}

.new-session-btn {
  background: rgba(255, 255, 255, 0.2);
  backdrop-filter: blur(10px);
  color: white;
}

.new-session-btn:hover {
  background: rgba(255, 255, 255, 0.3);
  transform: translateY(-2px);
}

.back-btn-composite {
  position: absolute;
  top: 20px;
  left: 20px;
  background: rgba(255, 255, 255, 0.2);
  backdrop-filter: blur(10px);
  color: white;
  border: none;
  padding: 12px 24px;
  border-radius: 12px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.back-btn-composite:hover {
  background: rgba(255, 255, 255, 0.3);
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
