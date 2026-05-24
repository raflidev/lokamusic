"use strict";
const electron = require("electron");
const wrappers = /* @__PURE__ */ new Map();
const electronAPI = {
  invoke: (channel, ...args) => electron.ipcRenderer.invoke(channel, ...args),
  on: (channel, cb) => {
    const wrapper = (_event, ...args) => cb(...args);
    wrappers.set(cb, wrapper);
    electron.ipcRenderer.on(channel, wrapper);
  },
  off: (channel, cb) => {
    const wrapper = wrappers.get(cb);
    if (wrapper) {
      electron.ipcRenderer.removeListener(channel, wrapper);
      wrappers.delete(cb);
    }
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
