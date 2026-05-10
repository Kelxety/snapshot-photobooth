<script lang="ts">
  import { goto } from "$app/navigation";
  import { getAllEvents, updateEvent, deleteEvent, createEvent, type Event } from "$lib/database";
  import { onMount } from "svelte";

  let events = $state<Event[]>([]);
  let loading = $state(true);

  // Menu state
  let openMenuId = $state<number | null>(null);

  // Rename modal state
  let renameModalOpen = $state(false);
  let renameEventId = $state<number | null>(null);
  let renameValue = $state("");

  // Delete confirm state
  let deleteModalOpen = $state(false);
  let deleteEventId = $state<number | null>(null);
  let deleteEventName = $state("");

  onMount(async () => {
    try {
      events = await getAllEvents();
    } catch (error) {
      console.error("Error loading events:", error);
    } finally {
      loading = false;
    }

    // Close menu when clicking outside
    function handleClickOutside(e: MouseEvent) {
      const target = e.target as HTMLElement;
      if (!target.closest(".event-menu-wrapper")) {
        openMenuId = null;
      }
    }
    document.addEventListener("click", handleClickOutside);
    return () => document.removeEventListener("click", handleClickOutside);
  });

  function handleLogout() {
    goto("/signin");
  }

  function handleEventClick(eventId: number) {
    goto(`/event/${eventId}`);
  }

  function handleCreateEvent() {
    goto("/create-event");
  }

  function toggleMenu(e: MouseEvent, eventId: number) {
    e.stopPropagation();
    openMenuId = openMenuId === eventId ? null : eventId;
  }

  // --- Rename ---
  function openRename(e: MouseEvent, event: Event) {
    e.stopPropagation();
    openMenuId = null;
    renameEventId = event.id!;
    renameValue = event.name;
    renameModalOpen = true;
  }

  async function confirmRename() {
    if (!renameEventId || !renameValue.trim()) return;
    await updateEvent(renameEventId, { name: renameValue.trim() });
    events = events.map(ev =>
      ev.id === renameEventId ? { ...ev, name: renameValue.trim() } : ev
    );
    renameModalOpen = false;
    renameEventId = null;
  }

  // --- Delete ---
  function openDelete(e: MouseEvent, event: Event) {
    e.stopPropagation();
    openMenuId = null;
    deleteEventId = event.id!;
    deleteEventName = event.name;
    deleteModalOpen = true;
  }

  async function confirmDelete() {
    if (!deleteEventId) return;
    await deleteEvent(deleteEventId);
    events = events.filter(ev => ev.id !== deleteEventId);
    deleteModalOpen = false;
    deleteEventId = null;
  }

  // --- Duplicate ---
  async function duplicateEvent(e: MouseEvent, event: Event) {
    e.stopPropagation();
    openMenuId = null;
    const newId = await createEvent({
      name: `${event.name} (Copy)`,
      date: event.date,
      time: event.time,
      location: event.location,
      description: event.description,
      max_photos: event.max_photos,
      paper_size: event.paper_size,
      template_image: event.template_image,
      photo_boxes: event.photo_boxes,
    });
    events = await getAllEvents();
  }
</script>

