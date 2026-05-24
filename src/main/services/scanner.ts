import fs from 'fs'
import path from 'path'
import crypto from 'crypto'
import * as mm from 'music-metadata'
import type { Song } from './store'

const AUDIO_EXTENSIONS = new Set(['.mp3', '.flac', '.wav', '.ogg', '.m4a', '.aac', '.wma', '.opus'])
const MAX_ART_BYTES = 500 * 1024

function collectAudioFiles(dir: string): string[] {
  const results: string[] = []
  try {
    const entries = fs.readdirSync(dir, { withFileTypes: true })
    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name)
      if (entry.isDirectory()) {
        results.push(...collectAudioFiles(fullPath))
      } else if (AUDIO_EXTENSIONS.has(path.extname(entry.name).toLowerCase())) {
        results.push(fullPath)
      }
    }
  } catch {
    // skip unreadable dirs
  }
  return results
}

export async function readFileMeta(filePath: string): Promise<Song> {
  const id = crypto.createHash('md5').update(filePath).digest('hex')
  let title = path.basename(filePath, path.extname(filePath))
  let artist = 'Unknown Artist'
  let album = 'Unknown Album'
  let duration = 0
  let albumArt: string | undefined
  let lyrics: string | undefined

  try {
    const stat = fs.statSync(filePath)
    const meta = await mm.parseFile(filePath, { duration: true, skipCovers: false })
    const common = meta.common
    const format = meta.format

    if (common.title) title = common.title
    if (common.artist) artist = common.artist
    if (common.album) album = common.album
    if (format.duration) duration = Math.round(format.duration)

    const cover = mm.selectCover(common.picture)
    if (cover && cover.data.length <= MAX_ART_BYTES) {
      const b64 = Buffer.from(cover.data).toString('base64')
      albumArt = `data:${cover.format};base64,${b64}`
    }

    // prefer .lrc sidecar (has timestamps) over embedded lyrics
    const lrcPath = filePath.replace(/\.[^.]+$/, '.lrc')
    if (fs.existsSync(lrcPath)) {
      try { lyrics = fs.readFileSync(lrcPath, 'utf8').trim() } catch { /* skip */ }
    }
    if (!lyrics) {
      const rawLyrics = common.lyrics?.[0]
      if (rawLyrics?.trim()) lyrics = rawLyrics.trim()
    }

    return {
      id,
      path: filePath,
      title,
      artist,
      album,
      duration,
      albumArt,
      lyrics,
      dateAdded: stat.mtimeMs,
      liked: false,
      playCount: 0
    }
  } catch {
    return {
      id,
      path: filePath,
      title,
      artist,
      album,
      duration,
      dateAdded: Date.now(),
      liked: false,
      playCount: 0
    }
  }
}

export async function scanFolder(
  folderPath: string,
  onProgress: (scanned: number, total: number, percent: number) => void
): Promise<{ songs: Song[]; errors: number }> {
  const files = collectAudioFiles(folderPath)
  const total = files.length
  const songs: Song[] = []
  let errors = 0

  for (let i = 0; i < files.length; i++) {
    try {
      const song = await readFileMeta(files[i])
      songs.push(song)
    } catch {
      errors++
    }
    const percent = Math.round(((i + 1) / total) * 100)
    onProgress(i + 1, total, percent)
  }

  return { songs, errors }
}

export function getFolderSize(dirPath: string): number {
  let size = 0
  try {
    const entries = fs.readdirSync(dirPath, { withFileTypes: true })
    for (const entry of entries) {
      const full = path.join(dirPath, entry.name)
      if (entry.isDirectory()) {
        size += getFolderSize(full)
      } else {
        try {
          size += fs.statSync(full).size
        } catch {
          // skip
        }
      }
    }
  } catch {
    // skip
  }
  return size
}
