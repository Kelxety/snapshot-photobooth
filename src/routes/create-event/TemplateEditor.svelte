<script lang="ts">
  interface PhotoBox {
    id: number;
    x: number;
    y: number;
    width: number;
    height: number;
    color: string;
    number: number;
  }

  let { templateImage = $bindable(), photoBoxes = $bindable([]) }: { 
    templateImage: string;
    photoBoxes: PhotoBox[];
  } = $props();

  let containerRef = $state<HTMLDivElement>();
  let selectedBox = $state<number | null>(null);
  let isDragging = $state(false);
  let isResizing = $state(false);
  let dragStart = $state({ x: 0, y: 0 });
  let nextBoxNumber = $state(1);

  const colors = ['#3b82f6', '#ef4444', '#10b981', '#f59e0b', '#8b5cf6', '#ec4899'];
  let selectedColor = $state('#3b82f6');

  function addPhotoBox() {
    const newBox: PhotoBox = {
      id: Date.now(),
      x: 50,
      y: 50,
      width: 150,
      height: 150,
      color: selectedColor,
      number: nextBoxNumber
    };
    photoBoxes.push(newBox);
    photoBoxes = photoBoxes;
    nextBoxNumber++;
    selectedBox = newBox.id;
  }

  function deleteBox(id: number) {
    photoBoxes = photoBoxes.filter(box => box.id !== id);
    // Renumber remaining boxes
    photoBoxes.sort((a, b) => a.number - b.number).forEach((box, index) => {
      box.number = index + 1;
    });
    nextBoxNumber = photoBoxes.length + 1;
    selectedBox = null;
  }

  function handleMouseDown(e: MouseEvent, boxId: number, isResize: boolean = false) {
    e.stopPropagation();
    selectedBox = boxId;
    dragStart = { x: e.clientX, y: e.clientY };
    
    if (isResize) {
      isResizing = true;
    } else {
      isDragging = true;
    }

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }

  function handleMouseMove(e: MouseEvent) {
    if (!containerRef || selectedBox === null) return;

    const box = photoBoxes.find(b => b.id === selectedBox);
    if (!box) return;

    const dx = e.clientX - dragStart.x;
    const dy = e.clientY - dragStart.y;

    const containerRect = containerRef.getBoundingClientRect();
    const scale = containerRect.width / 1000; // Assuming 1000px base width

    if (isDragging) {
      box.x = Math.max(0, Math.min(1000 - box.width, box.x + dx / scale));
      box.y = Math.max(0, Math.min(1000 - box.height, box.y + dy / scale));
    } else if (isResizing) {
      box.width = Math.max(50, box.width + dx / scale);
      box.height = Math.max(50, box.height + dy / scale);
    }

    dragStart = { x: e.clientX, y: e.clientY };
    photoBoxes = photoBoxes;
  }

  function handleMouseUp() {
    isDragging = false;
    isResizing = false;
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
  }

  function changeBoxColor(boxId: number, color: string) {
    const box = photoBoxes.find(b => b.id === boxId);
    if (box) {
      box.color = color;
      photoBoxes = photoBoxes;
    }
  }
</script>

