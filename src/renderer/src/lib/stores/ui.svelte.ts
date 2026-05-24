export type View = 'library' | 'folders' | 'player' | 'liked' | 'albums' | 'artists' | 'playlist'

export interface ScanProgress {
  folderId: string
  scanned: number
  total: number
  percent: number
}

let currentView = $state<View>('library')
let scanProgress = $state<ScanProgress | null>(null)
let searchQuery = $state('')
let selectedFolderId = $state<string | null>(null)
let selectedPlaylistId = $state<string | null>(null)
let pendingArtistName = $state<string | null>(null)
let showQueue = $state(false)

export const ui = {
  get currentView() { return currentView },
  get scanProgress() { return scanProgress },
  get searchQuery() { return searchQuery },
  get selectedFolderId() { return selectedFolderId },
  get selectedPlaylistId() { return selectedPlaylistId },
  get pendingArtistName() { return pendingArtistName },
  get showQueue() { return showQueue },

  navigate(view: View) {
    currentView = view
    selectedFolderId = null
    selectedPlaylistId = null
    searchQuery = ''
  },
  navigateToFolder(folderId: string) {
    currentView = 'library'
    selectedFolderId = folderId
    selectedPlaylistId = null
    searchQuery = ''
  },
  navigateToArtist(name: string) {
    pendingArtistName = name
    currentView = 'artists'
    selectedFolderId = null
    selectedPlaylistId = null
    searchQuery = ''
  },
  navigateToPlaylist(id: string) {
    currentView = 'playlist'
    selectedPlaylistId = id
    selectedFolderId = null
    searchQuery = ''
  },
  clearPendingArtist() { pendingArtistName = null },
  toggleQueue() { showQueue = !showQueue },
  clearFolderFilter() { selectedFolderId = null },
  setScanProgress(p: ScanProgress | null) { scanProgress = p },
  setSearchQuery(q: string) { searchQuery = q }
}
