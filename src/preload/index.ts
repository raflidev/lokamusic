import { contextBridge, ipcRenderer } from 'electron'

const wrappers = new Map<(...args: unknown[]) => void, (...args: unknown[]) => void>()

const electronAPI = {
  invoke: (channel: string, ...args: unknown[]) => ipcRenderer.invoke(channel, ...args),
  on: (channel: string, cb: (...args: unknown[]) => void) => {
    const wrapper = (_event: Electron.IpcRendererEvent, ...args: unknown[]) => cb(...args)
    wrappers.set(cb, wrapper)
    ipcRenderer.on(channel, wrapper)
  },
  off: (channel: string, cb: (...args: unknown[]) => void) => {
    const wrapper = wrappers.get(cb)
    if (wrapper) {
      ipcRenderer.removeListener(channel, wrapper as never)
      wrappers.delete(cb)
    }
  }
}

if (process.contextIsolated) {
  try {
    contextBridge.exposeInMainWorld('electronAPI', electronAPI)
  } catch (error) {
    console.error(error)
  }
} else {
  // @ts-ignore
  window.electronAPI = electronAPI
}
