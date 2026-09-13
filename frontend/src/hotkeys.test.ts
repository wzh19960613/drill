import { beforeEach, describe, expect, it } from 'vitest'
import {
  applyAnswerActions,
  codeIndex,
  defaultHotkeys,
  fmtKey,
  keysOf,
  loadHotkeys,
  primaryCode,
  saveHotkeys,
} from './hotkeys'

beforeEach(() => {
  localStorage.clear()
})

describe('loadHotkeys', () => {
  it('returns defaults with no stored data', () => {
    expect(loadHotkeys()).toEqual(defaultHotkeys())
  })

  it('applies v3 overrides and ignores unknown actions', () => {
    saveHotkeys({ ...defaultHotkeys(), markWrong: 'KeyR', bogus: 'KeyQ' } as never)
    const m = loadHotkeys()
    expect(m.markWrong).toBe('KeyR')
    expect('bogus' in m).toBe(false)
  })

  it('discards maps from an unknown schema version', () => {
    localStorage.setItem('drill:hotkeys', JSON.stringify({ v: 2, map: { prev: 'KeyK' } }))
    expect(loadHotkeys().prev).toBe(defaultHotkeys().prev)
  })

  it('discards maps from an unknown schema version', () => {
    localStorage.setItem('drill:hotkeys', JSON.stringify({ v: 1, map: { prev: 'KeyK' } }))
    expect(loadHotkeys().prev).toBe(defaultHotkeys().prev)
  })
})

describe('key formatting', () => {
  it('splits bindings and drops empties', () => {
    expect(keysOf('KeyX|KeyY')).toEqual(['KeyX', 'KeyY'])
    expect(keysOf('')).toEqual([])
    expect(keysOf('KeyX|')).toEqual(['KeyX'])
  })

  it('formats primary and readable labels', () => {
    expect(primaryCode('Comma|ArrowLeft')).toBe('Comma')
    expect(fmtKey('Comma|ArrowLeft')).toBe(', / ←')
    expect(fmtKey('Space')).toBe('空格')
    expect(fmtKey('Digit1')).toBe('1')
    expect(fmtKey('Numpad5')).toBe('小键盘 5')
    expect(fmtKey('')).toBe('未绑定')
  })
})

describe('applyAnswerActions', () => {
  it('toggles with one key bound to both actions', () => {
    const both = ['showAnswer', 'hideAnswer'] as const
    expect(applyAnswerActions(false, [...both])).toBe(true)
    expect(applyAnswerActions(true, [...both])).toBe(false)
  })

  it('each action is a no-op in its already-active state', () => {
    expect(applyAnswerActions(true, ['showAnswer'])).toBe(true)
    expect(applyAnswerActions(false, ['hideAnswer'])).toBe(false)
  })

  it('non-answer actions leave the state alone', () => {
    expect(applyAnswerActions(true, ['markRight'])).toBe(true)
    expect(applyAnswerActions(false, ['next'])).toBe(false)
  })
})

describe('codeIndex', () => {
  it('indexes every binding and allows one key to serve multiple actions', () => {
    const idx = codeIndex(defaultHotkeys())
    expect(idx.get('Space')).toEqual(['showAnswer', 'hideAnswer'])
    expect(idx.get('Enter')).toEqual(['markRight'])
    expect(idx.get('Slash')).toEqual(['markMastered'])
    expect(idx.get('Backspace')).toEqual(['markMastered'])
    expect(idx.get('ArrowLeft')).toEqual(['prev'])
    expect(idx.get('KeyQ')).toBeUndefined()
  })
})
