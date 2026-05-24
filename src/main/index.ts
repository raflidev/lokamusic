import { app, shell, BrowserWindow, ipcMain, dialog, nativeImage } from 'electron'
import { join, basename } from 'path'
import fs from 'fs'
import { electronApp, optimizer, is } from '@electron-toolkit/utils'
import crypto from 'crypto'
import store from './services/store'
import { scanFolder, readFileMeta, getFolderSize } from './services/scanner'
import type { WatchedFolder, Playlist } from './services/store'

app.name = 'lokamusic'

let mainWindow: BrowserWindow | null = null

function createWindow(): void {
  let iconPath = join(__dirname, '../../public/logo.png')
  if (!fs.existsSync(iconPath)) {
    iconPath = join(__dirname, '../renderer/logo.png')
  }
  const iconImage = fs.existsSync(iconPath) ? nativeImage.createFromPath(iconPath) : undefined

  mainWindow = new BrowserWindow({
    width: 1300,
    height: 760,
    minWidth: 1300,
    minHeight: 640,
    show: false,
    autoHideMenuBar: true,
    titleBarStyle: 'hiddenInset',
    backgroundColor: '#111316',
    icon: iconImage,
    title: 'lokamusic',
    webPreferences: {
      preload: join(__dirname, '../preload/index.js'),
      sandbox: false,
      webSecurity: false
    }
  })

  mainWindow.on('ready-to-show', () => {
    mainWindow!.show()
  })

  mainWindow.webContents.setWindowOpenHandler((details) => {
    shell.openExternal(details.url)
    return { action: 'deny' }
  })

  if (is.dev && process.env['ELECTRON_RENDERER_URL']) {
    mainWindow.loadURL(process.env['ELECTRON_RENDERER_URL'])
  } else {
    mainWindow.loadFile(join(__dirname, '../renderer/index.html'))
  }
}

function registerIpcHandlers(): void {
  ipcMain.handle('dialog:select-folder', async () => {
    const result = await dialog.showOpenDialog(mainWindow!, {
      properties: ['openDirectory']
    })
    return result.canceled ? null : result.filePaths[0]
  })

  ipcMain.handle('dialog:select-files', async () => {
    const result = await dialog.showOpenDialog(mainWindow!, {
      properties: ['openFile', 'multiSelections'],
      filters: [{ name: 'Audio', extensions: ['mp3', 'flac', 'wav', 'ogg', 'm4a', 'aac', 'wma', 'opus'] }]
    })
    return result.canceled ? [] : result.filePaths
  })

  ipcMain.handle('library:get-all-songs', () => store.get('songs'))
  ipcMain.handle('library:get-folders', () => store.get('folders'))

  ipcMain.handle('library:add-folder', async (_, folderPath: string) => {
    const folders = store.get('folders')
    if (folders.find(f => f.path === folderPath)) return null

    const id = crypto.createHash('md5').update(folderPath).digest('hex')
    const folder: WatchedFolder = {
      id,
      path: folderPath,
      name: basename(folderPath),
      songCount: 0,
      sizeBytes: 0
    }

    folders.push(folder)
    store.set('folders', folders)

    // scan in background
    doScan(id, folderPath)

    return folder
  })

  ipcMain.handle('library:remove-folder', (_, folderId: string) => {
    const folder = store.get('folders').find(f => f.id === folderId)
    const folders = store.get('folders').filter(f => f.id !== folderId)
    store.set('folders', folders)
    if (folder) {
      const songs = store.get('songs').filter(s => !s.path.startsWith(folder.path))
      store.set('songs', songs)
      mainWindow?.webContents.send('library:songs-updated', songs)
    }
  })

  ipcMain.handle('library:scan-folder', (_, folderId: string) => {
    const folder = store.get('folders').find(f => f.id === folderId)
    if (folder) doScan(folderId, folder.path)
  })

  ipcMain.handle('library:import-files', async (_, filePaths: string[]) => {
    const songs = store.get('songs')
    const existingIds = new Set(songs.map(s => s.id))
    let added = 0
    let errors = 0

    for (const fp of filePaths) {
      try {
        const song = await readFileMeta(fp)
        if (!existingIds.has(song.id)) {
          songs.push(song)
          existingIds.add(song.id)
          added++
        }
      } catch {
        errors++
      }
    }

    store.set('songs', songs)
    mainWindow?.webContents.send('library:songs-updated', store.get('songs'))

    const history = store.get('scanHistory')
    history.unshift({ timestamp: Date.now(), folderPath: 'Individual files', success: added, errors })
    store.set('scanHistory', history.slice(0, 50))

    return { added, errors }
  })

  ipcMain.handle('library:toggle-like', (_, songId: string) => {
    const songs = store.get('songs').map(s =>
      s.id === songId ? { ...s, liked: !s.liked } : s
    )
    store.set('songs', songs)
    return songs.find(s => s.id === songId)?.liked ?? false
  })

  ipcMain.handle('library:update-play', (_, songId: string) => {
    const songs = store.get('songs').map(s =>
      s.id === songId
        ? { ...s, playCount: s.playCount + 1, lastPlayed: Date.now() }
        : s
    )
    store.set('songs', songs)
  })

  ipcMain.handle('playlist:get-all', () => store.get('playlists'))

  ipcMain.handle('playlist:create', (_, name: string) => {
    const playlists = store.get('playlists')
    const playlist: Playlist = {
      id: crypto.randomUUID(),
      name,
      songIds: [],
      createdAt: Date.now()
    }
    playlists.push(playlist)
    store.set('playlists', playlists)
    return playlist
  })

  ipcMain.handle('playlist:delete', (_, id: string) => {
    const playlists = store.get('playlists').filter(p => p.id !== id)
    store.set('playlists', playlists)
  })

  ipcMain.handle('playlist:rename', (_, id: string, name: string) => {
    const playlists = store.get('playlists').map(p =>
      p.id === id ? { ...p, name } : p
    )
    store.set('playlists', playlists)
    return playlists.find(p => p.id === id) ?? null
  })

  ipcMain.handle('playlist:add-song', (_, playlistId: string, songId: string) => {
    const playlists = store.get('playlists').map(p =>
      p.id === playlistId && !p.songIds.includes(songId)
        ? { ...p, songIds: [...p.songIds, songId] }
        : p
    )
    store.set('playlists', playlists)
  })

  ipcMain.handle('playlist:remove-song', (_, playlistId: string, songId: string) => {
    const playlists = store.get('playlists').map(p =>
      p.id === playlistId ? { ...p, songIds: p.songIds.filter(id => id !== songId) } : p
    )
    store.set('playlists', playlists)
  })
}

