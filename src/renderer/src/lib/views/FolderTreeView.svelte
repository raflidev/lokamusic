<script lang="ts">
  import Icon from '../components/Icon.svelte'
  import { library } from '../stores/library.svelte'
  import { player } from '../stores/player.svelte'
  import type { Song, WatchedFolder } from '../../types'
  import { api } from '../api'
  import { getArt } from '../stores/artCache.svelte'

  type Level = { name: string; path: string; folderId?: string }

  let stack = $state<Level[]>([])
  let hoveredRowId = $state<string | null>(null)

  const currentLevel = $derived(stack.length > 0 ? stack[stack.length - 1] : null)

  // Sub-folders at current level
  const subFolders = $derived.by(() => {
    if (!currentLevel) {
      // Root: show watched folders as items
      return library.folders.map(f => ({ name: f.name, path: f.path, folderId: f.id, songCount: f.songCount }))
    }
    const prefix = currentLevel.path + '/'
    const childMap = new Map<string, number>()
    for (const song of library.songs) {
      if (!song.path.startsWith(prefix)) continue
      const rel = song.path.slice(prefix.length)
      const idx = rel.indexOf('/')
      if (idx === -1) continue
      const childPath = prefix + rel.slice(0, idx)
      childMap.set(childPath, (childMap.get(childPath) ?? 0) + 1)
    }
    return Array.from(childMap.entries())
      .map(([path, count]) => ({ name: path.split('/').pop()!, path, songCount: count }))
      .sort((a, b) => a.name.localeCompare(b.name))
  })

  // Direct + nested songs at current level (for leaf or mixed folders)
  const songs = $derived.by<Song[]>(() => {
    if (!currentLevel) return []
    const prefix = currentLevel.path + '/'
    return library.songs.filter(s => s.path.startsWith(prefix))
  })

  // Songs directly in the current folder (not in sub-folders)
  const directSongs = $derived.by<Song[]>(() => {
    if (!currentLevel) return []
    const prefix = currentLevel.path + '/'
    return library.songs.filter(s => {
      if (!s.path.startsWith(prefix)) return false
      const rel = s.path.slice(prefix.length)
      return !rel.includes('/')
    })
  })

  function getFolderArtIds(path: string): string[] {
    const prefix = path + '/'
    const result: string[] = []
    for (const song of library.songs) {
      if (!song.path.startsWith(prefix)) continue
      result.push(song.id)
      if (result.length === 4) break
    }
    return result
  }

  function enter(item: { name: string; path: string; folderId?: string }) {
    stack = [...stack, { name: item.name, path: item.path, folderId: item.folderId }]
  }

  function goTo(index: number) {
    stack = stack.slice(0, index + 1)
  }

  function goRoot() {
    stack = []
  }

  function formatDuration(s: number): string {
    if (!s) return '--'
    return `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, '0')}`
  }

  function playSong(song: Song, queue: Song[]) {
    player.playSong(song, queue)
    api.invoke('library:update-play', song.id)
  }

  function playAll() {
    if (songs.length) playSong(songs[0], songs)
  }
</script>

