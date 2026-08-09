<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { listen, emit, type UnlistenFn } from '@tauri-apps/api/event'
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
  import Icon from './lib/components/Icon.svelte'

  interface MiniSong {
    id: string
    title: string
    artist: string
    album?: string
    albumArt?: string
  }

  let song = $state<MiniSong | null>(null)
  let isPlaying = $state(false)
  let isLoading = $state(false)
  let currentTime = $state(0)
  let duration = $state(1)

  let unlisten: UnlistenFn | null = null

  const pct = $derived(duration > 0 ? (currentTime / duration) * 100 : 0)

  function formatTime(s: number): string {
    if (!Number.isFinite(s) || s < 0) s = 0
    const m = Math.floor(s / 60)
    const sec = Math.floor(s % 60)
    return `${m}:${sec.toString().padStart(2, '0')}`
  }

  function sendCmd(cmd: string) {
    emit('miniplayer:cmd', cmd)
  }

  function seek(e: MouseEvent) {
    const bar = e.currentTarget as HTMLElement
    const r = bar.getBoundingClientRect()
    const value = Math.max(0, Math.min(1, (e.clientX - r.left) / r.width)) * duration
    emit('miniplayer:cmd', { action: 'seek', value })
  }

  function closeWindow() {
    getCurrentWebviewWindow().close()
  }

  onMount(async () => {
    unlisten = await listen<{
      song: MiniSong | null
      isPlaying: boolean
      isLoading: boolean
      currentTime: number
      duration: number
    }>('miniplayer:state', (e) => {
      song = e.payload.song
      isPlaying = e.payload.isPlaying
      isLoading = e.payload.isLoading
      currentTime = e.payload.currentTime
      duration = e.payload.duration || 1
    })
    await emit('miniplayer:request-state', null)
    window.addEventListener('blur', closeWindow)
  })

  onDestroy(() => {
    unlisten?.()
    window.removeEventListener('blur', closeWindow)
  })
</script>

<div class="popover">
  <div class="top">
    {#if song?.albumArt}
      <img src={song.albumArt} alt="" class="art" draggable="false" />
    {:else}
      <div class="art-placeholder">
        <Icon name="music" size={22} />
      </div>
    {/if}
    <div class="info">
      <div class="title">{song?.title ?? 'Not Playing'}</div>
      <div class="subtitle">{[song?.artist, song?.album].filter(Boolean).join(' - ')}</div>
    </div>
  </div>

  <div class="controls">
    <button class="ctrl-btn" onclick={() => sendCmd('prev')} disabled={isLoading} title="Previous">
      <Icon name="skip-back" size={18} />
    </button>
    <button class="play-btn" onclick={() => sendCmd('toggle-play')} title={isPlaying ? 'Pause' : 'Play'}>
      <Icon name={isPlaying ? 'pause' : 'play'} size={20} />
    </button>
    <button class="ctrl-btn" onclick={() => sendCmd('next')} disabled={isLoading} title="Next">
      <Icon name="skip-forward" size={18} />
    </button>
  </div>

  <div class="progress-row">
    <span class="time">{formatTime(currentTime)}</span>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="track" onmousedown={seek}>
      <div class="fill" style="width: {pct}%"></div>
    </div>
    <span class="time">-{formatTime(Math.max(0, duration - currentTime))}</span>
  </div>
</div>

<style>
  :global(html), :global(body) {
    margin: 0;
    background: transparent;
    overflow: hidden;
    height: 100%;
    -webkit-user-select: none;
    user-select: none;
  }

  :global(#app) { height: 100%; }

  .popover {
    height: 100vh;
    width: 100vw;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px;
    background: var(--surface-container-low);
    border: 1px solid var(--outline-variant);
    border-radius: 12px;
  }

  .top {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }

  .art, .art-placeholder {
    width: 56px;
    height: 56px;
    flex-shrink: 0;
    border-radius: 8px;
    object-fit: cover;
  }

  .art-placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--surface-container-highest);
    color: var(--outline);
  }

  .info {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .title {
    font-size: 13px;
    font-weight: 600;
    color: var(--on-surface);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .subtitle {
    font-size: 11px;
    color: var(--on-surface-variant);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .controls {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 18px;
  }

  .ctrl-btn, .play-btn {
    border: none;
    background: none;
    font-family: inherit;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--on-surface);
    cursor: pointer;
    border-radius: 50%;
    transition: background 0.12s, transform 0.1s;
  }

  .ctrl-btn { width: 30px; height: 30px; }
  .ctrl-btn:hover { background: var(--surface-container-high); transform: scale(1.08); }
  .ctrl-btn:disabled { opacity: 0.4; cursor: default; pointer-events: none; transform: none; }

  .play-btn {
    width: 34px;
    height: 34px;
    background: var(--secondary-container);
    color: var(--on-secondary-container);
  }
  .play-btn:hover { background: var(--secondary); color: var(--on-secondary); transform: scale(1.06); }

  .progress-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .time {
    font-size: 10px;
    color: var(--on-surface-variant);
    flex-shrink: 0;
    width: 30px;
    text-align: center;
  }

  .track {
    position: relative;
    flex: 1;
    height: 12px;
    display: flex;
    align-items: center;
    cursor: pointer;
  }

  .track::before {
    content: '';
    position: absolute;
    inset: 4px 0;
    border-radius: 2px;
    background: var(--surface-container-highest);
  }

  .fill {
    position: absolute;
    top: 4px;
    left: 0;
    height: 4px;
    border-radius: 2px;
    background: var(--secondary);
    max-width: 100%;
    pointer-events: none;
  }
</style>
