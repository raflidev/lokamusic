import type { Song, WatchedFolder, ScanEvent } from '../../types'

let songs = $state<Song[]>([])
let folders = $state<WatchedFolder[]>([])
let scanHistory = $state<ScanEvent[]>([])

const recentlyPlayed = $derived(
  [...songs]
    .filter(s => s.lastPlayed)
    .sort((a, b) => (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0))
    .slice(0, 6)
)

const likedSongs = $derived(songs.filter(s => s.liked))

export const library = {
  get songs() { return songs },
  get folders() { return folders },
  get scanHistory() { return scanHistory },
  get recentlyPlayed() { return recentlyPlayed },
  get likedSongs() { return likedSongs },

  setSongs(s: Song[]) { songs = s },
  setFolders(f: WatchedFolder[]) { folders = f },
  setScanHistory(h: ScanEvent[]) { scanHistory = h },

  updateSong(updated: Song) {
    songs = songs.map(s => s.id === updated.id ? updated : s)
  },

  mergeSongs(newSongs: Song[]) {
    const ids = new Set(songs.map(s => s.id))
    songs = [...songs, ...newSongs.filter(s => !ids.has(s.id))]
  },

  addFolder(folder: WatchedFolder) {
    folders = [...folders, folder]
  },

  updateFolder(updated: WatchedFolder) {
    folders = folders.map(f => f.id === updated.id ? updated : f)
  },

  removeFolder(id: string) {
    folders = folders.filter(f => f.id !== id)
  },

  toggleLike(songId: string) {
    songs = songs.map(s => s.id === songId ? { ...s, liked: !s.liked } : s)
  },

  markPlayed(songId: string) {
    songs = songs.map(s =>
      s.id === songId
        ? { ...s, playCount: s.playCount + 1, lastPlayed: Date.now() }
        : s
    )
  }
}
