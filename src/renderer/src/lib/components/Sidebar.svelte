<script lang="ts">
  import Icon from './Icon.svelte'
  import FolderArt from './FolderArt.svelte'
  import { ui, type View } from '../stores/ui.svelte'
  import { library } from '../stores/library.svelte'
  import { playlistStore } from '../stores/playlists.svelte'
  import type { Playlist } from '../../../../../preload/index.d'

  const navItems: { view: View; icon: string; label: string }[] = [
    { view: 'library', icon: 'library', label: 'Library' },
    { view: 'folder-tree', icon: 'hierarchy', label: 'Folders Hierarchy' },
    { view: 'folders', icon: 'folder', label: 'Folders' },
    { view: 'player', icon: 'headphones', label: 'Player' }
  ]

  const libraryNavActive = $derived(
    ui.currentView === 'library' && ui.selectedFolderId === null
  )

  let creatingPlaylist = $state(false)
  let newPlaylistName = $state('')

  async function confirmCreatePlaylist() {
    if (!newPlaylistName.trim()) {
      creatingPlaylist = false
      return
    }
    const pl = await window.electronAPI.invoke('playlist:create', newPlaylistName.trim()) as Playlist
    playlistStore.add(pl)
    newPlaylistName = ''
    creatingPlaylist = false
    ui.navigateToPlaylist(pl.id)
  }

  function cancelCreate() {
    creatingPlaylist = false
    newPlaylistName = ''
  }
</script>

