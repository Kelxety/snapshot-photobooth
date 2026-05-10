<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";

  let eventId = $derived($page.params.id);

  // ── Settings ──────────────────────────────────────────────────────────────
  let viewDimension = $state("Pixels");
  let paperSize = $state("4 x 6");
  let printToSecondary = $state(false);
  let resolution = $state("300 dpi");
  let orientation = $state("Horizontal");

  // ── Templates ─────────────────────────────────────────────────────────────
  interface Template { id: string; name: string; photoCount: number; cols: number; rows: number; }
  const TEMPLATES: Template[] = [
    { id: "1x1", name: "1 Photo",        photoCount: 1, cols: 1, rows: 1 },
    { id: "2h",  name: "2 Photos H",     photoCount: 2, cols: 2, rows: 1 },
    { id: "2v",  name: "2 Photos V",     photoCount: 2, cols: 1, rows: 2 },
    { id: "2x2", name: "4 Photos",       photoCount: 4, cols: 2, rows: 2 },
    { id: "3h",  name: "3 Photos H",     photoCount: 3, cols: 3, rows: 1 },
    { id: "4h",  name: "4 Photos strip", photoCount: 4, cols: 4, rows: 1 },
    { id: "2x3", name: "6 Photos",       photoCount: 6, cols: 3, rows: 2 },
  ];
  let selectedTemplateId = $state("2x2");
  let selectedTemplate = $derived(TEMPLATES.find(t => t.id === selectedTemplateId) ?? TEMPLATES[3]);
  const SLOT_COLORS = ["#4a7c59","#5b8fa8","#9b59b6","#e67e22","#e74c3c","#1abc9c"];

  // ── Layer model ───────────────────────────────────────────────────────────
  // All layers are absolutely positioned so they can overlap freely.
  interface CLayer {
    id: string; name: string; visible: boolean;
    type: "photo" | "frame" | "bg";
    x: number; y: number; w: number; h: number;
    rotation: number; color: string; zIndex: number;
  }

  // Logical canvas size (px). Actual display is scaled.
  const CW = 800, CH = 533, MIN = 40;

  function buildLayers(t: Template): CLayer[] {
    const pad = 10, gap = 4;
    const sw = Math.floor((CW - pad * 2 - (t.cols - 1) * gap) / t.cols);
    const sh = Math.floor((CH - pad * 2 - (t.rows - 1) * gap) / t.rows);
    const photos: CLayer[] = Array.from({ length: t.photoCount }, (_, i) => {
      const col = i % t.cols, row = Math.floor(i / t.cols);
      return {
        id: `photo-${i + 1}`, name: `Photo ${i + 1}`,
        visible: true, type: "photo" as const,
        x: pad + col * (sw + gap), y: pad + row * (sh + gap),
        w: sw, h: sh, rotation: 0,
        color: SLOT_COLORS[i % SLOT_COLORS.length], zIndex: i + 1,
      };
    });
    return [
      { id: "bg",    name: "Background", visible: true, type: "bg",    x: 0, y: 0, w: CW, h: CH, rotation: 0, color: "#1a1a2e", zIndex: 0 },
      { id: "frame", name: "Frame",      visible: true, type: "frame", x: 0, y: 0, w: CW, h: CH, rotation: 0, color: "#c9a84c", zIndex: photos.length + 1 },
      ...photos,
    ];
  }

  let layers = $state<CLayer[]>(buildLayers(TEMPLATES[3]));
  let selectedId = $state<string | null>("photo-1");

  // ── Canvas ref + scale ────────────────────────────────────────────────────
  let canvasRef = $state<HTMLDivElement | null>(null);
  let sc = $state(1); // display scale: canvasRef.offsetWidth / CW
  $effect(() => {
    if (!canvasRef) return;
    const obs = new ResizeObserver(() => { sc = canvasRef!.offsetWidth / CW; });
    obs.observe(canvasRef);
    sc = canvasRef.offsetWidth / CW;
    return () => obs.disconnect();
  });

  // ── Drag / resize interaction ─────────────────────────────────────────────
  type IMode = "drag" | "resize-nw" | "resize-ne" | "resize-sw" | "resize-se";
  let interact = $state<{ mode: IMode; id: string; sx: number; sy: number; ox: number; oy: number; ow: number; oh: number; } | null>(null);

  function getL(id: string) { return layers.find(l => l.id === id); }
  function patchL(id: string, p: Partial<CLayer>) {
    layers = layers.map(l => l.id === id ? { ...l, ...p } : l);
  }

  function startInteract(e: MouseEvent, id: string, mode: IMode) {
    e.stopPropagation(); e.preventDefault();
    selectedId = id;
    const l = getL(id)!;
    interact = { mode, id, sx: e.clientX, sy: e.clientY, ox: l.x, oy: l.y, ow: l.w, oh: l.h };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", stopI);
  }

  function onMove(e: MouseEvent) {
    if (!interact) return;
    const { mode, id, sx, sy, ox, oy, ow, oh } = interact;
    const dx = (e.clientX - sx) / sc, dy = (e.clientY - sy) / sc;
    if (mode === "drag") patchL(id, { x: ox + dx, y: oy + dy });
    else if (mode === "resize-se") patchL(id, { w: Math.max(MIN, ow + dx), h: Math.max(MIN, oh + dy) });
    else if (mode === "resize-sw") { const nw = Math.max(MIN, ow - dx); patchL(id, { x: ox + ow - nw, w: nw, h: Math.max(MIN, oh + dy) }); }
    else if (mode === "resize-ne") { const nh = Math.max(MIN, oh - dy); patchL(id, { y: oy + oh - nh, w: Math.max(MIN, ow + dx), h: nh }); }
    else if (mode === "resize-nw") { const nw = Math.max(MIN, ow - dx), nh = Math.max(MIN, oh - dy); patchL(id, { x: ox + ow - nw, y: oy + oh - nh, w: nw, h: nh }); }
  }

  function stopI() {
    interact = null;
    window.removeEventListener("mousemove", onMove);
    window.removeEventListener("mouseup", stopI);
    pushH();
  }

  // ── Layer panel drag-to-reorder ───────────────────────────────────────────
  // Track last Y position so we move one slot at a time (not jump based on original index)
  let layerDrag = $state<{ id: string; lastY: number; } | null>(null);

  function startLayerDrag(e: MouseEvent, id: string) {
    e.stopPropagation();
    e.preventDefault();
    layerDrag = { id, lastY: e.clientY };
    window.addEventListener("mousemove", onLDMove);
    window.addEventListener("mouseup", stopLD);
  }

  function onLDMove(e: MouseEvent) {
    if (!layerDrag) return;
    const dy = e.clientY - layerDrag.lastY;
    const ROW_H = 46;
    if (Math.abs(dy) < ROW_H) return;
    const dir = dy > 0 ? 1 : -1;
    const arr = [...layers];
    const from = arr.findIndex(l => l.id === layerDrag!.id);
    // Panel is reversed: dragging down in panel = lower z = move toward index 0
    const to = Math.max(0, Math.min(arr.length - 1, from - dir));
    if (from === to) return;
    const [item] = arr.splice(from, 1);
    arr.splice(to, 0, item);
    layers = arr;
    layerDrag = { id: layerDrag.id, lastY: layerDrag.lastY + dir * ROW_H };
  }

  function stopLD() {
    layerDrag = null;
    window.removeEventListener("mousemove", onLDMove);
    window.removeEventListener("mouseup", stopLD);
    pushH();
  }

  // ── Layer operations ──────────────────────────────────────────────────────
  function toggleVis(id: string) { patchL(id, { visible: !getL(id)!.visible }); pushH(); }

  // Array order: index 0 = bottom (bg), last index = top (frame).
  // "Bring Forward" = move toward end of array (higher z).
  // "Send Backward" = move toward start of array (lower z).
  // Layers panel shows top-first (reversed array order).
  function bringFwd() {
    if (!selectedId) return;
    const i = layers.findIndex(l => l.id === selectedId);
    // Don't go above frame (last item)
    if (i < layers.length - 1) {
      const a = [...layers];
      [a[i], a[i + 1]] = [a[i + 1], a[i]];
      layers = a;
      pushH();
    }
  }

  function sendBwd() {
    if (!selectedId) return;
    const i = layers.findIndex(l => l.id === selectedId);
    // Don't go below bg (index 0)
    if (i > 0) {
      const a = [...layers];
      [a[i], a[i - 1]] = [a[i - 1], a[i]];
      layers = a;
      pushH();
    }
  }

  function selectTpl(id: string) {
    selectedTemplateId = id;
    layers = buildLayers(TEMPLATES.find(x => x.id === id)!);
    selectedId = "photo-1";
    pushH();
  }

  // ── History ───────────────────────────────────────────────────────────────
  type HE = { tid: string; layers: CLayer[] };
  let hStack: HE[] = [{ tid: selectedTemplateId, layers: JSON.parse(JSON.stringify(layers)) }];
  let hIdx = $state(0);

  function pushH() {
    hStack = hStack.slice(0, hIdx + 1);
    hStack.push({ tid: selectedTemplateId, layers: JSON.parse(JSON.stringify(layers)) });
    hIdx = hStack.length - 1;
  }

  function undo() {
    if (hIdx <= 0) return;
    hIdx--;
    const e = hStack[hIdx];
    selectedTemplateId = e.tid;
    layers = JSON.parse(JSON.stringify(e.layers));
  }

  function redo() {
    if (hIdx >= hStack.length - 1) return;
    hIdx++;
    const e = hStack[hIdx];
    selectedTemplateId = e.tid;
    layers = JSON.parse(JSON.stringify(e.layers));
  }

  // ── Helpers ───────────────────────────────────────────────────────────────
  function getSlots(t: Template) { return Array.from({ length: t.photoCount }, (_, i) => i + 1); }
  function saveAsNew() { alert("Saved as new copy."); }

  function getSelectedRotation() {
    const l = layers.find(x => x.id === selectedId);
    return l ? Math.round(l.rotation) : 0;
  }

  function setSelectedRotation(val: number) {
    if (selectedId) patchL(selectedId, { rotation: val });
  }

  // Layers render in array order — index 0 = bottom, last = top.
  // z-index is derived from position so reordering the array immediately
  // changes what's on top on the canvas.
  // NO separate sortedLayers — we render in array order directly.
