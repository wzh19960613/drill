import { afterEach, describe, expect, it } from 'vitest'
import { keysWithPrefix, readJSON, readString, removeKey, writeJSON, writeString } from './storage'

const origGet = localStorage.getItem.bind(localStorage)
const origSet = localStorage.setItem.bind(localStorage)
const origRemove = localStorage.removeItem.bind(localStorage)

afterEach(() => {
  localStorage.getItem = origGet
  localStorage.setItem = origSet
  localStorage.removeItem = origRemove
  localStorage.clear()
})

describe('readString / writeString', () => {
  it('round-trips values', () => {
    writeString('k', 'v')
    expect(readString('k')).toBe('v')
  })

  it('returns null for missing keys', () => {
    expect(readString('nope')).toBeNull()
  })

  it('does not throw when storage access throws', () => {
    localStorage.getItem = () => {
      throw new Error('SecurityError')
    }
    localStorage.setItem = () => {
      throw new Error('QuotaExceededError')
    }
    expect(readString('k')).toBeNull()
    expect(() => writeString('k', 'v')).not.toThrow()
  })
})

describe('readJSON / writeJSON', () => {
  it('round-trips JSON values', () => {
    writeJSON('j', { a: 1, b: [true] })
    expect(readJSON('j')).toEqual({ a: 1, b: [true] })
  })

  it('returns undefined for missing keys', () => {
    expect(readJSON('nope')).toBeUndefined()
  })

  it('returns undefined for corrupted JSON', () => {
    origSet('bad', '{oops')
    expect(readJSON('bad')).toBeUndefined()
  })

  it('does not throw when writing fails and reports failure', () => {
    localStorage.setItem = () => {
      throw new Error('QuotaExceededError')
    }
    expect(() => writeJSON('j', { a: 1 })).not.toThrow()
    expect(writeJSON('j', { a: 1 })).toBe(false)
    expect(writeJSON('ok', 1)).toBe(false)
  })
})

describe('removeKey / keysWithPrefix', () => {
  it('removes keys safely and lists keys by prefix', () => {
    writeString('drill:x:1', 'a')
    writeString('drill:x:2', 'b')
    writeString('other', 'c')
    expect(keysWithPrefix('drill:x:').sort()).toEqual(['drill:x:1', 'drill:x:2'])
    removeKey('drill:x:1')
    expect(readString('drill:x:1')).toBeNull()
    expect(keysWithPrefix('drill:x:')).toEqual(['drill:x:2'])
    localStorage.removeItem = () => {
      throw new Error('nope')
    }
    expect(() => removeKey('drill:x:2')).not.toThrow()
    expect(() => keysWithPrefix('drill:x:')).not.toThrow()
  })
})
