<script lang="ts">
  import { onMount } from 'svelte'
  import { relaunch } from '@tauri-apps/plugin-process'
  import { getVersion } from '@tauri-apps/api/app'
  import type { Update } from '@tauri-apps/plugin-updater'
  import { player } from '../stores/player.svelte'
  import { api } from '../api'

  let { update = null }: { update: Update | null } = $props()

  let discordPresence = $state(true)
  let toggling = $state(false)
  let appVersion = $state('')
  let installing = $state(false)

  onMount(async () => {
    const [settings, version] = await Promise.all([
      api.invoke('settings:get') as Promise<{ discordPresence: boolean }>,
      getVersion(),
    ])
    discordPresence = settings.discordPresence ?? true
    appVersion = version
  })

  async function toggleDiscord() {
    if (toggling) return
    toggling = true
    discordPresence = !discordPresence

    // Fire-and-forget: Discord IPC runs in Rust background thread
    api.invoke('settings:set-discord-presence', discordPresence).then(() => {
      if (discordPresence && player.currentSong) {
        const song = player.currentSong
        api.invoke('discord:update-presence', {
          title: song.title,
          artist: song.artist,
          album: song.album,
          isPlaying: player.isPlaying,
          duration: player.duration || song.duration,
          currentTime: player.currentTime,
        })
      }
      toggling = false
    }).catch(() => {
      toggling = false
    })
  }

  async function installUpdate() {
    if (!update || installing) return
    installing = true
    try {
      await update.downloadAndInstall()
      await relaunch()
    } catch {
      installing = false
    }
  }
</script>

<div class="settings-view">
  <div class="header">
    <h1>Settings</h1>
  </div>

  <div class="section">
    <p class="section-title">Updates</p>

    <div class="setting-row" class:loading={installing}>
      <div class="setting-info">
        {#if installing}
          <span class="setting-label">Mengunduh update…</span>
          <span class="setting-desc">App akan restart otomatis setelah selesai</span>
        {:else if update}
          <span class="setting-label">Update tersedia: <strong>v{update.version}</strong></span>
          <span class="setting-desc">Versi saat ini: v{appVersion}</span>
        {:else}
          <span class="setting-label">lokamusic is up to date</span>
          <span class="setting-desc">{appVersion ? `v${appVersion}` : ''}</span>
        {/if}
      </div>
      {#if installing}
        <span class="spinner" aria-label="Installing…"></span>
      {:else if update}
        <button class="install-btn" onclick={installUpdate}>
          Install
        </button>
      {:else}
        <span class="up-to-date-icon">✓</span>
      {/if}
    </div>
  </div>

  <div class="section">
    <p class="section-title">Integrations</p>

    <div class="setting-row" class:loading={toggling}>
      <div class="setting-info">
        <span class="setting-label">Discord Rich Presence</span>
        <span class="setting-desc">Tampilkan lagu yang sedang diputar di Discord</span>
      </div>
      {#if toggling}
        <span class="spinner" aria-label="Loading…"></span>
      {:else}
        <button
          class="toggle"
          class:on={discordPresence}
          onclick={toggleDiscord}
          aria-label="Toggle Discord Rich Presence"
        >
          <span class="thumb"></span>
        </button>
      {/if}
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

  .setting-row.loading {
    background: linear-gradient(
      90deg,
      var(--surface-container) 0%,
      var(--surface-container-high, color-mix(in srgb, var(--surface-container) 60%, var(--on-surface) 40%)) 50%,
      var(--surface-container) 100%
    );
    background-size: 200% 100%;
    animation: shimmer 1.4s ease-in-out infinite;
  }

  @keyframes shimmer {
    0%   { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }

  .spinner {
    flex-shrink: 0;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: 2px solid var(--outline);
    border-top-color: var(--primary);
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .install-btn {
    flex-shrink: 0;
    padding: 7px 16px;
    border-radius: var(--radius-lg);
    background: var(--primary);
    color: var(--on-primary);
    font-size: 13px;
    font-weight: 500;
    transition: opacity 0.15s;
  }

  .install-btn:hover { opacity: 0.85; }

  .up-to-date-icon {
    flex-shrink: 0;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--primary);
    font-size: 16px;
    font-weight: 600;
  }
</style>
