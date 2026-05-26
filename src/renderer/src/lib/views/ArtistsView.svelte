<script lang="ts">
  import Icon from '../components/Icon.svelte'
  import { library } from '../stores/library.svelte'
  import { player } from '../stores/player.svelte'
  import { ui } from '../stores/ui.svelte'
  import type { Song } from '../../types'
  import { api } from '../api'

  interface Artist {
    name: string
    art?: string
    songs: Song[]
    albumCount: number
  }

  const artists = $derived.by(() => {
    const map = new Map<string, { songs: Song[]; albums: Set<string> }>()
    for (const song of library.songs) {
      if (!map.has(song.artist)) map.set(song.artist, { songs: [], albums: new Set() })
      const entry = map.get(song.artist)!
      entry.songs.push(song)
      entry.albums.add(song.album)
    }
    return [...map.entries()]
      .map(([name, { songs, albums }]) => ({
        name,
        art: songs.find(s => s.albumArt)?.albumArt,
        songs,
        albumCount: albums.size
      }))
      .sort((a, b) => a.name.localeCompare(b.name))
  })

  let selectedArtist = $state<Artist | null>(null)
  let hoveredRowId = $state<string | null>(null)

  $effect(() => {
    if (ui.pendingArtistName) {
      selectedArtist = artists.find(a => a.name === ui.pendingArtistName) ?? null
      ui.clearPendingArtist()
    }
  })

  function formatDuration(s: number): string {
    if (!s) return '--'
    return `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, '0')}`
  }

  function playArtist(artist: Artist, startSong?: Song) {
    const song = startSong ?? artist.songs[0]
    player.playSong(song, artist.songs)
    api.invoke('library:update-play', song.id)
  }
</script>

