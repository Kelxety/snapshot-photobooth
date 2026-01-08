<script lang="ts">
  import { goto } from "$app/navigation";
  import { getAllEvents, type Event } from "$lib/database";
  import { onMount } from "svelte";

  let savedEvents = $state<Event[]>([]);
  let selectedEvent = $state<Event | null>(null);
  let searchQuery = $state("");

  // Capture modes - toggleable
  let captureModes = $state({
    photo: true,
    gif: true,
    boomerang: true,
    video: false
  });

  function toggleCaptureMode(mode: "photo" | "gif" | "boomerang" | "video") {
    captureModes[mode] = !captureModes[mode];
  }

  // Effects
  let effects = $state({
    digitalProps: false,
    beautyFilter: false,
    watermark: false,
    postProcessing: false,
    disclaimer: false,
    greenScreen: false,
    survey: false,
    virtualAttendant: true,
    triggers: false
  });

  // Printing settings
  let printing = $state({
    printAutomatically: true,
    displayPrintButton: true,
    mainPrinter: "Primary",
    secondaryPrinter: ""
  });

  // Sharing settings
  let sharing = $state({
    email: true,
    emailReply: "no-reply@mail.fotoshare.co",
    emailSubject: "Here is your photo",
    sms: true,
    smsCountryCode: "",
    smsMessage: "Here is your photo",
    twitter: true,
    twitterMessage: "",
    qrCode: true,
    facebookPage: ""
  });

  onMount(async () => {
    try {
      savedEvents = await getAllEvents();
      if (savedEvents.length > 0) {
        selectedEvent = savedEvents[0];
      }
    } catch (error) {
      console.error("Error loading events:", error);
    }
  });

  function handleEventSelect(event: Event) {
    selectedEvent = event;
  }

  function handleLaunchEvent() {
    if (!selectedEvent) {
      alert("Please select an event first");
      return;
    }
    // Navigate to the actual camera/booth screen
    console.log("Launching booth for event:", selectedEvent);
    goto(`/booth/camera/${selectedEvent.id}`);
  }

  function handleNewEvent() {
    goto("/create-event");
  }

  function handleCopySettings() {
    console.log("Copy event settings");
  }

  function handleEditEvent() {
    if (selectedEvent) {
      goto(`/event/${selectedEvent.id}`);
    }
  }

  let filteredEvents = $derived(savedEvents.filter(event =>
    event.name.toLowerCase().includes(searchQuery.toLowerCase())
  ));
</script>

