<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";

  let eventId = $derived($page.params.id);

  let screenTab = $state<"welcome" | "capture" | "sharing">("welcome");
  let editorPanel = $state<"design" | "settings">("settings");
  let aspectRatio = $state("16:9");
  let orientation = $state<"landscape" | "portrait">("landscape");

  // ── Theme colors ──────────────────────────────────────────────────────────
  let themePrimary = $state("#c0395a");
  let themeBackground = $state("#000000");

  // ── Collapsible sections ──────────────────────────────────────────────────
  let colorsOpen = $state(true);
  let alignOpen = $state(true);
  let layersOpen = $state(true);

  // ── Canvas elements ───────────────────────────────────────────────────────
  interface CanvasElement {
    id: string;
    type: "image" | "text" | "shape";
    x: number; y: number;
    w: number; h: number;
    label: string;
    src?: string;
    // text-specific
    text?: string;
    fontSize?: number;
    color?: string;
    rotation?: number;
  }

  let canvasElements = $state<CanvasElement[]>([
    { id: "placeholder", type: "image", x: 20, y: 20, w: 120, h: 100, label: "Image placeholder" }
  ]);
  let selectedElementId = $state<string | null>(null);
  let editingTextId = $state<string | null>(null);

  // ── History (undo/redo) ───────────────────────────────────────────────────
  const INITIAL_STATE: CanvasElement[] = [
    { id: "placeholder", type: "image", x: 20, y: 20, w: 120, h: 100, label: "Image placeholder" }
  ];

  // Keep history as plain non-reactive arrays to avoid proxy issues
  let historyStack: CanvasElement[][] = [JSON.parse(JSON.stringify(INITIAL_STATE))];
  let historyIndex = $state(0);

  function snapshot(): CanvasElement[] {
    // JSON round-trip strips the Svelte proxy and gives a plain array
    return JSON.parse(JSON.stringify(canvasElements));
  }

  function pushHistory() {
    // Trim any redo future
    historyStack = historyStack.slice(0, historyIndex + 1);
    historyStack.push(snapshot());
    historyIndex = historyStack.length - 1;
  }

  function undo() {
    if (historyIndex <= 0) return;
    historyIndex -= 1;
    canvasElements = JSON.parse(JSON.stringify(historyStack[historyIndex]));
    selectedElementId = null;
  }

  function redo() {
    if (historyIndex >= historyStack.length - 1) return;
    historyIndex += 1;
    canvasElements = JSON.parse(JSON.stringify(historyStack[historyIndex]));
    selectedElementId = null;
  }

  function resetCanvas() {
    canvasElements = JSON.parse(JSON.stringify(INITIAL_STATE));
    selectedElementId = null;
    historyStack = [JSON.parse(JSON.stringify(INITIAL_STATE))];
    historyIndex = 0;
  }

  // ── Interaction state ─────────────────────────────────────────────────────
  type InteractMode = "drag" | "resize-nw" | "resize-ne" | "resize-sw" | "resize-se" | "rotate";
  let interact = $state<{
    mode: InteractMode;
    id: string;
    startX: number; startY: number;
    origX: number; origY: number;
    origW: number; origH: number;
    origRot: number;
    cx: number; cy: number;
  } | null>(null);

  const MIN_SIZE = 40;

  function getSelected(): CanvasElement | undefined {
    return canvasElements.find(e => e.id === selectedElementId);
  }

  function updateSelected(patch: Partial<CanvasElement>) {
    if (!interact) return;
    const targetId = interact.id;
    canvasElements = canvasElements.map(el =>
      el.id === targetId ? { ...el, ...patch } : el
    );
  }

  // panel-side updates (not during drag)
  function patchSelected(patch: Partial<CanvasElement>) {
    if (!selectedElementId) return;
    canvasElements = canvasElements.map(el =>
      el.id === selectedElementId ? { ...el, ...patch } : el
    );
    pushHistory();
  }

  // ── Pointer handlers ──────────────────────────────────────────────────────
  function startInteract(e: MouseEvent, id: string, mode: InteractMode) {
    e.stopPropagation();
    e.preventDefault();
    selectedElementId = id;
    const el = canvasElements.find(c => c.id === id)!;
    // Snapshot all values into a plain object (not reactive proxy)
    const snap = {
      mode, id,
      startX: e.clientX, startY: e.clientY,
      origX: el.x, origY: el.y,
      origW: el.w, origH: el.h,
      origRot: el.rotation ?? 0,
      cx: el.x + el.w / 2,
      cy: el.y + el.h / 2,
    };
    interact = snap;
    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", stopInteract);
  }

  function onMouseMove(e: MouseEvent) {
    if (!interact) return;
    // Read all values from the snapshot directly
    const { mode, id, startX, startY, origX, origY, origW, origH } = interact;
    const dx = e.clientX - startX;
    const dy = e.clientY - startY;

    let patch: Partial<CanvasElement> = {};

    if (mode === "drag") {
      patch = { x: origX + dx, y: origY + dy };
    } else if (mode === "resize-se") {
      patch = { w: Math.max(MIN_SIZE, origW + dx), h: Math.max(MIN_SIZE, origH + dy) };
    } else if (mode === "resize-sw") {
      const newW = Math.max(MIN_SIZE, origW - dx);
      patch = { x: origX + origW - newW, w: newW, h: Math.max(MIN_SIZE, origH + dy) };
    } else if (mode === "resize-ne") {
      const newH = Math.max(MIN_SIZE, origH - dy);
      patch = { y: origY + origH - newH, w: Math.max(MIN_SIZE, origW + dx), h: newH };
    } else if (mode === "resize-nw") {
      const newW = Math.max(MIN_SIZE, origW - dx);
      const newH = Math.max(MIN_SIZE, origH - dy);
      patch = { x: origX + origW - newW, y: origY + origH - newH, w: newW, h: newH };
    } else if (mode === "rotate") {
      const el = canvasElements.find(c => c.id === id)!;
      const cx = el.x + el.w / 2;
      const cy = el.y + el.h / 2;
      const angle = Math.atan2(e.clientY - cy, e.clientX - cx) * (180 / Math.PI) + 90;
      patch = { rotation: angle };
    }

    // Apply patch directly by id, not via selectedElementId
    canvasElements = canvasElements.map(el => el.id === id ? { ...el, ...patch } : el);
  }

  function stopInteract() {
    interact = null;
    window.removeEventListener("mousemove", onMouseMove);
    window.removeEventListener("mouseup", stopInteract);
    pushHistory();
  }

  function selectElement(e: MouseEvent, id: string) {
    e.stopPropagation();
    if (editingTextId && editingTextId !== id) editingTextId = null;
    selectedElementId = id;
  }

  function deselectAll() {
    selectedElementId = null;
    editingTextId = null;
  }

  function startTextEdit(e: MouseEvent, id: string) {
    e.stopPropagation();
    editingTextId = id;
    selectedElementId = id;
  }

  // ── Element operations ────────────────────────────────────────────────────
  function addText() {
    const id = crypto.randomUUID();
    canvasElements = [...canvasElements, {
      id, type: "text",
      x: 100, y: 100, w: 200, h: 80,
      label: "Text", text: "Hello",
      fontSize: 40, color: "#ffffff", rotation: 0
    }];
    selectedElementId = id;
    pushHistory();
  }

  function addImage() {
    const input = document.createElement("input");
    input.type = "file"; input.accept = "image/*";
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;
      const src = URL.createObjectURL(file);
      const id = crypto.randomUUID();
      canvasElements = [...canvasElements, {
        id, type: "image",
        x: 60, y: 60, w: 200, h: 150,
        label: file.name, src, rotation: 0
      }];
      selectedElementId = id;
      pushHistory();
    };
    input.click();
  }

  function deleteSelected() {
    if (!selectedElementId) return;
    canvasElements = canvasElements.filter(el => el.id !== selectedElementId);
    selectedElementId = null;
    pushHistory();
  }

  function bringForward() {
    if (!selectedElementId) return;
    const idx = canvasElements.findIndex(el => el.id === selectedElementId);
    if (idx < canvasElements.length - 1) {
      const arr = [...canvasElements];
      [arr[idx], arr[idx + 1]] = [arr[idx + 1], arr[idx]];
      canvasElements = arr;
      pushHistory();
    }
  }

  function sendBackward() {
    if (!selectedElementId) return;
    const idx = canvasElements.findIndex(el => el.id === selectedElementId);
    if (idx > 0) {
      const arr = [...canvasElements];
      [arr[idx], arr[idx - 1]] = [arr[idx - 1], arr[idx]];
      canvasElements = arr;
      pushHistory();
    }
  }

  // ── Alignment ─────────────────────────────────────────────────────────────
  let canvasRef = $state<HTMLDivElement | null>(null);

  function align(type: string) {
    const el = getSelected();
    if (!el || !canvasRef) return;
    const cw = canvasRef.offsetWidth;
    const ch = canvasRef.offsetHeight;
    const patches: Partial<CanvasElement> = {};
    if (type === "left")    patches.x = 0;
    if (type === "hcenter") patches.x = (cw - el.w) / 2;
    if (type === "right")   patches.x = cw - el.w;
    if (type === "top")     patches.y = 0;
    if (type === "vcenter") patches.y = (ch - el.h) / 2;
    if (type === "bottom")  patches.y = ch - el.h;
    patchSelected(patches);
  }
  let welcomeSettings = $state({
    boothIcons: true, boothIconLabels: true,
    showLiveView: false, galleryButton: true,
    startScreenVideo: null as File | null,
    startScreenVideoName: ""
  });
  let boothIconModes = $state({ print: true, gif: true, slowMo: true, video: true });

  // ── Sharing settings ──────────────────────────────────────────────────────
  let sharingSettings = $state({
    skipSharingScreen: false,
    doneButton: true,
    iconsLocation: "Right" as "Right" | "Left" | "Bottom",
    textLabels: true,
    finalScreenTimeout: 30,
    showOriginalPhotos: true,
    retakeButton: false,
    email: true,
    sms: true,
    qrCode: true,
    print: true,
    whatsApp: false,
  });
  let eventName = $state("Test Event");

  function handleFileChoose() {
    const input = document.createElement("input");
    input.type = "file"; input.accept = "video/*";
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (file) {
        welcomeSettings.startScreenVideo = file;
        welcomeSettings.startScreenVideoName = file.name;
      }
    };
    input.click();
  }

  function handleLaunchEvent() { goto("/booth"); }
  function handleBackToHome() { goto("/home"); }
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

  <main class="content">
    <div class="se-root">
      <!-- Top toolbar -->
        <div class="se-toolbar">
          <div class="se-toolbar-left">
            <button class="se-tool-btn" onclick={addText} title="Add Text">
              <span class="se-tool-icon">T</span>
              <span class="se-tool-label">Text</span>
            </button>
            <button class="se-tool-btn" onclick={addImage} title="Add Image">
              <span class="se-tool-icon">🖼</span>
              <span class="se-tool-label">Image</span>
            </button>
            <button class="se-tool-btn" title="Shape">
              <span class="se-tool-icon">⬟</span>
              <span class="se-tool-label">Shape</span>
            </button>
            <div class="se-divider"></div>
            <button class="se-tool-btn" onclick={undo} disabled={historyIndex <= 0} title="Undo">
              <span class="se-tool-icon">↩</span>
              <span class="se-tool-label">Undo</span>
            </button>
            <button class="se-tool-btn" onclick={redo} disabled={historyIndex >= historyStack.length - 1} title="Redo">
              <span class="se-tool-icon">↪</span>
              <span class="se-tool-label">Redo</span>
            </button>
            <button class="se-tool-btn" onclick={resetCanvas} title="Reset">
              <span class="se-tool-icon">⟳</span>
              <span class="se-tool-label">Reset</span>
            </button>
            <div class="se-divider"></div>
            <div class="se-aspect-group">
              <span class="se-aspect-label">Aspect Ratio</span>
              <select class="se-select" bind:value={aspectRatio}>
                <option value="16:9">16:9</option>
                <option value="4:3">4:3</option>
                <option value="1:1">1:1</option>
              </select>
            </div>
            <div class="se-orient-group">
              <span class="se-aspect-label">Screen Orientation</span>
              <div class="se-orient-btns">
                <button
                  class="se-orient-btn"
                  class:active={orientation === "landscape"}
                  onclick={() => orientation = "landscape"}
                  title="Landscape"
                >▬</button>
                <button
                  class="se-orient-btn"
                  class:active={orientation === "portrait"}
                  onclick={() => orientation = "portrait"}
                  title="Portrait"
                >▮</button>
              </div>
            </div>
          </div>
          <div class="se-toolbar-right">
            <button class="se-photo-layout-btn" onclick={() => goto(`/event/${eventId}/photo-layout`)}>
              Photo Layout ›
            </button>
          </div>
        </div>

        <!-- Screen sub-tabs -->
        <div class="se-subtabs">
          <button class="se-subtab" class:active={screenTab === "welcome"} onclick={() => screenTab = "welcome"}>WELCOME SCREEN</button>
          <button class="se-subtab" class:active={screenTab === "capture"} onclick={() => screenTab = "capture"}>CAPTURE SCREEN</button>
          <button class="se-subtab" class:active={screenTab === "sharing"} onclick={() => screenTab = "sharing"}>SHARING SCREEN</button>
        </div>

        <!-- Editor body: canvas + right panel -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="se-body">
          <!-- Canvas area -->
          <div class="se-canvas-wrap">
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="se-canvas"
              class:portrait={orientation === "portrait"}
              role="region"
              aria-label="Screen canvas"
              bind:this={canvasRef}
              onmousedown={deselectAll}
              style="background-color:{themeBackground}"
            >
              <!-- Canvas content per tab -->
              {#if screenTab === "welcome"}
                <!-- Elements -->
                {#each canvasElements as el (el.id)}
                  {@const isSelected = selectedElementId === el.id}
                  {@const rot = el.rotation ?? 0}
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <div
                    class="se-element"
                    class:selected={isSelected}
                    class:text-el={el.type === "text"}
                    style="left:{el.x}px;top:{el.y}px;width:{el.w}px;height:{el.h}px;transform:rotate({rot}deg)"
                    onmousedown={(e) => { selectElement(e, el.id); startInteract(e, el.id, 'drag'); }}
                    ondblclick={(e) => el.type === 'text' && startTextEdit(e, el.id)}
                    role="button"
                    tabindex="0"
                    aria-label={el.label}
                    onkeydown={(e) => e.key === "Delete" && deleteSelected()}
                  >
                    {#if el.type === "image" && el.src}
                      <img src={el.src} alt={el.label} style="width:100%;height:100%;object-fit:contain;pointer-events:none" />
                    {:else if el.type === "image"}
                      <div class="se-img-placeholder">
                        <span class="se-img-icon">🖼</span>
                      </div>
                    {:else if el.type === "text"}
                      {#if editingTextId === el.id}
                        <!-- svelte-ignore a11y_autofocus -->
                        <textarea
                          class="se-text-editor"
                          style="font-size:{el.fontSize ?? 40}px"
                          value={el.text ?? ""}
                          autofocus
                          onmousedown={(e) => e.stopPropagation()}
                          oninput={(e) => {
                            const val = (e.target as HTMLTextAreaElement).value;
                            canvasElements = canvasElements.map(c => c.id === el.id ? { ...c, text: val } : c);
                          }}
                          onblur={() => editingTextId = null}
                        ></textarea>
                      {:else}
                        <span
                          class="se-text-display"
                          style="font-size:{el.fontSize ?? 40}px;color:{el.color ?? '#fff'}"
                        >{el.text ?? "Text"}</span>
                      {/if}
                    {/if}

                    <!-- Selection handles (only when selected) -->
                    {#if isSelected}
                      <!-- Rotation handle -->
                      <!-- svelte-ignore a11y_no_static_element_interactions -->
                      <div
                        class="se-handle se-rotate-handle"
                        onmousedown={(e) => startInteract(e, el.id, 'rotate')}
                      ></div>
                      <!-- Corner resize handles -->
                      <!-- svelte-ignore a11y_no_static_element_interactions -->
                      <div class="se-handle se-handle-nw" onmousedown={(e) => startInteract(e, el.id, 'resize-nw')}></div>
                      <!-- svelte-ignore a11y_no_static_element_interactions -->
                      <div class="se-handle se-handle-ne" onmousedown={(e) => startInteract(e, el.id, 'resize-ne')}></div>
                      <!-- svelte-ignore a11y_no_static_element_interactions -->
                      <div class="se-handle se-handle-sw" onmousedown={(e) => startInteract(e, el.id, 'resize-sw')}></div>
                      <!-- svelte-ignore a11y_no_static_element_interactions -->
                      <div class="se-handle se-handle-se" onmousedown={(e) => startInteract(e, el.id, 'resize-se')}></div>
                    {/if}
                  </div>
                {/each}

                <!-- Booth icons preview -->
                <div class="se-booth-icons">
                  {#if boothIconModes.print}
                    <div class="se-booth-icon">
                      <div class="se-icon-circle" style="background:{themePrimary}">
                        <svg viewBox="0 0 24 24" fill="white" width="28" height="28"><path d="M19 8H5c-1.66 0-3 1.34-3 3v6h4v4h12v-4h4v-6c0-1.66-1.34-3-3-3zm-3 11H8v-5h8v5zm3-7c-.55 0-1-.45-1-1s.45-1 1-1 1 .45 1 1-.45 1-1 1zm-1-9H6v4h12V3z"/></svg>
                      </div>
                      {#if welcomeSettings.boothIconLabels}<span class="se-icon-label" style="color:{themePrimary}">Print</span>{/if}
                    </div>
                  {/if}
                  {#if boothIconModes.gif}
                    <div class="se-booth-icon">
                      <div class="se-icon-circle" style="background:{themePrimary}">
                        <svg viewBox="0 0 24 24" fill="white" width="28" height="28"><path d="M11.5 2C6.81 2 3 5.81 3 10.5S6.81 19 11.5 19h.5v3c4.86-2.34 8-7 8-11.5C20 5.81 16.19 2 11.5 2zm1 14.5h-2v-2h2v2zm0-4h-2c0-3.25 3-3 3-5 0-1.1-.9-2-2-2s-2 .9-2 2h-2c0-2.21 1.79-4 4-4s4 1.79 4 4c0 2.5-3 2.75-3 5z"/></svg>
                      </div>
                      {#if welcomeSettings.boothIconLabels}<span class="se-icon-label" style="color:{themePrimary}">GIF</span>{/if}
                    </div>
                  {/if}
                  {#if boothIconModes.slowMo}
                    <div class="se-booth-icon">
                      <div class="se-icon-circle" style="background:{themePrimary}">
                        <svg viewBox="0 0 24 24" fill="white" width="28" height="28"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 14.5v-9l6 4.5-6 4.5z"/></svg>
                      </div>
                      {#if welcomeSettings.boothIconLabels}<span class="se-icon-label" style="color:{themePrimary}">360/Slow-mo</span>{/if}
                    </div>
                  {/if}
                  {#if boothIconModes.video}
                    <div class="se-booth-icon">
                      <div class="se-icon-circle" style="background:{themePrimary}">
                        <svg viewBox="0 0 24 24" fill="white" width="28" height="28"><path d="M17 10.5V7c0-.55-.45-1-1-1H4c-.55 0-1 .45-1 1v10c0 .55.45 1 1 1h12c.55 0 1-.45 1-1v-3.5l4 4v-11l-4 4z"/></svg>
                      </div>
                      {#if welcomeSettings.boothIconLabels}<span class="se-icon-label" style="color:{themePrimary}">Video</span>{/if}
                    </div>
                  {/if}
                </div>
              {:else if screenTab === "capture"}
                <div class="se-empty-screen">
                  <span>📷</span>
                  <p>Capture Screen</p>
                  <p class="se-empty-sub">Configure your capture screen layout here</p>
                </div>
              {:else}
                <!-- SHARING SCREEN PREVIEW -->
                <div class="ss-preview">
                  <!-- Left: media type tiles -->
                  <div class="ss-tiles">
                    <div class="ss-tile">Photos</div>
                    <div class="ss-tile">Print</div>
                    <div class="ss-tile">GIF</div>
                  </div>

                  <!-- Center: photo placeholder -->
                  <div class="ss-photo-area">
                    <div class="ss-photo-placeholder">
                      <span>�</span>
                      <p>FINAL PHOTO/VIDEO</p>
                    </div>
                  </div>

                  <!-- Top-right: Done button -->
                  {#if sharingSettings.doneButton}
                    <button class="ss-done-btn">Done</button>
                  {/if}

                  <!-- Right/Left sharing icons -->
                  <div class="ss-share-icons" class:left={sharingSettings.iconsLocation === 'Left'} class:bottom={sharingSettings.iconsLocation === 'Bottom'}>
                    {#if sharingSettings.email}
                      <div class="ss-share-item">
                        {#if sharingSettings.textLabels}<span class="ss-share-label">Email</span>{/if}
                        <div class="ss-share-circle" style="background:#e8a020">
                          <svg viewBox="0 0 24 24" fill="white" width="24" height="24"><path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/></svg>
                        </div>
                      </div>
                    {/if}
                    {#if sharingSettings.sms}
                      <div class="ss-share-item">
                        {#if sharingSettings.textLabels}<span class="ss-share-label">SMS</span>{/if}
                        <div class="ss-share-circle" style="background:#22c55e">
                          <svg viewBox="0 0 24 24" fill="white" width="24" height="24"><path d="M20 2H4c-1.1 0-2 .9-2 2v18l4-4h14c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2z"/></svg>
                        </div>
                      </div>
                    {/if}
                    {#if sharingSettings.qrCode}
                      <div class="ss-share-item">
                        {#if sharingSettings.textLabels}<span class="ss-share-label">Scan QR</span>{/if}
                        <div class="ss-share-circle" style="background:#7c3aed">
                          <svg viewBox="0 0 24 24" fill="white" width="24" height="24"><path d="M3 11h8V3H3v8zm2-6h4v4H5V5zM3 21h8v-8H3v8zm2-6h4v4H5v-4zM13 3v8h8V3h-8zm6 6h-4V5h4v4zM13 13h2v2h-2zm2 2h2v2h-2zm2-2h2v2h-2zm-2 4h2v2h-2zm2 0h2v2h-2z"/></svg>
                        </div>
                      </div>
                    {/if}
                    {#if sharingSettings.print}
                      <div class="ss-share-item">
                        {#if sharingSettings.textLabels}<span class="ss-share-label">Print</span>{/if}
                        <div class="ss-share-circle" style="background:{themePrimary}">
                          <svg viewBox="0 0 24 24" fill="white" width="24" height="24"><path d="M19 8H5c-1.66 0-3 1.34-3 3v6h4v4h12v-4h4v-6c0-1.66-1.34-3-3-3zm-3 11H8v-5h8v5zm3-7c-.55 0-1-.45-1-1s.45-1 1-1 1 .45 1 1-.45 1-1 1zm-1-9H6v4h12V3z"/></svg>
                        </div>
                      </div>
                    {/if}
                    {#if sharingSettings.whatsApp}
                      <div class="ss-share-item">
                        {#if sharingSettings.textLabels}<span class="ss-share-label">WhatsApp</span>{/if}
                        <div class="ss-share-circle" style="background:#25d366">
                          <svg viewBox="0 0 24 24" fill="white" width="24" height="24"><path d="M17.472 14.382c-.297-.149-1.758-.867-2.03-.967-.273-.099-.471-.148-.67.15-.197.297-.767.966-.94 1.164-.173.199-.347.223-.644.075-.297-.15-1.255-.463-2.39-1.475-.883-.788-1.48-1.761-1.653-2.059-.173-.297-.018-.458.13-.606.134-.133.298-.347.446-.52.149-.174.198-.298.298-.497.099-.198.05-.371-.025-.52-.075-.149-.669-1.612-.916-2.207-.242-.579-.487-.5-.669-.51-.173-.008-.371-.01-.57-.01-.198 0-.52.074-.792.372-.272.297-1.04 1.016-1.04 2.479 0 1.462 1.065 2.875 1.213 3.074.149.198 2.096 3.2 5.077 4.487.709.306 1.262.489 1.694.625.712.227 1.36.195 1.871.118.571-.085 1.758-.719 2.006-1.413.248-.694.248-1.289.173-1.413-.074-.124-.272-.198-.57-.347z"/><path d="M12 0C5.373 0 0 5.373 0 12c0 2.127.558 4.126 1.533 5.858L.057 23.5l5.797-1.52A11.95 11.95 0 0012 24c6.627 0 12-5.373 12-12S18.627 0 12 0zm0 22c-1.885 0-3.65-.52-5.16-1.426l-.37-.22-3.44.902.918-3.352-.24-.386A9.944 9.944 0 012 12C2 6.477 6.477 2 12 2s10 4.477 10 10-4.477 10-10 10z"/></svg>
                        </div>
                      </div>
                    {/if}
                  </div>
                </div>
              {/if}
            </div>

            <!-- Bottom layer controls -->
            <div class="se-layer-controls">
              <button class="se-layer-btn" onclick={bringForward}>
                <span>⬆</span> Bring Forward
              </button>
              <button class="se-layer-btn" onclick={sendBackward}>
                <span>⬇</span> Send Backward
              </button>
            </div>
          </div>

          <!-- Right panel -->
          <div class="se-panel">
            <div class="se-panel-tabs">
              <button
                class="se-panel-tab"
                class:active={editorPanel === "design"}
                onclick={() => editorPanel = "design"}
              >✏️ DESIGN</button>
              <button
                class="se-panel-tab"
                class:active={editorPanel === "settings"}
                onclick={() => editorPanel = "settings"}
              >⚙️ SETTINGS</button>
            </div>

            <div class="se-panel-body">
              {#if editorPanel === "design"}
                <div class="se-design-panel">
                  <!-- Font size slider (shown when text selected) -->
                  {#if getSelected()?.type === "text"}
                    {@const sel = getSelected()!}
                    <div class="se-font-row">
                      <input
                        type="range" min="8" max="200"
                        value={sel.fontSize ?? 40}
                        oninput={(e) => patchSelected({ fontSize: Number((e.target as HTMLInputElement).value) })}
                        class="se-font-slider"
                      />
                      <input
                        type="number" min="8" max="200"
                        value={sel.fontSize ?? 40}
                        oninput={(e) => patchSelected({ fontSize: Number((e.target as HTMLInputElement).value) })}
                        class="se-font-num"
                      />
                      <span class="se-font-unit">px</span>
                    </div>
                    <div class="se-prop-group">
                      <label class="se-prop-label">Text Color</label>
                      <input type="color" value={sel.color ?? "#ffffff"}
                        oninput={(e) => patchSelected({ color: (e.target as HTMLInputElement).value })}
                        class="se-color-input" />
                    </div>
                    <div class="se-prop-group">
                      <label class="se-prop-label">Content</label>
                      <input class="se-prop-input" type="text" value={sel.text ?? ""}
                        oninput={(e) => patchSelected({ text: (e.target as HTMLInputElement).value })} />
                    </div>
                  {/if}

                  <!-- Theme Colors -->
                  <div class="se-section-header" onclick={() => colorsOpen = !colorsOpen}>
                    <span>Theme Colors</span>
                    <span class="se-chevron">{colorsOpen ? "∧" : "∨"}</span>
                  </div>
                  {#if colorsOpen}
                    <div class="se-section-body">
                      <div class="se-color-row">
                        <div class="se-color-info">
                          <span class="se-color-name">Primary</span>
                          <span class="se-color-desc">Changes primary buttons and border colors</span>
                        </div>
                        <input type="color" bind:value={themePrimary} class="se-color-swatch" />
                      </div>
                      <div class="se-color-row">
                        <div class="se-color-info">
                          <span class="se-color-name">Background</span>
                          <span class="se-color-desc">Changes background color for all screens</span>
                        </div>
                        <input type="color" bind:value={themeBackground} class="se-color-swatch" />
                      </div>
                    </div>
                  {/if}

                  <!-- Alignment -->
                  <div class="se-section-header" onclick={() => alignOpen = !alignOpen}>
                    <span>Alignment</span>
                    <span class="se-chevron">{alignOpen ? "∧" : "∨"}</span>
                  </div>
                  {#if alignOpen}
                    <div class="se-section-body">
                      <div class="se-align-grid">
                        <button class="se-align-btn" title="Align Left"    onclick={() => align('left')}   >⬛▪▪</button>
                        <button class="se-align-btn" title="Center H"      onclick={() => align('hcenter')} >▪⬛▪</button>
                        <button class="se-align-btn" title="Align Right"   onclick={() => align('right')}  >▪▪⬛</button>
                        <button class="se-align-btn" title="Distribute H"  onclick={() => {}}>⬛▪⬛</button>
                        <button class="se-align-btn" title="Align Top"     onclick={() => align('top')}    >⬛</button>
                        <button class="se-align-btn" title="Center V"      onclick={() => align('vcenter')} >▪⬛</button>
                        <button class="se-align-btn" title="Align Bottom"  onclick={() => align('bottom')} >▪▪⬛</button>
                        <button class="se-align-btn" title="Distribute V"  onclick={() => {}}>⬛▪⬛</button>
                      </div>
                    </div>
                  {/if}

                  <!-- Layers -->
                  <div class="se-section-header" onclick={() => layersOpen = !layersOpen}>
                    <span>Layers</span>
                    <span class="se-chevron">{layersOpen ? "∧" : "∨"}</span>
                  </div>
                  {#if layersOpen}
                    <div class="se-layers-list">
                      {#each [...canvasElements].reverse() as el (el.id)}
                        <!-- svelte-ignore a11y_no_static_element_interactions -->
                        <div
                          class="se-layer-item"
                          class:active={selectedElementId === el.id}
                          onmousedown={() => selectedElementId = el.id}
                        >
                          <div class="se-layer-thumb">
                            {#if el.type === "image" && el.src}
                              <img src={el.src} alt={el.label} />
                            {:else if el.type === "text"}
                              <span style="font-size:10px;color:#fff">{el.text ?? "T"}</span>
                            {:else}
                              <span style="font-size:16px">🖼</span>
                            {/if}
                          </div>
                          <span class="se-layer-name">{el.label}</span>
                        </div>
                      {/each}
                    </div>
                  {/if}

                  {#if selectedElementId}
                    <button class="se-delete-btn" onclick={deleteSelected}>🗑 Delete element</button>
                  {/if}
                </div>
              {:else}
                <!-- SETTINGS panel -->
                <div class="se-settings-panel">
                  <h3 class="se-settings-title">
                    {screenTab === "welcome" ? "Welcome Screen Settings" : screenTab === "capture" ? "Capture Screen Settings" : "Sharing Screen Settings"}
                  </h3>

                  {#if screenTab === "welcome"}
                    <!-- Booth Icons -->
                    <div class="se-setting-row">
                      <div class="se-setting-info">
                        <span class="se-setting-name">Booth Icons</span>
                        <span class="se-setting-desc">If Off, touching anywhere on the screen will start a session. Set booth mode to use under Capture Settings.</span>
                        <a href="#" class="se-setting-link" onclick={(e) => e.preventDefault()}>Capture Settings ›</a>
                      </div>
                      <button class="se-toggle" class:on={welcomeSettings.boothIcons} onclick={() => welcomeSettings.boothIcons = !welcomeSettings.boothIcons} role="switch" aria-checked={welcomeSettings.boothIcons}>
                        <span class="se-toggle-knob"></span>
                        <span class="se-toggle-label">{welcomeSettings.boothIcons ? "ON" : "OFF"}</span>
                      </button>
                    </div>

                    {#if welcomeSettings.boothIcons}
                      <div class="se-icon-modes">
                        <label class="se-icon-mode-row"><input type="checkbox" bind:checked={boothIconModes.print} /><span>Print</span></label>
                        <label class="se-icon-mode-row"><input type="checkbox" bind:checked={boothIconModes.gif} /><span>GIF</span></label>
                        <label class="se-icon-mode-row"><input type="checkbox" bind:checked={boothIconModes.slowMo} /><span>360/Slow-mo</span></label>
                        <label class="se-icon-mode-row"><input type="checkbox" bind:checked={boothIconModes.video} /><span>Video</span></label>
                      </div>
                    {/if}

                    <div class="se-setting-row">
                      <div class="se-setting-info">
                        <span class="se-setting-name">Booth icons labels</span>
                        <span class="se-setting-desc">Display text labels below icons</span>
                      </div>
                      <button class="se-toggle" class:on={welcomeSettings.boothIconLabels} onclick={() => welcomeSettings.boothIconLabels = !welcomeSettings.boothIconLabels} role="switch" aria-checked={welcomeSettings.boothIconLabels}>
                        <span class="se-toggle-knob"></span>
                        <span class="se-toggle-label">{welcomeSettings.boothIconLabels ? "ON" : "OFF"}</span>
                      </button>
                    </div>

                    <div class="se-setting-row">
                      <div class="se-setting-info">
                        <span class="se-setting-name">Show live view</span>
                        <span class="se-setting-desc">Displays live camera preview</span>
                      </div>
                      <button class="se-toggle" class:on={welcomeSettings.showLiveView} onclick={() => welcomeSettings.showLiveView = !welcomeSettings.showLiveView} role="switch" aria-checked={welcomeSettings.showLiveView}>
                        <span class="se-toggle-knob"></span>
                        <span class="se-toggle-label">{welcomeSettings.showLiveView ? "ON" : "OFF"}</span>
                      </button>
                    </div>

                    <div class="se-setting-row">
                      <div class="se-setting-info">
                        <span class="se-setting-name">Gallery button</span>
                        <span class="se-setting-desc">Allow guests to see previous sessions</span>
                      </div>
                      <button class="se-toggle" class:on={welcomeSettings.galleryButton} onclick={() => welcomeSettings.galleryButton = !welcomeSettings.galleryButton} role="switch" aria-checked={welcomeSettings.galleryButton}>
                        <span class="se-toggle-knob"></span>
                        <span class="se-toggle-label">{welcomeSettings.galleryButton ? "ON" : "OFF"}</span>
                      </button>
                    </div>

                    <div class="se-setting-block">
                      <span class="se-setting-name">Start screen video</span>
                      <span class="se-setting-desc">Animation will play in a loop before the welcome screen appears.</span>
                      <button class="se-video-drop" onclick={handleFileChoose}>
                        {#if welcomeSettings.startScreenVideoName}
                          <span class="se-video-name">🎬 {welcomeSettings.startScreenVideoName}</span>
                        {:else}
                          <span class="se-video-choose">Choose</span>
                        {/if}
                      </button>
                    </div>

                    <a href="#" class="se-setting-link se-virtual-link" onclick={(e) => e.preventDefault()}>Virtual Attendant ›</a>
                  {:else if screenTab === "sharing"}
                    <!-- Skip Sharing Screen -->
                    <div class="se-setting-row">
                      <div class="se-setting-info">
                        <span class="se-setting-name">Skip Sharing Screen</span>
                      </div>
                      <button class="se-toggle" class:on={sharingSettings.skipSharingScreen} onclick={() => sharingSettings.skipSharingScreen = !sharingSettings.skipSharingScreen} role="switch" aria-checked={sharingSettings.skipSharingScreen}>
                        <span class="se-toggle-knob"></span>
                        <span class="se-toggle-label">{sharingSettings.skipSharingScreen ? "ON" : "OFF"}</span>
                      </button>
                    </div>

                    <!-- Done button -->
                    <div class="se-setting-row">
                      <div class="se-setting-info">
                        <span class="se-setting-name">Done button</span>
                      </div>
                      <button class="se-toggle" class:on={sharingSettings.doneButton} onclick={() => sharingSettings.doneButton = !sharingSettings.doneButton} role="switch" aria-checked={sharingSettings.doneButton}>
                        <span class="se-toggle-knob"></span>
                        <span class="se-toggle-label">{sharingSettings.doneButton ? "ON" : "OFF"}</span>
                      </button>
                    </div>

                    <!-- Sharing icons location -->
                    <div class="se-setting-block">
                      <span class="se-setting-name">Sharing icons location</span>
                      <select class="se-setting-select" bind:value={sharingSettings.iconsLocation}>
                        <option value="Right">Right</option>
                        <option value="Left">Left</option>
                        <option value="Bottom">Bottom</option>
                      </select>
                    </div>

                    <!-- Text labels -->
                    <div class="se-setting-row">
                      <div class="se-setting-info">
                        <span class="se-setting-name">Text labels</span>
                      </div>
                      <button class="se-toggle" class:on={sharingSettings.textLabels} onclick={() => sharingSettings.textLabels = !sharingSettings.textLabels} role="switch" aria-checked={sharingSettings.textLabels}>
                        <span class="se-toggle-knob"></span>
                        <span class="se-toggle-label">{sharingSettings.textLabels ? "ON" : "OFF"}</span>
                      </button>
                    </div>

                    <!-- Final screen timeout -->
                    <div class="se-setting-block">
                      <span class="se-setting-name">Final screen timeout</span>
                      <span class="se-setting-desc">Duration to display sharing screen</span>
                      <div class="se-timeout-row">
                        <input type="range" min="5" max="120" bind:value={sharingSettings.finalScreenTimeout} class="se-font-slider" />
                        <span class="se-timeout-val">{sharingSettings.finalScreenTimeout}</span>
                        <span class="se-font-unit">SEC</span>
                      </div>
                    </div>

                    <!-- Show original photos -->
                    <div class="se-setting-row">
                      <div class="se-setting-info">
                        <span class="se-setting-name">Show original photos</span>
                        <span class="se-setting-desc">Show single images without template</span>
                      </div>
                      <button class="se-toggle" class:on={sharingSettings.showOriginalPhotos} onclick={() => sharingSettings.showOriginalPhotos = !sharingSettings.showOriginalPhotos} role="switch" aria-checked={sharingSettings.showOriginalPhotos}>
                        <span class="se-toggle-knob"></span>
                        <span class="se-toggle-label">{sharingSettings.showOriginalPhotos ? "ON" : "OFF"}</span>
                      </button>
                    </div>

                    <!-- Retake button -->
                    <div class="se-setting-row">
                      <div class="se-setting-info">
                        <span class="se-setting-name">Retake button</span>
                      </div>
                      <button class="se-toggle" class:on={sharingSettings.retakeButton} onclick={() => sharingSettings.retakeButton = !sharingSettings.retakeButton} role="switch" aria-checked={sharingSettings.retakeButton}>
                        <span class="se-toggle-knob"></span>
                        <span class="se-toggle-label">{sharingSettings.retakeButton ? "ON" : "OFF"}</span>
                      </button>
                    </div>

                    <!-- Share method toggles -->
                    <div class="se-setting-row">
                      <div class="se-setting-info"><span class="se-setting-name">Email</span></div>
                      <button class="se-toggle" class:on={sharingSettings.email} onclick={() => sharingSettings.email = !sharingSettings.email} role="switch" aria-checked={sharingSettings.email}>
                        <span class="se-toggle-knob"></span>
                        <span class="se-toggle-label">{sharingSettings.email ? "ON" : "OFF"}</span>
                      </button>
                    </div>
                    <div class="se-setting-row">
                      <div class="se-setting-info"><span class="se-setting-name">Sms</span></div>
                      <button class="se-toggle" class:on={sharingSettings.sms} onclick={() => sharingSettings.sms = !sharingSettings.sms} role="switch" aria-checked={sharingSettings.sms}>
                        <span class="se-toggle-knob"></span>
                        <span class="se-toggle-label">{sharingSettings.sms ? "ON" : "OFF"}</span>
                      </button>
                    </div>
                    <div class="se-setting-row">
                      <div class="se-setting-info"><span class="se-setting-name">QR code</span></div>
                      <button class="se-toggle" class:on={sharingSettings.qrCode} onclick={() => sharingSettings.qrCode = !sharingSettings.qrCode} role="switch" aria-checked={sharingSettings.qrCode}>
                        <span class="se-toggle-knob"></span>
                        <span class="se-toggle-label">{sharingSettings.qrCode ? "ON" : "OFF"}</span>
                      </button>
                    </div>
                    <div class="se-setting-row">
                      <div class="se-setting-info"><span class="se-setting-name">Print</span></div>
                      <button class="se-toggle" class:on={sharingSettings.print} onclick={() => sharingSettings.print = !sharingSettings.print} role="switch" aria-checked={sharingSettings.print}>
                        <span class="se-toggle-knob"></span>
                        <span class="se-toggle-label">{sharingSettings.print ? "ON" : "OFF"}</span>
                      </button>
                    </div>
                    <div class="se-setting-row">
                      <div class="se-setting-info"><span class="se-setting-name">WhatsApp</span></div>
                      <button class="se-toggle" class:on={sharingSettings.whatsApp} onclick={() => sharingSettings.whatsApp = !sharingSettings.whatsApp} role="switch" aria-checked={sharingSettings.whatsApp}>
                        <span class="se-toggle-knob"></span>
                        <span class="se-toggle-label">{sharingSettings.whatsApp ? "ON" : "OFF"}</span>
                      </button>
                    </div>

                    <a href="#" class="se-setting-link se-virtual-link" onclick={(e) => e.preventDefault()}>Sharing Settings ›</a>
                  {:else}
                    <p class="se-panel-hint">Settings for this screen will appear here.</p>
                  {/if}
                </div>
              {/if}
            </div>
          </div>
        </div>
      </div>
  </main>
</div>

<style>
.event-dashboard {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: #2a2a2a;
  color: #ffffff;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
  overflow: hidden;
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
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 0;
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

/* ══════════════════════════════════════════════════════════
   SCREEN EDITOR
══════════════════════════════════════════════════════════ */
.se-root {
  display: flex;
  flex-direction: column;
  flex: 1;
  overflow: hidden;
  background: #2a2a2a;
}

.se-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #333;
  border-bottom: 1px solid #222;
  padding: 6px 12px;
  gap: 4px;
  flex-shrink: 0;
}

.se-toolbar-right {
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.se-photo-layout-btn {
  background: transparent;
  border: 1px solid #555;
  color: #ccc;
  border-radius: 6px;
  padding: 6px 14px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s, color 0.15s, border-color 0.15s;
  white-space: nowrap;
}

.se-photo-layout-btn:hover {
  background: #c0395a;
  border-color: #c0395a;
  color: #fff;
}

.se-toolbar-left {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-wrap: wrap;
}

.se-tool-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  background: transparent;
  border: none;
  color: #ccc;
  cursor: pointer;
  padding: 6px 10px;
  border-radius: 6px;
  transition: background 0.15s, color 0.15s;
  min-width: 44px;
}

.se-tool-btn:hover {
  background: #444;
  color: #fff;
}

.se-tool-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.se-tool-btn:disabled:hover {
  background: transparent;
  color: #ccc;
}

.se-tool-icon {
  font-size: 18px;
  line-height: 1;
}

.se-tool-label {
  font-size: 10px;
  color: #aaa;
}

.se-divider {
  width: 1px;
  height: 36px;
  background: #555;
  margin: 0 6px;
}

.se-aspect-group,
.se-orient-group {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin: 0 4px;
}

.se-aspect-label {
  font-size: 10px;
  color: #aaa;
}

.se-select {
  background: #444;
  color: #fff;
  border: 1px solid #555;
  border-radius: 4px;
  padding: 4px 6px;
  font-size: 12px;
  cursor: pointer;
}

.se-orient-btns {
  display: flex;
  gap: 4px;
}

.se-orient-btn {
  background: #444;
  border: 1px solid #555;
  color: #ccc;
  border-radius: 4px;
  padding: 4px 8px;
  font-size: 14px;
  cursor: pointer;
  transition: background 0.15s;
}

.se-orient-btn.active {
  background: #c0395a;
  border-color: #c0395a;
  color: #fff;
}

/* Sub-tabs */
.se-subtabs {
  display: flex;
  background: #2d2d2d;
  border-bottom: 1px solid #1a1a1a;
  padding: 0 16px;
  flex-shrink: 0;
}

.se-subtab {
  background: transparent;
  border: none;
  border-bottom: 3px solid transparent;
  color: #888;
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.5px;
  padding: 12px 20px;
  transition: color 0.15s, border-color 0.15s;
}

.se-subtab:hover { color: #ccc; }

.se-subtab.active {
  color: #fff;
  border-bottom-color: #c0395a;
}

/* Body */
.se-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* Canvas */
.se-canvas-wrap {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: #1e1e1e;
}

.se-canvas {
  flex: 1;
  position: relative;
  background: #2c2c2c;
  margin: 16px;
  border-radius: 8px;
  overflow: hidden;
  user-select: none;
}

.se-canvas.portrait {
  aspect-ratio: 9/16;
  max-width: 320px;
  margin: 16px auto;
}

/* Empty screen placeholders */
.se-empty-screen {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: #555;
}

.se-empty-screen span { font-size: 48px; }
.se-empty-screen p { font-size: 18px; color: #777; margin: 0; }
.se-empty-sub { font-size: 13px !important; color: #555 !important; }

/* ── Sharing Screen Preview ─────────────────────────────────────────── */
.ss-preview {
  position: absolute;
  inset: 0;
  display: flex;
  overflow: hidden;
}

.ss-tiles {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 16px 12px;
  flex-shrink: 0;
}

.ss-tile {
  width: 100px;
  height: 80px;
  border: 1px solid #555;
  border-radius: 6px;
  background: #1a1a1a;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  color: #ccc;
  cursor: pointer;
}

.ss-photo-area {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.ss-photo-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-end;
  padding-bottom: 16px;
  gap: 8px;
  background: #1e1e1e;
}

.ss-photo-placeholder span {
  font-size: 64px;
  opacity: 0.2;
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -60%);
}

.ss-photo-placeholder p {
  font-size: 13px;
  color: #666;
  letter-spacing: 1px;
  margin: 0;
  position: absolute;
  bottom: 12px;
  left: 50%;
  transform: translateX(-50%);
  white-space: nowrap;
}

.ss-done-btn {
  position: absolute;
  top: 12px;
  right: 12px;
  background: #c0395a;
  color: white;
  border: none;
  border-radius: 24px;
  padding: 10px 28px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  z-index: 10;
}

.ss-share-icons {
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 16px;
  padding: 16px 12px;
  flex-shrink: 0;
}

.ss-share-icons.left {
  order: -1;
}

.ss-share-icons.bottom {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  flex-direction: row;
  justify-content: center;
  padding: 12px;
  background: rgba(0,0,0,0.4);
}

.ss-share-item {
  display: flex;
  align-items: center;
  gap: 10px;
  justify-content: flex-end;
}

.ss-share-icons.bottom .ss-share-item {
  flex-direction: column;
  gap: 4px;
}

.ss-share-label {
  font-size: 12px;
  color: #ccc;
}

.ss-share-circle {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-shadow: 0 2px 8px rgba(0,0,0,0.3);
}

/* Sharing settings extras */
.se-setting-select {
  background: #3a3a3a;
  border: 1px solid #555;
  color: #fff;
  border-radius: 6px;
  padding: 8px 10px;
  font-size: 12px;
  cursor: pointer;
  margin-top: 6px;
  width: 100%;
}

.se-setting-select:focus {
  outline: none;
  border-color: #c0395a;
}

.se-timeout-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 6px;
}

.se-timeout-val {
  font-size: 13px;
  color: #fff;
  min-width: 24px;
  text-align: right;
}

/* Draggable elements */
.se-element {
  position: absolute;
  cursor: move;
  border: 2px solid transparent;
  border-radius: 2px;
}

.se-element.selected {
  border: 2px dashed #ffffff;
}

.se-element.text-el.selected {
  border: 2px dashed #ffffff;
}

/* Rotation handle */
.se-handle {
  position: absolute;
  z-index: 10;
}

.se-rotate-handle {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: #22c55e;
  top: -24px;
  left: 50%;
  transform: translateX(-50%);
  cursor: grab;
  border: 2px solid #fff;
}

.se-rotate-handle:active { cursor: grabbing; }

/* Corner resize handles */
.se-handle-nw, .se-handle-ne, .se-handle-sw, .se-handle-se {
  width: 12px;
  height: 12px;
  background: #3b5bdb;
  border: 1px solid #fff;
  border-radius: 2px;
}

.se-handle-nw { top: -6px;  left: -6px;  cursor: nw-resize; }
.se-handle-ne { top: -6px;  right: -6px; cursor: ne-resize; }
.se-handle-sw { bottom: -6px; left: -6px;  cursor: sw-resize; }
.se-handle-se { bottom: -6px; right: -6px; cursor: se-resize; }

/* Text element display */
.se-text-display {
  display: flex;
  width: 100%;
  height: 100%;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  pointer-events: none;
  user-select: none;
  text-align: center;
  word-break: break-word;
  overflow: hidden;
}

.se-text-editor {
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.55);
  border: none;
  outline: none;
  resize: none;
  color: #ffffff;
  font-size: inherit;
  font-weight: 600;
  text-align: center;
  padding: 4px;
  font-family: inherit;
  caret-color: #fff;
}

.se-img-placeholder {
  width: 100%;
  height: 100%;
  background: #3a3a3a;
  border: 2px dashed #555;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.se-img-icon {
  font-size: 36px;
  opacity: 0.5;
}

.se-text-el {
  font-size: 16px;
  color: #fff;
  white-space: nowrap;
}

/* Booth icons on canvas */
.se-booth-icons {
  position: absolute;
  bottom: 60px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 24px;
  align-items: flex-end;
}

.se-booth-icon {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.se-icon-circle {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  background: #c0395a;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 12px rgba(192, 57, 90, 0.4);
}

.se-icon-label {
  font-size: 12px;
  color: #c0395a;
  font-weight: 500;
}

/* Layer controls */
.se-layer-controls {
  display: flex;
  gap: 8px;
  padding: 8px 16px;
  background: #252525;
  border-top: 1px solid #333;
  flex-shrink: 0;
}

.se-layer-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  background: #3a3a3a;
  border: 1px solid #555;
  color: #ccc;
  padding: 6px 14px;
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.15s;
}

.se-layer-btn:hover {
  background: #4a4a4a;
  color: #fff;
}

/* Right panel */
.se-panel {
  width: 280px;
  flex-shrink: 0;
  background: #2d2d2d;
  border-left: 1px solid #222;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.se-panel-tabs {
  display: flex;
  border-bottom: 1px solid #222;
  flex-shrink: 0;
}

.se-panel-tab {
  flex: 1;
  background: transparent;
  border: none;
  border-bottom: 3px solid transparent;
  color: #888;
  cursor: pointer;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.4px;
  padding: 10px 8px;
  transition: color 0.15s, border-color 0.15s;
}

.se-panel-tab:hover { color: #ccc; }

.se-panel-tab.active {
  color: #fff;
  border-bottom-color: #c0395a;
}

.se-panel-body {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
}

/* Design panel */
.se-design-panel {
  display: flex;
  flex-direction: column;
  gap: 0;
}

/* Font size row */
.se-font-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 0 14px;
  border-bottom: 1px solid #3a3a3a;
  margin-bottom: 4px;
}

.se-font-slider {
  flex: 1;
  accent-color: #c0395a;
  cursor: pointer;
}

.se-font-num {
  width: 48px;
  background: #3a3a3a;
  border: 1px solid #555;
  color: #fff;
  border-radius: 4px;
  padding: 4px 6px;
  font-size: 12px;
  text-align: center;
}

.se-font-unit {
  font-size: 11px;
  color: #888;
}

/* Collapsible section headers */
.se-section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0 8px;
  font-size: 13px;
  font-weight: 700;
  color: #fff;
  cursor: pointer;
  border-bottom: 1px solid #3a3a3a;
  user-select: none;
}

.se-chevron {
  font-size: 11px;
  color: #888;
}

.se-section-body {
  padding: 10px 0;
  border-bottom: 1px solid #3a3a3a;
}

/* Color rows */
.se-color-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 6px 0;
}

.se-color-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.se-color-name {
  font-size: 12px;
  color: #ddd;
  font-weight: 500;
}

.se-color-desc {
  font-size: 10px;
  color: #888;
}

.se-color-swatch {
  width: 52px;
  height: 32px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  padding: 0;
  background: none;
  flex-shrink: 0;
}

.se-color-input {
  width: 40px;
  height: 28px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  padding: 0;
}

/* Alignment grid */
.se-align-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 4px;
}

.se-align-btn {
  background: #3a3a3a;
  border: 1px solid #555;
  color: #ccc;
  border-radius: 4px;
  padding: 6px 4px;
  font-size: 11px;
  cursor: pointer;
  transition: background 0.15s;
  text-align: center;
}

.se-align-btn:hover {
  background: #4a4a4a;
  color: #fff;
}

/* Layers list */
.se-layers-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px 0;
  border-bottom: 1px solid #3a3a3a;
}

.se-layer-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 8px;
  border-radius: 6px;
  cursor: pointer;
  background: #333;
  transition: background 0.15s;
}

.se-layer-item:hover { background: #3d3d3d; }

.se-layer-item.active {
  background: #3b5bdb;
}

.se-layer-thumb {
  width: 40px;
  height: 40px;
  background: #222;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  flex-shrink: 0;
  border: 1px solid #444;
}

.se-layer-thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.se-layer-name {
  font-size: 12px;
  color: #ddd;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.se-panel-hint {
  font-size: 12px;
  color: #666;
  text-align: center;
  padding: 16px 0;
}

.se-prop-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.se-prop-label {
  font-size: 11px;
  color: #aaa;
}

.se-prop-input {
  background: #3a3a3a;
  border: 1px solid #555;
  color: #fff;
  border-radius: 4px;
  padding: 6px 8px;
  font-size: 13px;
  width: 100%;
}

.se-prop-input:focus {
  outline: none;
  border-color: #c0395a;
}

.se-delete-btn {
  background: #5a1a1a;
  border: 1px solid #7a2a2a;
  color: #fc8181;
  border-radius: 6px;
  padding: 8px;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.15s;
  margin-top: 8px;
}

.se-delete-btn:hover {
  background: #7a2a2a;
}

/* Settings panel */
.se-settings-panel {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.se-settings-title {
  font-size: 13px;
  font-weight: 600;
  color: #fff;
  margin: 0 0 12px 0;
}

.se-setting-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
  padding: 10px 0;
  border-bottom: 1px solid #3a3a3a;
}

.se-setting-block {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px 0;
  border-bottom: 1px solid #3a3a3a;
}

.se-setting-info {
  display: flex;
  flex-direction: column;
  gap: 3px;
  flex: 1;
}

.se-setting-name {
  font-size: 12px;
  font-weight: 600;
  color: #ddd;
}

.se-setting-desc {
  font-size: 11px;
  color: #888;
  line-height: 1.4;
}

.se-setting-link {
  font-size: 11px;
  color: #c0395a;
  text-decoration: none;
  margin-top: 2px;
}

.se-setting-link:hover {
  text-decoration: underline;
}

/* Toggle switch */
.se-toggle {
  display: flex;
  align-items: center;
  gap: 4px;
  background: #555;
  border: none;
  border-radius: 20px;
  padding: 3px 8px 3px 4px;
  cursor: pointer;
  transition: background 0.2s;
  flex-shrink: 0;
  min-width: 56px;
}

.se-toggle.on {
  background: #c0395a;
}

.se-toggle-knob {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: white;
  transition: transform 0.2s;
  flex-shrink: 0;
}

.se-toggle.on .se-toggle-knob {
  transform: translateX(0);
}

.se-toggle-label {
  font-size: 10px;
  font-weight: 700;
  color: white;
  letter-spacing: 0.3px;
}

/* Icon mode checkboxes */
.se-icon-modes {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 6px 0 10px 8px;
  border-bottom: 1px solid #3a3a3a;
}

.se-icon-mode-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: #ccc;
  cursor: pointer;
}

.se-icon-mode-row input[type="checkbox"] {
  width: 14px;
  height: 14px;
  cursor: pointer;
  accent-color: #c0395a;
}

/* Video drop zone */
.se-video-drop {
  border: 2px dashed #555;
  border-radius: 8px;
  background: #222;
  color: #888;
  padding: 24px;
  text-align: center;
  cursor: pointer;
  font-size: 13px;
  transition: border-color 0.2s, background 0.2s;
  width: 100%;
  margin-top: 4px;
}

.se-video-drop:hover {
  border-color: #c0395a;
  background: #2a1a20;
  color: #ccc;
}

.se-video-name {
  color: #c0395a;
  font-size: 11px;
  word-break: break-all;
}

.se-virtual-link {
  margin-top: 12px;
  font-size: 12px;
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
