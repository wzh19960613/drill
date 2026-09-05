import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import { applyZoom, loadZoom } from './theme'
import 'katex/dist/katex.min.css'
import './style.css'

applyZoom(loadZoom())

// PWA standalone window detection: on macOS standalone windows and iPadOS 26+ windowed
// mode, the traffic lights overlap the top-left of the page, and env(safe-area-inset-*)
// does not cover that overlap.
// Under windowed Safari the display mode is not necessarily standalone, so also check
// iOS's navigator.standalone.
const standalonePwa =
  window.matchMedia('(display-mode: standalone)').matches ||
  window.matchMedia('(display-mode: windowed)').matches ||
  (window.navigator as { standalone?: boolean }).standalone === true
if (standalonePwa) {
  document.documentElement.classList.add('pwa')
  // Traffic-light avoidance only for iPad (windowed and standalone both show traffic lights):
  // iPadOS's UA pretends to be a Mac (platform=MacIntel) but has multi-touch.
  // The fullscreen display mode has no window chrome (no traffic lights, and the
  // web view covers the home-indicator strip), so the avoidance must not apply.
  const fullscreenPwa = window.matchMedia('(display-mode: fullscreen)').matches
  const iPad = navigator.platform === 'MacIntel' && navigator.maxTouchPoints > 1
  if (iPad && !fullscreenPwa) document.documentElement.classList.add('pwa-pad')
}

createApp(App).use(router).mount('#app')