<div class="booth-screen">
  <!-- Left Sidebar -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <h2>Saved Events</h2>
      <button class="sort-btn">
        <span class="icon">↕️</span>
        Sort by name
      </button>
    </div>

    <div class="search-box">
      <input
        type="text"
        placeholder="Find your event"
        bind:value={searchQuery}
      />
      <button class="search-icon">🔍</button>
    </div>

    <div class="events-list">
      {#each filteredEvents as event}
        <div
          class="event-item"
          class:active={selectedEvent?.id === event.id}
          onclick={() => handleEventSelect(event)}
          role="button"
          tabindex="0"
          onkeydown={(e) => e.key === 'Enter' && handleEventSelect(event)}
        >
          <div class="event-name">{event.name}</div>
          <div class="event-date">{event.date}</div>
          <button class="settings-btn" onclick={(e) => e.stopPropagation()}>⚙️</button>
        </div>
      {/each}
    </div>

    <div class="sidebar-actions">
      <button class="copy-btn" onclick={handleCopySettings}>
        Copy event settings >
      </button>
      <button class="launch-btn-sidebar" onclick={handleLaunchEvent}>
        ⚡ Launch event
      </button>
      <div class="or-text">or</div>
      <button class="new-event-btn" onclick={handleNewEvent}>
        + New event
      </button>
    </div>
  </aside>

  <!-- Main Content -->
  <main class="main-content">
    <div class="content-grid">
      <div class="left-column">
        <!-- Start Screen Section -->
        <section class="section">
          <h3>Start screen</h3>
          <div class="start-modes">
            {#if captureModes.photo}
              <button class="mode-btn">
                <span class="mode-icon">📷</span>
                <span>Print</span>
              </button>
            {/if}
            {#if captureModes.gif}
              <button class="mode-btn">
                <span class="mode-icon">🎬</span>
                <span>GIF</span>
              </button>
            {/if}
            {#if captureModes.boomerang}
              <button class="mode-btn">
                <span class="mode-icon">🔄</span>
                <span>Boomerang</span>
              </button>
            {/if}
            {#if captureModes.video}
              <button class="mode-btn">
                <span class="mode-icon">📹</span>
                <span>Video</span>
              </button>
            {/if}
          </div>
        </section>

        <!-- Capture Section -->
        <section class="section">
        <h3>Capture</h3>
        <div class="capture-modes">
          <button
            class="capture-btn"
            class:active={captureModes.photo}
            onclick={() => toggleCaptureMode("photo")}
          >
            <span class="capture-icon blue">📷</span>
            <span>Photo</span>
          </button>
          <button
            class="capture-btn"
            class:active={captureModes.gif}
            onclick={() => toggleCaptureMode("gif")}
          >
            <span class="capture-icon blue">🎬</span>
            <span>Gif</span>
          </button>
          <button
            class="capture-btn"
            class:active={captureModes.boomerang}
            onclick={() => toggleCaptureMode("boomerang")}
          >
            <span class="capture-icon blue">🔄</span>
            <span>Boomerang</span>
          </button>
          <button
            class="capture-btn"
            class:active={captureModes.video}
            onclick={() => toggleCaptureMode("video")}
          >
            <span class="capture-icon">📹</span>
            <span>Video</span>
          </button>
        </div>

        <div class="effects-list">
          <h4>Effects</h4>
          <label class="effect-item">
            <input type="checkbox" bind:checked={effects.digitalProps} />
            <span>Digital Props</span>
          </label>
          <label class="effect-item">
            <input type="checkbox" bind:checked={effects.beautyFilter} />
            <span>Beauty Filter</span>
          </label>
          <label class="effect-item">
            <input type="checkbox" bind:checked={effects.watermark} />
            <span>Watermark</span>
          </label>
          <label class="effect-item">
            <input type="checkbox" bind:checked={effects.postProcessing} />
            <span>Post Processing</span>
          </label>
          <label class="effect-item">
            <input type="checkbox" bind:checked={effects.disclaimer} />
            <span>Disclaimer</span>
          </label>
          <label class="effect-item">
            <input type="checkbox" bind:checked={effects.greenScreen} />
            <span>Green Screen</span>
          </label>
          <label class="effect-item">
            <input type="checkbox" bind:checked={effects.survey} />
            <span>Survey</span>
          </label>
          <label class="effect-item">
            <input type="checkbox" bind:checked={effects.virtualAttendant} />
            <span>Virtual Attendant</span>
          </label>
          <label class="effect-item">
            <input type="checkbox" bind:checked={effects.triggers} />
            <span>Triggers</span>
          </label>
        </div>
      </section>
      </div>

      <!-- Right Sidebar - Printing & Sharing -->
      <aside class="right-sidebar">
        <!-- Printing Section -->
        <section class="section">
          <h3>Printing</h3>
          <label class="setting-item">
            <input type="checkbox" bind:checked={printing.printAutomatically} />
            <span>Print Automatically</span>
          </label>
          <label class="setting-item">
            <input type="checkbox" bind:checked={printing.displayPrintButton} />
            <span>Display Print Button</span>
          </label>
          <div class="printer-settings">
            <div>Main Printer:</div>
            <div>Secondary Printer:</div>
          </div>
          <div class="printer-preview">
            <div class="preview-box red">
              <div class="preview-number">1</div>
            </div>
            <div class="preview-box green">
              <div class="preview-number">2</div>
            </div>
          </div>
        </section>

        <!-- Sharing Configuration -->
        <section class="section">
          <h3>Sharing Configuration</h3>
          <label class="setting-item">
            <input type="checkbox" bind:checked={sharing.email} />
            <span>Email</span>
          </label>
          {#if sharing.email}
            <div class="setting-details">
              <div class="detail-row">
                <span class="label">Reply to:</span>
                <input type="text" bind:value={sharing.emailReply} class="small-input" />
              </div>
              <div class="detail-row">
                <span class="label">Subject:</span>
                <input type="text" bind:value={sharing.emailSubject} class="small-input" />
              </div>
            </div>
          {/if}

          <label class="setting-item">
            <input type="checkbox" bind:checked={sharing.sms} />
            <span>SMS</span>
          </label>
          {#if sharing.sms}
            <div class="setting-details">
              <div class="detail-row">
                <span class="label">Country code:</span>
                <input type="text" bind:value={sharing.smsCountryCode} class="small-input" />
              </div>
              <div class="detail-row">
                <span class="label">Message:</span>
                <input type="text" bind:value={sharing.smsMessage} class="small-input" />
              </div>
            </div>
          {/if}

          <label class="setting-item">
            <input type="checkbox" bind:checked={sharing.twitter} />
            <span>Twitter</span>
          </label>
          {#if sharing.twitter}
            <div class="setting-details">
              <div class="detail-row">
                <span class="label">Message:</span>
                <input type="text" bind:value={sharing.twitterMessage} class="small-input" />
              </div>
            </div>
          {/if}

          <label class="setting-item">
            <input type="checkbox" bind:checked={sharing.qrCode} />
            <span>QR Code</span>
          </label>
          {#if sharing.qrCode}
            <div class="setting-details">
              <div class="detail-row">
                <span class="label">Facebook Page</span>
              </div>
            </div>
          {/if}
        </section>
      </aside>
    </div>
  </main>
</div>

<style>
.booth-screen {
  display: flex;
  min-height: 100vh;
  background-color: #2a2a2a;
  color: #ffffff;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

/* Sidebar */
.sidebar {
  width: 220px;
  background-color: #1f1f1f;
  display: flex;
  flex-direction: column;
  border-right: 1px solid #3a3a3a;
  height: 100vh;
  position: sticky;
  top: 0;
}

.sidebar-header {
  padding: 16px;
  border-bottom: 1px solid #3a3a3a;
}

.sidebar-header h2 {
  font-size: 14px;
  font-weight: 600;
  margin: 0 0 12px 0;
}

.sort-btn {
  background: transparent;
  border: 1px solid #4a4a4a;
  color: #ccc;
  padding: 6px 12px;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 6px;
}

.search-box {
  padding: 12px 16px;
  position: relative;
}

.search-box input {
  width: calc(100% - 32px);
  background: #2a2a2a;
  border: 1px solid #4a4a4a;
  color: #fff;
  padding: 8px 32px 8px 12px;
  border-radius: 4px;
  font-size: 13px;
  box-sizing: border-box;
}

.search-icon {
  position: absolute;
  right: 24px;
  top: 50%;
  transform: translateY(-50%);
  background: transparent;
  border: none;
  cursor: pointer;
}

.events-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
  min-height: 0;
}

.event-item {
  width: 100%;
  background: #2a2a2a;
  border: none;
  color: #fff;
  padding: 12px;
  margin-bottom: 8px;
  border-radius: 6px;
  cursor: pointer;
  text-align: left;
  position: relative;
  transition: background 0.2s;
}

.event-item:hover,
.event-item.active {
  background: #3a3a3a;
}

.event-item.active {
  border-left: 3px solid #3b82f6;
}

.event-name {
  font-size: 13px;
  font-weight: 500;
  margin-bottom: 4px;
}

.event-date {
  font-size: 11px;
  color: #888;
}

.settings-btn {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  background: transparent;
  border: none;
  font-size: 16px;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.2s;
}

.event-item:hover .settings-btn {
  opacity: 1;
}

.sidebar-actions {
  padding: 16px;
  border-top: 1px solid #3a3a3a;
  display: flex;
  flex-direction: column;
  gap: 8px;
  background-color: #1f1f1f;
  position: sticky;
  bottom: 0;
  margin-top: auto;
}

.copy-btn {
  background: transparent;
  border: none;
  color: #3b82f6;
  padding: 8px;
  font-size: 12px;
  cursor: pointer;
  text-align: left;
}

.launch-btn-sidebar {
  background: #3b82f6;
  border: none;
  color: white;
  padding: 10px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
}

.launch-btn-sidebar:hover {
  background: #2563eb;
}

.or-text {
  text-align: center;
  font-size: 12px;
  color: #888;
}

.new-event-btn {
  background: transparent;
  border: 1px solid #4a4a4a;
  color: #fff;
  padding: 10px;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
}

.new-event-btn:hover {
  background: #3a3a3a;
}

/* Main Content */
.main-content {
  flex: 1;
  padding: 24px;
  overflow-y: auto;
}

.content-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}
left-column {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.section {
  background: #3a3a3a;
  border-radius: 8px;
  padding: 20px;
}

.section h3 {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 16px 0;
}

.section h4 {
  font-size: 14px;
  font-weight: 500;
  margin: 20px 0 12px 0;
}

/* Start Screen */
.start-modes {
  display: flex;
  gap: 16px;
}

.mode-btn {
  flex: 1;
  background: #505050;
  border: none;
  color: #fff;
  padding: 20px;
  border-radius: 50%;
  width: 80px;
  height: 80px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  cursor: pointer;
  font-size: 12px;
}

.mode-icon {
  font-size: 24px;
}

/* Capture Modes */
.capture-modes {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  margin-bottom: 20px;
}

.capture-btn {
  background: #505050;
  border: 2px solid transparent;
  color: #fff;
  padding: 16px;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s;
}

.capture-btn:hover {
  background: #5a5a5a;
}

.capture-btn.active {
  border-color: #3b82f6;
}

.capture-icon {
  font-size: 32px;
}

.capture-icon.blue {
  color: #3b82f6;
}

/* Effects List */
.effects-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.effect-item {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
  cursor: pointer;
}

.effect-item input[type="checkbox"] {
  width: 16px;
  height: 16px;
}

/* Right Sidebar */
.right-sidebar {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.setting-item {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
  cursor: pointer;
  margin-bottom: 12px;
}

.setting-item input[type="checkbox"] {
  width: 16px;
  height: 16px;
}

.setting-details {
  margin-left: 26px;
  margin-bottom: 12px;
  font-size: 12px;
}

.detail-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.detail-row .label {
  min-width: 100px;
  color: #ccc;
}

.small-input {
  flex: 1;
  background: #2a2a2a;
  border: 1px solid #4a4a4a;
  color: #fff;
  padding: 6px 8px;
  border-radius: 4px;
  font-size: 12px;
}

.printer-settings {
  font-size: 12px;
  color: #ccc;
  margin: 12px 0;
}

.printer-preview {
  display: flex;
  gap: 12px;
  margin-top: 12px;
}

.preview-box {
  flex: 1;
  height: 120px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.preview-box.red {
  background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
}

.preview-box.green {
  background: linear-gradient(135deg, #84cc16 0%, #65a30d 100%);
}

.preview-number {
  font-size: 48px;
  font-weight: bold;
  color: white;
}

@media (max-width: 1200px) {
  .content-grid {
    grid-template-columns: 1fr;
  }
}
</style>
