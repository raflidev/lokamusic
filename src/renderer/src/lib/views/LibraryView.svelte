<script lang="ts">
  import Icon from '../components/Icon.svelte'
  import { library } from '../stores/library.svelte'
  import { player } from '../stores/player.svelte'
  import { ui } from '../stores/ui.svelte'
  import { playlistStore } from '../stores/playlists.svelte'
  import type { Song, Playlist } from '../../types'
  import { api } from '../api'
  import { getArt } from '../stores/artCache.svelte'

  let searchQuery = $derived(ui.searchQuery)
  let hoveredRowId = $state<string | null>(null)

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

  // Filter state
  type FilterMode = 'all' | 'liked'
  let filterMode = $state<FilterMode>('all')
  let showFilterDropdown = $state(false)

  // Playlist menu state
  let playlistMenuSongId = $state<string | null>(null)
  let playlistMenuAnchor = $state<{ top: number; right: number } | null>(null)
  let newPlaylistName = $state('')
  let creatingFromMenu = $state(false)

  function toggleSort(key: SortKey) {
    if (sortKey === key) {
      sortDir = sortDir === 'asc' ? 'desc' : 'asc'
    } else {
      sortKey = key
      sortDir = defaultDir[key]
    }
  }

  const filterLabel = $derived(filterMode === 'liked' ? 'Filter: Liked' : 'Filter')

  const activeFolder = $derived(
    ui.selectedFolderId ? library.folders.find(f => f.id === ui.selectedFolderId) ?? null : null
  )

  const baseSongs = $derived(
    ui.selectedSubFolderPath
      ? library.songs.filter(s => s.path.startsWith(ui.selectedSubFolderPath! + '/'))
      : activeFolder
        ? library.songs.filter(s => s.path.startsWith(activeFolder.path))
        : library.songs
  )

  const filteredSongs = $derived(
    searchQuery.trim()
      ? baseSongs.filter(s =>
          s.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
          s.artist.toLowerCase().includes(searchQuery.toLowerCase()) ||
          s.album.toLowerCase().includes(searchQuery.toLowerCase())
        )
      : baseSongs
  )

  const afterFilterSongs = $derived(
    filterMode === 'liked' ? filteredSongs.filter(s => s.liked) : filteredSongs
  )

  const displaySongs = $derived((() => {
    const arr = [...afterFilterSongs]
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

  function openPlaylistMenu(e: MouseEvent, songId: string) {
    e.stopPropagation()
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
    playlistMenuSongId = songId
    playlistMenuAnchor = { top: rect.bottom + 4, right: window.innerWidth - rect.right }
    newPlaylistName = ''
    creatingFromMenu = false
  }

  function closePlaylistMenu() {
    playlistMenuSongId = null
    playlistMenuAnchor = null
    newPlaylistName = ''
    creatingFromMenu = false
  }

  async function addToPlaylist(playlist: Playlist) {
    if (!playlistMenuSongId) return
    await api.invoke('playlist:add-song', playlist.id, playlistMenuSongId)
    playlistStore.addSong(playlist.id, playlistMenuSongId)
    closePlaylistMenu()
  }

  async function createPlaylistAndAdd() {
    if (!playlistMenuSongId || !newPlaylistName.trim()) return
    const pl = await api.invoke('playlist:create', newPlaylistName.trim()) as Playlist
    playlistStore.add(pl)
    await api.invoke('playlist:add-song', pl.id, playlistMenuSongId)
    playlistStore.addSong(pl.id, playlistMenuSongId)
    closePlaylistMenu()
  }
</script>

<!-- Backdrop for filter dropdown -->
{#if showFilterDropdown}
  <div class="backdrop" onclick={() => showFilterDropdown = false}></div>
{/if}

<!-- Backdrop for playlist menu -->
{#if playlistMenuSongId}
  <div class="backdrop" onclick={closePlaylistMenu}></div>
{/if}

<!-- Playlist menu (fixed position) -->
{#if playlistMenuSongId && playlistMenuAnchor}
  <div class="playlist-menu" style="top: {playlistMenuAnchor.top}px; right: {playlistMenuAnchor.right}px;">
    {#if playlistStore.all.length === 0 && !creatingFromMenu}
      <p class="playlist-menu-empty">No playlists yet</p>
    {:else}
      {#each playlistStore.all as pl}
        <button class="playlist-menu-item" onclick={() => addToPlaylist(pl)}>
          <Icon name="list-music" size={13} />
          <span>{pl.name}</span>
        </button>
      {/each}
    {/if}
    {#if creatingFromMenu}
      <div class="playlist-menu-create">
        <input
          type="text"
          placeholder="Playlist name…"
          bind:value={newPlaylistName}
          onkeydown={(e) => {
            if (e.key === 'Enter') createPlaylistAndAdd()
            if (e.key === 'Escape') { creatingFromMenu = false; newPlaylistName = '' }
          }}
          autofocus
        />
        <button class="playlist-menu-confirm" onclick={createPlaylistAndAdd}>Create</button>
      </div>
    {:else}
      <div class="playlist-menu-divider"></div>
      <button class="playlist-menu-item playlist-menu-new" onclick={() => { creatingFromMenu = true }}>
        <Icon name="plus" size={13} />
        <span>New Playlist</span>
      </button>
    {/if}
  </div>
{/if}

<div class="library-view">
  <div class="topbar">
    <div class="search-wrap">
      <Icon name="search" size={15} />
      <input
        type="text"
        placeholder="Search tracks, albums, artists…"
        value={ui.searchQuery}
        oninput={(e) => ui.setSearchQuery((e.target as HTMLInputElement).value)}
      />
    </div>
    <div class="topbar-actions">
    </div>
  </div>

  <div class="content">
    {#if library.recentlyPlayed.length > 0 && !searchQuery && !activeFolder && !ui.selectedSubFolderPath}
      <section class="recently-played">
        <h2 class="section-title">Recently Played</h2>
        <div class="recent-strip">
          {#each library.recentlyPlayed as song, i}
            <button class="strip-card" onclick={() => playSong(song)}>
              <div class="strip-art">
                {#if getArt(song.id)}
                  <img src={getArt(song.id)} alt="" />
                {:else}
                  <div class="strip-art-placeholder"><Icon name="music" size={28} /></div>
                {/if}
                {#if i === 0}
                  <span class="strip-badge">MOST PLAYED</span>
                {/if}
                <div class="strip-play-overlay"><Icon name="play" size={20} /></div>
              </div>
              <p class="strip-title">{song.title}</p>
              <p class="strip-artist">{song.artist}</p>
            </button>
          {/each}
        </div>
      </section>
    {/if}

    <section class="all-songs">
      <div class="all-songs-header">
        <div class="title-wrap">
          {#if activeFolder || ui.selectedSubFolderPath}
            <button class="back-btn" onclick={() => ui.clearFolderFilter()} title="Back to all songs">
              <Icon name="arrow-right" size={14} class="back-arrow" />
            </button>
          {/if}
          <h2 class="section-title">
            {ui.selectedSubFolderPath
              ? (ui.selectedSubFolderPath.split('/').pop() ?? 'Folder')
              : activeFolder ? activeFolder.name : 'All Songs'}
          </h2>
          {#if activeFolder || ui.selectedSubFolderPath}
            <span class="folder-count">{displaySongs.length} songs</span>
          {/if}
        </div>
        <div class="header-actions">
          <!-- Filter dropdown -->
          <div class="dropdown-wrap">
            <button
              class="pill-btn"
              class:active={filterMode !== 'all'}
              onclick={() => { showFilterDropdown = !showFilterDropdown }}
            >
              <Icon name="filter" size={13} /> {filterLabel}
            </button>
            {#if showFilterDropdown}
              <div class="dropdown">
                <button
                  class="dropdown-item"
                  class:selected={filterMode === 'all'}
                  onclick={() => { filterMode = 'all'; showFilterDropdown = false }}
                >All</button>
                <button
                  class="dropdown-item"
                  class:selected={filterMode === 'liked'}
                  onclick={() => { filterMode = 'liked'; showFilterDropdown = false }}
                >Liked Only</button>
              </div>
            {/if}
          </div>
        </div>
      </div>

      {#if library.songs.length === 0}
        <div class="empty-state">
          <Icon name="music" size={48} />
          <p>No songs in your library</p>
          <p class="sub">Go to Folders to scan your music</p>
          <button class="action-btn" onclick={() => ui.navigate('folders')}>Open Folders</button>
        </div>
      {:else}
        <table class="songs-table">
          <thead>
            <tr>
              <th class="col-num">#</th>
              <th class="col-title sortable" class:sort-active={sortKey === 'title'} onclick={() => toggleSort('title')}>
                Title {#if sortKey === 'title'}<span class="sort-ind">{sortDir === 'asc' ? '↑' : '↓'}</span>{/if}
              </th>
              <th class="col-album sortable" class:sort-active={sortKey === 'album'} onclick={() => toggleSort('album')}>
                Album {#if sortKey === 'album'}<span class="sort-ind">{sortDir === 'asc' ? '↑' : '↓'}</span>{/if}
              </th>
              <th class="col-date sortable" class:sort-active={sortKey === 'dateAdded'} onclick={() => toggleSort('dateAdded')}>
                Date Added {#if sortKey === 'dateAdded'}<span class="sort-ind">{sortDir === 'asc' ? '↑' : '↓'}</span>{/if}
              </th>
              <th class="col-duration sortable" class:sort-active={sortKey === 'duration'} onclick={() => toggleSort('duration')}>
                <Icon name="queue" size={13} /> {#if sortKey === 'duration'}<span class="sort-ind">{sortDir === 'asc' ? '↑' : '↓'}</span>{/if}
              </th>
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
                      {#if getArt(song.id)}
                        <img src={getArt(song.id)} alt="" />
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
                <td class="col-album">
                  <button class="album-link" onclick={(e) => { e.stopPropagation(); ui.navigateToAlbum(song.album, song.artist) }}>
                    {song.album}
                  </button>
                </td>
                <td class="col-date text-muted">{formatDate(song.dateAdded)}</td>
                <td class="col-duration text-muted">{formatDuration(song.duration)}</td>
                <td class="col-actions">
                  <button
                    class="row-action-btn"
                    title="Add to playlist"
                    onclick={(e) => openPlaylistMenu(e, song.id)}
                  >
                    <Icon name="plus" size={14} />
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>
  </div>
</div>

<style>
  .library-view {
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
  }

  .search-wrap {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 10px;
    background: var(--surface-container);
    border: 1px solid var(--outline-variant);
    border-radius: var(--radius-xl);
    padding: 8px 14px;
    color: var(--on-surface-variant);
  }

  .search-wrap input {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    color: var(--on-surface);
    font-size: 14px;
  }

  .search-wrap input::placeholder { color: var(--outline); }

  .topbar-actions {
    display: flex;
    gap: 4px;
  }

  .icon-btn {
    color: var(--on-surface-variant);
    padding: 8px;
    border-radius: var(--radius-lg);
    display: flex;
    transition: color 0.12s, background 0.12s;
  }

  .icon-btn:hover {
    color: var(--on-surface);
    background: var(--surface-container);
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 0 32px 32px;
  }

  .section-title {
    font-size: 20px;
    font-weight: 600;
    letter-spacing: -0.02em;
    color: var(--secondary);
    margin-bottom: 16px;
  }

  /* Recently Played — horizontal strip */
  .recently-played { margin-bottom: 32px; }

  .recent-strip {
    display: flex;
    gap: 12px;
    overflow-x: auto;
    padding-bottom: 8px;
  }

  .recent-strip::-webkit-scrollbar { height: 4px; }
  .recent-strip::-webkit-scrollbar-thumb { background: var(--outline-variant); border-radius: 2px; }

  .strip-card {
    flex-shrink: 0;
    width: 130px;
    text-align: left;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
  }

  .strip-art {
    position: relative;
    width: 130px;
    height: 130px;
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: var(--surface-container-high);
    margin-bottom: 8px;
  }

  .strip-art img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
    transition: filter 0.15s;
  }

  .strip-art-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--outline);
  }

  .strip-badge {
    position: absolute;
    top: 8px;
    left: 8px;
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.08em;
    background: var(--secondary-container);
    color: var(--on-secondary-container);
    padding: 3px 6px;
    border-radius: 2px;
  }

  .strip-play-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0,0,0,0.4);
    color: #fff;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .strip-card:hover .strip-play-overlay { opacity: 1; }
  .strip-card:hover .strip-art img { filter: brightness(0.75); }

  .strip-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--on-surface);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .strip-artist {
    font-size: 11px;
    color: var(--on-surface-variant);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* All songs */
  .all-songs-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }

  .title-wrap {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .back-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--on-surface-variant);
    transform: rotate(180deg);
    transition: color 0.12s;
  }

  .back-btn:hover { color: var(--on-surface); }

  .folder-count {
    font-size: 13px;
    color: var(--outline);
    align-self: flex-end;
    padding-bottom: 2px;
  }

  .header-actions {
    display: flex;
    gap: 8px;
  }

  /* Dropdown wrapper */
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
    transition: border-color 0.12s, color 0.12s, background 0.12s;
  }

  .pill-btn:hover {
    border-color: var(--outline);
    color: var(--on-surface);
  }

  .pill-btn.active {
    border-color: var(--primary);
    color: var(--primary);
    background: var(--primary-container);
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

  /* Backdrop */
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 99;
  }

  /* Playlist menu */
  .playlist-menu {
    position: fixed;
    background: var(--surface-container-high);
    border: 1px solid var(--outline-variant);
    border-radius: var(--radius-lg);
    padding: 4px;
    min-width: 200px;
    z-index: 1000;
    box-shadow: 0 4px 20px rgba(0,0,0,0.4);
  }

  .playlist-menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 7px 12px;
    border-radius: var(--radius);
    font-size: 13px;
    color: var(--on-surface-variant);
    text-align: left;
    transition: background 0.1s, color 0.1s;
  }

  .playlist-menu-item:hover {
    background: var(--surface-container);
    color: var(--on-surface);
  }

  .playlist-menu-new {
    color: var(--primary);
  }

  .playlist-menu-new:hover {
    color: var(--primary);
    background: var(--primary-container);
  }

  .playlist-menu-empty {
    font-size: 12px;
    color: var(--outline);
    padding: 6px 12px;
  }

  .playlist-menu-divider {
    height: 1px;
    background: var(--outline-variant);
    margin: 4px 8px;
  }

  .playlist-menu-create {
    display: flex;
    gap: 6px;
    padding: 6px 8px;
  }

  .playlist-menu-create input {
    flex: 1;
    background: var(--surface-container);
    border: 1px solid var(--outline-variant);
    border-radius: var(--radius);
    padding: 5px 8px;
    font-size: 13px;
    color: var(--on-surface);
    outline: none;
  }

  .playlist-menu-create input:focus {
    border-color: var(--primary);
  }

  .playlist-menu-confirm {
    padding: 5px 10px;
    background: var(--primary-container);
    color: var(--on-primary-container);
    border-radius: var(--radius);
    font-size: 12px;
    font-weight: 500;
    transition: background 0.1s;
  }

  .playlist-menu-confirm:hover {
    background: var(--primary);
    color: var(--on-primary);
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

  .sortable {
    cursor: pointer;
    user-select: none;
    white-space: nowrap;
  }
  .sortable:hover { color: var(--on-surface); }
  .sort-active { color: var(--primary); }
  .sort-ind { margin-left: 3px; font-size: 10px; }

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

  .album-link {
    background: none;
    border: none;
    padding: 0;
    font-size: 13px;
    color: var(--on-surface-variant);
    cursor: pointer;
    text-align: left;
    transition: color 0.1s;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    max-width: 100%;
    white-space: normal;
  }

  .album-link:hover {
    color: var(--on-surface);
    text-decoration: underline;
  }

  /* Row action button (add to playlist) */
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
    color: var(--on-surface);
    background: var(--surface-container-high);
  }

  /* Empty state */
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
