<script lang="ts">
  import { goto } from "$app/navigation";
  import { getAllEvents, type Event } from "$lib/database";
  import { onMount } from "svelte";

  let events = $state<Event[]>([]);
  let loading = $state(true);

  onMount(async () => {
    try {
      events = await getAllEvents();
    } catch (error) {
      console.error("Error loading events:", error);
    } finally {
      loading = false;
    }
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
        {/each}
      </div>
    {/if}
  </div>
</main>

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
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
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

.event-card {
  background: rgba(255, 255, 255, 0.95);
  color: #333;
  border: none;
  border-radius: 16px;
  padding: 24px;
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
}
</style>
