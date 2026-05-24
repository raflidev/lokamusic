"use strict";
const electron = require("electron");
const path = require("path");
const fs = require("fs");
const crypto = require("crypto");
const Store = require("electron-store");
const mm = require("music-metadata");
function _interopNamespaceDefault(e) {
  const n = Object.create(null, { [Symbol.toStringTag]: { value: "Module" } });
  if (e) {
    for (const k in e) {
      if (k !== "default") {
        const d = Object.getOwnPropertyDescriptor(e, k);
        Object.defineProperty(n, k, d.get ? d : {
          enumerable: true,
          get: () => e[k]
        });
      }
    }
  }
  n.default = e;
  return Object.freeze(n);
}
const mm__namespace = /* @__PURE__ */ _interopNamespaceDefault(mm);
const is = {
  dev: !electron.app.isPackaged
};
const platform = {
  isWindows: process.platform === "win32",
  isMacOS: process.platform === "darwin",
  isLinux: process.platform === "linux"
};
const electronApp = {
  setAppUserModelId(id) {
    if (platform.isWindows)
      electron.app.setAppUserModelId(is.dev ? process.execPath : id);
  },
  setAutoLaunch(auto) {
    if (platform.isLinux)
      return false;
    const isOpenAtLogin = () => {
      return electron.app.getLoginItemSettings().openAtLogin;
    };
    if (isOpenAtLogin() !== auto) {
      electron.app.setLoginItemSettings({ openAtLogin: auto });
      return isOpenAtLogin() === auto;
    } else {
      return true;
    }
  },
  skipProxy() {
    return electron.session.defaultSession.setProxy({ mode: "direct" });
  }
};
const optimizer = {
  watchWindowShortcuts(window, shortcutOptions) {
    if (!window)
      return;
    const { webContents } = window;
    const { escToCloseWindow = false, zoom = false } = shortcutOptions || {};
    webContents.on("before-input-event", (event, input) => {
      if (input.type === "keyDown") {
        if (!is.dev) {
          if (input.code === "KeyR" && (input.control || input.meta))
            event.preventDefault();
          if (input.code === "KeyI" && (input.alt && input.meta || input.control && input.shift)) {
            event.preventDefault();
          }
        } else {
          if (input.code === "F12") {
            if (webContents.isDevToolsOpened()) {
              webContents.closeDevTools();
            } else {
              webContents.openDevTools({ mode: "undocked" });
              console.log("Open dev tool...");
            }
          }
        }
        if (escToCloseWindow) {
          if (input.code === "Escape" && input.key !== "Process") {
            window.close();
            event.preventDefault();
          }
        }
        if (!zoom) {
          if (input.code === "Minus" && (input.control || input.meta))
            event.preventDefault();
          if (input.code === "Equal" && input.shift && (input.control || input.meta))
            event.preventDefault();
        }
      }
    });
  },
  registerFramelessWindowIpc() {
    electron.ipcMain.on("win:invoke", (event, action) => {
      const win = electron.BrowserWindow.fromWebContents(event.sender);
      if (win) {
        if (action === "show") {
          win.show();
        } else if (action === "showInactive") {
          win.showInactive();
        } else if (action === "min") {
          win.minimize();
        } else if (action === "max") {
          const isMaximized = win.isMaximized();
          if (isMaximized) {
            win.unmaximize();
          } else {
            win.maximize();
          }
        } else if (action === "close") {
          win.close();
        }
      }
    });
  }
};
const store = new Store({
  defaults: {
    songs: [],
    folders: [],
    scanHistory: [],
    playlists: [],
    settings: { volume: 0.8, shuffle: false, repeat: "none" }
  }
});
const AUDIO_EXTENSIONS = /* @__PURE__ */ new Set([".mp3", ".flac", ".wav", ".ogg", ".m4a", ".aac", ".wma", ".opus"]);
const MAX_ART_BYTES = 5 * 1024 * 1024;
function collectAudioFiles(dir) {
  const results = [];
  try {
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        results.push(...collectAudioFiles(fullPath));
      } else if (AUDIO_EXTENSIONS.has(path.extname(entry.name).toLowerCase())) {
        results.push(fullPath);
      }
    }
  } catch {
  }
  return results;
}
async function readFileMeta(filePath) {
  const id = crypto.createHash("md5").update(filePath).digest("hex");
  let title = path.basename(filePath, path.extname(filePath));
  let artist = "Unknown Artist";
  let album = "Unknown Album";
  let duration = 0;
  let albumArt;
  let lyrics;
  try {
    const stat = fs.statSync(filePath);
    const meta = await mm__namespace.parseFile(filePath, { duration: true, skipCovers: false });
    const common = meta.common;
    const format = meta.format;
    if (common.title) title = common.title;
    if (common.artist) artist = common.artist;
    if (common.album) album = common.album;
    if (format.duration) duration = Math.round(format.duration);
    const cover = mm__namespace.selectCover(common.picture);
    if (cover && cover.data.length <= MAX_ART_BYTES) {
      const b64 = Buffer.from(cover.data).toString("base64");
      const mime = cover.format === "image/jpg" ? "image/jpeg" : cover.format || "image/jpeg";
      albumArt = `data:${mime};base64,${b64}`;
    }
    const lrcPath = filePath.replace(/\.[^.]+$/, ".lrc");
    if (fs.existsSync(lrcPath)) {
      try {
        lyrics = fs.readFileSync(lrcPath, "utf8").trim();
      } catch {
      }
    }
    if (!lyrics) {
      const rawLyrics = common.lyrics?.[0];
      if (rawLyrics?.trim()) lyrics = rawLyrics.trim();
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
    };
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
    };
  }
}
async function scanFolder(folderPath, onProgress) {
  const files = collectAudioFiles(folderPath);
  const total = files.length;
  const songs = [];
  let errors = 0;
  for (let i = 0; i < files.length; i++) {
    try {
      const song = await readFileMeta(files[i]);
      songs.push(song);
    } catch {
      errors++;
    }
    const percent = Math.round((i + 1) / total * 100);
    onProgress(i + 1, total, percent);
  }
  return { songs, errors };
}
function getFolderSize(dirPath) {
  let size = 0;
  try {
    const entries = fs.readdirSync(dirPath, { withFileTypes: true });
    for (const entry of entries) {
      const full = path.join(dirPath, entry.name);
      if (entry.isDirectory()) {
        size += getFolderSize(full);
      } else {
        try {
          size += fs.statSync(full).size;
        } catch {
        }
      }
    }
  } catch {
  }
  return size;
}
electron.app.name = "lokamusic";
let mainWindow = null;
function createWindow() {
  let iconPath = path.join(__dirname, "../../public/logo.png");
  if (!fs.existsSync(iconPath)) {
    iconPath = path.join(__dirname, "../renderer/logo.png");
  }
  const iconImage = fs.existsSync(iconPath) ? electron.nativeImage.createFromPath(iconPath) : void 0;
  mainWindow = new electron.BrowserWindow({
    width: 1300,
    height: 760,
    minWidth: 1300,
    minHeight: 640,
    show: false,
    autoHideMenuBar: true,
    titleBarStyle: "hiddenInset",
    backgroundColor: "#111316",
    icon: iconImage,
    title: "lokamusic",
    webPreferences: {
      preload: path.join(__dirname, "../preload/index.js"),
      sandbox: false,
      webSecurity: false
    }
  });
  mainWindow.on("ready-to-show", () => {
    mainWindow.show();
  });
  mainWindow.webContents.setWindowOpenHandler((details) => {
    electron.shell.openExternal(details.url);
    return { action: "deny" };
  });
  if (is.dev && process.env["ELECTRON_RENDERER_URL"]) {
    mainWindow.loadURL(process.env["ELECTRON_RENDERER_URL"]);
  } else {
    mainWindow.loadFile(path.join(__dirname, "../renderer/index.html"));
  }
}
function registerIpcHandlers() {
  electron.ipcMain.handle("dialog:select-folder", async () => {
    const result = await electron.dialog.showOpenDialog(mainWindow, {
      properties: ["openDirectory"]
    });
    return result.canceled ? null : result.filePaths[0];
  });
  electron.ipcMain.handle("dialog:select-files", async () => {
    const result = await electron.dialog.showOpenDialog(mainWindow, {
      properties: ["openFile", "multiSelections"],
      filters: [{ name: "Audio", extensions: ["mp3", "flac", "wav", "ogg", "m4a", "aac", "wma", "opus"] }]
    });
    return result.canceled ? [] : result.filePaths;
  });
  electron.ipcMain.handle("library:get-all-songs", () => store.get("songs"));
  electron.ipcMain.handle("library:get-folders", () => store.get("folders"));
  electron.ipcMain.handle("library:add-folder", async (_, folderPath) => {
    const folders = store.get("folders");
    if (folders.find((f) => f.path === folderPath)) return null;
    const id = crypto.createHash("md5").update(folderPath).digest("hex");
    const folder = {
      id,
      path: folderPath,
      name: path.basename(folderPath),
      songCount: 0,
      sizeBytes: 0
    };
    folders.push(folder);
    store.set("folders", folders);
    doScan(id, folderPath);
    return folder;
  });
  electron.ipcMain.handle("library:remove-folder", (_, folderId) => {
    const folder = store.get("folders").find((f) => f.id === folderId);
    const folders = store.get("folders").filter((f) => f.id !== folderId);
    store.set("folders", folders);
    if (folder) {
      const songs = store.get("songs").filter((s) => !s.path.startsWith(folder.path));
      store.set("songs", songs);
      mainWindow?.webContents.send("library:songs-updated", songs);
    }
  });
  electron.ipcMain.handle("library:scan-folder", (_, folderId) => {
    const folder = store.get("folders").find((f) => f.id === folderId);
    if (folder) doScan(folderId, folder.path);
  });
  electron.ipcMain.handle("library:import-files", async (_, filePaths) => {
    const songs = store.get("songs");
    const existingIds = new Set(songs.map((s) => s.id));
    let added = 0;
    let errors = 0;
    for (const fp of filePaths) {
      try {
        const song = await readFileMeta(fp);
        if (!existingIds.has(song.id)) {
          songs.push(song);
          existingIds.add(song.id);
          added++;
        }
      } catch {
        errors++;
      }
    }
    store.set("songs", songs);
    mainWindow?.webContents.send("library:songs-updated", store.get("songs"));
    const history = store.get("scanHistory");
    history.unshift({ timestamp: Date.now(), folderPath: "Individual files", success: added, errors });
    store.set("scanHistory", history.slice(0, 50));
    return { added, errors };
  });
  electron.ipcMain.handle("library:toggle-like", (_, songId) => {
    const songs = store.get("songs").map(
      (s) => s.id === songId ? { ...s, liked: !s.liked } : s
    );
    store.set("songs", songs);
    return songs.find((s) => s.id === songId)?.liked ?? false;
  });
  electron.ipcMain.handle("library:update-play", (_, songId) => {
    const songs = store.get("songs").map(
      (s) => s.id === songId ? { ...s, playCount: s.playCount + 1, lastPlayed: Date.now() } : s
    );
    store.set("songs", songs);
  });
  electron.ipcMain.handle("playlist:get-all", () => store.get("playlists"));
  electron.ipcMain.handle("playlist:create", (_, name) => {
    const playlists = store.get("playlists");
    const playlist = {
      id: crypto.randomUUID(),
      name,
      songIds: [],
      createdAt: Date.now()
    };
    playlists.push(playlist);
    store.set("playlists", playlists);
    return playlist;
  });
  electron.ipcMain.handle("playlist:delete", (_, id) => {
    const playlists = store.get("playlists").filter((p) => p.id !== id);
    store.set("playlists", playlists);
  });
  electron.ipcMain.handle("playlist:rename", (_, id, name) => {
    const playlists = store.get("playlists").map(
      (p) => p.id === id ? { ...p, name } : p
    );
    store.set("playlists", playlists);
    return playlists.find((p) => p.id === id) ?? null;
  });
  electron.ipcMain.handle("playlist:add-song", (_, playlistId, songId) => {
    const playlists = store.get("playlists").map(
      (p) => p.id === playlistId && !p.songIds.includes(songId) ? { ...p, songIds: [...p.songIds, songId] } : p
    );
    store.set("playlists", playlists);
  });
  electron.ipcMain.handle("playlist:remove-song", (_, playlistId, songId) => {
    const playlists = store.get("playlists").map(
      (p) => p.id === playlistId ? { ...p, songIds: p.songIds.filter((id) => id !== songId) } : p
    );
    store.set("playlists", playlists);
  });
}
let scanQueue = Promise.resolve();
async function doScan(folderId, folderPath) {
  scanQueue = scanQueue.then(() => runScan(folderId, folderPath));
  return scanQueue;
}
async function runScan(folderId, folderPath) {
  try {
    const { songs: newSongs, errors } = await scanFolder(folderPath, (scanned, total, percent) => {
      mainWindow?.webContents.send("library:scan-progress", { folderId, scanned, total, percent });
    });
    const existing = store.get("songs");
    const existingMap = new Map(existing.map((s) => [s.id, s]));
    const kept = existing.filter((s) => !s.path.startsWith(folderPath));
    const updated = newSongs.map((s) => {
      const prev = existingMap.get(s.id);
      if (!prev) return s;
      return { ...s, liked: prev.liked, playCount: prev.playCount, lastPlayed: prev.lastPlayed };
    });
    const merged = [...kept, ...updated];
    store.set("songs", merged);
    const folders = store.get("folders").map(
      (f) => f.id === folderId ? { ...f, songCount: newSongs.length, sizeBytes: getFolderSize(folderPath), lastScanned: Date.now() } : f
    );
    store.set("folders", folders);
    mainWindow?.webContents.send("library:scan-complete", {
      folderId,
      songs: merged,
      folders: store.get("folders")
    });
    const history = store.get("scanHistory");
    const folder = folders.find((f) => f.id === folderId);
    history.unshift({ timestamp: Date.now(), folderPath: folder?.path ?? folderPath, success: newSongs.length, errors });
    store.set("scanHistory", history.slice(0, 50));
  } catch {
    mainWindow?.webContents.send("library:scan-complete", {
      folderId,
      songs: store.get("songs"),
      folders: store.get("folders")
    });
  }
}
function purgeOrphanSongs() {
  const folderPaths = store.get("folders").map((f) => f.path);
  const songs = store.get("songs").filter((s) => folderPaths.some((fp) => s.path.startsWith(fp)));
  store.set("songs", songs);
}
electron.app.whenReady().then(() => {
  electronApp.setAppUserModelId("com.lokamusic");
  purgeOrphanSongs();
  let iconPath = path.join(__dirname, "../../public/logo.png");
  if (!fs.existsSync(iconPath)) {
    iconPath = path.join(__dirname, "../renderer/logo.png");
  }
  if (process.platform === "darwin" && fs.existsSync(iconPath)) {
    electron.app.dock.setIcon(electron.nativeImage.createFromPath(iconPath));
  }
  electron.app.on("browser-window-created", (_, window) => {
    optimizer.watchWindowShortcuts(window);
  });
  registerIpcHandlers();
  createWindow();
  electron.app.on("activate", function() {
    if (electron.BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});
electron.app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    electron.app.quit();
  }
});
