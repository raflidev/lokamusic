<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { listen, emit, type UnlistenFn } from '@tauri-apps/api/event'
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
  import Icon from './lib/components/Icon.svelte'

  interface MiniSong {
    id: string
    title: string
    artist: string
    albumArt?: string
  }

  interface PlayerState {
    song: MiniSong | null
    isPlaying: boolean
    isLoading: boolean
    currentTime: number
    duration: number
    liked: boolean
    shuffle: boolean
    repeat: 'none' | 'one' | 'all'
    volume: number
  }

  let ps = $state({
    song: null as MiniSong | null,
    isPlaying: false,
    isLoading: false,
    currentTime: 0,
    duration: 1,
    liked: false,
    shuffle: false,
    repeat: 'none' as 'none' | 'one' | 'all',
    volume: 0.8,
  })

  let isHovering = $state(false)
  let isSeeking = $state(false)
  let seekValue = $state(0)
  let volHovering = $state(false)
  let volDragging = $state(false)

  let volBarEl = $state(null as HTMLElement | null)
  let volDragStartY = 0
  let volDragStartVal = 0

  let unlistenPs: UnlistenFn | null = null

  const displayPct = $derived(ps.duration > 0 ? (ps.currentTime / ps.duration) * 100 : 0)

  $effect(() => {
    if (!isSeeking) seekValue = ps.currentTime
  })

  onMount(async () => {
    unlistenPs = await listen<PlayerState>('miniplayer:state', (e) => {
      const d = e.payload
      ps.song = d.song
      ps.isPlaying = d.isPlaying
      ps.isLoading = d.isLoading
      ps.currentTime = d.currentTime
      ps.duration = d.duration
      ps.liked = d.liked
      ps.shuffle = d.shuffle
      ps.repeat = d.repeat
      if (!volDragging) ps.volume = d.volume
    })
    await emit('miniplayer:request-state', null)
  })

  onDestroy(() => {
    unlistenPs?.()
    window.removeEventListener('mousemove', doVolDrag)
    window.removeEventListener('mouseup', endVolDrag)
  })

  function sendCmd(cmd: string | object) {
    emit('miniplayer:cmd', cmd)
  }

  async function backToApp() {
    await emit('miniplayer:cmd', 'open-player')
    getCurrentWebviewWindow().close()
  }

  // Progress seek
  function startSeek(e: MouseEvent) {
    isSeeking = true
    applySeek(e)
  }

  function applySeek(e: MouseEvent) {
    const bar = e.currentTarget as HTMLElement
    const r = bar.getBoundingClientRect()
    seekValue = Math.max(0, Math.min(1, (e.clientX - r.left) / r.width)) * ps.duration
  }

  function moveSeek(e: MouseEvent) {
    if (isSeeking) applySeek(e)
  }

  function commitSeek() {
    if (!isSeeking) return
    sendCmd({ action: 'seek', value: seekValue })
    isSeeking = false
  }

  // Volume bar drag - click sets position, drag adjusts
  function startVolBarDrag(e: MouseEvent) {
    e.preventDefault()
    volDragging = true
    if (volBarEl) {
      const rect = volBarEl.getBoundingClientRect()
      const clicked = Math.max(0, Math.min(1, 1 - (e.clientY - rect.top) / rect.height))
      ps.volume = clicked
      sendCmd({ action: 'set-volume', value: clicked })
      volDragStartY = e.clientY
      volDragStartVal = clicked
    } else {
      volDragStartY = e.clientY
      volDragStartVal = ps.volume
    }
    window.addEventListener('mousemove', doVolDrag)
    window.addEventListener('mouseup', endVolDrag)
  }

  function doVolDrag(e: MouseEvent) {
    const delta = (volDragStartY - e.clientY) / 120
    const newVol = Math.max(0, Math.min(1, volDragStartVal + delta))
    ps.volume = newVol
    sendCmd({ action: 'set-volume', value: newVol })
  }

  function endVolDrag() {
    volDragging = false
    window.removeEventListener('mousemove', doVolDrag)
    window.removeEventListener('mouseup', endVolDrag)
  }

  const volIcon = $derived(
    ps.volume === 0 ? 'volume' : 'volume-low'
  )
</script>

