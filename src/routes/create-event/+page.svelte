<script lang="ts">
  import { goto } from "$app/navigation";
  import { createEvent } from "$lib/database";
  import TemplateEditor from "./TemplateEditor.svelte";

  interface PhotoBox {
    id: number;
    x: number;
    y: number;
    width: number;
    height: number;
    color: string;
    number: number;
  }

  let eventName = $state("");
  let eventDate = $state("");
  let eventTime = $state("");
  let eventLocation = $state("");
  let eventDescription = $state("");
  let maxPhotos = $state(50);
  let paperSize = $state("4R");
  let templateImage = $state<string | null>(null);
  let templateFileName = $state<string>("");
  let photoBoxes = $state<PhotoBox[]>([]);
  let errorMsg = $state("");
  let successMsg = $state("");
  let isLoading = $state(false);

  const paperSizes = [
    { 
      name: '4R', 
      value: '4R', 
      dimensions: '4×6 inches',
      pixels: '1200×1800 px',
      description: 'Standard photo print size'
    },
    { 
      name: '5R', 
      value: '5R', 
      dimensions: '5×7 inches',
      pixels: '1500×2100 px',
      description: 'Larger print format'
    }
  ];

  async function handleTemplateUpload(event: Event & { currentTarget: HTMLInputElement }) {
    const file = event.currentTarget.files?.[0];
    if (!file) return;

    // Validate file type
    if (!file.type.startsWith('image/png')) {
      errorMsg = 'Please upload a PNG image file';
      return;
    }

    // Validate file size (max 5MB)
    if (file.size > 5 * 1024 * 1024) {
      errorMsg = 'Template image must be less than 5MB';
      return;
    }

    // Read file as base64
    const reader = new FileReader();
    reader.onload = (e) => {
      templateImage = e.target?.result as string;
      templateFileName = file.name;
      errorMsg = '';
    };
    reader.onerror = () => {
      errorMsg = 'Failed to read template image';
    };
    reader.readAsDataURL(file);
  }

  function removeTemplate() {
    templateImage = null;
    templateFileName = '';
  }

  async function handleCreateEvent(event: Event) {
    event.preventDefault();
    errorMsg = "";
    successMsg = "";
    
    // Basic validation
    if (!eventName || !eventDate || !eventTime) {
      errorMsg = "Please fill in required fields (Name, Date, Time)";
      return;
    }

    if (!templateImage) {
      errorMsg = "Please upload a template image";
      return;
    }

    if (photoBoxes.length === 0) {
      errorMsg = "Please add at least one photo box to the template";
      return;
    }

    isLoading = true;

    try {
      // Save event to database
      const eventId = await createEvent({
        name: eventName,
        date: eventDate,
        time: eventTime,
        location: eventLocation,
        description: eventDescription,
        max_photos: maxPhotos,
        paper_size: paperSize,
        template_image: templateImage,
        photo_boxes: JSON.stringify(photoBoxes)
      });
      
      successMsg = "Event created successfully!";
      
      // Redirect to event dashboard after 1 second
      setTimeout(() => {
        goto(`/event/${eventId}`);
      }, 1000);
    } catch (error) {
      errorMsg = "Failed to create event. Please try again.";
      console.error("Event creation error:", error);
    } finally {
      isLoading = false;
    }
  }

  function handleCancel() {
    goto("/home");
  }
</script>

