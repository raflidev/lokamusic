import { invoke } from '@tauri-apps/api/core'

let cache = $state<Record<string, string>>({})
const pending = new Set<string>()
let active = 0
const MAX_CONCURRENT = 8
const waitQueue: Array<() => void> = []

async function doFetch(songId: string) {
  active++
  try {
    const art = await invoke<string | null>('library_get_song_art', { songId })
    cache[songId] = art ?? ''
  } catch {
    cache[songId] = ''
  } finally {
    active--
    pending.delete(songId)
    waitQueue.shift()?.()
  }
}

function enqueue(songId: string) {
  if (active < MAX_CONCURRENT) {
    doFetch(songId)
  } else {
    waitQueue.push(() => doFetch(songId))
  }
}

export function getArt(songId: string | undefined): string | undefined {
  if (!songId) return undefined
  if (songId in cache) return cache[songId] || undefined
  if (!pending.has(songId)) {
    pending.add(songId)
    enqueue(songId)
  }
  return undefined
}

export function clearArtCache() {
  cache = {}
}
