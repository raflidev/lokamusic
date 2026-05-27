# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
npm run dev              # start Tauri app in dev mode (Rust + Vite HMR)
npm run build            # production build (Tauri bundle)
npm run dev:renderer     # start Vite renderer only (no Tauri)
npm run build:renderer   # build renderer only
npm run check            # type-check Svelte files (renderer only)
```

## Stack

**Tauri 2** + Svelte 5 + TypeScript, built with Vite 5 + `@sveltejs/vite-plugin-svelte@4`. Rust backend via `src-tauri/`.

> Do NOT upgrade vite or `@sveltejs/vite-plugin-svelte` beyond these major versions — `@sveltejs/vite-plugin-svelte@6` has a bug with the Vite Environment API that causes the compiler to process `.ts` files as Svelte.

## Project structure

```
src-tauri/
  src/
    lib.rs              — Tauri commands, app setup, Discord RPC, store helpers
    scanner.rs          — folder scanning + audio metadata extraction (Rust)
    types.rs            — Song, WatchedFolder, Playlist, Settings, ScanEvent types
  Cargo.toml            — Rust deps (tauri, tauri-plugin-store, discord-rich-presence, etc.)
  tauri.conf.json       — app config, bundle identifiers, permissions
  capabilities/
    default.json        — Tauri capability grants

src/renderer/
  index.html
  src/
    App.svelte           — app shell: Sidebar + view routing + BottomBar + QueueSidebar
    app.css              — design tokens (CSS vars), Geist font, global resets
    main.ts              — renderer entry point
    types.ts             — shared TS types (Song, WatchedFolder, Playlist, etc.)
    lib/
      api.ts             — Tauri invoke/listen wrapper (maps channel names to commands)
      stores/
        library.svelte.ts  — songs, folders, scanHistory, recentlyPlayed, likedSongs
        player.svelte.ts   — audio playback state + HTML5 Audio + Discord presence trigger
        playlists.svelte.ts — playlist CRUD state
        ui.svelte.ts       — currentView, scanProgress, search, navigation helpers
      components/
        Sidebar.svelte
        BottomBar.svelte   — persistent mini player
        FolderArt.svelte   — folder artwork display
        FolderTreeNode.svelte — recursive folder tree node
        QueueSidebar.svelte — slide-out queue panel
        Icon.svelte        — SVG icon component (inline paths)
      views/
        LibraryView.svelte
        FoldersView.svelte
        FolderTreeView.svelte
        PlayerView.svelte
        LikedSongsView.svelte
        AlbumsView.svelte
        ArtistsView.svelte
        PlaylistView.svelte
        SettingsView.svelte
        AboutView.svelte

public/
  Geist-Variable.woff2  — variable font (copied from node_modules/geist)
  icons.svg             — SVG sprite
  favicon.svg
```

## Architecture

### IPC: Tauri commands (frontend → backend)
Renderer calls via `api.invoke(channel, ...args)` (see `lib/api.ts`), which maps to Rust `#[tauri::command]` functions:

- `dialog:select-folder` → `Option<String>`
- `dialog:select-files` → `Vec<String>`
- `library:get-all-songs` / `library:get-folders`
- `library:add-folder(path)` → `Option<WatchedFolder>`
- `library:remove-folder(id)` / `library:scan-folder(id)`
- `library:import-files(paths[])` → `{ added, errors }`
- `library:toggle-like(song_id)` / `library:update-play(song_id)`
- `playlist:get-all` / `playlist:create(name)` / `playlist:delete(id)`
- `playlist:rename(id, name)` / `playlist:add-song(playlist_id, song_id)` / `playlist:remove-song(playlist_id, song_id)`
- `settings:get` / `settings:set-discord-presence(enabled)`
- `discord:update-presence(payload)`

`api.ts` converts channel names (`library:add-folder`) to snake_case Tauri commands (`library_add_folder`) and maps positional args to named param objects.

### IPC: Tauri events (backend → frontend)
Backend emits via `app.emit(...)`, renderer listens via `api.on(channel, cb)`:
- `library:scan-progress` → `{ folderId, scanned, total, percent }`
- `library:scan-complete` → `{ folderId, songs, folders }`
- `library:songs-updated` → `Song[]`

### State management
Svelte 5 runes in `.svelte.ts` store files. Stores export plain objects with getters/setters (`library`, `player`, `ui`, `playlistStore`). Components import stores directly — no writable/readable pattern.

### Audio playback
HTML5 `<audio>` element created lazily in `player.svelte.ts`. Files loaded via `convertFileSrc(path)` from `@tauri-apps/api/core` (converts native paths to Tauri asset URLs). Player store triggers Discord presence updates via `api.invoke('discord:update-presence', ...)` on song/state changes.

### Discord Rich Presence
Managed in `lib.rs` via `DiscordState` (wraps `discord-rich-presence` crate). Artwork fetched from iTunes Search API and cached per `artist::album` key. Enabled/disabled via `Settings.discord_presence` persisted in store.

### Persistence
`tauri-plugin-store` persists JSON to the app data directory (`config.json`). Keys: `songs`, `folders`, `scanHistory`, `playlists`, `settings`. Store helpers in `lib.rs`: `store_get`, `store_get_obj`, `store_set`.

Pinned folders use `localStorage` (renderer-side only, keyed `lokamusic:pinnedFolders`).

### Font
Geist Variable font at `public/Geist-Variable.woff2`. Declared with `@font-face` in `app.css`. Do not import from `node_modules` directly.

## Design system
Colors and tokens follow `DESIGN.md` ("Organic Precision" theme). All tokens are CSS custom properties on `:root` in `app.css`. Primary accent: `--primary` (teal). Secondary/highlight: `--secondary` (amber `#ffb77d`) used for playback controls and active states.
