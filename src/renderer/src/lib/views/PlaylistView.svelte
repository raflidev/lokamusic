<script lang="ts">
  import Icon from '../components/Icon.svelte'
  import { library } from '../stores/library.svelte'
  import { player } from '../stores/player.svelte'
  import { ui } from '../stores/ui.svelte'
  import { playlistStore } from '../stores/playlists.svelte'
  import type { Song, Playlist } from '../../types'
  import { api } from '../api'

  let hoveredRowId = $state<string | null>(null)
  let editingName = $state(false)
  let nameInput = $state('')

  // Sort state
  type SortKey = 'title' | 'artist' | 'album' | 'dateAdded' | 'duration'
  type SortDir = 'asc' | 'desc'
  const defaultDir: Record<SortKey, SortDir> = {
    title: 'asc',
    artist: 'asc',
    album: 'asc',
    dateAdded: 'desc',
    duration: 'desc'
  }
  const sortLabels: Record<SortKey, string> = {
    title: 'Title',
    artist: 'Artist',
    album: 'Album',
    dateAdded: 'Date Added',
    duration: 'Duration'
  }
  let sortKey = $state<SortKey>('title')
  let sortDir = $state<SortDir>('asc')
  let showSortDropdown = $state(false)

  function toggleSort(key: SortKey) {
    if (sortKey === key) {
      sortDir = sortDir === 'asc' ? 'desc' : 'asc'
    } else {
      sortKey = key
      sortDir = defaultDir[key]
    }
    showSortDropdown = false
  }

  const sortArrow = $derived(sortDir === 'asc' ? '↑' : '↓')
  const sortLabel = $derived(`Sort: ${sortLabels[sortKey]} ${sortArrow}`)

  const playlist = $derived(
    playlistStore.all.find(p => p.id === ui.selectedPlaylistId) ?? null
  )

  const playlistSongs = $derived(
    playlist
      ? library.songs.filter(s => playlist.songIds.includes(s.id))
      : []
  )

  const displaySongs = $derived((() => {
    const arr = [...playlistSongs]
    arr.sort((a, b) => {
      let cmp = 0
      if (sortKey === 'title') cmp = a.title.localeCompare(b.title)
      else if (sortKey === 'artist') cmp = a.artist.localeCompare(b.artist)
      else if (sortKey === 'album') cmp = a.album.localeCompare(b.album)
      else if (sortKey === 'dateAdded') cmp = a.dateAdded - b.dateAdded
      else if (sortKey === 'duration') cmp = a.duration - b.duration
      return sortDir === 'asc' ? cmp : -cmp
    })
    return arr
  })())

  function playSong(song: Song) {
    player.playSong(song, displaySongs)
    api.invoke('library:update-play', song.id)
  }

  function formatDuration(s: number): string {
    if (!s) return '--'
    const m = Math.floor(s / 60)
    const sec = Math.floor(s % 60)
    return `${m}:${sec.toString().padStart(2, '0')}`
  }

  function formatDate(ts: number): string {
    if (!ts) return '--'
    const d = new Date(ts)
    return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })
  }

  function startEditName() {
    if (!playlist) return
    nameInput = playlist.name
    editingName = true
  }

  async function saveNameEdit() {
    if (!playlist || !nameInput.trim()) { editingName = false; return }
    const updated = await api.invoke('playlist:rename', playlist.id, nameInput.trim()) as Playlist | null
    if (updated) playlistStore.update(updated)
    editingName = false
  }

  async function deletePlaylist() {
    if (!playlist) return
    if (!confirm(`Delete playlist "${playlist.name}"?`)) return
    await api.invoke('playlist:delete', playlist.id)
    playlistStore.remove(playlist.id)
    ui.navigate('library')
  }

  async function removeSong(songId: string) {
    if (!playlist) return
    await api.invoke('playlist:remove-song', playlist.id, songId)
    playlistStore.removeSong(playlist.id, songId)
  }
</script>

