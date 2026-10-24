import { untrack } from 'svelte'
import { convertFileSrc } from '@tauri-apps/api/core'
import type { Song } from '../../types'
import { api } from '../api'
import { library } from './library.svelte'
import { getArt } from './artCache.svelte'

let currentSong = $state<Song | null>(null)
let queue = $state<Song[]>([])
let queueIndex = $state(0)
let isPlaying = $state(false)
let isLoading = $state(false)
let currentTime = $state(0)
let duration = $state(0)
let volume = $state(0.8)
let shuffle = $state(false)
let repeat = $state<'none' | 'one' | 'all'>('none')
let originalQueue = $state<Song[]>([])
let error = $state<string | null>(null)

let audio: HTMLAudioElement | null = null
let persistTimer: ReturnType<typeof setTimeout> | null = null

function shuffleArray<T>(items: T[]): T[] {
  const result = [...items]
  for (let i = result.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1))
    ;[result[i], result[j]] = [result[j], result[i]]
  }
  return result
}

function playbackError(song: Song | null) {
  error = song ? `Unable to play “${song.title}”.` : 'Unable to play this file.'
}

function persistPlayback() {
  if (persistTimer) clearTimeout(persistTimer)
  persistTimer = setTimeout(() => {
    persistTimer = null
    api.invoke('settings:set-playback', volume, shuffle, repeat).catch(() => {})
  }, 300)
}

function getAudio(): HTMLAudioElement {
  if (!audio) {
    audio = new Audio()
    audio.volume = volume

    audio.ontimeupdate = () => { currentTime = audio!.currentTime }
    audio.ondurationchange = () => { duration = audio!.duration || 0 }
    audio.onplay = () => { isPlaying = true; isLoading = false; error = null }
    audio.onpause = () => { isPlaying = false }
    audio.onended = () => { player.next(true) }
    audio.onerror = () => { isLoading = false; playbackError(currentSong) }

    if ('mediaSession' in navigator) {
      navigator.mediaSession.setActionHandler('play', () => player.togglePlay())
      navigator.mediaSession.setActionHandler('pause', () => player.togglePlay())
      navigator.mediaSession.setActionHandler('previoustrack', () => player.prev())
      navigator.mediaSession.setActionHandler('nexttrack', () => player.next())
    }
  }
  return audio
}

