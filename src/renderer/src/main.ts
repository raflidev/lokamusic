import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'
import MiniPlayerApp from './MiniPlayerApp.svelte'
import { initAnalytics } from './lib/analytics'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'

initAnalytics()

let isMiniPlayer = false
try {
  isMiniPlayer = getCurrentWebviewWindow().label === 'miniplayer'
} catch {
  // Not in Tauri context (browser dev mode)
}

const app = mount(isMiniPlayer ? MiniPlayerApp : App, {
  target: document.getElementById('app')!,
})

export default app