<div class="folder-tree-view">
  <!-- Topbar / breadcrumbs -->
  <div class="topbar">
    <div class="breadcrumbs">
      <button class="crumb" class:active={stack.length === 0} onclick={goRoot}>
        Folders
      </button>
      {#each stack as crumb, i}
        <span class="crumb-sep"><Icon name="chevron-right" size={12} /></span>
        <button
          class="crumb"
          class:active={i === stack.length - 1}
          onclick={() => goTo(i)}
        >{crumb.name}</button>
      {/each}
    </div>
    {#if currentLevel && songs.length > 0}
      <button class="play-all-btn" onclick={playAll}>
        <Icon name="play" size={13} /> Play All
      </button>
    {/if}
  </div>

  <div class="content">
    <!-- Sub-folder grid -->
    {#if subFolders.length > 0}
      {#if currentLevel && directSongs.length > 0}
        <p class="section-label">Sub-folders</p>
      {/if}
      <div class="folders-grid">
        {#each subFolders as item}
          {@const artIds = getFolderArtIds(item.path)}
          <button class="folder-card" onclick={() => enter(item)}>
            <div class="card-art">
              {#if artIds.length === 0 || !getArt(artIds[0])}
                <div class="art-placeholder"><Icon name="folder" size={36} /></div>
              {:else if artIds.length === 1}
                <img src={getArt(artIds[0])} alt="" class="art-single" />
              {:else}
                <div class="art-grid">
                  {#each Array(4) as _, i}
                    <div class="art-cell">
                      {#if getArt(artIds[i])}
                        <img src={getArt(artIds[i])} alt="" />
                      {:else if getArt(artIds[artIds.length - 1])}
                        <img src={getArt(artIds[artIds.length - 1])} alt="" />
                      {/if}
                    </div>
                  {/each}
                </div>
              {/if}
              <div class="card-hover-overlay">
                <Icon name="chevron-right" size={22} />
              </div>
            </div>
            <p class="card-name">{item.name}</p>
            <p class="card-meta">{item.songCount} songs</p>
          </button>
        {/each}
      </div>
    {/if}

    <!-- Songs at this level -->
    {#if currentLevel && directSongs.length > 0}
      {#if subFolders.length > 0}
        <p class="section-label songs-label">Songs in this folder</p>
      {/if}
      <table class="songs-table">
        <thead>
          <tr>
            <th class="col-num">#</th>
            <th class="col-title">Title</th>
            <th class="col-album">Album</th>
            <th class="col-dur"><Icon name="queue" size={13} /></th>
          </tr>
        </thead>
        <tbody>
          {#each directSongs as song, i}
            {@const isActive = player.currentSong?.id === song.id}
            <tr
              class="song-row"
              class:active={isActive}
              onmouseenter={() => hoveredRowId = song.id}
              onmouseleave={() => hoveredRowId = null}
              ondblclick={() => playSong(song, directSongs)}
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
                    <button class="num-play-btn" onclick={() => playSong(song, directSongs)}>
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
                    <span class="song-name" class:active-name={isActive}>{song.title}</span>
                    <span class="song-artist">{song.artist}</span>
                  </div>
                </div>
              </td>
              <td class="col-album">
                <span class="album-text">{song.album}</span>
              </td>
              <td class="col-dur text-muted">{formatDuration(song.duration)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}

    <!-- Empty root state -->
    {#if !currentLevel && library.folders.length === 0}
      <div class="empty-state">
        <Icon name="folder" size={48} />
        <p>No folders yet</p>
        <p class="sub">Go to Folders to add a watched folder</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .folder-tree-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 32px 12px;
    flex-shrink: 0;
    gap: 16px;
  }

  .breadcrumbs {
    display: flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
    flex: 1;
  }

  .crumb {
    font-size: 14px;
    color: var(--on-surface-variant);
    transition: color 0.12s;
    white-space: nowrap;
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .crumb:hover { color: var(--on-surface); }

  .crumb.active {
    font-size: 20px;
    font-weight: 600;
    letter-spacing: -0.02em;
    color: var(--secondary);
    cursor: default;
  }

  .crumb-sep {
    color: var(--outline);
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  .play-all-btn {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 8px 18px;
    background: var(--secondary);
    color: var(--on-secondary);
    border-radius: var(--radius-lg);
    font-size: 13px;
    font-weight: 500;
    flex-shrink: 0;
    transition: background 0.12s;
  }

  .play-all-btn:hover { background: var(--tertiary); }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 4px 32px 32px;
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .section-label {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--outline);
    margin-bottom: 14px;
  }

  .songs-label { margin-top: 28px; }

  /* Folder grid */
  .folders-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 20px 16px;
    margin-bottom: 8px;
  }

  .folder-card {
    text-align: left;
    cursor: pointer;
    padding: 0;
    background: none;
    border: none;
  }

  .card-art {
    position: relative;
    width: 100%;
    aspect-ratio: 1;
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: var(--surface-container-high);
    margin-bottom: 8px;
    transition: transform 0.15s;
  }

  .folder-card:hover .card-art { transform: scale(1.03); }

  .art-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--on-surface-variant);
  }

  .art-single {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .art-grid {
    width: 100%;
    height: 100%;
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-template-rows: 1fr 1fr;
    gap: 1px;
    background: var(--outline-variant);
  }

  .art-cell {
    overflow: hidden;
    background: var(--surface-container-high);
  }

  .art-cell img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .card-hover-overlay {
    position: absolute;
    inset: 0;
    background: rgba(0,0,0,0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .folder-card:hover .card-hover-overlay { opacity: 1; }

  .card-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--on-surface);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 2px;
  }

  .card-meta {
    font-size: 12px;
    color: var(--outline);
  }

  /* Songs table */
  .songs-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  .songs-table thead th {
    padding: 6px 12px;
    text-align: left;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    color: var(--outline);
    border-bottom: 1px solid var(--outline-variant);
  }

  .col-num { width: 40px; text-align: center; }
  .col-title { min-width: 0; }
  .col-album { width: 180px; }
  .col-dur { width: 56px; text-align: right; padding-right: 4px !important; }

  .song-row {
    border-bottom: 1px solid var(--outline-variant);
    cursor: pointer;
    transition: background 0.1s;
  }

  .song-row:hover { background: var(--surface-container-low); }
  .song-row.active { background: var(--primary-container); }

  .song-row td {
    padding: 8px 12px;
    color: var(--on-surface-variant);
  }

  .col-num {
    text-align: center;
    vertical-align: middle;
  }

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
    min-width: 0;
  }

  .thumb {
    width: 36px;
    height: 36px;
    border-radius: var(--radius-sm);
    overflow: hidden;
    flex-shrink: 0;
    background: var(--surface-container-high);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--on-surface-variant);
  }

  .thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .title-meta {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }

  .song-name { color: var(--on-surface); font-weight: 500; }
  .song-name.active-name { color: var(--primary); }

  .song-artist { font-size: 11px; color: var(--outline); }

  .album-text {
    font-size: 13px;
    color: var(--on-surface-variant);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    white-space: normal;
  }

  .text-muted { color: var(--on-surface-variant); }

  /* Empty state */
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--on-surface-variant);
    padding: 60px 0;
  }

  .empty-state p { font-size: 16px; font-weight: 500; color: var(--on-surface); }
  .empty-state .sub { font-size: 13px; color: var(--outline); font-weight: 400; }
</style>
