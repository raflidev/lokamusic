import { invoke } from '@tauri-apps/api/core'
import { library } from './library.svelte'

// Covers are downscaled to 512px JPEG in Rust (~30-60 KB each), so caching a
// large library stays in the tens of MB while avoiding re-reads on scroll.
const MAX_CACHE = 1500
const MAX_CONCURRENT = 8

let cache = $state<Record<string, string>>({})
let order: string[] = []
const pending = new Set<string>()
let active = 0
const waitQueue: Array<() => void> = []

function remember(songId: string, art: string) {
  cache[songId] = art
  order = order.filter(id => id !== songId)
  order.push(songId)
  while (order.length > MAX_CACHE) {
    const evicted = order.shift()
    if (!evicted) break
    delete cache[evicted]
  }
}

async function doFetch(songId: string, path: string) {
  active++
  try {
    const art = await invoke<string | null>('library_get_song_art', { path })
    remember(songId, art ?? '')
  } catch {
    remember(songId, '')
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
  if (songId in cache) {
    // Touch the entry so frequently visible rows are evicted last.
    if (order[order.length - 1] !== songId) {
      order = order.filter(id => id !== songId)
      order.push(songId)
    }
    return cache[songId] || undefined
  }
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
  order = []
}
