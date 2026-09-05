import { ref, watch } from 'vue'
import { readString, writeString, StorageKeys } from './storage'
export type ThemeMode = 'auto' | 'light' | 'dark'

const media = window.matchMedia('(prefers-color-scheme: dark)')

export const themeMode = ref<ThemeMode>(loadMode())
export const themeDark = ref(false)

function loadMode(): ThemeMode {
  const v = readString(StorageKeys.theme)
  return v === 'light' || v === 'dark' || v === 'auto' ? v : 'auto'
}

function apply() {
  const dark = themeMode.value === 'dark' || (themeMode.value === 'auto' && media.matches)
  themeDark.value = dark
  document.documentElement.dataset.theme = dark ? 'dark' : 'light'
}

watch(themeMode, (m) => {
  writeString(StorageKeys.theme, m)
  apply()
})

media.addEventListener('change', () => {
  if (themeMode.value === 'auto') apply()
})

apply()

const THEME_ORDER: ThemeMode[] = ['auto', 'light', 'dark']
const THEME_LABELS: Record<ThemeMode, string> = { auto: '系统', light: '浅色', dark: '深色' }

export function cycleTheme(): ThemeMode {
  const i = THEME_ORDER.indexOf(themeMode.value)
  themeMode.value = THEME_ORDER[(i + 1) % THEME_ORDER.length]
  return themeMode.value
}

export function themeLabel(m: ThemeMode): string {
  return THEME_LABELS[m]
}

export const BASE_FONT_PX = 16

export const ZOOM_STEPS: { v: number; label: string }[] = [
  { v: 0.8, label: '更小' },
  { v: 0.9, label: '小' },
  { v: 1, label: '标准' },
  { v: 1.15, label: '大' },
  { v: 1.3, label: '更大' },
  { v: 1.5, label: '超大' },
]

export function loadZoom(): number {
  const v = Number(readString(StorageKeys.appZoom))
  return ZOOM_STEPS.some((s) => s.v === v) ? v : 1
}

export function applyZoom(v: number) {
  writeString(StorageKeys.appZoom, String(v))
  document.documentElement.style.fontSize = `${Math.round(v * BASE_FONT_PX * 100) / 100}px`
}
