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
  import { getArt } from './lib/stores/artCache.svelte'
  import { initAnalytics, trackEvent } from './lib/analytics'
  import { listen, emit } from '@tauri-apps/api/event'
  import type { Song, WatchedFolder, Playlist } from './types'
  import type { UnlistenFn } from '@tauri-apps/api/event'

  type Settings = {
    volume: number
    shuffle: boolean
    repeat: string
    discordPresence: boolean
    analytics: boolean
  }

  type MiniState = {
    song: { id: string; title: string; artist: string; album: string; albumArt?: string } | null
    isPlaying: boolean
    isLoading: boolean
    liked: boolean
    shuffle: boolean
    repeat: string
  }

  let pendingUpdate = $state<Update | null>(null)
  let prevVolume = $state(0.8)
  let unlistenMpCmd: UnlistenFn | null = null
  let unlistenMpReq: UnlistenFn | null = null
  let lastMiniKey = ''
  let lastProgressAt = 0

  function buildMiniState(): MiniState {
    const song = player.currentSong
    return {
      song: song
        ? {
            id: song.id,
            title: song.title,
            artist: song.artist,
            album: song.album,
            albumArt: getArt(song.id),
          }
        : null,
      isPlaying: player.isPlaying,
      isLoading: player.isLoading,
      liked: song ? library.songs.find(s => s.id === song.id)?.liked ?? false : false,
      shuffle: player.shuffle,
      repeat: player.repeat,
    }
  }

  function miniStateKey(state: MiniState): string {
    const art = state.song?.albumArt
    return [
      state.song?.id ?? '',
      art ? `${art.length}:${art.slice(0, 24)}` : '',
      state.isPlaying ? 1 : 0,
      state.isLoading ? 1 : 0,
      state.liked ? 1 : 0,
      state.shuffle ? 1 : 0,
      state.repeat,
    ].join('|')
  }

  function emitMiniState() {
    emit('miniplayer:state', buildMiniState()).catch(() => {})
  }

  function emitMiniProgress() {
    emit('miniplayer:progress', {
      currentTime: player.currentTime,
      duration: player.duration,
      volume: player.volume,
    }).catch(() => {})
  }

  // Metadata changes (including cover art) are rare: send them as they happen.
  $effect(() => {
    const state = buildMiniState()
    const key = miniStateKey(state)
    if (key === lastMiniKey) return
    lastMiniKey = key
    emit('miniplayer:state', state).catch(() => {})
  })

  // Playback progress is high frequency: throttle to two updates per second.
  $effect(() => {
    const currentTime = player.currentTime
    const duration = player.duration
    const volume = player.volume
    const now = Date.now()
    if (now - lastProgressAt < 500 && currentTime < duration) return
    lastProgressAt = now
    emit('miniplayer:progress', { currentTime, duration, volume }).catch(() => {})
  })

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
        api.invoke('library:toggle-like', player.currentSong.id).catch(() => {})
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
    const [songs, folders, playlists, settings] = await Promise.all([
      api.invoke('library:get-all-songs') as Promise<Song[]>,
      api.invoke('library:get-folders') as Promise<WatchedFolder[]>,
      api.invoke('playlist:get-all') as Promise<Playlist[]>,
      api.invoke('settings:get').catch(() => null) as Promise<Settings | null>,
    ])
    library.setSongs(songs ?? [])
    library.setFolders(folders ?? [])
    playlistStore.set(playlists ?? [])
    player.restore(settings)

    await initAnalytics(settings?.analytics !== false)
    trackEvent('app_opened')

    await api.on('library:scan-progress', onScanProgress as (...args: unknown[]) => void)
    await api.on('library:scan-complete', onScanComplete as (...args: unknown[]) => void)
    await api.on('library:songs-updated', onSongsUpdated as (...args: unknown[]) => void)

    type MpCmd = string | { action: 'seek'; value: number } | { action: 'set-volume'; value: number }
    unlistenMpCmd = await listen<MpCmd>('miniplayer:cmd', (e) => {
      const cmd = e.payload
      if (typeof cmd === 'string') {
        switch (cmd) {
          case 'toggle-play': player.togglePlay(); break
          case 'prev': player.prev(); break
          case 'next': player.next(); break
          case 'toggle-like':
            if (player.currentSong) {
              api.invoke('library:toggle-like', player.currentSong.id).catch(() => {})
              library.toggleLike(player.currentSong.id)
            }
            break
          case 'toggle-shuffle': player.toggleShuffle(); break
          case 'toggle-repeat': player.toggleRepeat(); break
          case 'open-player': ui.navigate('player'); break
        }
      } else if (cmd && typeof cmd === 'object') {
        if (cmd.action === 'seek') {
          player.seek(cmd.value)
        } else if (cmd.action === 'set-volume') {
          player.setVolume(cmd.value)
        }
      }
    })

    // When mini player opens, it requests current state immediately
    unlistenMpReq = await listen('miniplayer:request-state', () => {
      lastMiniKey = miniStateKey(buildMiniState())
      emitMiniState()
      emitMiniProgress()
    })

    check().then(update => { pendingUpdate = update ?? null }).catch(() => {})
  })

  onDestroy(() => {
    api.off('library:scan-progress', onScanProgress as (...args: unknown[]) => void)
    api.off('library:scan-complete', onScanComplete as (...args: unknown[]) => void)
    api.off('library:songs-updated', onSongsUpdated as (...args: unknown[]) => void)
    unlistenMpCmd?.()
    unlistenMpReq?.()
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

  {#if player.error}
    <div class="toast" role="alert">
      <span>{player.error}</span>
      <button class="toast-close" onclick={() => player.clearError()} aria-label="Dismiss">×</button>
    </div>
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

  .toast {
    position: fixed;
    left: 50%;
    bottom: calc(var(--bottom-bar-height) + 16px);
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 12px;
    max-width: min(560px, calc(100vw - 32px));
    padding: 10px 14px;
    background: var(--error-container);
    color: var(--on-surface);
    border: 1px solid var(--error);
    border-radius: var(--radius-lg);
    font-size: 13px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    z-index: 2000;
  }

  .toast-close {
    flex-shrink: 0;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    color: inherit;
    font-size: 16px;
    line-height: 1;
    opacity: 0.7;
  }

  .toast-close:hover { opacity: 1; }
</style>
