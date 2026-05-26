<script lang="ts">
  import Icon from '../components/Icon.svelte'
  import { library } from '../stores/library.svelte'
  import { player } from '../stores/player.svelte'
  import type { Song } from '../../types'
  import { api } from '../api'

  function formatDuration(s: number): string {
    if (!s) return '--'
    const m = Math.floor(s / 60)
    const sec = Math.floor(s % 60)
    return `${m}:${sec.toString().padStart(2, '0')}`
  }

  function playSong(song: Song) {
    player.playSong(song, library.likedSongs)
  }

  async function toggleLike(song: Song) {
    await api.invoke('library:toggle-like', song.id)
    library.toggleLike(song.id)
  }
</script>

<div class="liked-view">
  <div class="header">
    <div class="header-art">
      <Icon name="heart-filled" size={48} />
    </div>
    <div class="header-info">
      <span class="label-sm">Playlist</span>
      <h1>Liked Songs</h1>
      <p>{library.likedSongs.length} songs</p>
    </div>
  </div>

  {#if library.likedSongs.length === 0}
    <div class="empty-state">
      <Icon name="heart" size={48} />
      <p>No liked songs yet</p>
      <p class="sub">Click the heart icon on any track to like it</p>
    </div>
  {:else}
    <div class="content">
      <table class="songs-table">
        <thead>
          <tr>
            <th class="col-num">#</th>
            <th class="col-title">Title</th>
            <th class="col-album">Album</th>
            <th class="col-duration"><Icon name="queue" size={13} /></th>
            <th class="col-like"></th>
          </tr>
        </thead>
        <tbody>
          {#each library.likedSongs as song, i}
            {@const isActive = player.currentSong?.id === song.id}
            <tr class="song-row" class:active={isActive} ondblclick={() => playSong(song)}>
              <td class="col-num">
                {#if isActive && player.isPlaying}
                  <span class="playing-indicator">▶</span>
                {:else}
                  <span class="row-num">{i + 1}</span>
                {/if}
              </td>
              <td class="col-title">
                <div class="title-cell">
                  <div class="thumb">
                    {#if song.albumArt}
                      <img src={song.albumArt} alt="" />
                    {:else}
                      <Icon name="music" size={14} />
                    {/if}
                  </div>
                  <div class="title-meta">
                    <span class="song-name">{song.title}</span>
                    <span class="song-artist">{song.artist}</span>
                  </div>
                </div>
              </td>
              <td class="col-album text-muted">{song.album}</td>
              <td class="col-duration text-muted">{formatDuration(song.duration)}</td>
              <td class="col-like">
                <button class="like-btn liked" onclick={() => toggleLike(song)}>
                  <Icon name="heart-filled" size={14} />
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .liked-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .header {
    display: flex;
    align-items: flex-end;
    gap: 24px;
    padding: 40px 32px 24px;
    background: linear-gradient(to bottom, var(--primary-container), var(--surface));
    flex-shrink: 0;
  }

  .header-art {
    width: 120px;
    height: 120px;
    border-radius: var(--radius-lg);
    background: var(--primary-container);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--secondary);
    flex-shrink: 0;
  }

  .header-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .header-info h1 {
    font-size: 40px;
    font-weight: 700;
    letter-spacing: -0.03em;
    color: var(--on-surface);
  }

  .header-info p {
    font-size: 13px;
    color: var(--on-surface-variant);
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 0 32px 32px;
  }

  .songs-table {
    width: 100%;
    border-collapse: collapse;
  }

  .songs-table thead tr {
    border-bottom: 1px solid var(--outline-variant);
  }

  .songs-table th {
    padding: 8px 12px;
    text-align: left;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--on-surface-variant);
  }

  .songs-table td { padding: 6px 12px; }

  .song-row {
    border-radius: var(--radius);
    transition: background 0.1s;
    cursor: pointer;
  }

  .song-row:hover { background: var(--surface-container); }
  .song-row.active { background: var(--primary-container); }

  .col-num { width: 40px; text-align: center; }
  .col-album { width: 200px; }
  .col-duration { width: 60px; text-align: right; }
  .col-like { width: 40px; text-align: center; }

  .row-num, .playing-indicator { font-size: 12px; color: var(--on-surface-variant); }
  .playing-indicator { color: var(--secondary); }

  .title-cell { display: flex; align-items: center; gap: 10px; }

  .thumb {
    width: 36px;
    height: 36px;
    border-radius: var(--radius);
    overflow: hidden;
    flex-shrink: 0;
    background: var(--surface-container-high);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--outline);
  }

  .thumb img { width: 100%; height: 100%; object-fit: cover; }

  .title-meta { display: flex; flex-direction: column; min-width: 0; }

  .song-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--on-surface);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .song-artist { font-size: 11px; color: var(--on-surface-variant); }
  .text-muted { color: var(--on-surface-variant); font-size: 13px; }

  .like-btn { color: var(--secondary); display: flex; }
  .like-btn:hover { opacity: 0.7; }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 80px 0;
    color: var(--outline);
    text-align: center;
  }

  .empty-state .sub { font-size: 13px; }
</style>
