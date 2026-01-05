<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";

  let eventId = $derived($page.params.id);
  let activeTab = $state("general");
  
  // Mock event data - replace with actual data from your backend
  let eventName = $state("Test Event");
  let launchMessage = $state("");
  let showLaunchMessage = $state(false);
  
  const tabs = [
    { id: "general", label: "General", icon: "⚙️" },
    { id: "camera", label: "Camera", icon: "📷" },
    { id: "display", label: "Display", icon: "🖥️" },
    { id: "effects", label: "Effects", icon: "✨" },
    { id: "gif", label: "GIF", icon: "🎬" },
    { id: "video", label: "Video/360", icon: "📹" },
    { id: "virtual", label: "Virtual Attendant", icon: "👤" },
    { id: "survey", label: "Survey", icon: "📋" },
    { id: "sharing", label: "Sharing", icon: "🔄" },
    { id: "printing", label: "Printing", icon: "🖨️" },
    { id: "language", label: "Language", icon: "🌐" }
  ];

  const shareTypes = [
    { type: "Email", total: 0, event: 0, pendings: 0 },
    { type: "SMS", total: 0, event: 0, pendings: 0 },
    { type: "Facebook Page", total: 0, event: 0, pendings: 0 },
    { type: "Twitter", total: 0, event: 0, pendings: 0 },
    { type: "Print", total: 0, event: 0, pendings: 0 },
    { type: "Upload", total: 0, event: 0, pendings: 0 }
  ];

  let boothModes = $state({
    photo: false,
    gif: false,
    boomerang: false,
    video: false,
    createGifWithPhotos: false,
    displayTextLabels: true
  });

  // Camera settings
  let cameraSettings = $state({
    autoTrigger: true,
    enableWebcams: true,
    webcamResolution: 50,
    liveViewEnable: false,
    mirrorImage: false,
    rotation: "0",
    displayOnStartScreen: false
  });

  function handleLaunchEvent() {
    console.log("Launch event clicked!"); // Debug log
    
    // Check if at least one booth mode is selected
    const hasSelectedMode = boothModes.photo || boothModes.gif || boothModes.boomerang || boothModes.video;
    
    console.log("Has selected mode:", hasSelectedMode); // Debug log
    console.log("Current booth modes:", boothModes); // Debug log
    
    if (!hasSelectedMode) {
      launchMessage = "⚠️ Please select at least one booth mode (Photo, GIF, Boomerang, or Video) before launching the event.";
      showLaunchMessage = true;
      setTimeout(() => showLaunchMessage = false, 5000);
      return;
    }

    // Select all booth modes when launching
    boothModes.photo = true;
    boothModes.gif = true;
    boothModes.boomerang = true;
    boothModes.video = true;
    boothModes.createGifWithPhotos = true;
    boothModes.displayTextLabels = true;
    
    console.log("Launching event:", eventId);
    console.log("Booth modes:", boothModes);
    console.log("Camera settings:", cameraSettings);
    
    // Navigate to booth screen
    goto("/booth");
  }

  function handleBackToHome() {
    goto("/home");
  }
</script>

