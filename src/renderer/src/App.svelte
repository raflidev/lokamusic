<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { check, type Update } from '@tauri-apps/plugin-updater'
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
  import { trackEvent } from './lib/analytics'
  import type { Song, WatchedFolder, Playlist } from './types'

  let pendingUpdate = $state<Update | null>(null)
  let prevVolume = $state(0.8)

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
    const tag = (e.target as HTMLElement).tagName
    if (tag === 'INPUT' || tag === 'TEXTAREA') return

    switch (e.code) {
      case 'Space':
        e.preventDefault()
        if (player.currentSong) player.togglePlay()
        break
      case 'ArrowRight':
        if (!player.currentSong) break
        e.preventDefault()
        player.seek(Math.min(player.currentTime + 5, player.duration))
        break
      case 'ArrowLeft':
        if (!player.currentSong) break
        e.preventDefault()
        player.seek(Math.max(player.currentTime - 5, 0))
        break
      case 'ArrowUp':
        e.preventDefault()
        player.setVolume(Math.min(player.volume + 0.05, 1))
        break
      case 'ArrowDown':
        e.preventDefault()
        player.setVolume(Math.max(player.volume - 0.05, 0))
        break
      case 'KeyN':
        if (!player.currentSong) break
        e.preventDefault()
        player.next()
        break
      case 'KeyP':
        if (!player.currentSong) break
        e.preventDefault()
        player.prev()
        break
      case 'KeyL':
        if (!player.currentSong) break
        e.preventDefault()
        api.invoke('library:toggle-like', player.currentSong.id)
        library.toggleLike(player.currentSong.id)
        break
      case 'KeyS':
        e.preventDefault()
        player.toggleShuffle()
        break
      case 'KeyR':
        e.preventDefault()
        player.toggleRepeat()
        break
      case 'KeyQ':
        e.preventDefault()
        ui.toggleQueue()
        break
      case 'KeyM':
        e.preventDefault()
        if (player.volume > 0) {
          prevVolume = player.volume
          player.setVolume(0)
        } else {
          player.setVolume(prevVolume)
        }
        break
      case 'MediaPlayPause':
        if (player.currentSong) player.togglePlay()
        break
      case 'MediaTrackNext':
        player.next()
        break
      case 'MediaTrackPrevious':
        player.prev()
        break
    }
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

    check().then(update => { pendingUpdate = update ?? null }).catch(() => {})

    trackEvent('app_opened')
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
        <SettingsView update={pendingUpdate} />
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