export const player = {
  get currentSong() { return currentSong },
  get queue() { return queue },
  get queueIndex() { return queueIndex },
  get isPlaying() { return isPlaying },
  get isLoading() { return isLoading },
  get currentTime() { return currentTime },
  get duration() { return duration },
  get volume() { return volume },
  get shuffle() { return shuffle },
  get repeat() { return repeat },
  get error() { return error },

  playSong(song: Song, songList?: Song[]) {
    if (songList) {
      originalQueue = songList
      if (shuffle) {
        queue = [song, ...shuffleArray(songList.filter(s => s.id !== song.id))]
        queueIndex = 0
      } else {
        const idx = songList.findIndex(s => s.id === song.id)
        queue = songList
        queueIndex = idx >= 0 ? idx : 0
      }
    }

    currentSong = song
    isLoading = true
    error = null
    const a = getAudio()
    a.src = convertFileSrc(song.path)
    a.load()
    a.play().catch((err) => {
      isLoading = false
      playbackError(song)
      console.error('[player] play failed:', err)
    })

    api.invoke('library:update-play', song.id).catch(() => {})
    library.markPlayed(song.id)
  },

  togglePlay() {
    const a = getAudio()
    if (isPlaying) {
      a.pause()
    } else if (currentSong) {
      a.play().catch(() => playbackError(currentSong))
    }
  },

  prev() {
    if (currentTime > 3) {
      getAudio().currentTime = 0
      return
    }
    if (queue.length === 0) return
    queueIndex = (queueIndex - 1 + queue.length) % queue.length
    player.playSong(queue[queueIndex])
  },

  /** `fromEnded` is true when the audio element reached the end by itself. */
  next(fromEnded = false) {
    if (fromEnded && repeat === 'one') {
      const a = getAudio()
      a.currentTime = 0
      a.play().catch(() => {})
      return
    }
    if (queue.length === 0) return
    if (queueIndex < queue.length - 1) {
      queueIndex++
    } else if (repeat === 'all') {
      queueIndex = 0
    } else {
      isPlaying = false
      return
    }
    player.playSong(queue[queueIndex])
  },

  seek(time: number) {
    currentTime = time
    getAudio().currentTime = time
  },

  playAt(index: number) {
    if (index < 0 || index >= queue.length) return
    queueIndex = index
    const song = queue[index]
    currentSong = song
    isLoading = true
    error = null
    const a = getAudio()
    a.src = convertFileSrc(song.path)
    a.load()
    a.play().catch((err) => {
      isLoading = false
      playbackError(song)
      console.error('[player] play failed:', err)
    })
    library.markPlayed(song.id)
    api.invoke('library:update-play', song.id).catch(() => {})
  },

  setVolume(v: number) {
    volume = Math.max(0, Math.min(1, v))
    getAudio().volume = volume
    persistPlayback()
  },

  toggleShuffle() {
    shuffle = !shuffle
    persistPlayback()
    if (!currentSong || queue.length === 0) return
    if (shuffle) {
      queue = [currentSong!, ...shuffleArray(queue.filter(s => s.id !== currentSong!.id))]
      queueIndex = 0
    } else {
      const src = originalQueue.length > 0 ? originalQueue : queue
      const idx = src.findIndex(s => s.id === currentSong!.id)
      queue = src
      queueIndex = idx >= 0 ? idx : 0
    }
  },
  toggleRepeat() {
    repeat = repeat === 'none' ? 'all' : repeat === 'all' ? 'one' : 'none'
    persistPlayback()
  },

  /** Applies the persisted playback settings at startup. */
  restore(settings?: { volume?: number; shuffle?: boolean; repeat?: string } | null) {
    if (!settings) return
    if (typeof settings.volume === 'number' && Number.isFinite(settings.volume)) {
      volume = Math.max(0, Math.min(1, settings.volume))
      if (audio) audio.volume = volume
    }
    shuffle = !!settings.shuffle
    repeat = settings.repeat === 'one' || settings.repeat === 'all' ? settings.repeat : 'none'
  },

  clearError() {
    error = null
  },

  updateCurrentSongLyrics(lyrics: string) {
    if (currentSong) currentSong = { ...currentSong, lyrics }
  },

  get upNext(): Song | null {
    if (queue.length === 0) return null
    const next = queueIndex + 1
    return next < queue.length ? queue[next] : null
  }
}

$effect.root(() => {
  let pauseTimer: ReturnType<typeof setTimeout> | null = null

  $effect(() => {
    const song = currentSong
    const playing = isPlaying
    const art = song ? getArt(song.id) : undefined

    api.invoke('tray:set-now-playing', song ? {
      title: song.title,
      artist: song.artist,
      album: song.album,
      isPlaying: playing,
      art: art ?? null,
    } : null).catch(() => {})

    if ('mediaSession' in navigator) {
      navigator.mediaSession.metadata = song ? new MediaMetadata({
        title: song.title,
        artist: song.artist,
        album: song.album,
        artwork: art ? [{ src: art, sizes: '512x512', type: art.match(/^data:([^;]+);/)?.[1] ?? 'image/jpeg' }] : []
      }) : null
      navigator.mediaSession.playbackState = song ? (playing ? 'playing' : 'paused') : 'none'
    }

    if (pauseTimer) {
      clearTimeout(pauseTimer)
      pauseTimer = null
    }

    if (!song) {
      api.invoke('discord:update-presence', null).catch(() => {})
      return
    }

    if (!playing) {
      pauseTimer = setTimeout(() => {
        api.invoke('discord:update-presence', null).catch(() => {})
        pauseTimer = null
      }, 5000)
      return
    }

    const ct = untrack(() => currentTime)
    const dur = untrack(() => duration) || song.duration

    api.invoke('discord:update-presence', {
      title: song.title,
      artist: song.artist,
      album: song.album,
      isPlaying: true,
      duration: dur,
      currentTime: ct,
    }).catch(() => {})
  })
})

api.on('tray:control', (action) => {
  if (action === 'prev') player.prev()
  else if (action === 'next') player.next()
  else if (action === 'play-pause') player.togglePlay()
})
