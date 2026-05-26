<script lang="ts">
  import Icon from '../components/Icon.svelte'
  import { player } from '../stores/player.svelte'
  import { library } from '../stores/library.svelte'
  import { ui } from '../stores/ui.svelte'
  import { api } from '../api'

  function formatTime(s: number): string {
    if (!s || isNaN(s)) return '0:00'
    const m = Math.floor(s / 60)
    const sec = Math.floor(s % 60)
    return `${m}:${sec.toString().padStart(2, '0')}`
  }

  async function toggleLike() {
    if (!player.currentSong) return
    await api.invoke('library:toggle-like', player.currentSong.id)
    library.toggleLike(player.currentSong.id)
  }

  let isSeeking = $state(false)
  let seekValue = $state(0)

  $effect(() => {
    if (!isSeeking) seekValue = player.currentTime
  })

  const displayPct = $derived(
    player.duration > 0 ? (seekValue / player.duration) * 100 : 0
  )

  const songLiked = $derived(
    player.currentSong
      ? library.songs.find(s => s.id === player.currentSong!.id)?.liked ?? false
      : false
  )

  const hasLyrics = $derived(!!player.currentSong?.lyrics)
  let showLyrics = $state(false)
  let lyricsFullscreen = $state(false)

  interface LrcLine { time: number; text: string }

  const lrcLines = $derived.by((): LrcLine[] => {
    const text = player.currentSong?.lyrics
    if (!text) return []
    const result: LrcLine[] = []
    for (const line of text.split('\n')) {
      const m = /^\[(\d{1,2}):(\d{2}[.,]\d{1,3})\](.*)/.exec(line)
      if (m) {
        const time = parseInt(m[1]) * 60 + parseFloat(m[2].replace(',', '.'))
        const txt = m[3].trim()
        if (txt) result.push({ time, text: txt })
      }
    }
    return result.sort((a, b) => a.time - b.time)
  })

  const isLrc = $derived(lrcLines.length > 0)

  const currentLineIdx = $derived.by(() => {
    if (!isLrc) return -1
    let idx = 0
    for (let i = 0; i < lrcLines.length; i++) {
      if (lrcLines[i].time <= player.currentTime) idx = i
      else break
    }
    return idx
  })

  let lineEls: HTMLElement[] = []
  let lrcContainer = $state<HTMLElement | null>(null)
  let lineElsFs: HTMLElement[] = []
  let lrcFsContainer = $state<HTMLElement | null>(null)

  function scrollToCenter(container: HTMLElement, el: HTMLElement) {
    const containerRect = container.getBoundingClientRect()
    const elRect = el.getBoundingClientRect()
    const scrollTarget = container.scrollTop + (elRect.top - containerRect.top) - (container.clientHeight / 2) + (el.clientHeight / 2)
    container.scrollTo({ top: scrollTarget, behavior: 'smooth' })
  }

  $effect(() => {
    const idx = currentLineIdx
    if (idx < 0) return
    if (lineEls[idx] && lrcContainer) scrollToCenter(lrcContainer, lineEls[idx])
    if (lineElsFs[idx] && lrcFsContainer) scrollToCenter(lrcFsContainer, lineElsFs[idx])
  })
</script>

