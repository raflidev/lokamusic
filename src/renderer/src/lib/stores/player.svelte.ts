import { untrack } from 'svelte'
import { convertFileSrc } from '@tauri-apps/api/core'
import type { Song } from '../../types'
import { api } from '../api'
import { library } from './library.svelte'

let currentSong = $state<Song | null>(null)
let queue = $state<Song[]>([])
let queueIndex = $state(0)
let isPlaying = $state(false)
let currentTime = $state(0)
let duration = $state(0)
let volume = $state(0.8)
let shuffle = $state(false)
let repeat = $state<'none' | 'one' | 'all'>('none')
let originalQueue = $state<Song[]>([])

let audio: HTMLAudioElement | null = null

function getAudio(): HTMLAudioElement {
  if (!audio) {
    audio = new Audio()
    audio.volume = volume

    audio.ontimeupdate = () => { currentTime = audio!.currentTime }
    audio.ondurationchange = () => { duration = audio!.duration || 0 }
    audio.onplay = () => { isPlaying = true }
    audio.onpause = () => { isPlaying = false }
    audio.onended = () => { player.next() }
  }
  return audio
}

export const player = {
  get currentSong() { return currentSong },
  get queue() { return queue },
  get queueIndex() { return queueIndex },
  get isPlaying() { return isPlaying },
  get currentTime() { return currentTime },
  get duration() { return duration },
  get volume() { return volume },
  get shuffle() { return shuffle },
  get repeat() { return repeat },

  playSong(song: Song, songList?: Song[]) {
    if (songList) {
      originalQueue = songList
      if (shuffle) {
        const others = songList.filter(s => s.id !== song.id).sort(() => Math.random() - 0.5)
        queue = [song, ...others]
        queueIndex = 0
      } else {
        const idx = songList.findIndex(s => s.id === song.id)
        queue = songList
        queueIndex = idx >= 0 ? idx : 0
      }
    }

    currentSong = song
    const a = getAudio()
    a.src = convertFileSrc(song.path)
    a.load()
    a.play().catch((err) => console.error('[player] play failed:', err))

    api.invoke('library:update-play', song.id)
    library.markPlayed(song.id)
  },

  togglePlay() {
    const a = getAudio()
    if (isPlaying) {
      a.pause()
    } else if (currentSong) {
      a.play()
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

  next() {
    if (repeat === 'one') {
      getAudio().currentTime = 0
      getAudio().play()
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
    const a = getAudio()
    a.src = convertFileSrc(song.path)
    a.load()
    a.play().catch((err) => console.error('[player] play failed:', err))
    library.markPlayed(song.id)
    api.invoke('library:update-play', song.id)
  },

  setVolume(v: number) {
    volume = v
    getAudio().volume = v
  },

  toggleShuffle() {
    shuffle = !shuffle
    if (!currentSong || queue.length === 0) return
    if (shuffle) {
      const others = queue.filter(s => s.id !== currentSong!.id).sort(() => Math.random() - 0.5)
      queue = [currentSong!, ...others]
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

    if (pauseTimer) {
      clearTimeout(pauseTimer)
      pauseTimer = null
    }

    if (!song) {
      api.invoke('discord:update-presence', null)
      return
    }

    if (!playing) {
      pauseTimer = setTimeout(() => {
        api.invoke('discord:update-presence', null)
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
    })
  })
})
