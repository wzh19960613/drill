import { vi } from 'vitest'

function fakeStorage(): Storage {
  const map = new Map<string, string>()
  return new Proxy({} as Record<string, string>, {
    get(_, key: string) {

      if (map.has(key)) return map.get(key)
      switch (key) {
        case 'getItem':
          return (k: string) => map.get(k) ?? null
        case 'setItem':
          return (k: string, v: string) => void map.set(k, v)
        case 'removeItem':
          return (k: string) => void map.delete(k)
        case 'clear':
          return () => map.clear()
        case 'key':
          return (i: number) => [...map.keys()][i] ?? null
        case 'length':
          return map.size
        default:
          return map.get(key)
      }
    },
    ...enumeratingHandlers(map),
  }) as unknown as Storage
}

function enumeratingHandlers(map: Map<string, string>): ProxyHandler<Record<string, string>> {
  return {
    set(_, key: string, v) {

      map.set(key, typeof v === 'function' ? (v as string) : String(v))
      return true
    },
    ownKeys() {
      return [...map.keys()]
    },
    getOwnPropertyDescriptor(_, key: string) {
      return map.has(key)
        ? { enumerable: true, configurable: true, writable: true, value: map.get(key) }
        : undefined
    },
    has(_, key: string) {
      return map.has(key)
    },
    deleteProperty(_, key: string) {
      return map.delete(key)
    },
  }
}

vi.stubGlobal('localStorage', fakeStorage())
