<script lang="ts">
  import Icon from './Icon.svelte'
  import { player } from '../stores/player.svelte'
  import { ui } from '../stores/ui.svelte'

  function formatDuration(s: number): string {
    if (!s) return '--'
    return `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, '0')}`
  }
</script>

<aside class="queue-sidebar">
  <div class="queue-header">
    <span class="queue-title">Queue</span>
    <button class="close-btn" onclick={() => ui.toggleQueue()}>
      <Icon name="arrow-right" size={15} />
    </button>
  </div>

  {#if player.currentSong}
    <div class="section">
      <p class="section-label">Now Playing</p>
      <div class="queue-item current">
        <div class="item-art">
          {#if player.currentSong.albumArt}
            <img src={player.currentSong.albumArt} alt="" />
          {:else}
            <div class="art-placeholder"><Icon name="music" size={12} /></div>
          {/if}
          <div class="playing-dot"></div>
        </div>
        <div class="item-meta">
          <span class="item-title">{player.currentSong.title}</span>
          <span class="item-artist">{player.currentSong.artist}</span>
        </div>
        <span class="item-dur">{formatDuration(player.currentSong.duration)}</span>
      </div>
    </div>
  {/if}

  {#if player.queue.length > 0}
    {@const upNext = player.queue.slice(player.queueIndex + 1)}
    {#if upNext.length > 0}
      <div class="section">
        <p class="section-label">Up Next <span class="count">{upNext.length}</span></p>
        <div class="queue-list">
          {#each upNext as song, i}
            {@const absIdx = player.queueIndex + 1 + i}
            <button class="queue-item" onclick={() => player.playAt(absIdx)}>
              <span class="item-num">{i + 1}</span>
              <div class="item-art">
                {#if song.albumArt}
                  <img src={song.albumArt} alt="" />
                {:else}
                  <div class="art-placeholder"><Icon name="music" size={12} /></div>
                {/if}
              </div>
              <div class="item-meta">
                <span class="item-title">{song.title}</span>
                <span class="item-artist">{song.artist}</span>
              </div>
              <span class="item-dur">{formatDuration(song.duration)}</span>
            </button>
          {/each}
        </div>
      </div>
    {:else}
      <div class="empty-queue">
        <Icon name="queue" size={32} />
        <p>Queue kosong</p>
      </div>
    {/if}
  {:else}
    <div class="empty-queue">
      <Icon name="queue" size={32} />
      <p>Belum ada queue</p>
    </div>
  {/if}
</aside>

<style>
  .queue-sidebar {
    width: 280px;
    min-width: 280px;
    height: 100%;
    background: var(--surface-container-lowest);
    border-left: 1px solid var(--outline-variant);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .queue-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 16px 12px;
    flex-shrink: 0;
    border-bottom: 1px solid var(--outline-variant);
  }

  .queue-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--on-surface);
  }

  .close-btn {
    display: flex;
    align-items: center;
    padding: 4px;
    color: var(--on-surface-variant);
    border-radius: var(--radius);
    transition: color 0.12s;
  }

  .close-btn:hover { color: var(--on-surface); }

  .section {
    padding: 12px 8px 0;
    flex-shrink: 0;
  }

  .section-label {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--outline);
    padding: 0 8px 8px;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .count {
    background: var(--surface-container-high);
    color: var(--on-surface-variant);
    font-size: 10px;
    padding: 1px 5px;
    border-radius: 10px;
    letter-spacing: 0;
    text-transform: none;
    font-weight: 500;
  }

  .queue-list {
    overflow-y: auto;
    max-height: calc(100vh - 220px);
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .queue-list::-webkit-scrollbar { width: 3px; }
  .queue-list::-webkit-scrollbar-thumb { background: var(--outline-variant); border-radius: 2px; }

  .queue-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 7px 8px;
    border-radius: var(--radius);
    text-align: left;
    width: 100%;
    transition: background 0.1s;
  }

  .queue-item:not(.current):hover { background: var(--surface-container); }

  .queue-item.current {
    background: var(--primary-container);
  }

  .item-num {
    font-size: 11px;
    color: var(--outline);
    width: 16px;
    text-align: center;
    flex-shrink: 0;
  }

  .item-art {
    position: relative;
    width: 36px;
    height: 36px;
    border-radius: var(--radius);
    overflow: hidden;
    flex-shrink: 0;
    background: var(--surface-container-high);
  }

  .item-art img {
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

  .playing-dot {
    position: absolute;
    inset: 0;
    background: rgba(0,0,0,0.45);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .playing-dot::after {
    content: '▶';
    font-size: 10px;
    color: var(--secondary);
  }

  .item-meta {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .item-title {
    font-size: 12px;
    font-weight: 500;
    color: var(--on-surface);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .queue-item.current .item-title { color: var(--on-primary-container); }

  .item-artist {
    font-size: 11px;
    color: var(--on-surface-variant);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-dur {
    font-size: 11px;
    color: var(--outline);
    flex-shrink: 0;
    font-variant-numeric: tabular-nums;
  }

  .empty-queue {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--outline);
    font-size: 13px;
    padding: 40px 0;
  }
</style>
