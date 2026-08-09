import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

type Callback = (...args: unknown[]) => void
const listeners = new Map<Callback, UnlistenFn>()

function channelToCmd(channel: string): string {
  return channel.replace(/[:\-]/g, '_')
}

function mapArgs(channel: string, args: unknown[]): Record<string, unknown> {
  switch (channel) {
    case 'library:add-folder':    return { path: args[0] }
    case 'library:remove-folder': return { id: args[0] }
    case 'library:scan-folder':   return { id: args[0] }
    case 'library:import-files':  return { paths: args[0] }
    case 'library:toggle-like':   return { song_id: args[0] }
    case 'library:set-lyrics':    return { song_id: args[0], lyrics: args[1] }
    case 'library:fetch-lyrics':  return { songId: args[0], title: args[1], artist: args[2], album: args[3], duration: args[4] }
    case 'library:update-play':   return { song_id: args[0] }
    case 'playlist:create':       return { name: args[0] }
    case 'playlist:delete':       return { id: args[0] }
    case 'playlist:rename':       return { id: args[0], name: args[1] }
    case 'playlist:add-song':     return { playlist_id: args[0], song_id: args[1] }
    case 'playlist:remove-song':  return { playlist_id: args[0], song_id: args[1] }
    case 'settings:set-discord-presence': return { enabled: args[0] }
    case 'discord:update-presence':       return { payload: args[0] ?? null }
    case 'tray:set-now-playing':          return { payload: args[0] ?? null }
    default: return {}
  }
}

export const api = {
  invoke(channel: string, ...args: unknown[]): Promise<unknown> {
    return invoke(channelToCmd(channel), mapArgs(channel, args))
  },

  async on(channel: string, cb: Callback): Promise<void> {
    const unlisten = await listen(channel, (e) => cb(e.payload))
    listeners.set(cb, unlisten)
  },

  off(_channel: string, cb: Callback): void {
    const unlisten = listeners.get(cb)
    if (unlisten) {
      unlisten()
      listeners.delete(cb)
    }
  }
}
