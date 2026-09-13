export const StorageKeys = {
  subject: 'drill:subject',
  theme: 'drill:theme',
  appZoom: 'drill:app-zoom',
  hotkeys: 'drill:hotkeys',
  bookConfig: 'drill:book-config',
  exportOpts: 'drill:export-opts',
} as const

export function readString(key: string): string | null {
  try {
    return localStorage.getItem(key)
  } catch {
    return null
  }
}

export function writeString(key: string, v: string): void {
  try {
    localStorage.setItem(key, v)
  } catch {

  }
}

export function readJSON<T = unknown>(key: string): T | undefined {
  try {
    const raw = localStorage.getItem(key)
    return raw ? (JSON.parse(raw) as T) : undefined
  } catch {
    return undefined
  }
}

export function writeJSON(key: string, v: unknown): boolean {
  try {
    localStorage.setItem(key, JSON.stringify(v))
    return true
  } catch {
    return false
  }
}

export function removeKey(key: string): void {
  try {
    localStorage.removeItem(key)
  } catch {
  }
}

export function keysWithPrefix(prefix: string): string[] {
  try {
    return Object.keys(localStorage).filter((k) => k.startsWith(prefix))
  } catch {
    return []
  }
}
