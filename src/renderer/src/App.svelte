<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import Sidebar from './lib/components/Sidebar.svelte'
  import BottomBar from './lib/components/BottomBar.svelte'
  import LibraryView from './lib/views/LibraryView.svelte'
  import FoldersView from './lib/views/FoldersView.svelte'
  import FolderTreeView from './lib/views/FolderTreeView.svelte'
  import PlayerView from './lib/views/PlayerView.svelte'
  import LikedSongsView from './lib/views/LikedSongsView.svelte'
  import AlbumsView from './lib/views/AlbumsView.svelte'
  import ArtistsView from './lib/views/ArtistsView.svelte'
  import PlaylistView from './lib/views/PlaylistView.svelte'
  import SettingsView from './lib/views/SettingsView.svelte'
  import QueueSidebar from './lib/components/QueueSidebar.svelte'
  import { library } from './lib/stores/library.svelte'
  import { ui } from './lib/stores/ui.svelte'
  import { player } from './lib/stores/player.svelte'
  import { playlistStore } from './lib/stores/playlists.svelte'
  import { api } from './lib/api'
  import type { Song, WatchedFolder, Playlist } from './types'

  function onScanProgress(progress: { folderId: string; scanned: number; total: number; percent: number }) {
    ui.setScanProgress(progress)
  }
  function onScanComplete(data: { folderId: string; songs: Song[]; folders: WatchedFolder[] }) {
    library.setSongs(data.songs ?? [])
    library.setFolders(data.folders ?? [])
    ui.setScanProgress(null)
  }
  function onSongsUpdated(songs: Song[]) {
    library.setSongs(songs ?? [])
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.code !== 'Space') return
    const tag = (e.target as HTMLElement).tagName
    if (tag === 'INPUT' || tag === 'TEXTAREA') return
    e.preventDefault()
    if (player.currentSong) player.togglePlay()
  }

  onMount(async () => {
    const [songs, folders, playlists] = await Promise.all([
      api.invoke('library:get-all-songs') as Promise<Song[]>,
      api.invoke('library:get-folders') as Promise<WatchedFolder[]>,
      api.invoke('playlist:get-all') as Promise<Playlist[]>,
    ])
    library.setSongs(songs ?? [])
    library.setFolders(folders ?? [])
    playlistStore.set(playlists ?? [])

    await api.on('library:scan-progress', onScanProgress as (...args: unknown[]) => void)
    await api.on('library:scan-complete', onScanComplete as (...args: unknown[]) => void)
    await api.on('library:songs-updated', onSongsUpdated as (...args: unknown[]) => void)
  })

  onDestroy(() => {
    api.off('library:scan-progress', onScanProgress as (...args: unknown[]) => void)
    api.off('library:scan-complete', onScanComplete as (...args: unknown[]) => void)
    api.off('library:songs-updated', onSongsUpdated as (...args: unknown[]) => void)
  })
</script>

<svelte:window onkeydown={onKeyDown} />

<div class="app-shell">
  <div class="main-area">
    <Sidebar />
    <main class="view-area">
      {#if ui.currentView === 'library'}
        <LibraryView />
      {:else if ui.currentView === 'folder-tree'}
        <FolderTreeView />
      {:else if ui.currentView === 'folders'}
        <FoldersView />
      {:else if ui.currentView === 'player'}
        <PlayerView />
      {:else if ui.currentView === 'liked'}
        <LikedSongsView />
      {:else if ui.currentView === 'albums'}
        <AlbumsView />
      {:else if ui.currentView === 'artists'}
        <ArtistsView />
      {:else if ui.currentView === 'playlist'}
        <PlaylistView />
      {:else if ui.currentView === 'settings'}
        <SettingsView />
      {/if}
    </main>
    {#if ui.showQueue}
      <QueueSidebar />
    {/if}
  </div>
  {#if ui.currentView !== 'player'}
    <BottomBar />
  {/if}
</div>

<style>
  .app-shell {
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .main-area {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .view-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--surface);
  }
</style>
