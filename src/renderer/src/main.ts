import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'
import { initAnalytics } from './lib/analytics'

initAnalytics()

const app = mount(App, {
  target: document.getElementById('app')!,
})

export default app
