<script lang="ts">
  import { goto } from "$app/navigation";
  import { createEvent } from "$lib/database";

  let eventName = $state("");
  let eventDate = $state("");
  let eventTime = $state("");
  let eventLocation = $state("");
  let eventDescription = $state("");
  let maxPhotos = $state(50);
  let errorMsg = $state("");
  let successMsg = $state("");
  let isLoading = $state(false);

  async function handleCreateEvent(event: Event) {
    event.preventDefault();
    errorMsg = "";
    successMsg = "";
    
    // Basic validation
    if (!eventName || !eventDate || !eventTime) {
      errorMsg = "Please fill in required fields (Name, Date, Time)";
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
        max_photos: maxPhotos
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
          rows="4"
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
  max-width: 600px;
  width: 100%;
}

.header {
  margin-bottom: 30px;
  text-align: center;
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
}

@media (max-width: 640px) {
  .form-row {
    grid-template-columns: 1fr;
  }

  .event-card {
    padding: 30px 20px;
  }
}
</style>