let scanQueue = Promise.resolve()

async function doScan(folderId: string, folderPath: string): Promise<void> {
  scanQueue = scanQueue.then(() => runScan(folderId, folderPath))
  return scanQueue
}

async function runScan(folderId: string, folderPath: string): Promise<void> {
  try {
    const { songs: newSongs, errors } = await scanFolder(folderPath, (scanned, total, percent) => {
      mainWindow?.webContents.send('library:scan-progress', { folderId, scanned, total, percent })
    })

    // re-read store after scan completes to pick up any changes (likes/plays) made during scan
    const existing = store.get('songs')
    const existingMap = new Map(existing.map(s => [s.id, s]))
    // keep songs from OTHER folders as-is; for THIS folder, only keep what the scan found
    const kept = existing.filter(s => !s.path.startsWith(folderPath))
    const updated = newSongs.map(s => {
      const prev = existingMap.get(s.id)
      if (!prev) return s
      return { ...s, liked: prev.liked, playCount: prev.playCount, lastPlayed: prev.lastPlayed }
    })
    const merged = [...kept, ...updated]
    store.set('songs', merged)

    const folders = store.get('folders').map(f =>
      f.id === folderId
        ? { ...f, songCount: newSongs.length, sizeBytes: getFolderSize(folderPath), lastScanned: Date.now() }
        : f
    )
    store.set('folders', folders)

    mainWindow?.webContents.send('library:scan-complete', {
      folderId,
      songs: merged,
      folders: store.get('folders')
    })

    const history = store.get('scanHistory')
    const folder = folders.find(f => f.id === folderId)
    history.unshift({ timestamp: Date.now(), folderPath: folder?.path ?? folderPath, success: newSongs.length, errors })
    store.set('scanHistory', history.slice(0, 50))
  } catch {
    mainWindow?.webContents.send('library:scan-complete', {
      folderId,
      songs: store.get('songs'),
      folders: store.get('folders')
    })
  }
}

function purgeOrphanSongs(): void {
  const folderPaths = store.get('folders').map(f => f.path)
  const songs = store.get('songs').filter(s => folderPaths.some(fp => s.path.startsWith(fp)))
  store.set('songs', songs)
}

app.whenReady().then(() => {
  electronApp.setAppUserModelId('com.lokamusic')
  purgeOrphanSongs()

  // Set Dock Icon on macOS
  let iconPath = join(__dirname, '../../public/logo.png')
  if (!fs.existsSync(iconPath)) {
    iconPath = join(__dirname, '../renderer/logo.png')
  }
  if (process.platform === 'darwin' && fs.existsSync(iconPath)) {
    app.dock.setIcon(nativeImage.createFromPath(iconPath))
  }

  app.on('browser-window-created', (_, window) => {
    optimizer.watchWindowShortcuts(window)
  })

  registerIpcHandlers()
  createWindow()

  app.on('activate', function () {
    if (BrowserWindow.getAllWindows().length === 0) createWindow()
  })
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  }
})
