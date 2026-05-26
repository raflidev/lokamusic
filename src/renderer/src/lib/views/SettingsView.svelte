<script lang="ts">
  import { onMount } from 'svelte'
  import { player } from '../stores/player.svelte'

  let discordPresence = $state(true)

  onMount(async () => {
    const settings = await window.electronAPI.invoke('settings:get') as { discordPresence: boolean }
    discordPresence = settings.discordPresence ?? true
  })

  async function toggleDiscord() {
    discordPresence = !discordPresence
    await window.electronAPI.invoke('settings:set-discord-presence', discordPresence)

    if (discordPresence && player.currentSong) {
      const song = player.currentSong
      window.electronAPI.invoke('discord:update-presence', {
        title: song.title,
        artist: song.artist,
        album: song.album,
        isPlaying: player.isPlaying,
        duration: player.duration || song.duration,
        currentTime: player.currentTime,
      })
    }
  }
</script>

<div class="settings-view">
  <div class="header">
    <h1>Settings</h1>
  </div>

  <div class="section">
    <p class="section-title">Integrations</p>

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">Discord Rich Presence</span>
        <span class="setting-desc">Tampilkan lagu yang sedang diputar di Discord</span>
      </div>
      <button
        class="toggle"
        class:on={discordPresence}
        onclick={toggleDiscord}
        aria-label="Toggle Discord Rich Presence"
      >
        <span class="thumb"></span>
      </button>
    </div>
  </div>
</div>

<style>
  .settings-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
    padding: 40px 48px;
  }

  .header {
    margin-bottom: 40px;
  }

  h1 {
    font-size: 24px;
    font-weight: 600;
    letter-spacing: -0.02em;
    color: var(--on-surface);
    margin: 0;
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .section-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--outline);
    margin: 0 0 12px;
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    background: var(--surface-container);
    border-radius: var(--radius-lg);
    gap: 24px;
  }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .setting-label {
    font-size: 14px;
    font-weight: 500;
    color: var(--on-surface);
  }

  .setting-desc {
    font-size: 12px;
    color: var(--outline);
  }

  .toggle {
    position: relative;
    width: 44px;
    height: 24px;
    border-radius: 999px;
    background: var(--surface-container-highest);
    transition: background 0.2s;
    flex-shrink: 0;
  }

  .toggle.on {
    background: var(--primary);
  }

  .thumb {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--outline);
    transition: transform 0.2s, background 0.2s;
  }

  .toggle.on .thumb {
    transform: translateX(20px);
    background: var(--on-primary);
  }
</style>