<div class="template-editor">
  <div class="editor-toolbar">
    <button type="button" class="tool-btn" onclick={addPhotoBox}>
      ➕ Add Photo Box
    </button>
    
    <div class="color-picker">
      <span>Box Color:</span>
      {#each colors as color}
        <button
          type="button"
          class="color-btn"
          class:active={selectedColor === color}
          style="background: {color}"
          onclick={() => selectedColor = color}
        ></button>
      {/each}
    </div>

    {#if selectedBox !== null}
      <button 
        type="button" 
        class="tool-btn delete-btn" 
        onclick={() => deleteBox(selectedBox!)}
      >
        🗑️ Delete Selected
      </button>
    {/if}
  </div>

  <div class="editor-container" bind:this={containerRef}>
    <img src={templateImage} alt="Template" class="template-bg" />
    
    <div class="boxes-layer">
      {#each photoBoxes as box (box.id)}
        <div
          class="photo-box"
          class:selected={selectedBox === box.id}
          style="
            left: {box.x}px;
            top: {box.y}px;
            width: {box.width}px;
            height: {box.height}px;
            border-color: {box.color};
          "
          onmousedown={(e) => handleMouseDown(e, box.id)}
        >
          <div class="box-number" style="background: {box.color}">
            {box.number}
          </div>
          
          <div 
            class="resize-handle"
            onmousedown={(e) => handleMouseDown(e, box.id, true)}
          ></div>

          {#if selectedBox === box.id}
            <div class="box-colors">
              {#each colors as color}
                <button
                  type="button"
                  class="mini-color-btn"
                  style="background: {color}"
                  onclick={() => changeBoxColor(box.id, color)}
                ></button>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </div>

  <div class="editor-info">
    <p>📝 Click "Add Photo Box" to define photo placement areas</p>
    <p>🎯 Drag boxes to reposition, use handle to resize</p>
    <p>🔢 Numbers indicate photo sequence (1st photo → Box 1, 2nd photo → Box 2, etc.)</p>
  </div>
</div>

<style>
.template-editor {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.editor-toolbar {
  display: flex;
  align-items: center;
  gap: 15px;
  padding: 15px;
  background: #f9f9f9;
  border-radius: 12px;
  flex-wrap: wrap;
}

.tool-btn {
  background: #3b82f6;
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.tool-btn:hover {
  background: #2563eb;
  transform: translateY(-1px);
}

.delete-btn {
  background: #ef4444;
}

.delete-btn:hover {
  background: #dc2626;
}

.color-picker {
  display: flex;
  align-items: center;
  gap: 8px;
}

.color-picker span {
  font-size: 14px;
  color: #555;
  font-weight: 500;
}

.color-btn {
  width: 32px;
  height: 32px;
  border-radius: 6px;
  border: 3px solid transparent;
  cursor: pointer;
  transition: all 0.2s;
}

.color-btn:hover {
  transform: scale(1.1);
}

.color-btn.active {
  border-color: #333;
  box-shadow: 0 0 0 2px white;
}

.editor-container {
  position: relative;
  width: 100%;
  max-width: 1000px;
  margin: 0 auto;
  background: repeating-conic-gradient(#ddd 0% 25%, white 0% 50%) 50% / 20px 20px;
  border-radius: 12px;
  overflow: hidden;
  border: 2px solid #e0e0e0;
}

.template-bg {
  width: 100%;
  display: block;
  pointer-events: none;
}

.boxes-layer {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
}

.photo-box {
  position: absolute;
  border: 3px solid;
  background: rgba(255, 255, 255, 0.3);
  cursor: move;
  transition: box-shadow 0.2s;
  user-select: none;
}

.photo-box:hover {
  box-shadow: 0 0 0 2px rgba(0, 0, 0, 0.2);
}

.photo-box.selected {
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.5);
  z-index: 10;
}

.box-number {
  position: absolute;
  top: -12px;
  left: -12px;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 700;
  font-size: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  z-index: 2;
}

.resize-handle {
  position: absolute;
  bottom: -6px;
  right: -6px;
  width: 16px;
  height: 16px;
  background: white;
  border: 2px solid #3b82f6;
  border-radius: 50%;
  cursor: nwse-resize;
  z-index: 2;
}

.box-colors {
  position: absolute;
  top: -50px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 4px;
  background: white;
  padding: 6px;
  border-radius: 8px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.2);
  z-index: 3;
}

.mini-color-btn {
  width: 24px;
  height: 24px;
  border-radius: 4px;
  border: 2px solid white;
  cursor: pointer;
  transition: all 0.2s;
}

.mini-color-btn:hover {
  transform: scale(1.1);
  border-color: #333;
}

.editor-info {
  padding: 15px;
  background: #e0f2fe;
  border-radius: 8px;
  border-left: 4px solid #3b82f6;
}

.editor-info p {
  margin: 5px 0;
  font-size: 14px;
  color: #555;
}

@media (prefers-color-scheme: dark) {
  .editor-toolbar {
    background: #2a2a2a;
  }

  .color-picker span {
    color: #d0d0d0;
  }

  .editor-container {
    background: repeating-conic-gradient(#3a3a3a 0% 25%, #2a2a2a 0% 50%) 50% / 20px 20px;
    border-color: #444;
  }

  .box-colors {
    background: #2a2a2a;
  }

  .editor-info {
    background: #1e3a5f;
    border-color: #3b82f6;
  }

  .editor-info p {
    color: #d0d0d0;
  }
}
</style>
