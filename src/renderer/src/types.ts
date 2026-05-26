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
