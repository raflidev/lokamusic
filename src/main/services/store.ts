import Store from 'electron-store'

export interface Song {
  id: string
  path: string
  title: string
  artist: string
  album: string
  duration: number
  albumArt?: string
  lyrics?: string
  dateAdded: number
  liked: boolean
  playCount: number
  lastPlayed?: number
}

export interface WatchedFolder {
  id: string
  path: string
  name: string
  songCount: number
  sizeBytes: number
  lastScanned?: number
}

export interface ScanEvent {
  timestamp: number
  folderPath: string
  success: number
  errors: number
}

export interface Playlist {
  id: string
  name: string
  songIds: string[]
  createdAt: number
}

interface Schema {
  songs: Song[]
  folders: WatchedFolder[]
  scanHistory: ScanEvent[]
  playlists: Playlist[]
  settings: {
    volume: number
    shuffle: boolean
    repeat: 'none' | 'one' | 'all'
  }
}

const store = new Store<Schema>({
  defaults: {
    songs: [],
    folders: [],
    scanHistory: [],
    playlists: [],
    settings: { volume: 0.8, shuffle: false, repeat: 'none' }
  }
})

export default store
