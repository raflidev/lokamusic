export type View = 'library' | 'folders' | 'folder-tree' | 'player' | 'liked' | 'albums' | 'artists' | 'playlist' | 'settings'

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
let pendingAlbumKey = $state<string | null>(null)
let showQueue = $state(false)
let selectedSubFolderPath = $state<string | null>(null)
let pinnedFolders = $state<{ path: string; name: string }[]>(
  JSON.parse(localStorage.getItem('lokamusic:pinnedFolders') ?? '[]')
)

export const ui = {
  get currentView() { return currentView },
  get scanProgress() { return scanProgress },
  get searchQuery() { return searchQuery },
  get selectedFolderId() { return selectedFolderId },
  get selectedPlaylistId() { return selectedPlaylistId },
  get pendingArtistName() { return pendingArtistName },
  get pendingAlbumKey() { return pendingAlbumKey },
  get showQueue() { return showQueue },
  get selectedSubFolderPath() { return selectedSubFolderPath },
  get pinnedFolders() { return pinnedFolders },

  navigate(view: View) {
    currentView = view
    selectedFolderId = null
    selectedPlaylistId = null
    selectedSubFolderPath = null
    searchQuery = ''
  },
  navigateToFolder(folderId: string) {
    currentView = 'library'
    selectedFolderId = folderId
    selectedPlaylistId = null
    selectedSubFolderPath = null
    searchQuery = ''
  },
  navigateToSubFolder(path: string) {
    currentView = 'library'
    selectedSubFolderPath = path
    selectedFolderId = null
    selectedPlaylistId = null
    searchQuery = ''
  },
  pinFolder(path: string, name: string) {
    if (!pinnedFolders.find(f => f.path === path)) {
      pinnedFolders = [...pinnedFolders, { path, name }]
      localStorage.setItem('lokamusic:pinnedFolders', JSON.stringify(pinnedFolders))
    }
  },
  unpinFolder(path: string) {
    pinnedFolders = pinnedFolders.filter(f => f.path !== path)
    localStorage.setItem('lokamusic:pinnedFolders', JSON.stringify(pinnedFolders))
  },
  navigateToAlbum(albumName: string, artist: string) {
    pendingAlbumKey = `${albumName}__${artist}`
    currentView = 'albums'
    selectedFolderId = null
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
  clearPendingAlbum() { pendingAlbumKey = null },
  toggleQueue() { showQueue = !showQueue },
  clearFolderFilter() { selectedFolderId = null; selectedSubFolderPath = null },
  setScanProgress(p: ScanProgress | null) { scanProgress = p },
  setSearchQuery(q: string) { searchQuery = q }
}