<main class="container">
  <div class="header">
    <div class="logo-section">
      <div class="app-logo">📸</div>
      <h1>SnapBooth</h1>
    </div>
    <button class="logout-btn" onclick={handleLogout}>
      <span class="icon">🚪</span>
      Logout
    </button>
  </div>

  <div class="content">
    <div class="welcome-section">
      <h2>Welcome to Your Events</h2>
      <p>Manage and launch your photobooth events</p>
    </div>

    <div class="actions">
      <button class="create-event-btn" onclick={handleCreateEvent}>
        <span class="icon">➕</span>
        Create New Event
      </button>
    </div>

    {#if loading}
      <div class="loading">Loading events...</div>
    {:else if events.length === 0}
      <div class="no-events">
        <p>No events yet. Create your first event to get started!</p>
      </div>
    {:else}
      <div class="events-grid">
        {#each events as event}
          <div class="event-card-wrapper">
            <button
              class="event-card"
              onclick={() => event.id && handleEventClick(event.id)}
            >
              <div class="event-icon">🎉</div>
              <div class="event-details">
                <h3 class="event-name">{event.name}</h3>
                <div class="event-meta">
                  <span class="date">📅 {event.date}</span>
                  <span class="time">🕐 {event.time}</span>
                </div>
                {#if event.location}
                  <div class="event-location">📍 {event.location}</div>
                {/if}
                {#if event.max_photos}
                  <div class="event-capacity">📷 {event.max_photos} photos max</div>
                {/if}
              </div>
            </button>

            <!-- Three-dot menu -->
            <div class="event-menu-wrapper">
              <button
                class="menu-trigger"
                onclick={(e) => toggleMenu(e, event.id!)}
                title="More options"
                aria-label="More options"
              >
                ⋯
              </button>
              {#if openMenuId === event.id}
                <div class="dropdown-menu" role="menu">
                  <button class="menu-item" onclick={(e) => openRename(e, event)}>
                    ✏️ Rename
                  </button>
                  <button class="menu-item" onclick={(e) => duplicateEvent(e, event)}>
                    📋 Duplicate
                  </button>
                  <button class="menu-item danger" onclick={(e) => openDelete(e, event)}>
                    🗑️ Delete
                  </button>
                </div>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</main>

<!-- Rename Modal -->
{#if renameModalOpen}
  <div class="modal-backdrop" onclick={() => (renameModalOpen = false)}>
    <div class="modal" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" aria-labelledby="rename-title">
      <h3 id="rename-title">Rename Event</h3>
      <input
        class="modal-input"
        type="text"
        bind:value={renameValue}
        onkeydown={(e) => e.key === "Enter" && confirmRename()}
        autofocus
      />
      <div class="modal-actions">
        <button class="modal-btn secondary" onclick={() => (renameModalOpen = false)}>Cancel</button>
        <button class="modal-btn primary" onclick={confirmRename}>Rename</button>
      </div>
    </div>
  </div>
{/if}

<!-- Delete Confirm Modal -->
{#if deleteModalOpen}
  <div class="modal-backdrop" onclick={() => (deleteModalOpen = false)}>
    <div class="modal" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" aria-labelledby="delete-title">
      <h3 id="delete-title">Delete Event</h3>
      <p class="modal-body">Are you sure you want to delete <strong>{deleteEventName}</strong>? This cannot be undone.</p>
      <div class="modal-actions">
        <button class="modal-btn secondary" onclick={() => (deleteModalOpen = false)}>Cancel</button>
        <button class="modal-btn danger" onclick={confirmDelete}>Delete</button>
      </div>
    </div>
  </div>
{/if}

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

:root {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

.container {
  min-height: 100vh;
  background: #000000;
  color: white;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 40px;
  background: rgba(0, 0, 0, 0.2);
  backdrop-filter: blur(10px);
}

.logo-section {
  display: flex;
  align-items: center;
  gap: 15px;
}

.app-logo {
  font-size: 48px;
  text-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
}

.logo-section h1 {
  font-size: 32px;
  font-weight: 700;
  margin: 0;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.logout-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(255, 255, 255, 0.2);
  border: 1px solid rgba(255, 255, 255, 0.3);
  color: white;
  padding: 10px 20px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s;
}

.logout-btn:hover {
  background: rgba(255, 255, 255, 0.3);
  transform: translateY(-2px);
}

.content {
  max-width: 1200px;
  margin: 0 auto;
  padding: 40px 20px;
}

.welcome-section {
  text-align: center;
  margin-bottom: 40px;
}

.welcome-section h2 {
  font-size: 36px;
  font-weight: 700;
  margin-bottom: 10px;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.welcome-section p {
  font-size: 18px;
  opacity: 0.9;
}

.actions {
  display: flex;
  justify-content: center;
  margin-bottom: 40px;
}

.create-event-btn {
  display: flex;
  align-items: center;
  gap: 10px;
  background: white;
  color: #667eea;
  border: none;
  padding: 15px 30px;
  border-radius: 12px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  transition: all 0.3s;
}

.create-event-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.2);
}

.create-event-btn .icon {
  font-size: 20px;
}

.loading {
  text-align: center;
  font-size: 18px;
  padding: 40px;
  opacity: 0.8;
}

.no-events {
  text-align: center;
  padding: 60px 20px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  backdrop-filter: blur(10px);
}

.no-events p {
  font-size: 18px;
  opacity: 0.9;
}

.events-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 24px;
  padding: 20px 0;
}

/* Card wrapper holds the card + the menu button */
.event-card-wrapper {
  position: relative;
}

.event-card {
  width: 100%;
  background: rgba(255, 255, 255, 0.95);
  color: #333;
  border: none;
  border-radius: 16px;
  padding: 24px;
  padding-right: 48px; /* room for the menu button */
  cursor: pointer;
  transition: all 0.3s;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  text-align: left;
  display: flex;
  gap: 16px;
  align-items: flex-start;
}

.event-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
  background: white;
}

/* Three-dot button — sits in the top-right corner of the wrapper */
.event-menu-wrapper {
  position: absolute;
  top: 12px;
  right: 12px;
}

.menu-trigger {
  background: transparent;
  border: none;
  color: #718096;
  font-size: 20px;
  line-height: 1;
  width: 32px;
  height: 32px;
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s, color 0.2s;
  letter-spacing: 1px;
}

.menu-trigger:hover {
  background: rgba(0, 0, 0, 0.08);
  color: #2d3748;
}

.dropdown-menu {
  position: absolute;
  top: calc(100% + 4px);
  right: 0;
  background: white;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  min-width: 150px;
  z-index: 100;
  overflow: hidden;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 10px 14px;
  background: transparent;
  border: none;
  font-size: 14px;
  color: #2d3748;
  cursor: pointer;
  text-align: left;
  transition: background 0.15s;
}

.menu-item:hover {
  background: #f7fafc;
}

.menu-item.danger {
  color: #e53e3e;
}

.menu-item.danger:hover {
  background: #fff5f5;
}

.event-icon {
  font-size: 48px;
  flex-shrink: 0;
}

.event-details {
  flex: 1;
  min-width: 0;
}

.event-name {
  font-size: 20px;
  font-weight: 600;
  margin-bottom: 12px;
  color: #2d3748;
  word-wrap: break-word;
}

.event-meta {
  display: flex;
  gap: 12px;
  margin-bottom: 8px;
  font-size: 14px;
  color: #718096;
  flex-wrap: wrap;
}

.event-location,
.event-capacity {
  font-size: 14px;
  color: #718096;
  margin-top: 6px;
}

/* ---- Modals ---- */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
  backdrop-filter: blur(4px);
}

.modal {
  background: white;
  color: #2d3748;
  border-radius: 16px;
  padding: 32px;
  width: 100%;
  max-width: 420px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.modal h3 {
  font-size: 20px;
  font-weight: 700;
  margin-bottom: 16px;
}

.modal-body {
  font-size: 15px;
  color: #4a5568;
  margin-bottom: 24px;
  line-height: 1.5;
}

.modal-input {
  width: 100%;
  padding: 10px 14px;
  border: 1px solid #cbd5e0;
  border-radius: 8px;
  font-size: 15px;
  margin-bottom: 20px;
  outline: none;
  transition: border-color 0.2s;
}

.modal-input:focus {
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.2);
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.modal-btn {
  padding: 10px 20px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  border: none;
  transition: all 0.2s;
}

.modal-btn.secondary {
  background: #edf2f7;
  color: #4a5568;
}

.modal-btn.secondary:hover {
  background: #e2e8f0;
}

.modal-btn.primary {
  background: #667eea;
  color: white;
}

.modal-btn.primary:hover {
  background: #5a67d8;
}

.modal-btn.danger {
  background: #e53e3e;
  color: white;
}

.modal-btn.danger:hover {
  background: #c53030;
}

@media (max-width: 768px) {
  .header {
    padding: 15px 20px;
  }

  .logo-section h1 {
    font-size: 24px;
  }

  .app-logo {
    font-size: 36px;
  }

  .welcome-section h2 {
    font-size: 28px;
  }

  .events-grid {
    grid-template-columns: 1fr;
  }

  .event-card {
    padding: 20px;
    padding-right: 48px;
  }
}

@media (prefers-color-scheme: dark) {
  .event-card {
    background: rgba(45, 55, 72, 0.95);
    color: white;
  }

  .event-card:hover {
    background: rgba(45, 55, 72, 1);
  }

  .event-name {
    color: white;
  }

  .event-meta,
  .event-location,
  .event-capacity {
    color: #cbd5e0;
  }

  .menu-trigger {
    color: #a0aec0;
  }

  .menu-trigger:hover {
    background: rgba(255, 255, 255, 0.1);
    color: white;
  }

  .dropdown-menu {
    background: #2d3748;
    border-color: #4a5568;
  }

  .menu-item {
    color: #e2e8f0;
  }

  .menu-item:hover {
    background: #3a4a5c;
  }

  .menu-item.danger {
    color: #fc8181;
  }

  .menu-item.danger:hover {
    background: #3a2020;
  }

  .modal {
    background: #2d3748;
    color: #e2e8f0;
  }

  .modal-body {
    color: #a0aec0;
  }

  .modal-input {
    background: #1a202c;
    border-color: #4a5568;
    color: #e2e8f0;
  }

  .modal-input:focus {
    border-color: #667eea;
  }

  .modal-btn.secondary {
    background: #4a5568;
    color: #e2e8f0;
  }

  .modal-btn.secondary:hover {
    background: #606878;
  }
}
</style>
