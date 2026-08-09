import { invoke } from '@tauri-apps/api/core'
import { library } from './library.svelte'

let cache = $state<Record<string, string>>({})
const pending = new Set<string>()
let active = 0
const MAX_CONCURRENT = 8
const waitQueue: Array<() => void> = []

async function doFetch(songId: string, path: string) {
  active++
  try {
    const art = await invoke<string | null>('library_get_song_art', { path })
    cache[songId] = art ?? ''
  } catch {
    cache[songId] = ''
  } finally {
    active--
    pending.delete(songId)
    waitQueue.shift()?.()
  }
}

function enqueue(songId: string, path: string) {
  if (active < MAX_CONCURRENT) {
    doFetch(songId, path)
  } else {
    waitQueue.push(() => doFetch(songId, path))
  }
}

export function getArt(songId: string | undefined): string | undefined {
  if (!songId) return undefined
  if (songId in cache) return cache[songId] || undefined
  if (!pending.has(songId)) {
    const path = library.songs.find(s => s.id === songId)?.path
    if (!path) return undefined
    pending.add(songId)
    enqueue(songId, path)
  }
  return undefined
}

export function clearArtCache() {
  cache = {}
}
