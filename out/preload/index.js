"use strict";
const electron = require("electron");
const electronAPI = {
  invoke: (channel, ...args) => electron.ipcRenderer.invoke(channel, ...args),
  on: (channel, cb) => {
    electron.ipcRenderer.on(channel, (_event, ...args) => cb(...args));
  },
  off: (channel, cb) => {
    electron.ipcRenderer.removeListener(channel, cb);
  }
};
if (process.contextIsolated) {
  try {
    electron.contextBridge.exposeInMainWorld("electronAPI", electronAPI);
  } catch (error) {
    console.error(error);
  }
} else {
  window.electronAPI = electronAPI;
}
