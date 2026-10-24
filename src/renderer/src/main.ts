import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'
import MiniPlayerApp from './MiniPlayerApp.svelte'
import TrayPopoverApp from './TrayPopoverApp.svelte'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'

let label = ''
try {
  label = getCurrentWebviewWindow().label
} catch {
  // Not in Tauri context (browser dev mode)
}

const component = label === 'miniplayer' ? MiniPlayerApp : label === 'tray-popover' ? TrayPopoverApp : App

const app = mount(component, {
  target: document.getElementById('app')!,
})

export default app