<main class="container">
  <div class="event-card">
    <div class="header">
      <h1>Create New Event</h1>
      <p class="subtitle">Set up your photobooth event</p>
    </div>

    <form class="event-form" onsubmit={handleCreateEvent}>
      <div class="form-layout">
        <!-- Left Column: Form Fields -->
        <div class="form-column">
          <div class="form-group">
            <label for="event-name">Event Name <span class="required">*</span></label>
            <input
              id="event-name"
              type="text"
              placeholder="e.g., Wedding Reception, Birthday Party"
              bind:value={eventName}
              disabled={isLoading}
              required
            />
          </div>

          <div class="form-row">
            <div class="form-group">
              <label for="event-date">Event Date <span class="required">*</span></label>
              <input
                id="event-date"
                type="date"
                bind:value={eventDate}
                disabled={isLoading}
                required
              />
            </div>

            <div class="form-group">
              <label for="event-time">Event Time <span class="required">*</span></label>
              <input
                id="event-time"
                type="time"
                bind:value={eventTime}
                disabled={isLoading}
                required
              />
            </div>
          </div>

          <div class="form-group">
            <label for="event-location">Location</label>
            <input
              id="event-location"
              type="text"
              placeholder="Event venue address"
              bind:value={eventLocation}
              disabled={isLoading}
            />
          </div>

          <div class="form-group">
            <label for="event-description">Description</label>
            <textarea
              id="event-description"
              placeholder="Add any notes or special instructions..."
              bind:value={eventDescription}
              disabled={isLoading}
              rows="3"
            ></textarea>
          </div>

          <div class="form-group">
            <label for="max-photos">Maximum Photos</label>
            <input
              id="max-photos"
              type="number"
              min="1"
              max="1000"
              bind:value={maxPhotos}
              disabled={isLoading}
            />
            <span class="help-text">Set a limit for the number of photos</span>
          </div>
        </div>

        <!-- Right Column: Template Preview -->
        <div class="preview-column">
          <div class="form-group">
            <label>Paper Size <span class="required">*</span></label>
            <p class="help-text">Select print format for your photobooth</p>
            <div class="paper-size-selector">
              {#each paperSizes as size}
                <button
                  type="button"
                  class="size-card"
                  class:selected={paperSize === size.value}
                  onclick={() => paperSize = size.value}
                  disabled={isLoading}
                >
                  <h3>{size.name}</h3>
                  <div class="size-details">
                    <span class="dimensions">{size.dimensions}</span>
                    <span class="pixels">{size.pixels}</span>
                  </div>
                  <p>{size.description}</p>
                </button>
              {/each}
            </div>
          </div>

          <div class="form-group">
            <label>Template Image <span class="required">*</span></label>
            <p class="help-text">Upload a PNG template with transparent areas for photos (max 5MB)</p>
            
            {#if templateImage}
              <div class="template-preview">
                <img src={templateImage} alt="Template preview" />
                <div class="template-info">
                  <span class="file-name">📄 {templateFileName}</span>
                  <button type="button" class="remove-btn" onclick={removeTemplate}>
                    🗑️ Remove
                  </button>
                </div>
              </div>

              <!-- Template Editor -->
              <div class="editor-section">
                <h3>📐 Define Photo Placement</h3>
                <TemplateEditor bind:templateImage bind:photoBoxes />
              </div>
            {:else}
              <div class="upload-area">
                <input
                  type="file"
                  id="template-upload"
                  accept="image/png"
                  onchange={handleTemplateUpload}
                  disabled={isLoading}
                  class="file-input"
                />
                <label for="template-upload" class="upload-label">
                  <div class="upload-icon">📤</div>
                  <span class="upload-text">Click to upload PNG template</span>
                  <span class="upload-hint">Transparent areas will show photos</span>
                </label>
              </div>
            {/if}
          </div>
        </div>
      </div>

      {#if errorMsg}
        <div class="error-message">{errorMsg}</div>
      {/if}

      {#if successMsg}
        <div class="success-message">{successMsg}</div>
      {/if}

      <div class="button-group">
        <button type="button" class="cancel-button" onclick={handleCancel} disabled={isLoading}>
          Cancel
        </button>
        <button type="submit" class="create-button" disabled={isLoading}>
          {isLoading ? "Creating..." : "Create Event"}
        </button>
      </div>
    </form>
  </div>
</main>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  padding: 40px 20px;
}

.event-card {
  background: white;
  border-radius: 12px;
  padding: 40px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  max-width: 1200px;
  width: 100%;
}

.header {
  margin-bottom: 30px;
  text-align: center;
}

.event-form {
  width: 100%;
}

.form-layout {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 40px;
}

.form-column,
.preview-column {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.header h1 {
  font-size: 28px;
  margin: 0 0 10px 0;
  color: #0f0f0f;
}

.subtitle {
  color: #666;
  font-size: 14px;
  margin: 0;
}

.event-form {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 15px;
}

.form-group {
  display: flex;
  flex-direction: column;
  text-align: left;
  gap: 8px;
}

.form-group label {
  font-weight: 500;
  font-size: 14px;
  color: #333;
}

.required {
  color: #ff4444;
}

.form-group input,
.form-group textarea {
  padding: 12px 16px;
  border: 2px solid #e0e0e0;
  border-radius: 8px;
  font-size: 16px;
  transition: all 0.2s ease;
  outline: none;
  font-family: inherit;
}

.form-group textarea {
  resize: vertical;
  min-height: 100px;
}

.form-group input:focus,
.form-group textarea:focus {
  border-color: #646cff;
  box-shadow: 0 0 0 3px rgba(100, 108, 255, 0.1);
}

.form-group input:disabled,
.form-group textarea:disabled {
  background-color: #f5f5f5;
  cursor: not-allowed;
  opacity: 0.6;
}

.help-text {
  font-size: 12px;
  color: #888;
  margin-top: -4px;
}

.error-message {
  padding: 12px;
  background-color: #fee;
  border: 1px solid #fcc;
  border-radius: 6px;
  color: #c33;
  font-size: 14px;
  text-align: center;
}

.success-message {
  padding: 12px;
  background-color: #efe;
  border: 1px solid #cfc;
  border-radius: 6px;
  color: #3c3;
  font-size: 14px;
  text-align: center;
}

.button-group {
  display: flex;
  gap: 12px;
  margin-top: 10px;
}

.cancel-button,
.create-button {
  flex: 1;
  padding: 14px;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.cancel-button {
  background-color: #e0e0e0;
  color: #333;
}

.cancel-button:hover:not(:disabled) {
  background-color: #d0d0d0;
}

.create-button {
  background-color: #646cff;
  color: white;
}

.create-button:hover:not(:disabled) {
  background-color: #535bf2;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(100, 108, 255, 0.3);
}

.create-button:active:not(:disabled) {
  transform: translateY(0);
}

.cancel-button:disabled,
.create-button:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

/* Paper Size Selector */
.paper-size-selector {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 15px;
  margin-top: 15px;
}

.size-card {
  background: #f9f9f9;
  border: 3px solid #e0e0e0;
  border-radius: 12px;
  padding: 20px;
  cursor: pointer;
  transition: all 0.2s;
  text-align: center;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.size-card:hover:not(:disabled) {
  border-color: #646cff;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(100, 108, 255, 0.2);
}

.size-card.selected {
  border-color: #646cff;
  background: rgba(100, 108, 255, 0.1);
}

.size-card:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.size-card h3 {
  font-size: 24px;
  font-weight: 700;
  margin: 0;
  color: #333;
}

.size-details {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.dimensions {
  font-size: 16px;
  font-weight: 600;
  color: #555;
}

.pixels {
  font-size: 13px;
  color: #888;
}

.size-card p {
  font-size: 13px;
  margin: 0;
  color: #666;
}

/* Template Upload */
.upload-area {
  margin-top: 15px;
}

.file-input {
  display: none;
}

.upload-label {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 60px 40px;
  min-height: 400px;
  border: 2px dashed #ccc;
  border-radius: 12px;
  background: #f9f9f9;
  cursor: pointer;
  transition: all 0.2s;
}

.upload-label:hover {
  border-color: #646cff;
  background: rgba(100, 108, 255, 0.05);
}

.upload-icon {
  font-size: 48px;
}

.upload-text {
  font-size: 16px;
  font-weight: 600;
  color: #333;
}

.upload-hint {
  font-size: 13px;
  color: #888;
}

.template-preview {
  margin-top: 15px;
  border: 2px solid #e0e0e0;
  border-radius: 12px;
  padding: 15px;
  background: #f9f9f9;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.template-preview img {
  width: 100%;
  height: 400px;
  object-fit: contain;
  border-radius: 8px;
  background: repeating-conic-gradient(#ddd 0% 25%, white 0% 50%) 50% / 20px 20px;
  margin-bottom: 10px;
  flex: 1;
}

.template-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.file-name {
  font-size: 14px;
  color: #555;
}

.remove-btn {
  background: #ef4444;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.remove-btn:hover {
  background: #dc2626;
}

.editor-section {
  margin-top: 25px;
  padding: 20px;
  background: white;
  border-radius: 12px;
  border: 2px solid #e0e0e0;
}

.editor-section h3 {
  margin: 0 0 15px 0;
  font-size: 18px;
  color: #333;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  .event-card {
    background: #1a1a1a;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
  }

  .header h1 {
    color: #f6f6f6;
  }

  .subtitle {
    color: #a0a0a0;
  }

  .form-group label {
    color: #d0d0d0;
  }

  .form-group input,
  .form-group textarea {
    background-color: #2a2a2a;
    border-color: #444;
    color: #f6f6f6;
  }

  .form-group input:focus,
  .form-group textarea:focus {
    border-color: #7c84ff;
    box-shadow: 0 0 0 3px rgba(124, 132, 255, 0.2);
    background-color: #2f2f2f;
  }

  .form-group input:disabled,
  .form-group textarea:disabled {
    background-color: #1f1f1f;
    color: #888;
  }

  .help-text {
    color: #888;
  }

  .error-message {
    background-color: #3a1a1a;
    border-color: #5a2a2a;
    color: #ff6b6b;
  }

  .success-message {
    background-color: #1a3a1a;
    border-color: #2a5a2a;
    color: #6bff6b;
  }

  .cancel-button {
    background-color: #3a3a3a;
    color: #d0d0d0;
  }

  .cancel-button:hover:not(:disabled) {
    background-color: #4a4a4a;
  }

  .size-card {
    background: #2a2a2a;
    border-color: #444;
  }

  .size-card:hover:not(:disabled) {
    border-color: #7c84ff;
  }

  .size-card.selected {
    border-color: #7c84ff;
    background: rgba(124, 132, 255, 0.2);
  }

  .size-card h3 {
    color: #f6f6f6;
  }

  .dimensions {
    color: #d0d0d0;
  }

  .pixels {
    color: #a0a0a0;
  }

  .size-card p {
    color: #a0a0a0;
  }

  .upload-label {
    background: #2a2a2a;
    border-color: #444;
  }

  .upload-label:hover {
    border-color: #7c84ff;
    background: rgba(124, 132, 255, 0.1);
  }

  .upload-text {
    color: #f6f6f6;
  }

  .upload-hint {
    color: #a0a0a0;
  }

  .template-preview {
    background: #2a2a2a;
    border-color: #444;
  }

  .template-preview img {
    background: repeating-conic-gradient(#3a3a3a 0% 25%, #2a2a2a 0% 50%) 50% / 20px 20px;
  }

  .file-name {
    color: #d0d0d0;
  }

  .editor-section {
    background: #2a2a2a;
    border-color: #444;
  }

  .editor-section h3 {
    color: #f6f6f6;
  }
}

@media (max-width: 640px) {
  .form-row {
    grid-template-columns: 1fr;
  }

  .event-card {
    padding: 30px 20px;
  }

  .form-layout {
    grid-template-columns: 1fr;
    gap: 30px;
  }

  .template-preview img {
    height: 300px;
  }

  .upload-label {
    min-height: 300px;
    padding: 40px 20px;
  }
}
</style>
