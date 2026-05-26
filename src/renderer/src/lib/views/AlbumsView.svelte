<script lang="ts">
  import Icon from '../components/Icon.svelte'
  import { library } from '../stores/library.svelte'
  import { player } from '../stores/player.svelte'
  import { ui } from '../stores/ui.svelte'
  import type { Song } from '../../../../../preload/index.d'

  interface Album {
    name: string
    artist: string
    art?: string
    songs: Song[]
  }

  const albums = $derived.by(() => {
    const map = new Map<string, Album>()
    for (const song of library.songs) {
      const key = `${song.album}__${song.artist}`
      if (!map.has(key)) {
        map.set(key, { name: song.album, artist: song.artist, art: song.albumArt, songs: [] })
      }
      map.get(key)!.songs.push(song)
      if (!map.get(key)!.art && song.albumArt) map.get(key)!.art = song.albumArt
    }
    return [...map.values()].sort((a, b) => a.name.localeCompare(b.name))
  })

  let selectedAlbum = $state<Album | null>(null)
  let hoveredRowId = $state<string | null>(null)

  $effect(() => {
    const key = ui.pendingAlbumKey
    if (key) {
      selectedAlbum = albums.find(a => `${a.name}__${a.artist}` === key) ?? null
      ui.clearPendingAlbum()
    }
  })

  function formatDuration(s: number): string {
    if (!s) return '--'
    return `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, '0')}`
  }

  function playAlbum(album: Album, startSong?: Song) {
    const song = startSong ?? album.songs[0]
    player.playSong(song, album.songs)
    window.electronAPI.invoke('library:update-play', song.id)
  }
</script>

{#if selectedAlbum}
  <!-- Album detail -->
  <div class="album-detail">
    <div class="detail-header">
      <button class="back-btn" onclick={() => selectedAlbum = null}>
        <Icon name="arrow-right" size={14} />
      </button>
      <div class="detail-art">
        {#if selectedAlbum.art}
          <img src={selectedAlbum.art} alt="" />
        {:else}
          <div class="detail-art-placeholder"><Icon name="music" size={36} /></div>
        {/if}
      </div>
      <div class="detail-info">
        <span class="label-sm">Album</span>
        <h1>{selectedAlbum.name}</h1>
        <p>{selectedAlbum.artist} · {selectedAlbum.songs.length} songs</p>
        <button class="play-all-btn" onclick={() => playAlbum(selectedAlbum!)}>
          <Icon name="play" size={14} /> Play All
        </button>
      </div>
    </div>

    <table class="songs-table">
      <thead>
        <tr>
          <th class="col-num">#</th>
          <th>Title</th>
          <th class="col-duration"><Icon name="queue" size={13} /></th>
        </tr>
      </thead>
      <tbody>
        {#each selectedAlbum.songs as song, i}
          {@const isActive = player.currentSong?.id === song.id}
          <tr
            class="song-row"
            class:active={isActive}
            onmouseenter={() => hoveredRowId = song.id}
            onmouseleave={() => hoveredRowId = null}
            ondblclick={() => playAlbum(selectedAlbum!, song)}
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
                  <button class="num-play-btn" onclick={() => playAlbum(selectedAlbum!, song)}>
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
                <span class="song-artist">{song.artist}</span>
              </div>
            </td>
            <td class="col-duration text-muted">{formatDuration(song.duration)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{:else}
  <!-- Albums grid -->
  <div class="albums-view">
    <div class="view-header">
      <h2 class="section-title">Albums</h2>
      <span class="count">{albums.length} albums</span>
    </div>

    {#if albums.length === 0}
      <div class="empty-state">
        <Icon name="music" size={48} />
        <p>No albums yet</p>
        <p class="sub">Scan a folder to see your albums</p>
      </div>
    {:else}
      <div class="albums-grid">
        {#each albums as album}
          <button class="album-card" onclick={() => selectedAlbum = album}>
            <div class="album-art">
              {#if album.art}
                <img src={album.art} alt="" />
              {:else}
                <div class="art-placeholder"><Icon name="music" size={32} /></div>
              {/if}
            </div>
            <p class="album-name">{album.name}</p>
            <p class="album-artist">{album.artist}</p>
          </button>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  /* Albums grid */
  .albums-view {
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

  .count {
    font-size: 13px;
    color: var(--outline);
  }

  .albums-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 20px 16px;
  }

  .album-card {
    text-align: left;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
  }

  .album-art {
    position: relative;
    width: 100%;
    aspect-ratio: 1;
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: var(--surface-container-high);
    margin-bottom: 8px;
  }

  .album-art img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .art-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--outline);
  }

  .album-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--on-surface);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .album-artist {
    font-size: 12px;
    color: var(--on-surface-variant);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* Album detail */
  .album-detail {
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

  .detail-art {
    width: 140px;
    height: 140px;
    border-radius: var(--radius-lg);
    overflow: hidden;
    flex-shrink: 0;
    background: var(--surface-container-high);
    box-shadow: 0 8px 32px rgba(0,0,0,0.4);
  }

  .detail-art img { width: 100%; height: 100%; object-fit: cover; }

  .detail-art-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--outline);
  }

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
    transition: background 0.12s;
    width: fit-content;
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
  .song-artist { font-size: 11px; color: var(--on-surface-variant); }
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
