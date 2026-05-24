# Lokamusic

A local music player desktop app built with Electron, Svelte 5, and TypeScript. Designed around an "Organic Precision" aesthetic — dark, editorial, and tactile.

## Features

- Scan local folders and import music files
- Automatic metadata extraction (title, artist, album, duration, cover art)
- Persistent library across sessions
- Like songs and build a personal collection
- Full playback controls with seek, volume, and shuffle
- Recently played history
- Minimal, dark-mode interface with Geist typography

## Stack

| Layer | Technology |
|---|---|
| Desktop shell | Electron 42 |
| UI framework | Svelte 5 (runes) |
| Language | TypeScript 6 |
| Build tool | electron-vite 5 + Vite 5 |
| Styling | CSS custom properties (design tokens) |
| Persistence | electron-store 8 |
| Metadata | music-metadata 7 |
| Font | Geist Variable |

## Getting Started

```bash
npm install
npm run dev       # start in dev mode with HMR
npm run build     # production build → out/
npm run start     # preview production build
npm run check     # type-check Svelte files
```

## Project Structure

```
src/
  main/
    index.ts              — Electron main: window creation, IPC handlers
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
          BottomBar.svelte   — persistent mini player
          Icon.svelte        — SVG icon component
        views/
          LibraryView.svelte
          FoldersView.svelte
          PlayerView.svelte
          LikedSongsView.svelte
public/
  Geist-Variable.woff2    — variable font (copied from node_modules/geist)
```

## Design System

Colors and tokens follow `DESIGN.md` ("Organic Precision" theme). All tokens are CSS custom properties on `:root` in `app.css`.

- **Primary accent:** `--primary` — deep teal (`#aecccc`) for structural emphasis
- **Secondary accent:** `--secondary` — muted amber (`#ffb77d`) for playback controls and active states
- **Surface:** dark warm slate (`#111316`), never pure black
- **Typography:** Geist Variable, editorial weight/spacing hierarchy

## Notes

- Do **not** upgrade `@sveltejs/vite-plugin-svelte` beyond v4 or `vite` beyond v5 — `@sveltejs/vite-plugin-svelte@6` has a bug with the Vite Environment API that miscompiles `.ts` files as Svelte, and electron-vite does not yet support Vite 8.
- Data is persisted to `~/Library/Application Support/lokamusic/config.json` on macOS.
- Audio playback uses the HTML5 `<audio>` element with `file://` protocol paths.