<!-- Backdrop for sort dropdown -->
{#if showSortDropdown}
  <div class="backdrop" onclick={() => showSortDropdown = false}></div>
{/if}

<div class="playlist-view">
  {#if !playlist}
    <div class="not-found">
      <Icon name="music" size={48} />
      <p>Playlist not found</p>
      <button class="action-btn" onclick={() => ui.navigate('library')}>Back to Library</button>
    </div>
  {:else}
    <div class="topbar">
      <button class="back-btn" onclick={() => ui.navigate('library')} title="Back">
        <Icon name="arrow-right" size={16} />
      </button>

      <div class="playlist-title-wrap">
        {#if editingName}
          <input
            class="name-input"
            type="text"
            bind:value={nameInput}
            onkeydown={(e) => {
              if (e.key === 'Enter') saveNameEdit()
              if (e.key === 'Escape') editingName = false
            }}
            onblur={saveNameEdit}
            autofocus
          />
        {:else}
          <button class="playlist-name" onclick={startEditName} title="Click to rename">
            {playlist.name}
          </button>
        {/if}
        <span class="song-count">{displaySongs.length} songs</span>
      </div>

      <div class="topbar-actions">
        <!-- Sort dropdown -->
        <div class="dropdown-wrap">
          <button class="pill-btn" onclick={() => showSortDropdown = !showSortDropdown}>
            <Icon name="sort" size={13} /> {sortLabel}
          </button>
          {#if showSortDropdown}
            <div class="dropdown">
              {#each Object.entries(sortLabels) as [key, label]}
                <button
                  class="dropdown-item"
                  class:selected={sortKey === key}
                  onclick={() => toggleSort(key as SortKey)}
                >
                  {label}
                  {#if sortKey === key}
                    <span class="sort-dir-indicator">{sortArrow}</span>
                  {/if}
                </button>
              {/each}
            </div>
          {/if}
        </div>

        <button class="delete-btn" onclick={deletePlaylist} title="Delete playlist">
          <Icon name="trash" size={16} />
        </button>
      </div>
    </div>

    <div class="content">
      {#if displaySongs.length === 0}
        <div class="empty-state">
          <Icon name="list-music" size={48} />
          <p>No songs in this playlist.</p>
          <p class="sub">Add songs from your Library.</p>
          <button class="action-btn" onclick={() => ui.navigate('library')}>Go to Library</button>
        </div>
      {:else}
        <table class="songs-table">
          <thead>
            <tr>
              <th class="col-num">#</th>
              <th class="col-title">Title</th>
              <th class="col-album">Album</th>
              <th class="col-date">Date Added</th>
              <th class="col-duration"><Icon name="queue" size={13} /></th>
              <th class="col-actions"></th>
            </tr>
          </thead>
          <tbody>
            {#each displaySongs as song, i}
              {@const isActive = player.currentSong?.id === song.id}
              <tr
                class="song-row"
                class:active={isActive}
                onmouseenter={() => hoveredRowId = song.id}
                onmouseleave={() => hoveredRowId = null}
                ondblclick={() => playSong(song)}
              >
                <td class="col-num">
                  <div class="num-cell">
                    {#if isActive && player.isPlaying}
                      <button class="num-play-btn" onclick={() => player.togglePlay()}>
                        <span class="playing-indicator">▶</span>
                      </button>
                    {:else if isActive && !player.isPlaying}
                      <button class="num-play-btn" onclick={() => player.togglePlay()}>
                        <Icon name="play" size={13} />
                      </button>
                    {:else if hoveredRowId === song.id}
                      <button class="num-play-btn" onclick={() => playSong(song)}>
                        <Icon name="play" size={13} />
                      </button>
                    {:else}
                      <span class="row-num">{i + 1}</span>
                    {/if}
                  </div>
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
                <td class="col-date text-muted">{formatDate(song.dateAdded)}</td>
                <td class="col-duration text-muted">{formatDuration(song.duration)}</td>
                <td class="col-actions">
                  <button
                    class="row-action-btn"
                    title="Remove from playlist"
                    onclick={() => removeSong(song.id)}
                  >
                    <Icon name="trash" size={14} />
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  {/if}
</div>

<style>
  .playlist-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .topbar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 32px;
    flex-shrink: 0;
    border-bottom: 1px solid var(--outline-variant);
  }

  .back-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--on-surface-variant);
    transform: rotate(180deg);
    padding: 6px;
    border-radius: var(--radius);
    transition: color 0.12s, background 0.12s;
    flex-shrink: 0;
  }

  .back-btn:hover {
    color: var(--on-surface);
    background: var(--surface-container);
  }

  .playlist-title-wrap {
    flex: 1;
    display: flex;
    align-items: baseline;
    gap: 12px;
    min-width: 0;
  }

  .playlist-name {
    font-size: 20px;
    font-weight: 600;
    letter-spacing: -0.02em;
    color: var(--secondary);
    background: none;
    border: none;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: var(--radius);
    transition: background 0.12s;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 400px;
  }

  .playlist-name:hover {
    background: var(--surface-container);
  }

  .name-input {
    font-size: 20px;
    font-weight: 600;
    letter-spacing: -0.02em;
    color: var(--secondary);
    background: var(--surface-container);
    border: 1px solid var(--primary);
    border-radius: var(--radius);
    padding: 2px 6px;
    outline: none;
    min-width: 200px;
  }

  .song-count {
    font-size: 13px;
    color: var(--outline);
    flex-shrink: 0;
  }

  .topbar-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .dropdown-wrap {
    position: relative;
  }

  .pill-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 14px;
    border: 1px solid var(--outline-variant);
    border-radius: var(--radius-xl);
    font-size: 13px;
    color: var(--on-surface-variant);
    transition: border-color 0.12s, color 0.12s;
  }

  .pill-btn:hover {
    border-color: var(--outline);
    color: var(--on-surface);
  }

  .dropdown {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    background: var(--surface-container-high);
    border: 1px solid var(--outline-variant);
    border-radius: var(--radius-lg);
    padding: 4px;
    min-width: 160px;
    z-index: 200;
    box-shadow: 0 4px 16px rgba(0,0,0,0.3);
  }

  .dropdown-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 7px 12px;
    border-radius: var(--radius);
    font-size: 13px;
    color: var(--on-surface-variant);
    text-align: left;
    transition: background 0.1s, color 0.1s;
  }

  .dropdown-item:hover {
    background: var(--surface-container);
    color: var(--on-surface);
  }

  .dropdown-item.selected {
    color: var(--on-surface);
    font-weight: 500;
  }

  .sort-dir-indicator {
    font-size: 12px;
    color: var(--primary);
  }

  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 99;
  }

  .delete-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 7px;
    border-radius: var(--radius-lg);
    color: var(--on-surface-variant);
    transition: color 0.12s, background 0.12s;
  }

  .delete-btn:hover {
    color: #e57373;
    background: var(--surface-container);
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 16px 32px 32px;
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

  .songs-table td {
    padding: 0 12px;
    font-size: 13px;
    height: 48px;
  }

  .song-row {
    border-radius: var(--radius);
    transition: background 0.1s;
    cursor: pointer;
  }

  .song-row:hover { background: var(--surface-container); }
  .song-row.active { background: var(--primary-container); }
  .song-row.active .song-name { color: var(--on-primary-container); }

  .col-num { width: 40px; text-align: center; }
  .col-title { }
  .col-album { width: 200px; }
  .col-date { width: 140px; }
  .col-duration { width: 60px; text-align: right; }
  .col-actions { width: 40px; text-align: right; }

  .num-cell {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto;
  }

  .row-num {
    font-size: 12px;
    color: var(--on-surface-variant);
    line-height: 1;
  }

  .num-play-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    color: var(--on-surface);
    transition: color 0.1s;
    flex-shrink: 0;
  }

  .num-play-btn:hover { color: var(--secondary); }

  .playing-indicator {
    font-size: 10px;
    color: var(--secondary);
    line-height: 1;
  }

  .title-cell {
    display: flex;
    align-items: center;
    gap: 10px;
  }

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

  .thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .title-meta {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .song-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--on-surface);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .song-artist {
    font-size: 11px;
    color: var(--on-surface-variant);
  }

  .text-muted { color: var(--on-surface-variant); }

  .row-action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius);
    color: var(--on-surface-variant);
    opacity: 0;
    transition: opacity 0.1s, color 0.1s, background 0.1s;
    margin-left: auto;
  }

  .song-row:hover .row-action-btn { opacity: 1; }

  .row-action-btn:hover {
    color: #e57373;
    background: var(--surface-container-high);
  }

  .not-found, .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    flex: 1;
    padding: 80px 0;
    color: var(--outline);
    text-align: center;
  }

  .empty-state .sub {
    font-size: 13px;
    color: var(--outline);
  }

  .action-btn {
    margin-top: 8px;
    padding: 8px 20px;
    background: var(--primary-container);
    color: var(--on-primary-container);
    border-radius: var(--radius-lg);
    font-size: 13px;
    font-weight: 500;
    transition: background 0.12s;
  }

  .action-btn:hover { background: var(--primary); color: var(--on-primary); }
</style>