<div class="shell" data-tauri-drag-region>
  <!-- Art area -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="art-area"
    data-tauri-drag-region
    onmouseenter={() => isHovering = true}
    onmouseleave={() => isHovering = false}
  >
    {#if ps.song?.albumArt}
      <img src={ps.song.albumArt} alt="" class="art" draggable="false" />
    {:else}
      <div class="art-placeholder" data-tauri-drag-region>
        <Icon name="music" size={48} />
      </div>
    {/if}

    <!-- X button - always visible in top-right -->
    <button class="close-fab" onclick={backToApp} title="Back to app">
      <Icon name="x" size={14} />
    </button>

    <!-- Hover controls overlay -->
    {#if isHovering}
      <div class="controls-overlay">

        <!-- Single row: volume, prev, play/pause, next, shuffle -->
        <div class="overlay-controls">
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="vol-wrap"
            onmouseenter={() => volHovering = true}
            onmouseleave={() => { if (!volDragging) volHovering = false }}
          >
            {#if volHovering || volDragging}
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="vol-bar-track"
                bind:this={volBarEl}
                onmousedown={startVolBarDrag}
              >
                <div class="vol-bar-fill" style="height: {ps.volume * 100}%"></div>
              </div>
            {/if}
            <div class="ov-btn vol-icon-btn" class:dragging={volDragging}>
              <Icon name={volIcon} size={15} />
            </div>
          </div>
          <button class="ov-btn" onclick={() => sendCmd('prev')} disabled={ps.isLoading} title="Previous">
            <Icon name="skip-back" size={18} />
          </button>
          <button class="play-circle" onclick={() => sendCmd('toggle-play')} title={ps.isPlaying ? 'Pause' : 'Play'}>
            <Icon name={ps.isPlaying ? 'pause' : 'play'} size={18} />
          </button>
          <button class="ov-btn" onclick={() => sendCmd('next')} disabled={ps.isLoading} title="Next">
            <Icon name="skip-forward" size={18} />
          </button>
          <button
            class="ov-btn"
            class:active={ps.shuffle}
            onclick={() => sendCmd('toggle-shuffle')}
            title="Shuffle"
          >
            <Icon name="shuffle" size={15} />
          </button>
        </div>

        <!-- Progress bar -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="overlay-progress"
          onmousedown={startSeek}
          onmousemove={moveSeek}
          onmouseup={commitSeek}
        >
          <div class="overlay-track">
            <div class="overlay-fill" style="width: {isSeeking ? (seekValue / ps.duration * 100) : displayPct}%"></div>
          </div>
        </div>

      </div>
    {/if}
  </div>

  <!-- Info strip -->
  <div class="info-strip" data-tauri-drag-region>
    <div class="strip-row" data-tauri-drag-region>
      <span class="track-title" data-tauri-drag-region>{ps.song?.title ?? 'Not playing'}</span>
      <button class="icon-btn" class:liked={ps.liked} onclick={() => sendCmd('toggle-like')} title="Like">
        <Icon name={ps.liked ? 'heart-filled' : 'heart'} size={13} />
      </button>
    </div>
    <span class="track-artist" data-tauri-drag-region>{ps.song?.artist ?? ''}</span>
  </div>
</div>

<style>
  :global(html), :global(body) {
    margin: 0;
    background: var(--surface-container-low);
    overflow: hidden;
    height: 100%;
    -webkit-user-select: none;
    user-select: none;
  }

  :global(#app) { height: 100%; }

  .shell {
    height: 100vh;
    width: 100vw;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border: 1px solid var(--outline-variant);
    border-radius: 12px;
    cursor: grab;
  }

  .shell:active { cursor: grabbing; }

  /* Art area */
  .art-area {
    position: relative;
    width: 100%;
    flex: 1;
    min-height: 160px;
    background: var(--surface-container-highest);
    overflow: hidden;
    cursor: grab;
  }

  .art {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
    pointer-events: none;
  }

  .art-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--outline);
    cursor: grab;
  }

  /* X button */
  .close-fab {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.45);
    color: rgba(255, 255, 255, 0.85);
    border: none;
    font-family: inherit;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    z-index: 20;
    pointer-events: auto;
    transition: background 0.15s, color 0.15s, transform 0.1s;
    backdrop-filter: blur(4px);
  }

  .close-fab:hover {
    background: rgba(0, 0, 0, 0.7);
    color: #fff;
    transform: scale(1.1);
  }

  /* Hover overlay */
  .controls-overlay {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.52);
    backdrop-filter: blur(4px);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0;
    z-index: 10;
    pointer-events: auto;
  }

  /* Single controls row */
  .overlay-controls {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    flex: 1;
  }

  .ov-btn {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    border: none;
    background: none;
    font-family: inherit;
    display: flex;
    align-items: center;
    justify-content: center;
    color: rgba(255, 255, 255, 0.75);
    cursor: pointer;
    transition: color 0.12s, background 0.12s, transform 0.1s;
  }

  .ov-btn:hover { color: #fff; background: rgba(255, 255, 255, 0.12); transform: scale(1.08); }
  .ov-btn.active { color: var(--secondary); }
  .ov-btn:disabled { opacity: 0.4; cursor: default; pointer-events: none; transform: none; }

  .vol-wrap {
    position: relative;
    flex-shrink: 0;
    /* larger hover zone so the bar is easy to reach */
    padding: 4px 10px;
    margin: -4px -10px;
  }

  .vol-icon-btn { cursor: default; }
  .vol-icon-btn.dragging { color: var(--secondary); }

  /* Vertical bar - floats above the icon */
  .vol-bar-track {
    position: absolute;
    bottom: calc(100% + 2px);
    left: 50%;
    transform: translateX(-50%);
    /* wide transparent hit area, thin visual via inner fill */
    width: 24px;
    height: 80px;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    align-items: center;
    cursor: ns-resize;
    pointer-events: auto;
    padding: 0 10px;
    box-sizing: border-box;
  }

  /* the visible thin rail lives as a pseudo-element */
  .vol-bar-track::before {
    content: '';
    position: absolute;
    inset: 0 10px;
    background: rgba(255, 255, 255, 0.15);
    border-radius: 2px;
    transition: background 0.12s;
  }

  .vol-bar-track:hover::before { background: rgba(255, 255, 255, 0.28); }

  .vol-bar-fill {
    position: absolute;
    bottom: 0;
    left: 10px;
    right: 10px;
    background: var(--secondary);
    border-radius: 2px;
    transition: height 0.08s ease;
    min-height: 2px;
    pointer-events: none;
  }

  .play-circle {
    width: 44px;
    height: 44px;
    border-radius: 50%;
    border: none;
    font-family: inherit;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--secondary-container);
    color: var(--on-secondary-container);
    cursor: pointer;
    transition: background 0.15s, color 0.15s, transform 0.1s;
  }

  .play-circle:hover {
    background: var(--secondary);
    color: var(--on-secondary);
    transform: scale(1.08);
  }

  /* Progress bar */
  .overlay-progress {
    width: 100%;
    height: 24px;
    cursor: pointer;
    display: flex;
    align-items: center;
    flex-shrink: 0;
    padding: 0 4px;
    box-sizing: border-box;
  }

  .overlay-track {
    position: relative;
    width: 100%;
    height: 4px;
    background: rgba(255, 255, 255, 0.2);
    border-radius: 2px;
    pointer-events: none;
  }

  .overlay-fill {
    position: absolute;
    left: 0;
    top: 0;
    height: 100%;
    background: var(--secondary);
    border-radius: 2px;
    pointer-events: none;
    max-width: 100%;
    transition: width 0.3s linear;
  }

  /* Info strip */
  .info-strip {
    flex-shrink: 0;
    background: var(--surface-container-low);
    border-top: 1px solid var(--outline-variant);
    display: flex;
    flex-direction: column;
    justify-content: center;
    padding: 8px 12px;
    gap: 3px;
    cursor: grab;
  }

  .strip-row {
    display: flex;
    align-items: center;
    gap: 4px;
    cursor: grab;
  }

  .track-title {
    flex: 1;
    min-width: 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--on-surface);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    cursor: grab;
  }

  .track-artist {
    font-size: 11px;
    color: var(--on-surface-variant);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    cursor: grab;
  }

  .icon-btn {
    width: 22px;
    height: 22px;
    flex-shrink: 0;
    border-radius: 3px;
    border: none;
    background: none;
    font-family: inherit;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--outline);
    cursor: pointer;
    transition: color 0.12s, background 0.12s;
  }

  .icon-btn:hover { color: var(--on-surface); background: var(--surface-container-high); }
  .icon-btn.liked { color: var(--secondary); }
</style>