<aside class="sidebar">
  <div class="logo">
    <span>lokamusic</span>
  </div>

  <nav class="nav">
    {#each navItems as item}
      <button
        class="nav-item"
        class:active={item.view === 'library' ? libraryNavActive : ui.currentView === item.view}
        onclick={() => ui.navigate(item.view)}
      >
        <Icon name={item.icon} size={16} />
        <span>{item.label}</span>
      </button>
    {/each}
  </nav>

  <div class="section library-section">
    <p class="label-sm">Your Library</p>
    <button class="nav-item" class:active={ui.currentView === 'liked'} onclick={() => ui.navigate('liked')}>
      <Icon name="heart" size={15} />
      <span>Liked Songs</span>
    </button>
    <button class="nav-item" class:active={ui.currentView === 'albums'} onclick={() => ui.navigate('albums')}>
      <Icon name="music" size={15} />
      <span>Albums</span>
    </button>
    <button class="nav-item" class:active={ui.currentView === 'artists'} onclick={() => ui.navigate('artists')}>
      <Icon name="user" size={15} />
      <span>Artists</span>
    </button>

    {#if library.folders.length > 0}
      <div class="folder-divider"></div>
      {#each library.folders as folder}
        <button
          class="nav-item folder-item"
          class:active={ui.currentView === 'library' && ui.selectedFolderId === folder.id}
          onclick={() => ui.navigateToFolder(folder.id)}
          title={folder.name}
        >
          <FolderArt folderId={folder.id} size={28} />
          <div class="folder-item-meta">
            <span class="folder-item-name">{folder.name}</span>
            <span class="folder-item-count">{folder.songCount} songs</span>
          </div>
        </button>
      {/each}
    {/if}

    <div class="folder-divider"></div>
    <div class="section-header">
      <p class="label-sm">Playlists</p>
      <button class="icon-btn" title="New playlist" onclick={() => { creatingPlaylist = true; newPlaylistName = '' }}>
        <Icon name="plus" size={14} />
      </button>
    </div>

    {#if creatingPlaylist}
      <div class="new-playlist-input">
        <input
          type="text"
          placeholder="Playlist name…"
          bind:value={newPlaylistName}
          onkeydown={(e) => {
            if (e.key === 'Enter') confirmCreatePlaylist()
            if (e.key === 'Escape') cancelCreate()
          }}
          onblur={confirmCreatePlaylist}
          autofocus
        />
      </div>
    {/if}

    {#if playlistStore.all.length === 0 && !creatingPlaylist}
      <p class="empty-playlists">No playlists yet</p>
    {:else}
      {#each playlistStore.all as pl}
        <button
          class="nav-item"
          class:active={ui.currentView === 'playlist' && ui.selectedPlaylistId === pl.id}
          onclick={() => ui.navigateToPlaylist(pl.id)}
          title={pl.name}
        >
          <Icon name="queue" size={15} />
          <span class="playlist-name-label">{pl.name}</span>
        </button>
      {/each}
    {/if}
  </div>

  <div class="star-section">
    <button
      class="star-btn"
      class:active={ui.currentView === 'settings'}
      onclick={() => ui.navigate('settings')}
    >
      <Icon name="settings" size={14} />
      <span>Settings</span>
    </button>
    <button class="star-btn" onclick={() => window.open('https://github.com/raflidev/lokamusic', '_blank')}>
      <Icon name="github" size={14} />
      <span>Star Repository</span>
    </button>
  </div>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-width);
    min-width: var(--sidebar-width);
    height: 100%;
    background: var(--surface-container-lowest);
    border-right: 1px solid var(--outline-variant);
    display: flex;
    flex-direction: column;
    padding: 0 0 16px;
    overflow: hidden;
  }

  .logo {
    padding: 44px 20px 16px;
    font-size: 16px;
    font-weight: 600;
    letter-spacing: -0.02em;
    color: var(--on-surface);
    display: flex;
    align-items: center;
    gap: 10px;
    -webkit-app-region: drag;
    cursor: default;
  }

  .logo-img {
    width: 26px;
    height: 26px;
    object-fit: cover;
    border-radius: var(--radius);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4), 0 0 0 1px var(--outline-variant);
    transition: transform 0.5s cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  .logo:hover .logo-img {
    transform: scale(1.08) rotate(15deg);
  }

  .nav {
    display: flex;
    flex-direction: column;
    padding: 0 8px;
    gap: 2px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    border-radius: var(--radius-lg);
    font-size: 14px;
    color: var(--on-surface-variant);
    transition: background 0.12s, color 0.12s;
    text-align: left;
    width: 100%;
  }

  .nav-item:hover {
    background: var(--surface-container);
    color: var(--on-surface);
  }

  .nav-item.active {
    background: var(--primary-container);
    color: var(--on-primary-container);
  }

  .section {
    margin-top: 24px;
    padding: 0 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .library-section {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
  }

  .folder-divider {
    height: 1px;
    background: var(--outline-variant);
    margin: 6px 12px;
  }

  .folder-item {
    gap: 8px;
  }

  .folder-item-meta {
    display: flex;
    flex-direction: column;
    min-width: 0;
    gap: 1px;
  }

  .folder-item-name {
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .folder-item-count {
    font-size: 10px;
    color: var(--outline);
  }

  .section .label-sm {
    padding: 0 12px 8px;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px 8px;
  }

  .section-header .label-sm {
    padding: 0;
  }

  .icon-btn {
    color: var(--on-surface-variant);
    padding: 2px;
    border-radius: var(--radius);
    display: flex;
  }

  .icon-btn:hover {
    color: var(--on-surface);
  }

  .empty-playlists {
    font-size: 12px;
    color: var(--outline);
    padding: 4px 12px;
  }

  .new-playlist-input {
    padding: 0 4px 4px;
  }

  .new-playlist-input input {
    width: 100%;
    background: var(--surface-container);
    border: 1px solid var(--primary);
    border-radius: var(--radius);
    padding: 6px 10px;
    font-size: 13px;
    color: var(--on-surface);
    outline: none;
    box-sizing: border-box;
  }

  .playlist-name-label {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 13px;
  }

.star-section {
    padding: 7% 12px 0;
    border-top: 1px solid var(--outline-variant);
    margin-top: auto;
    flex-shrink: 0;
  }

  .star-btn {
    display: flex;
    align-items: center;
    gap: 7px;
    width: 100%;
    padding: 8px 10px;
    border-radius: var(--radius);
    font-size: 12px;
    color: var(--outline);
    transition: background 0.12s, color 0.12s;
  }

  .star-btn:hover {
    background: var(--surface-container);
    color: var(--on-surface-variant);
  }

  .star-btn.active {
    background: var(--primary-container);
    color: var(--on-primary-container);
  }
</style>
