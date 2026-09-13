import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import { applyZoom, loadZoom } from './theme'
import 'katex/dist/katex.min.css'
import './style.css'

applyZoom(loadZoom())

const standalonePwa =
  window.matchMedia('(display-mode: standalone)').matches ||
  window.matchMedia('(display-mode: windowed)').matches ||
  (window.navigator as { standalone?: boolean }).standalone === true
if (standalonePwa) {
  document.documentElement.classList.add('pwa')

  const fullscreenPwa = window.matchMedia('(display-mode: fullscreen)').matches
  const iPad = navigator.platform === 'MacIntel' && navigator.maxTouchPoints > 1
  if (iPad && !fullscreenPwa) document.documentElement.classList.add('pwa-pad')
}

createApp(App).use(router).mount('#app')