<div class="player-view">
  <!-- Decorative visualizer -->
  <div class="visualizer" aria-hidden="true">
    {#each Array(18) as _, i}
      <div class="bar" style="--delay: {i * 0.07}s; --h: {20 + Math.random() * 60}%"></div>
    {/each}
  </div>

  <div class="player-body" class:compact={ui.showQueue}>
    <!-- Album art / Lyrics — card flip -->
    <div class="art-wrap">
      <div class="card-flip">
        <div class="card-flip-inner" class:flipped={showLyrics}>

          <!-- Front: Album art -->
          <div class="card-front">
            {#key player.currentSong?.id}
              {#if player.currentSong?.albumArt}
                <img src={player.currentSong.albumArt} alt="" class="album-art" />
              {:else}
                <div class="album-art placeholder">
                  <Icon name="music" size={80} />
                </div>
              {/if}
            {/key}
          </div>

          <!-- Back: Lyrics -->
          <div class="card-back">
            <div class="lyrics-panel">
              <div class="lyrics-toolbar">
                <button class="art-toggle" onclick={() => showLyrics = false} title="Kembali ke album art">
                  <Icon name="arrow-right" size={14} />
                </button>
                {#if hasLyrics}
                  <button class="fullscreen-btn" onclick={() => lyricsFullscreen = true} title="Fullscreen">
                    <Icon name="expand" size={14} />
                  </button>
                {/if}
              </div>
              {#if hasLyrics}
                {#if isLrc}
                  <div class="lyrics-scroll lrc-scroll" bind:this={lrcContainer}>
                    {#each lrcLines as line, i}
                      <p
                        bind:this={lineEls[i]}
                        class="lrc-line"
                        class:lrc-past={i < currentLineIdx}
                        class:lrc-active={i === currentLineIdx}
                        class:lrc-future={i > currentLineIdx}
                        onclick={() => player.seek(line.time)}
                      >{line.text}</p>
                    {/each}
                  </div>
                {:else}
                  <div class="lyrics-scroll">
                    <p class="lyrics-text">{player.currentSong?.lyrics}</p>
                  </div>
                {/if}
              {:else}
                <div class="no-lyrics">
                  <Icon name="mic" size={32} />
                  <p>No lyrics available</p>
                </div>
              {/if}
            </div>
          </div>

        </div>
      </div>
    </div>

    <!-- Lyrics fullscreen overlay -->
    {#if lyricsFullscreen}
      <div class="lyrics-fullscreen">
        <button class="fs-close" onclick={() => lyricsFullscreen = false} title="Tutup">
          <Icon name="expand" size={18} />
        </button>
        <div class="fs-song-info">
          <span class="fs-title">{player.currentSong?.title}</span>
          <span class="fs-artist">{player.currentSong?.artist}</span>
        </div>
        <div class="fs-lyrics-scroll" bind:this={lrcFsContainer}>
          {#if isLrc}
            {#each lrcLines as line, i}
              <p
                bind:this={lineElsFs[i]}
                class="fs-lrc-line"
                class:fs-lrc-past={i < currentLineIdx}
                class:fs-lrc-active={i === currentLineIdx}
                class:fs-lrc-future={i > currentLineIdx}
                onclick={() => player.seek(line.time)}
              >{line.text}</p>
            {/each}
          {:else}
            <p class="fs-lyrics-text">{player.currentSong?.lyrics}</p>
          {/if}
        </div>
      </div>
    {/if}

    <!-- Info + controls -->
    <div class="right-panel">
      <!-- Now playing info -->
      <div class="now-playing-card">
        <div class="np-header">
          <span class="label-sm">Now Playing</span>
          <button class="like-btn" class:liked={songLiked} onclick={toggleLike}>
            <Icon name={songLiked ? 'heart-filled' : 'heart'} size={18} />
          </button>
        </div>

        {#if player.currentSong}
          <h1 class="track-title">{player.currentSong.title}</h1>
          <p class="track-meta">{player.currentSong.artist} — {player.currentSong.album}</p>
        {:else}
          <h1 class="track-title muted">No track selected</h1>
          <p class="track-meta">Pick a song from your library</p>
        {/if}

        <!-- Progress -->
        <div class="progress-area">
          <input
            type="range"
            class="range-bar"
            min="0"
            max={player.duration || 100}
            step="0.1"
            value={seekValue}
            oninput={(e) => { isSeeking = true; seekValue = +e.currentTarget.value }}
            onchange={(e) => { player.seek(+e.currentTarget.value); isSeeking = false }}
            style="--val: {displayPct}%"
          />
          <div class="timestamps">
            <span>{formatTime(player.currentTime)}</span>
            <span>{formatTime(player.duration)}</span>
          </div>
        </div>

        <!-- Controls -->
        <div class="controls-card">
          <button
            class="ctrl-btn"
            class:active={player.shuffle}
            onclick={() => player.toggleShuffle()}
          >
            <Icon name="shuffle" size={16} />
          </button>
          <button class="ctrl-btn" onclick={() => player.prev()}>
            <Icon name="skip-back" size={20} />
          </button>
          <button class="play-btn" onclick={() => player.togglePlay()}>
            <Icon name={player.isPlaying ? 'pause' : 'play'} size={22} />
          </button>
          <button class="ctrl-btn" onclick={() => player.next()}>
            <Icon name="skip-forward" size={20} />
          </button>
          <button
            class="ctrl-btn"
            class:active={player.repeat !== 'none'}
            onclick={() => player.toggleRepeat()}
          >
            <Icon name={player.repeat === 'one' ? 'repeat-one' : 'repeat'} size={16} />
          </button>
          <button
            class="ctrl-btn"
            class:active={showLyrics}
            onclick={() => showLyrics = !showLyrics}
            title="Lirik"
          >
            <Icon name="mic" size={16} />
          </button>
        </div>
      </div>

      <!-- Bottom cards -->
      <div class="bottom-cards">
        <div class="up-next-card">
          <Icon name="queue" size={14} />
          <div class="up-next-info">
            <span class="up-next-label">Up Next:</span>
            <span class="up-next-title">{player.upNext?.title ?? 'End of queue'}</span>
          </div>
          <button class="link-btn" onclick={() => ui.toggleQueue()}>View Queue</button>
        </div>

        <div class="volume-card">
          <Icon name="volume-low" size={15} />
          <input
            type="range"
            class="range-bar"
            min="0"
            max="1"
            step="0.01"
            value={player.volume}
            oninput={(e) => player.setVolume(+e.currentTarget.value)}
            style="--val: {player.volume * 100}%"
          />
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .player-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
    background: var(--surface);
  }

  /* Visualizer */
  .visualizer {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 160px;
    display: flex;
    align-items: flex-end;
    gap: 4px;
    padding: 0 40px;
    pointer-events: none;
    opacity: 0.15;
    overflow: hidden;
  }

  .bar {
    flex: 1;
    background: linear-gradient(to top, var(--primary), transparent);
    border-radius: 2px 2px 0 0;
    animation: pulse 2s ease-in-out infinite alternate;
    animation-delay: var(--delay);
    height: var(--h);
  }

  @keyframes pulse {
    0% { transform: scaleY(0.3); }
    100% { transform: scaleY(1); }
  }

  /* Body */
  .player-body {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 48px;
    padding: 32px 48px;
    position: relative;
    z-index: 1;
    transition: gap 0.3s, padding 0.3s;
    overflow: hidden;
  }

  .player-body.compact {
    gap: 28px;
    padding: 24px 32px;
  }

  /* Album art / Lyrics — card flip */
  .art-wrap {
    flex-shrink: 0;
    position: relative;
  }

  .card-flip {
    width: 420px;
    height: 420px;
    perspective: 1200px;
    transition: width 0.3s, height 0.3s;
  }

  .compact .card-flip {
    width: 300px;
    height: 300px;
  }

  .card-flip-inner {
    position: relative;
    width: 100%;
    height: 100%;
    transform-style: preserve-3d;
    transition: transform 0.55s cubic-bezier(0.4, 0.2, 0.2, 1);
  }

  .card-flip-inner.flipped {
    transform: rotateY(180deg);
  }

  .card-front,
  .card-back {
    position: absolute;
    inset: 0;
    backface-visibility: hidden;
    -webkit-backface-visibility: hidden;
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .card-back {
    transform: rotateY(180deg);
  }

  .album-art {
    width: 100%;
    height: 100%;
    object-fit: cover;
    box-shadow: 0 32px 80px rgba(0,0,0,0.6);
    animation: art-fade-in 0.4s ease;
  }

  @keyframes art-fade-in {
    from { opacity: 0; transform: scale(1.04); }
    to   { opacity: 1; transform: scale(1); }
  }

  .album-art.placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--surface-container);
    color: var(--outline);
  }

  /* Lyrics panel (replaces art) */
  .lyrics-panel {
    width: 100%;
    height: 100%;
    border-radius: var(--radius-lg);
    background: var(--surface-container-low);
    border: 1px solid var(--outline-variant);
    box-shadow: 0 32px 80px rgba(0,0,0,0.4);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .lyrics-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border-bottom: 1px solid var(--outline-variant);
    flex-shrink: 0;
  }

  .art-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 6px;
    border-radius: var(--radius);
    color: var(--on-surface-variant);
    transform: rotate(180deg);
    transition: color 0.12s, background 0.12s;
  }

  .fullscreen-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 6px;
    border-radius: var(--radius);
    color: var(--on-surface-variant);
    transition: color 0.12s, background 0.12s;
  }

  .art-toggle:hover, .fullscreen-btn:hover {
    color: var(--on-surface);
    background: var(--surface-container);
  }

  .lyrics-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 16px 20px;
  }

  .lyrics-scroll::-webkit-scrollbar { width: 4px; }
  .lyrics-scroll::-webkit-scrollbar-thumb { background: var(--outline-variant); border-radius: 2px; }

  .no-lyrics {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--outline);
    font-size: 13px;
  }

  .lyrics-text {
    font-size: 14px;
    line-height: 1.9;
    color: var(--on-surface);
    white-space: pre-wrap;
    word-break: break-word;
  }

  /* LRC sing-along */
  .lrc-scroll {
    padding: 80px 20px;
  }

  .lrc-line {
    font-size: 14px;
    font-weight: 500;
    line-height: 1.5;
    padding: 8px 0;
    text-align: center;
    cursor: pointer;
    transition: color 0.3s, font-size 0.3s, font-weight 0.3s, opacity 0.3s;
    word-break: break-word;
  }

  .lrc-line:hover { opacity: 0.9; }

  .lrc-past {
    color: var(--outline);
    font-size: 13px;
    font-weight: 400;
  }

  .lrc-active {
    color: var(--secondary);
    font-size: 17px;
    font-weight: 700;
  }

  .lrc-future {
    color: var(--on-surface-variant);
    font-size: 13px;
    font-weight: 400;
    opacity: 0.7;
  }

  /* Fullscreen overlay */
  .lyrics-fullscreen {
    position: absolute;
    inset: 0;
    z-index: 100;
    background: var(--surface-container-lowest);
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 32px 48px;
    gap: 16px;
  }

  .fs-close {
    position: absolute;
    top: 20px;
    right: 24px;
    display: flex;
    align-items: center;
    padding: 8px;
    border-radius: var(--radius);
    color: var(--on-surface-variant);
    transform: rotate(45deg);
    transition: color 0.12s;
  }

  .fs-close:hover { color: var(--on-surface); }

  .fs-song-info {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .fs-title {
    font-size: 20px;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--on-surface);
  }

  .fs-artist {
    font-size: 13px;
    color: var(--on-surface-variant);
  }

  .fs-lyrics-scroll {
    flex: 1;
    overflow-y: auto;
    width: 100%;
    max-width: 640px;
  }

  .fs-lyrics-scroll::-webkit-scrollbar { width: 4px; }
  .fs-lyrics-scroll::-webkit-scrollbar-thumb { background: var(--outline-variant); border-radius: 2px; }

  .fs-lyrics-text {
    font-size: 17px;
    line-height: 2;
    color: var(--on-surface);
    white-space: pre-wrap;
    word-break: break-word;
    text-align: center;
  }

  .fs-lrc-line {
    font-size: 16px;
    font-weight: 500;
    line-height: 1.5;
    padding: 10px 0;
    text-align: center;
    cursor: pointer;
    transition: color 0.3s, font-size 0.3s, font-weight 0.3s, opacity 0.3s;
    word-break: break-word;
  }

  .fs-lrc-past { color: var(--outline); font-size: 15px; font-weight: 400; }

  .fs-lrc-active {
    color: var(--secondary);
    font-size: 24px;
    font-weight: 700;
  }

  .fs-lrc-future { color: var(--on-surface-variant); font-size: 15px; font-weight: 400; opacity: 0.7; }

  /* Right panel */
  .right-panel {
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 280px;
    max-width: 420px;
    flex: 1;
    transition: gap 0.3s;
  }

  .compact .right-panel { gap: 8px; }

  .compact .now-playing-card { padding: 16px; gap: 8px; }
  .compact .track-title { font-size: 20px; }
  .compact .bottom-cards { gap: 8px; }
  .compact .up-next-card, .compact .volume-card { padding: 12px; }

  .now-playing-card {
    background: var(--surface-container-low);
    border: 1px solid var(--outline-variant);
    border-radius: var(--radius-lg);
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .np-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .like-btn {
    color: var(--outline);
    transition: color 0.12s;
  }

  .like-btn:hover, .like-btn.liked { color: var(--secondary); }

  .track-title {
    font-size: 28px;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--on-surface);
    line-height: 1.1;
  }

  .track-title.muted { color: var(--outline); font-weight: 400; font-size: 20px; }

  .track-meta {
    font-size: 14px;
    color: var(--on-surface-variant);
  }

  /* Progress */
  .progress-area {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .range-bar {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 20px;
    background: transparent;
    cursor: pointer;
    outline: none;
    flex: 1;
  }

  .range-bar::-webkit-slider-runnable-track {
    height: 4px;
    border-radius: 2px;
    background: linear-gradient(to right, var(--secondary) var(--val, 0%), var(--surface-container-high) var(--val, 0%));
    transition: height 0.15s;
  }

  .range-bar:hover::-webkit-slider-runnable-track { height: 6px; }

  .range-bar::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 0;
    height: 0;
    background: var(--secondary);
    border-radius: 50%;
    margin-top: -8px;
    transition: width 0.15s, height 0.15s, margin-top 0.15s;
    cursor: grab;
  }

  .range-bar:hover::-webkit-slider-thumb,
  .range-bar:active::-webkit-slider-thumb {
    width: 14px;
    height: 14px;
    margin-top: -5px;
  }

  .range-bar:active::-webkit-slider-thumb { cursor: grabbing; }

  .timestamps {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    color: var(--on-surface-variant);
  }

  /* Controls */
  .controls-card {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 12px 0 4px;
  }

  .ctrl-btn {
    color: var(--on-surface-variant);
    padding: 8px;
    border-radius: var(--radius);
    display: flex;
    transition: color 0.12s;
  }

  .ctrl-btn:hover { color: var(--on-surface); }
  .ctrl-btn.active { color: var(--secondary); }

  .play-btn {
    width: 56px;
    height: 56px;
    border-radius: 50%;
    background: var(--secondary-container);
    color: var(--on-secondary-container);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.12s, transform 0.1s;
  }

  .play-btn:hover {
    background: var(--secondary);
    color: var(--on-secondary);
    transform: scale(1.05);
  }

  /* Bottom cards */
  .bottom-cards {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }

  .up-next-card {
    background: var(--surface-container-low);
    border: 1px solid var(--outline-variant);
    border-radius: var(--radius-lg);
    padding: 16px;
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--on-surface-variant);
  }

  .up-next-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .up-next-label {
    font-size: 11px;
    color: var(--outline);
  }

  .up-next-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--on-surface);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .link-btn {
    font-size: 12px;
    color: var(--secondary);
    white-space: nowrap;
    transition: opacity 0.12s;
  }

  .link-btn:hover { opacity: 0.8; }

  .volume-card {
    background: var(--surface-container-low);
    border: 1px solid var(--outline-variant);
    border-radius: var(--radius-lg);
    padding: 16px;
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--on-surface-variant);
  }

</style>
