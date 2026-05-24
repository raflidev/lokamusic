<script lang="ts">
  import Icon from './Icon.svelte'
  import { player } from '../stores/player.svelte'
  import { library } from '../stores/library.svelte'
  import { ui } from '../stores/ui.svelte'

  function formatTime(s: number): string {
    if (!s || isNaN(s)) return '0:00'
    const m = Math.floor(s / 60)
    const sec = Math.floor(s % 60)
    return `${m}:${sec.toString().padStart(2, '0')}`
  }

  async function toggleLike() {
    if (!player.currentSong) return
    await window.electronAPI.invoke('library:toggle-like', player.currentSong.id)
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
</script>

<div class="bottom-bar">
  <div class="inner">
    <!-- Track info -->
    <div class="track-info">
      <button class="album-art" onclick={() => player.currentSong && ui.navigate('player')}>
        {#if player.currentSong?.albumArt}
          <img src={player.currentSong.albumArt} alt="" />
        {:else}
          <div class="art-placeholder"><Icon name="music" size={18} /></div>
        {/if}
      </button>
      {#if player.currentSong}
        <div class="meta">
          <span class="track-title">{player.currentSong.title}</span>
          <button class="track-artist-btn" onclick={() => player.currentSong && ui.navigateToArtist(player.currentSong.artist)}>{player.currentSong.artist}</button>
        </div>
        <button class="like-btn" class:liked={songLiked} onclick={toggleLike} title="Like">
          <Icon name={songLiked ? 'heart-filled' : 'heart'} size={16} />
        </button>
      {:else}
        <div class="meta">
          <span class="track-title muted">Not playing</span>
        </div>
      {/if}
    </div>

    <!-- Controls -->
    <div class="controls">
      <div class="buttons">
        <button
          class="ctrl-btn"
          class:active={player.shuffle}
          onclick={() => player.toggleShuffle()}
          title="Shuffle"
        >
          <Icon name="shuffle" size={15} />
        </button>
        <button class="ctrl-btn" onclick={() => player.prev()} title="Previous">
          <Icon name="skip-back" size={18} />
        </button>
        <button class="play-btn" onclick={() => player.togglePlay()} title={player.isPlaying ? 'Pause' : 'Play'}>
          <Icon name={player.isPlaying ? 'pause' : 'play'} size={18} />
        </button>
        <button class="ctrl-btn" onclick={() => player.next()} title="Next">
          <Icon name="skip-forward" size={18} />
        </button>
        <button
          class="ctrl-btn"
          class:active={player.repeat !== 'none'}
          onclick={() => player.toggleRepeat()}
          title="Repeat"
        >
          <Icon name={player.repeat === 'one' ? 'repeat-one' : 'repeat'} size={15} />
        </button>
      </div>

      <div class="progress-row">
        <span class="time">{formatTime(player.currentTime)}</span>
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
        <span class="time">{formatTime(player.duration)}</span>
      </div>
    </div>

    <!-- Volume + extras -->
    <div class="extras">
      <Icon name="volume-low" size={15} class="muted-icon" />
      <input
        type="range"
        class="range-bar volume-range"
        min="0"
        max="1"
        step="0.01"
        value={player.volume}
        oninput={(e) => player.setVolume(+e.currentTarget.value)}
        style="--val: {player.volume * 100}%"
      />
      <button class="ctrl-btn" class:active={ui.showQueue} onclick={() => ui.toggleQueue()} title="Queue">
        <Icon name="queue" size={15} />
      </button>
    </div>
  </div>
</div>

<style>
  .bottom-bar {
    height: var(--bottom-bar-height);
    background: var(--surface-container-lowest);
    border-top: 1px solid var(--outline-variant);
    flex-shrink: 0;
    position: relative;
  }

  .inner {
    height: 100%;
    display: grid;
    grid-template-columns: 1fr 2fr 1fr;
    align-items: center;
    padding: 0 16px;
    gap: 16px;
  }

  /* Track info */
  .track-info {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }

  .album-art {
    width: 42px;
    height: 42px;
    border-radius: var(--radius-md);
    overflow: hidden;
    flex-shrink: 0;
    background: var(--surface-container-high);
    padding: 0;
    transition: opacity 0.12s;
  }

  .album-art:hover { opacity: 0.85; }

  .album-art img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .art-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--outline);
  }

  .meta {
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .track-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--on-surface);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .track-title.muted { color: var(--outline); }

  .track-artist-btn {
    font-size: 11px;
    color: var(--on-surface-variant);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    text-align: left;
    transition: color 0.12s;
  }

  .track-artist-btn:hover { color: var(--on-surface); text-decoration: underline; }

  .like-btn {
    color: var(--outline);
    flex-shrink: 0;
    transition: color 0.12s;
  }

  .like-btn:hover, .like-btn.liked { color: var(--secondary); }

  /* Controls */
  .controls {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }

  .buttons {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .ctrl-btn {
    color: var(--on-surface-variant);
    padding: 6px;
    border-radius: var(--radius);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.12s;
  }

  .ctrl-btn:hover { color: var(--on-surface); }
  .ctrl-btn.active { color: var(--secondary); }

  .play-btn {
    width: 36px;
    height: 36px;
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

  .progress-row {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
  }

  .time {
    font-size: 11px;
    color: var(--on-surface-variant);
    min-width: 32px;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }

  .range-bar {
    -webkit-appearance: none;
    appearance: none;
    flex: 1;
    height: 20px;
    background: transparent;
    cursor: pointer;
    outline: none;
  }

  .range-bar::-webkit-slider-runnable-track {
    height: 3px;
    border-radius: 2px;
    background: linear-gradient(to right, var(--secondary) var(--val, 0%), var(--surface-container-high) var(--val, 0%));
    transition: height 0.15s;
  }

  .range-bar:hover::-webkit-slider-runnable-track { height: 5px; }

  .range-bar::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 0;
    height: 0;
    background: var(--secondary);
    border-radius: 50%;
    margin-top: -7px;
    transition: width 0.15s, height 0.15s, margin-top 0.15s;
    cursor: grab;
  }

  .range-bar:hover::-webkit-slider-thumb,
  .range-bar:active::-webkit-slider-thumb {
    width: 13px;
    height: 13px;
    margin-top: -5px;
  }

  .range-bar:active::-webkit-slider-thumb { cursor: grabbing; }

  /* Extras */
  .extras {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
  }

  :global(.muted-icon) {
    color: var(--on-surface-variant);
    flex-shrink: 0;
  }

  .volume-range { width: 80px; flex: none; }
</style>
