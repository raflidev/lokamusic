import type { Playlist } from '../../../../preload/index.d'

let playlists = $state<Playlist[]>([])

export const playlistStore = {
  get all() { return playlists },
  set(p: Playlist[]) { playlists = p },
  add(p: Playlist) { playlists = [...playlists, p] },
  remove(id: string) { playlists = playlists.filter(p => p.id !== id) },
  update(updated: Playlist) { playlists = playlists.map(p => p.id === updated.id ? updated : p) },
  addSong(playlistId: string, songId: string) {
    playlists = playlists.map(p =>
      p.id === playlistId && !p.songIds.includes(songId)
        ? { ...p, songIds: [...p.songIds, songId] }
        : p
    )
  },
  removeSong(playlistId: string, songId: string) {
    playlists = playlists.map(p =>
      p.id === playlistId ? { ...p, songIds: p.songIds.filter(id => id !== songId) } : p
    )
  }
}
