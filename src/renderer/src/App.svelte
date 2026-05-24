<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import Sidebar from './lib/components/Sidebar.svelte'
  import BottomBar from './lib/components/BottomBar.svelte'
  import LibraryView from './lib/views/LibraryView.svelte'
  import FoldersView from './lib/views/FoldersView.svelte'
  import PlayerView from './lib/views/PlayerView.svelte'
  import LikedSongsView from './lib/views/LikedSongsView.svelte'
  import AlbumsView from './lib/views/AlbumsView.svelte'
  import ArtistsView from './lib/views/ArtistsView.svelte'
  import PlaylistView from './lib/views/PlaylistView.svelte'
  import QueueSidebar from './lib/components/QueueSidebar.svelte'
  import { library } from './lib/stores/library.svelte'
  import { ui } from './lib/stores/ui.svelte'
  import { playlistStore } from './lib/stores/playlists.svelte'
  import type { Song, WatchedFolder, ScanEvent, Playlist } from '../../../preload/index.d'

  function onScanProgress(...args: unknown[]) {
    const progress = args[0] as { folderId: string; scanned: number; total: number; percent: number }
    ui.setScanProgress(progress)
  }
  function onScanComplete(...args: unknown[]) {
    const data = args[0] as { folderId: string; songs: Song[]; folders: WatchedFolder[] }
    library.setSongs(data.songs ?? [])
    library.setFolders(data.folders ?? [])
    ui.setScanProgress(null)
  }
  function onSongsUpdated(...args: unknown[]) {
    const songs = args[0] as Song[]
    library.setSongs(songs ?? [])
  }

  onMount(async () => {
    const [songs, folders, playlists] = await Promise.all([
      window.electronAPI.invoke('library:get-all-songs') as Promise<Song[]>,
      window.electronAPI.invoke('library:get-folders') as Promise<WatchedFolder[]>,
      window.electronAPI.invoke('playlist:get-all') as Promise<Playlist[]>,
    ])
    library.setSongs(songs ?? [])
    library.setFolders(folders ?? [])
    playlistStore.set(playlists ?? [])

    window.electronAPI.on('library:scan-progress', onScanProgress)
    window.electronAPI.on('library:scan-complete', onScanComplete)
    window.electronAPI.on('library:songs-updated', onSongsUpdated)
  })

  onDestroy(() => {
    window.electronAPI.off('library:scan-progress', onScanProgress)
    window.electronAPI.off('library:scan-complete', onScanComplete)
    window.electronAPI.off('library:songs-updated', onSongsUpdated)
  })
</script>

<div class="app-shell">
  <div class="main-area">
    <Sidebar />
    <main class="view-area">
      {#if ui.currentView === 'library'}
        <LibraryView />
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