{#if selectedArtist}
  <!-- Artist detail -->
  <div class="artist-detail">
    <div class="detail-header">
      <button class="back-btn" onclick={() => selectedArtist = null}>
        <Icon name="arrow-right" size={14} />
      </button>
      <div class="detail-avatar">
        {#if selectedArtist.art}
          <img src={selectedArtist.art} alt="" />
        {:else}
          <div class="avatar-placeholder"><Icon name="user" size={40} /></div>
        {/if}
      </div>
      <div class="detail-info">
        <span class="label-sm">Artist</span>
        <h1>{selectedArtist.name}</h1>
        <p>{selectedArtist.albumCount} albums · {selectedArtist.songs.length} songs</p>
        <button class="play-all-btn" onclick={() => playArtist(selectedArtist!)}>
          <Icon name="play" size={14} /> Play All
        </button>
      </div>
    </div>

    <table class="songs-table">
      <thead>
        <tr>
          <th class="col-num">#</th>
          <th>Title</th>
          <th class="col-album">Album</th>
          <th class="col-duration"><Icon name="queue" size={13} /></th>
        </tr>
      </thead>
      <tbody>
        {#each selectedArtist.songs as song, i}
          {@const isActive = player.currentSong?.id === song.id}
          <tr
            class="song-row"
            class:active={isActive}
            onmouseenter={() => hoveredRowId = song.id}
            onmouseleave={() => hoveredRowId = null}
            ondblclick={() => playArtist(selectedArtist!, song)}
          >
            <td class="col-num">
              <div class="num-cell">
                {#if isActive && player.isPlaying}
                  <button class="num-play-btn" onclick={() => player.togglePlay()}>
                    <span class="playing-indicator">▶</span>
                  </button>
                {:else if isActive}
                  <button class="num-play-btn" onclick={() => player.togglePlay()}>
                    <Icon name="play" size={13} />
                  </button>
                {:else if hoveredRowId === song.id}
                  <button class="num-play-btn" onclick={() => playArtist(selectedArtist!, song)}>
                    <Icon name="play" size={13} />
                  </button>
                {:else}
                  <span class="row-num">{i + 1}</span>
                {/if}
              </div>
            </td>
            <td>
              <div class="title-meta">
                <span class="song-name" class:active-name={isActive}>{song.title}</span>
              </div>
            </td>
            <td class="col-album text-muted">{song.album}</td>
            <td class="col-duration text-muted">{formatDuration(song.duration)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{:else}
  <!-- Artists list -->
  <div class="artists-view">
    <div class="view-header">
      <h2 class="section-title">Artists</h2>
      <span class="count">{artists.length} artists</span>
    </div>

    {#if artists.length === 0}
      <div class="empty-state">
        <Icon name="user" size={48} />
        <p>No artists yet</p>
        <p class="sub">Scan a folder to see your artists</p>
      </div>
    {:else}
      <div class="artists-list">
        {#each artists as artist}
          <button class="artist-row" onclick={() => selectedArtist = artist}>
            <div class="artist-avatar">
              {#if artist.art}
                <img src={artist.art} alt="" />
              {:else}
                <div class="avatar-placeholder"><Icon name="user" size={18} /></div>
              {/if}
            </div>
            <div class="artist-meta">
              <span class="artist-name">{artist.name}</span>
              <span class="artist-sub">{artist.albumCount} albums · {artist.songs.length} songs</span>
            </div>
            <Icon name="arrow-right" size={14} class="chevron" />
          </button>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  /* Artists list */
  .artists-view {
    flex: 1;
    overflow-y: auto;
    padding: 24px 32px 32px;
  }

  .view-header {
    display: flex;
    align-items: baseline;
    gap: 12px;
    margin-bottom: 20px;
  }

  .section-title {
    font-size: 20px;
    font-weight: 600;
    letter-spacing: -0.02em;
    color: var(--secondary);
  }

  .count { font-size: 13px; color: var(--outline); }

  .artists-list {
    display: flex;
    flex-direction: column;
    background: var(--surface-container-low);
    border: 1px solid var(--outline-variant);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .artist-row {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 10px 16px;
    border-bottom: 1px solid var(--outline-variant);
    cursor: pointer;
    transition: background 0.1s;
    text-align: left;
  }

  .artist-row:last-child { border-bottom: none; }
  .artist-row:hover { background: var(--surface-container); }

  .artist-avatar {
    width: 42px;
    height: 42px;
    border-radius: 50%;
    overflow: hidden;
    flex-shrink: 0;
    background: var(--surface-container-high);
  }

  .artist-avatar img { width: 100%; height: 100%; object-fit: cover; }

  .avatar-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--outline);
  }

  .artist-meta {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    gap: 2px;
  }

  .artist-name {
    font-size: 14px;
    font-weight: 500;
    color: var(--on-surface);
  }

  .artist-sub { font-size: 12px; color: var(--on-surface-variant); }

  :global(.chevron) { color: var(--outline); flex-shrink: 0; }

  /* Artist detail */
  .artist-detail {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .detail-header {
    display: flex;
    align-items: flex-end;
    gap: 20px;
    padding: 32px 32px 24px;
    background: linear-gradient(to bottom, var(--surface-container-low), var(--surface));
    flex-shrink: 0;
  }

  .back-btn {
    display: flex;
    align-items: center;
    color: var(--on-surface-variant);
    transform: rotate(180deg);
    align-self: flex-start;
    margin-top: 4px;
    flex-shrink: 0;
    transition: color 0.12s;
  }

  .back-btn:hover { color: var(--on-surface); }

  .detail-avatar {
    width: 140px;
    height: 140px;
    border-radius: 50%;
    overflow: hidden;
    flex-shrink: 0;
    background: var(--surface-container-high);
    box-shadow: 0 8px 32px rgba(0,0,0,0.4);
  }

  .detail-avatar img { width: 100%; height: 100%; object-fit: cover; }

  .detail-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .detail-info h1 {
    font-size: 28px;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--on-surface);
    margin: 4px 0;
  }

  .detail-info p { font-size: 13px; color: var(--on-surface-variant); }

  .play-all-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin-top: 12px;
    padding: 8px 18px;
    background: var(--secondary-container);
    color: var(--on-secondary-container);
    border-radius: var(--radius-xl);
    font-size: 13px;
    font-weight: 500;
    width: fit-content;
    transition: background 0.12s;
  }

  .play-all-btn:hover { background: var(--secondary); color: var(--on-secondary); }

  /* Song table */
  .songs-table {
    width: 100%;
    border-collapse: collapse;
    padding: 0 32px;
    display: table;
  }

  .songs-table thead tr { border-bottom: 1px solid var(--outline-variant); }

  .songs-table th {
    padding: 8px 12px;
    text-align: left;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--on-surface-variant);
  }

  .songs-table td { padding: 0 12px; height: 44px; font-size: 13px; }

  .song-row { cursor: pointer; transition: background 0.1s; }
  .song-row:hover { background: var(--surface-container); }
  .song-row.active { background: var(--primary-container); }

  .col-num { width: 40px; text-align: center; }
  .col-album { width: 180px; }
  .col-duration { width: 60px; text-align: right; }

  .num-cell {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto;
  }

  .row-num { font-size: 12px; color: var(--on-surface-variant); line-height: 1; }

  .num-play-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    color: var(--on-surface);
    transition: color 0.1s;
  }

  .num-play-btn:hover { color: var(--secondary); }
  .playing-indicator { font-size: 10px; color: var(--secondary); line-height: 1; }

  .title-meta { display: flex; flex-direction: column; min-width: 0; }

  .song-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--on-surface);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .song-name.active-name { color: var(--on-primary-container); }
  .text-muted { color: var(--on-surface-variant); }

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
