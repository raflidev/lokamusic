# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
npm run dev       # start Electron app in dev mode with HMR
npm run build     # production build (outputs to out/)
npm run start     # preview production build
npm run check     # type-check Svelte files (renderer only)
```

## Stack

Electron + Svelte 5 + TypeScript, built with **electron-vite 5** + Vite 5 + `@sveltejs/vite-plugin-svelte@4`.

> Do NOT upgrade vite or `@sveltejs/vite-plugin-svelte` beyond these major versions — `@sveltejs/vite-plugin-svelte@6` has a bug with the Vite Environment API that causes the compiler to process `.ts` files as Svelte, and electron-vite does not yet support Vite 8.

## Project structure

```
src/
  main/
    index.ts              — Electron main: window, IPC handlers
    services/
      scanner.ts          — folder scanning + music-metadata extraction
      store.ts            — electron-store wrapper (Song[], WatchedFolder[], settings)
  preload/
    index.ts              — exposes window.electronAPI (invoke/on/off)
    index.d.ts            — Song, WatchedFolder, ScanEvent types + ElectronAPI interface
  renderer/
    index.html
    src/
      App.svelte           — app shell: Sidebar + view routing + BottomBar
      app.css              — design tokens (CSS vars), Geist font, global resets
      lib/
        stores/
          library.svelte.ts  — songs, folders, scanHistory, recentlyPlayed, likedSongs
          player.svelte.ts   — audio playback state + HTML5 Audio element
          ui.svelte.ts       — currentView, scanProgress
        components/
          Sidebar.svelte
          BottomBar.svelte   — persistent mini player (audio lives here)
          Icon.svelte        — SVG icon component (inline paths)
        views/
          LibraryView.svelte
          FoldersView.svelte
          PlayerView.svelte
          LikedSongsView.svelte
public/
  Geist-Variable.woff2    — variable font (copied from node_modules/geist)
```

## Architecture

### IPC channels (main ↔ renderer)
Renderer calls via `window.electronAPI.invoke(channel, ...args)`:
- `dialog:select-folder` → `string | null`
- `dialog:select-files` → `string[]`
- `library:get-all-songs` / `library:get-folders`
- `library:add-folder(path)` → `WatchedFolder`
- `library:remove-folder(id)` / `library:scan-folder(id)`
- `library:import-files(paths[])` → `{ added, errors }`
- `library:toggle-like(id)` / `library:update-play(id)`

Main pushes to renderer via `win.webContents.send`:
- `library:scan-progress` → `{ folderId, scanned, total, percent }`
- `library:scan-complete` → `{ folderId, songs, folders }`
- `library:songs-updated` → `Song[]`

### State management
Svelte 5 runes in `.svelte.ts` store files. Stores export plain objects with getters/setters (`library`, `player`, `ui`). Components import stores directly — no writable/readable pattern.

### Audio playback
HTML5 `<audio>` element created in `player.svelte.ts`, loaded with `file://` protocol. `BottomBar.svelte` reads from player store for display; player store drives the audio element.

### Persistence
`electron-store@8` (CJS) persists songs, folders, scan history, settings to `~/Library/Application Support/lokamusic/config.json` (macOS).

### Font
Geist Variable font at `public/Geist-Variable.woff2`. Declared with `@font-face` in `app.css`. Do not import from `node_modules` directly — Vite/Electron can't resolve it at runtime.

## Design system
Colors and tokens follow `DESIGN.md` ("Organic Precision" theme). All tokens are CSS custom properties on `:root` in `app.css`. Primary accent: `--primary` (teal). Secondary/highlight: `--secondary` (amber `#ffb77d`) used for playback controls and active states.