</script>

<div class="pl-root">

  <!-- Header -->
  <header class="pl-header">
    <button class="pl-nav-btn" onclick={() => goto(`/event/${eventId}`)}>‹ Screen Editor</button>
    <h1 class="pl-title">Photo Layout</h1>
    <button class="pl-nav-btn" onclick={() => goto(`/event/${eventId}`)}>General ›</button>
  </header>

  <!-- Sub-header -->
  <div class="pl-subheader">
    <span class="pl-tpl-info">{selectedTemplate.name} · {paperSize}</span>
    <div class="pl-actions">
      <button class="pl-act-btn">＋ Add layout</button>
      <button class="pl-act-btn">🛒 Shop templates</button>
      <button class="pl-act-btn" onclick={saveAsNew}>💾 Save as New</button>
      <button class="pl-act-btn">📥 Import</button>
      <button class="pl-act-btn">📤 Export</button>
      <div class="pl-vdiv"></div>
      <button class="pl-act-btn" onclick={undo} disabled={hIdx <= 0}>↩ Undo</button>
      <button class="pl-act-btn" onclick={redo} disabled={hIdx >= hStack.length - 1}>↪ Redo</button>
      <button class="pl-act-btn">🖨 Print test Copy</button>
      <button class="pl-act-btn danger" onclick={() => selectTpl("2x2")}>🗑 Delete Template</button>
    </div>
  </div>

  <!-- Body -->
  <div class="pl-body">

    <!-- Left panel -->
    <div class="pl-left">
      <p class="pl-sec-title">Add</p>
      <button class="pl-lb">📷 Photo From Booth</button>
      <button class="pl-lb">🖼 Image</button>
      <button class="pl-lb">T&nbsp; Text</button>
      <button class="pl-lb">⬟ Shape</button>
      <button class="pl-lb">🎨 Background Color</button>
      <button class="pl-lb">⬛ QR Code</button>
      <button class="pl-lb">👤 Session Data <span class="pl-arr">›</span></button>

      <div class="pl-sep"></div>
      <p class="pl-sec-title">Layout</p>

      <div class="pl-tpl-grid">
        {#each TEMPLATES as t}
          <button
            class="pl-tpl-btn"
            class:active={selectedTemplateId === t.id}
            onclick={() => selectTpl(t.id)}
            title={t.name}
          >
            <div class="pl-tpl-inner"
              style="grid-template-columns:repeat({t.cols},1fr);grid-template-rows:repeat({t.rows},1fr)">
              {#each getSlots(t) as n, i}
                <div class="pl-tpl-cell" style="background:{SLOT_COLORS[i % SLOT_COLORS.length]}">{n}</div>
              {/each}
            </div>
          </button>
        {/each}
      </div>

      <div class="pl-sep"></div>
      <p class="pl-sec-title">Settings</p>

      <div class="pl-field">
        <label>View Dimension In</label>
        <select bind:value={viewDimension}>
          <option>Pixels</option><option>Inches</option><option>Centimeters</option>
        </select>
      </div>
      <div class="pl-field">
        <label>Paper Size</label>
        <select bind:value={paperSize}>
          <option>4 x 6</option><option>5 x 7</option><option>6 x 8</option><option>A4</option>
        </select>
      </div>
      <label class="pl-check">
        <input type="checkbox" bind:checked={printToSecondary} /> Print to Secondary Printer
      </label>
      <div class="pl-field">
        <label>Resolution</label>
        <select bind:value={resolution}>
          <option>72 dpi</option><option>150 dpi</option><option>300 dpi</option><option>600 dpi</option>
        </select>
      </div>
      <div class="pl-field">
        <label>Orientation</label>
        <select bind:value={orientation}>
          <option>Horizontal</option><option>Vertical</option>
        </select>
      </div>
    </div>

    <!-- Canvas -->
    <div class="pl-canvas-wrap">
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="pl-canvas"
        class:vertical={orientation === "Vertical"}
        bind:this={canvasRef}
        onmousedown={() => selectedId = null}
      >
        {#each layers as layer, idx (layer.id)}
          {#if layer.visible}
            {#if layer.type === "bg"}
              <div class="pl-layer-bg" style="background:{layer.color};z-index:{idx}"></div>
            {:else if layer.type === "frame"}
              <div class="pl-layer-frame" style="border-color:{layer.color};z-index:{idx}"></div>
            {:else}
              {@const isSel = selectedId === layer.id}
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="pl-slot"
                class:pl-sel={isSel}
                style="left:{layer.x * sc}px;top:{layer.y * sc}px;width:{layer.w * sc}px;height:{layer.h * sc}px;background:{layer.color};transform:rotate({layer.rotation}deg);z-index:{idx}"
                onmousedown={(e) => startInteract(e, layer.id, 'drag')}
              >
                <span class="pl-slot-num">{layer.name.replace('Photo ', '')}</span>

                {#if isSel}
                  <!-- Rotation handle -->
                  <div class="pl-rot-handle"></div>
                  <!-- Corner resize handles -->
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <div class="pl-h pl-nw" onmousedown={(e) => startInteract(e, layer.id, 'resize-nw')}></div>
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <div class="pl-h pl-ne" onmousedown={(e) => startInteract(e, layer.id, 'resize-ne')}></div>
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <div class="pl-h pl-sw" onmousedown={(e) => startInteract(e, layer.id, 'resize-sw')}></div>
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <div class="pl-h pl-se" onmousedown={(e) => startInteract(e, layer.id, 'resize-se')}></div>
                {/if}
              </div>
            {/if}
          {/if}
        {/each}
      </div>

      <div class="pl-footer">
        <button class="pl-foot-btn" onclick={bringFwd}>⬆ Bring Forward</button>
        <button class="pl-foot-btn" onclick={sendBwd}>⬇ Send Backward</button>
      </div>
    </div>

    <!-- Right panel -->
    <div class="pl-right">
      <!-- Rotate -->
      <div class="pl-right-row">
        <span class="pl-rl">Rotate</span>
        <input
          type="number" min="0" max="360"
          value={getSelectedRotation()}
          oninput={(e) => setSelectedRotation(Number((e.target as HTMLInputElement).value))}
          class="pl-rot-input"
        />
      </div>

      <!-- Alignment -->
      <div class="pl-rsec">
        <div class="pl-rsec-hdr"><span>Alignment</span><span>∧</span></div>
        <div class="pl-align-grid">
          <button class="pl-al-btn" title="Align Left">⬛▪▪</button>
          <button class="pl-al-btn" title="Center H">▪⬛▪</button>
          <button class="pl-al-btn" title="Align Right">▪▪⬛</button>
          <button class="pl-al-btn" title="Distribute H">⬛▪⬛</button>
          <button class="pl-al-btn" title="Align Top">⬛</button>
          <button class="pl-al-btn" title="Center V">▪⬛</button>
          <button class="pl-al-btn" title="Align Bottom">▪▪⬛</button>
          <button class="pl-al-btn" title="Distribute V">⬛▪⬛</button>
        </div>
      </div>

      <!-- Layers -->
      <div class="pl-rsec">
        <div class="pl-rsec-hdr"><span>Layers</span><span>∧</span></div>
        <div class="pl-layers">
          {#each [...layers].reverse() as layer (layer.id)}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="pl-layer-row"
              class:active={selectedId === layer.id}
              onmousedown={() => selectedId = layer.id}
            >
              <!-- Eye toggle -->
              <button
                class="pl-eye"
                onclick={(e) => { e.stopPropagation(); toggleVis(layer.id); }}
                title={layer.visible ? "Hide" : "Show"}
              >{layer.visible ? "👁" : "🚫"}</button>

              <!-- Thumbnail — mirrors canvas color -->
              <div
                class="pl-lthumb"
                style="background:{layer.visible ? (layer.color ?? '#333') : '#1a1a1a'};opacity:{layer.visible ? 1 : 0.4}"
              >
                {#if layer.type === "photo"}
                  <span class="pl-lnum">{layer.name.replace("Photo ", "")}</span>
                {:else if layer.type === "frame"}
                  <div class="pl-lframe-thumb" style="border-color:{layer.color}"></div>
                {:else}
                  <span class="pl-lnum" style="font-size:8px">BG</span>
                {/if}
              </div>

              <span class="pl-lname" style="color:{layer.visible ? '#ddd' : '#666'}">{layer.name}</span>

              <!-- Drag handle -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <span
                class="pl-drag-handle"
                onmousedown={(e) => startLayerDrag(e, layer.id)}
                title="Drag to reorder"
              >⋮</span>
            </div>
          {/each}
        </div>
      </div>
    </div>

  </div>
</div>

<style>
* { box-sizing: border-box; margin: 0; padding: 0; }

.pl-root {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #2a2a2a;
  color: #fff;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  overflow: hidden;
}

/* Header */
.pl-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  background: #1e1e1e;
  border-bottom: 1px solid #111;
  flex-shrink: 0;
}
.pl-nav-btn {
  background: transparent;
  border: none;
  color: #aaa;
  font-size: 13px;
  cursor: pointer;
  padding: 6px 10px;
  border-radius: 6px;
  transition: color .15s, background .15s;
}
.pl-nav-btn:hover { color: #fff; background: #333; }
.pl-title { font-size: 18px; font-weight: 600; }

/* Sub-header */
.pl-subheader {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 14px;
  background: #252525;
  border-bottom: 1px solid #111;
  flex-shrink: 0;
  flex-wrap: wrap;
}
.pl-tpl-info { font-size: 12px; color: #888; margin-right: 6px; }
.pl-actions { display: flex; align-items: center; gap: 2px; flex-wrap: wrap; }
.pl-act-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  background: transparent;
  border: none;
  color: #ccc;
  font-size: 12px;
  cursor: pointer;
  padding: 5px 9px;
  border-radius: 5px;
  transition: background .15s, color .15s;
  white-space: nowrap;
}
.pl-act-btn:hover { background: #3a3a3a; color: #fff; }
.pl-act-btn:disabled { opacity: .35; cursor: not-allowed; }
.pl-act-btn.danger:hover { background: #5a1a1a; color: #fc8181; }
.pl-vdiv { width: 1px; height: 22px; background: #444; margin: 0 4px; }

/* Body */
.pl-body { display: flex; flex: 1; overflow: hidden; }

/* Left panel */
.pl-left {
  width: 190px;
  flex-shrink: 0;
  background: #222;
  border-right: 1px solid #111;
  overflow-y: auto;
  padding: 10px 0;
}
.pl-sec-title {
  font-size: 10px;
  font-weight: 700;
  color: #777;
  letter-spacing: .5px;
  padding: 6px 12px 3px;
  text-transform: uppercase;
}
.pl-lb {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  background: transparent;
  border: none;
  color: #ccc;
  font-size: 12px;
  padding: 7px 12px;
  cursor: pointer;
  text-align: left;
  transition: background .15s;
}
.pl-lb:hover { background: #2d2d2d; color: #fff; }
.pl-arr { margin-left: auto; color: #555; }
.pl-sep { height: 1px; background: #333; margin: 8px 0; }

/* Template grid */
.pl-tpl-grid { display: grid; grid-template-columns: repeat(3,1fr); gap: 5px; padding: 6px 8px; }
.pl-tpl-btn {
  aspect-ratio: 1;
  background: #1a1a1a;
  border: 2px solid #444;
  border-radius: 4px;
  cursor: pointer;
  padding: 3px;
  transition: border-color .15s;
  overflow: hidden;
}
.pl-tpl-btn:hover { border-color: #888; }
.pl-tpl-btn.active { border-color: #c0395a; }
.pl-tpl-inner { display: grid; width: 100%; height: 100%; gap: 2px; }
.pl-tpl-cell {
  border-radius: 2px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 7px;
  color: rgba(255,255,255,.7);
  font-weight: 700;
}

/* Settings fields */
.pl-field { display: flex; flex-direction: column; gap: 3px; padding: 5px 12px; }
.pl-field label { font-size: 10px; color: #777; }
.pl-field select {
  background: #333;
  border: 1px solid #555;
  color: #fff;
  border-radius: 4px;
  padding: 4px 7px;
  font-size: 11px;
  cursor: pointer;
  width: 100%;
}
.pl-field select:focus { outline: none; border-color: #c0395a; }
.pl-check { display: flex; align-items: center; gap: 7px; padding: 5px 12px; font-size: 11px; color: #ccc; cursor: pointer; }
.pl-check input { accent-color: #c0395a; }

/* Canvas area */
.pl-canvas-wrap {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: #1a1a1a;
  overflow: hidden;
}

.pl-canvas {
  position: relative;
  width: min(88%, 800px);
  aspect-ratio: 3/2;
  overflow: hidden;
  border-radius: 4px;
  user-select: none;
  cursor: default;
}
.pl-canvas.vertical { aspect-ratio: 2/3; width: min(55%, 400px); }

/* Background layer */
.pl-layer-bg { position: absolute; inset: 0; z-index: 0; }

/* Frame layer — sits on top of everything */
.pl-layer-frame {
  position: absolute;
  inset: 0;
  z-index: 9999;
  border: 10px solid #c9a84c;
  border-radius: 3px;
  pointer-events: none;
}

/* Photo slots — absolutely positioned, can overlap */
.pl-slot {
  position: absolute;
  cursor: move;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 3px;
  outline: 2px solid transparent;
  outline-offset: 1px;
  transition: outline-color .1s;
}
.pl-slot:hover { outline-color: rgba(255,255,255,0.3); }
.pl-sel { outline: 2px dashed #ffffff; }

.pl-slot-num {
  font-size: clamp(16px, 4vw, 56px);
  font-weight: 900;
  color: rgba(255,255,255,.55);
  pointer-events: none;
  user-select: none;
  text-shadow: 0 2px 6px rgba(0,0,0,.5);
}

/* Resize handles */
.pl-h {
  position: absolute;
  width: 12px;
  height: 12px;
  background: #3b5bdb;
  border: 2px solid #fff;
  border-radius: 2px;
  z-index: 10;
}
.pl-nw { top: -6px;    left: -6px;   cursor: nw-resize; }
.pl-ne { top: -6px;    right: -6px;  cursor: ne-resize; }
.pl-sw { bottom: -6px; left: -6px;   cursor: sw-resize; }
.pl-se { bottom: -6px; right: -6px;  cursor: se-resize; }

/* Rotation handle */
.pl-rot-handle {
  position: absolute;
  top: -22px;
  left: 50%;
  transform: translateX(-50%);
  width: 12px;
  height: 12px;
  background: #22c55e;
  border: 2px solid #fff;
  border-radius: 50%;
  cursor: grab;
  z-index: 10;
}
.pl-rot-handle:active { cursor: grabbing; }

/* Footer */
.pl-footer {
  display: flex;
  gap: 8px;
  padding: 10px;
  background: #1e1e1e;
  border-top: 1px solid #111;
  width: 100%;
  justify-content: center;
}
.pl-foot-btn {
  background: #333;
  border: 1px solid #555;
  color: #ccc;
  border-radius: 5px;
  padding: 5px 12px;
  font-size: 12px;
  cursor: pointer;
  transition: background .15s;
}
.pl-foot-btn:hover { background: #444; color: #fff; }

/* Right panel */
.pl-right {
  width: 230px;
  flex-shrink: 0;
  background: #222;
  border-left: 1px solid #111;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}
.pl-right-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-bottom: 1px solid #333;
}
.pl-rl { font-size: 12px; color: #aaa; }
.pl-rot-input {
  width: 56px;
  background: #333;
  border: 1px solid #555;
  color: #fff;
  border-radius: 4px;
  padding: 4px 6px;
  font-size: 12px;
  text-align: center;
}
.pl-rot-input:focus { outline: none; border-color: #c0395a; }

.pl-rsec { border-bottom: 1px solid #333; }
.pl-rsec-hdr {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 9px 12px 5px;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  user-select: none;
}
.pl-align-grid { display: grid; grid-template-columns: repeat(4,1fr); gap: 3px; padding: 4px 8px 8px; }
.pl-al-btn {
  background: #333;
  border: 1px solid #555;
  color: #ccc;
  border-radius: 3px;
  padding: 5px 3px;
  font-size: 9px;
  cursor: pointer;
  text-align: center;
  transition: background .15s;
}
.pl-al-btn:hover { background: #444; color: #fff; }

/* Layers list */
.pl-layers { display: flex; flex-direction: column; gap: 2px; padding: 5px 7px 8px; }
.pl-layer-row {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 5px 7px;
  border-radius: 5px;
  cursor: pointer;
  background: #2a2a2a;
  border: 1px solid transparent;
  transition: background .15s;
  user-select: none;
}
.pl-layer-row:hover { background: #333; }
.pl-layer-row.active { background: #3b5bdb; border-color: #4a6ae0; }

.pl-eye {
  background: transparent;
  border: none;
  cursor: pointer;
  font-size: 12px;
  padding: 0;
  flex-shrink: 0;
  line-height: 1;
}

.pl-lthumb {
  width: 36px;
  height: 36px;
  border-radius: 3px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border: 1px solid #444;
  position: relative;
  overflow: hidden;
  transition: background .2s, opacity .2s;
}
.pl-lnum { font-size: 11px; font-weight: 700; color: #fff; }
.pl-lframe-thumb { position: absolute; inset: 3px; border: 3px solid #c9a84c; border-radius: 2px; }

.pl-lname {
  font-size: 11px;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pl-drag-handle {
  color: #555;
  font-size: 18px;
  cursor: grab;
  flex-shrink: 0;
  line-height: 1;
  padding: 0 2px;
}
.pl-drag-handle:hover { color: #aaa; }
.pl-drag-handle:active { cursor: grabbing; }
</style>
