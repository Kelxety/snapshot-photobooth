<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { onMount, onDestroy } from 'svelte';
  import { getEvent, type Event } from '$lib/database';

  let eventId = $derived($page.params.id);
  let event = $state<Event | null>(null);
  let videoElement = $state<HTMLVideoElement>();
  let canvasElement = $state<HTMLCanvasElement>();
  let stream: MediaStream | null = null;
  let captureMode = $state<'photo' | 'gif' | 'boomerang' | 'video'>('photo');
  let isCapturing = $state(false);
  let countdown = $state(0);
  let capturedImage = $state<string | null>(null);

  onMount(async () => {
    try {
      if (!eventId) {
        throw new Error('Event ID is required');
      }
      const eventData = await getEvent(parseInt(eventId));
      event = eventData;
      await initCamera();
    } catch (error) {
      console.error('Error loading event:', error);
    }
  });

  onDestroy(() => {
    stopCamera();
  });

  async function initCamera() {
    try {
      stream = await navigator.mediaDevices.getUserMedia({
        video: { width: 1280, height: 720 },
        audio: false
      });
      if (videoElement) {
        videoElement.srcObject = stream;
      }
    } catch (error) {
      console.error('Error accessing camera:', error);
      alert('Could not access camera. Please grant camera permissions.');
    }
  }

  function stopCamera() {
    if (stream) {
      stream.getTracks().forEach(track => track.stop());
    }
  }

  async function startCapture() {
    if (isCapturing) return;
    
    isCapturing = true;
    countdown = 3;

    const timer = setInterval(() => {
      countdown--;
      if (countdown === 0) {
        clearInterval(timer);
        capturePhoto();
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

  function retake() {
    capturedImage = null;
    countdown = 0;
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
      <video
        bind:this={videoElement}
        autoplay
        playsinline
        class="camera-feed"
      ></video>
      <canvas bind:this={canvasElement} style="display: none;"></canvas>

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
