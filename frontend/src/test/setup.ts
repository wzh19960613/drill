import { vi } from 'vitest'

/** The node test environment has no localStorage: provide a behaviour-equivalent
 *  Proxy fake (including Object.keys enumeration). */
function fakeStorage(): Storage {
  const map = new Map<string, string>()
  return new Proxy({} as Record<string, string>, {
    get(_, key: string) {
      // Test-injected overrides (e.g. a mocked setItem simulating quota
      // errors) take precedence over the built-in methods
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
    set(_, key: string, v) {
      // Functions are stored as-is (tests may inject a setItem override);
      // everything else is stringified per the Storage contract
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
  }) as unknown as Storage
}

vi.stubGlobal('localStorage', fakeStorage())