<div class="event-dashboard">
  <header class="header">
    <div class="header-left">
      <button class="back-btn" onclick={handleBackToHome}>← Back</button>
      <h1 class="event-title">{eventName}</h1>
    </div>
    <button class="launch-btn" onclick={handleLaunchEvent}>
      ⚡ Launch event
    </button>
  </header>

  {#if showLaunchMessage}
    <div class="launch-message" class:error={launchMessage.includes('⚠️')} class:success={launchMessage.includes('✅')}>
      {launchMessage}
    </div>
  {/if}

  <nav class="tabs">
    {#each tabs as tab}
      <button
        class="tab"
        class:active={activeTab === tab.id}
        onclick={() => activeTab = tab.id}
      >
        <span class="tab-icon">{tab.icon}</span>
        <span class="tab-label">{tab.label}</span>
      </button>
    {/each}
  </nav>

  <main class="content">
    {#if activeTab === "general"}
      <!-- STATS Section -->
      <section class="section">
        <h2 class="section-title">STATS</h2>
        <div class="table-container">
          <table class="stats-table">
            <thead>
              <tr>
                <th>Type</th>
                <th>Total</th>
                <th>Event</th>
                <th>Pendings*</th>
                <th>Last Error</th>
              </tr>
            </thead>
            <tbody>
              {#each shareTypes as share}
                <tr>
                  <td class="type-cell">{share.type}</td>
                  <td>{share.total}</td>
                  <td class="event-cell">{share.event}</td>
                  <td class="pendings-cell">{share.pendings}</td>
                  <td class="actions-cell">
                    <button class="icon-btn delete-btn" title="Delete">✕</button>
                    <button class="icon-btn info-btn" title="Info">ℹ</button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
        <div class="table-footer">
          <button class="btn-secondary">Export shares</button>
          <button class="btn-secondary">Delete all shares</button>
        </div>
        <p class="note">* These items will process once you have internet connectivity and correct settings.</p>
      </section>

      <!-- BOOTH MODE Section -->
      <section class="section">
        <h2 class="section-title">BOOTH MODE</h2>
        <div class="booth-modes">
          <label class="booth-option">
            <input type="checkbox" bind:checked={boothModes.photo} />
            <div class="option-content">
              <span class="option-icon">📷</span>
              <div class="option-text">
                <strong>Photo</strong>
                <span class="option-desc">Capture photos for print</span>
              </div>
            </div>
          </label>

          <label class="booth-option">
            <input type="checkbox" bind:checked={boothModes.gif} />
            <div class="option-content">
              <span class="option-icon">🎬</span>
              <div class="option-text">
                <strong>GIF</strong>
                <span class="option-desc">Capture photos for GIF</span>
              </div>
            </div>
          </label>

          <label class="booth-option">
            <input type="checkbox" bind:checked={boothModes.boomerang} />
            <div class="option-content">
              <span class="option-icon">🔄</span>
              <div class="option-text">
                <strong>Boomerang</strong>
                <span class="option-desc">Record a short video that will automatically loop</span>
              </div>
            </div>
          </label>

          <label class="booth-option">
            <input type="checkbox" bind:checked={boothModes.video} />
            <div class="option-content">
              <span class="option-icon">📹</span>
              <div class="option-text">
                <strong>Video</strong>
                <span class="option-desc">Record a single or multiple clips</span>
              </div>
            </div>
          </label>
        </div>

        <div class="additional-options">
          <label class="checkbox-option">
            <input type="checkbox" bind:checked={boothModes.createGifWithPhotos} />
            <span>Create GIF with photos</span>
          </label>
          <label class="checkbox-option">
            <input type="checkbox" bind:checked={boothModes.displayTextLabels} />
            <span>Display text labels below icons</span>
          </label>
        </div>
      </section>

      <!-- OPTIONS Section -->
      <section class="section">
        <h2 class="section-title">OPTIONS</h2>
        <p class="placeholder-text">Additional options will be displayed here</p>
      </section>
    {:else if activeTab === "camera"}
      <!-- CAMERA TAB -->
      <section class="section">
        <div class="camera-options">
          <label class="checkbox-option">
            <input type="checkbox" bind:checked={cameraSettings.autoTrigger} />
            <div class="option-label">
              <span>Auto Trigger Camera</span>
              <span class="option-desc-small">Trigger the camera shutter at the end of the countdown.</span>
            </div>
          </label>

          <label class="checkbox-option">
            <input type="checkbox" bind:checked={cameraSettings.enableWebcams} />
            <div class="option-label">
              <span>Enable Webcams</span>
              <span class="option-desc-small">If disabled, only Canon, Nikon and Sony cameras can be used.</span>
            </div>
          </label>

          <div class="slider-option">
            <label for="webcam-resolution">Webcam Resolution</label>
            <div class="slider-container">
              <span class="slider-label">Faster framerate</span>
              <input 
                type="range" 
                id="webcam-resolution"
                min="0" 
                max="100" 
                bind:value={cameraSettings.webcamResolution}
                class="slider"
              />
              <span class="slider-label">Higher Quality</span>
            </div>
          </div>
        </div>
      </section>

      <!-- LIVE VIEW Section -->
      <section class="section">
        <h2 class="section-title">LIVE VIEW</h2>
        <div class="camera-options">
          <label class="checkbox-option">
            <input type="checkbox" bind:checked={cameraSettings.liveViewEnable} />
            <div class="option-label">
              <span>Enable</span>
              <span class="option-desc-small">Display a video preview during countdown. Not all DSLR cameras support this. <a href="#" class="info-link">More Information</a></span>
            </div>
          </label>

          <label class="checkbox-option">
            <input type="checkbox" bind:checked={cameraSettings.mirrorImage} />
            <div class="option-label">
              <span>Mirror Image</span>
              <span class="option-desc-small">Mirrors live view image to guests similar to looking at a mirror</span>
            </div>
          </label>

          <div class="select-option">
            <label for="rotation">Rotation</label>
            <select id="rotation" bind:value={cameraSettings.rotation}>
              <option value="0">0 degrees</option>
              <option value="90">90 degrees</option>
              <option value="180">180 degrees</option>
              <option value="270">270 degrees</option>
            </select>
          </div>

          <label class="checkbox-option">
            <input type="checkbox" bind:checked={cameraSettings.displayOnStartScreen} />
            <span>Display on Start Screen</span>
          </label>
        </div>
      </section>
    {:else}
      <div class="placeholder-content">
        <p>Content for {tabs.find(t => t.id === activeTab)?.label} tab</p>
      </div>
    {/if}
  </main>
</div>

<style>
.event-dashboard {
  min-height: 100vh;
  background-color: #505050;
  color: #ffffff;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  background-color: #3a3a3a;
  border-bottom: 1px solid #2a2a2a;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.back-btn {
  background: transparent;
  border: none;
  color: #ffffff;
  cursor: pointer;
  font-size: 14px;
  padding: 8px 12px;
  border-radius: 6px;
  transition: background-color 0.2s;
}

.back-btn:hover {
  background-color: #505050;
}

.event-title {
  font-size: 24px;
  font-weight: 400;
  margin: 0;
  color: #ffffff;
}

.launch-btn {
  background-color: #3b82f6;
  color: white;
  border: none;
  padding: 12px 24px;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: all 0.2s;
}

.launch-btn:hover {
  background-color: #2563eb;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.4);
}

.launch-message {
  padding: 16px 24px;
  text-align: center;
  font-size: 14px;
  font-weight: 500;
  animation: slideDown 0.3s ease-out;
}

.launch-message.error {
  background-color: #7f1d1d;
  color: #fecaca;
  border-bottom: 2px solid #991b1b;
}

.launch-message.success {
  background-color: #14532d;
  color: #86efac;
  border-bottom: 2px solid #166534;
}

@keyframes slideDown {
  from {
    transform: translateY(-100%);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

.tabs {
  display: flex;
  background-color: #3a3a3a;
  border-bottom: 1px solid #2a2a2a;
  overflow-x: auto;
  padding: 0 24px;
}

.tab {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 12px 16px;
  background: transparent;
  border: none;
  color: #a0a0a0;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s;
  border-bottom: 3px solid transparent;
  white-space: nowrap;
}

.tab:hover {
  color: #ffffff;
  background-color: #454545;
}

.tab.active {
  color: #ffffff;
  border-bottom-color: #3b82f6;
}

.tab-icon {
  font-size: 20px;
}

.tab-label {
  font-size: 11px;
}

.content {
  padding: 24px;
  max-width: 1400px;
  margin: 0 auto;
}

.section {
  background-color: #3a3a3a;
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 20px;
}

.section-title {
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.5px;
  margin: 0 0 16px 0;
  color: #ffffff;
}

.table-container {
  overflow-x: auto;
  margin-bottom: 16px;
}

.stats-table {
  width: 100%;
  border-collapse: collapse;
}

.stats-table thead tr {
  background-color: #2a2a2a;
}

.stats-table th {
  padding: 12px;
  text-align: left;
  font-size: 13px;
  font-weight: 600;
  color: #ffffff;
  border: 1px solid #404040;
}

.stats-table td {
  padding: 12px;
  font-size: 13px;
  border: 1px solid #404040;
}

.stats-table tbody tr {
  background-color: #505050;
}

.stats-table tbody tr:hover {
  background-color: #5a5a5a;
}

.type-cell {
  color: #ffffff;
  font-weight: 500;
}

.event-cell {
  color: #10b981;
}

.pendings-cell {
  color: #f59e0b;
}

.actions-cell {
  display: flex;
  gap: 8px;
  align-items: center;
}

.icon-btn {
  background: transparent;
  border: none;
  color: #a0a0a0;
  cursor: pointer;
  font-size: 16px;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.2s;
}

.icon-btn:hover {
  background-color: #404040;
  color: #ffffff;
}

.delete-btn:hover {
  color: #ef4444;
}

.info-btn:hover {
  color: #3b82f6;
}

.table-footer {
  display: flex;
  gap: 12px;
  margin-bottom: 12px;
}

.btn-secondary {
  background-color: #505050;
  color: #ffffff;
  border: 1px solid #606060;
  padding: 10px 20px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-secondary:hover {
  background-color: #606060;
}

.note {
  font-size: 12px;
  color: #a0a0a0;
  margin: 0;
  font-style: italic;
}

.booth-modes {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 16px;
  margin-bottom: 20px;
}

.booth-option {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 16px;
  background-color: #454545;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.booth-option:hover {
  background-color: #505050;
}

.booth-option input[type="checkbox"] {
  margin-top: 4px;
  width: 18px;
  height: 18px;
  cursor: pointer;
}

.option-content {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  flex: 1;
}

.option-icon {
  font-size: 32px;
}

.option-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.option-text strong {
  font-size: 14px;
  color: #ffffff;
}

.option-desc {
  font-size: 12px;
  color: #a0a0a0;
}

.additional-options {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding-left: 8px;
}

.checkbox-option {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  font-size: 13px;
  color: #ffffff;
}

.checkbox-option input[type="checkbox"] {
  width: 16px;
  height: 16px;
  cursor: pointer;
}

.camera-options {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.camera-options .checkbox-option {
  align-items: flex-start;
}

.option-label {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.option-desc-small {
  font-size: 12px;
  color: #a0a0a0;
  font-weight: 400;
}

.info-link {
  color: #3b82f6;
  text-decoration: underline;
}

.info-link:hover {
  color: #60a5fa;
}

.slider-option {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.slider-option label {
  font-size: 13px;
  font-weight: 500;
}

.slider-container {
  display: flex;
  align-items: center;
  gap: 12px;
}

.slider-label {
  font-size: 12px;
  color: #a0a0a0;
  white-space: nowrap;
}

.slider {
  flex: 1;
  height: 6px;
  border-radius: 3px;
  background: linear-gradient(to right, #606060, #3b82f6);
  outline: none;
  -webkit-appearance: none;
}

.slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #ffffff;
  cursor: pointer;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
}

.slider::-moz-range-thumb {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #ffffff;
  cursor: pointer;
  border: none;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
}

.select-option {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.select-option label {
  font-size: 13px;
  font-weight: 500;
}

.select-option select {
  padding: 10px 12px;
  background-color: #505050;
  color: #ffffff;
  border: 1px solid #606060;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  max-width: 200px;
}

.select-option select:focus {
  outline: none;
  border-color: #3b82f6;
}

.placeholder-text,
.placeholder-content {
  color: #a0a0a0;
  font-size: 14px;
  padding: 20px;
  text-align: center;
}

@media (prefers-color-scheme: light) {
  .event-dashboard {
    background-color: #f3f4f6;
    color: #1f2937;
  }

  .header {
    background-color: #ffffff;
    border-bottom-color: #e5e7eb;
  }

  .back-btn {
    color: #374151;
  }

  .back-btn:hover {
    background-color: #f3f4f6;
  }

  .event-title {
    color: #1f2937;
  }

  .tabs {
    background-color: #ffffff;
    border-bottom-color: #e5e7eb;
  }

  .tab {
    color: #6b7280;
  }

  .tab:hover {
    color: #1f2937;
    background-color: #f9fafb;
  }

  .tab.active {
    color: #1f2937;
  }

  .section {
    background-color: #ffffff;
  }

  .section-title {
    color: #1f2937;
  }

  .stats-table thead tr {
    background-color: #f9fafb;
  }

  .stats-table th {
    color: #1f2937;
    border-color: #e5e7eb;
  }

  .stats-table td {
    border-color: #e5e7eb;
  }

  .stats-table tbody tr {
    background-color: #ffffff;
  }

  .stats-table tbody tr:hover {
    background-color: #f9fafb;
  }

  .type-cell {
    color: #1f2937;
  }

  .icon-btn {
    color: #6b7280;
  }

  .icon-btn:hover {
    background-color: #f3f4f6;
    color: #1f2937;
  }

  .btn-secondary {
    background-color: #f3f4f6;
    color: #1f2937;
    border-color: #d1d5db;
  }

  .btn-secondary:hover {
    background-color: #e5e7eb;
  }

  .note {
    color: #6b7280;
  }

  .booth-option {
    background-color: #f9fafb;
  }

  .booth-option:hover {
    background-color: #f3f4f6;
  }

  .option-text strong {
    color: #1f2937;
  }

  .option-desc {
    color: #6b7280;
  }

  .checkbox-option {
    color: #1f2937;
  }

  .option-desc-small {
    color: #6b7280;
  }

  .slider {
    background: linear-gradient(to right, #d1d5db, #3b82f6);
  }

  .select-option select {
    background-color: #ffffff;
    color: #1f2937;
    border-color: #d1d5db;
  }

  .placeholder-text,
  .placeholder-content {
    color: #6b7280;
  }
}

@media (max-width: 768px) {
  .header {
    flex-direction: column;
    gap: 12px;
    align-items: flex-start;
  }

  .tabs {
    padding: 0 12px;
  }

  .tab {
    padding: 10px 12px;
  }

  .content {
    padding: 16px;
  }

  .booth-modes {
    grid-template-columns: 1fr;
  }

  .table-footer {
    flex-direction: column;
  }
}
</style>
