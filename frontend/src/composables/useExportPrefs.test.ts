import { beforeEach, describe, expect, it } from 'vitest'
import { loadExportOpts } from './useExportPrefs'
import { StorageKeys } from '../storage'

beforeEach(() => {
  localStorage.clear()
})

function saveRaw(v: unknown) {
  localStorage.setItem(StorageKeys.exportOpts, JSON.stringify(v))
}

describe('loadExportOpts validation', () => {
  it('returns defaults when nothing is stored', () => {
    const o = loadExportOpts()
    expect(o.paper).toBe('A4')
    expect(o.sepQ).toEqual({ style: 'dashed', width: 0.6, gap: 2 })
    expect(o.sepFoot).toEqual({ style: 'none', width: 0.6, gap: 2 })
    expect(o.marginT).toBe(14)
    expect(o.marginR).toBe(16)
  })

  it('falls back to A4 for an invalid paper instead of breaking print', () => {
    saveRaw({ paper: 'A5' })
    expect(loadExportOpts().paper).toBe('A4')
    saveRaw({ paper: 42 })
    expect(loadExportOpts().paper).toBe('A4')
  })

  it('keeps a valid stored paper', () => {
    saveRaw({ paper: 'B5' })
    expect(loadExportOpts().paper).toBe('B5')
  })

  it('does not crash on sepQ: null and restores the default spec', () => {
    saveRaw({ sepQ: null, sepHead: 'garbage' })
    const o = loadExportOpts()
    expect(o.sepQ).toEqual({ style: 'dashed', width: 0.6, gap: 2 })
    expect(o.sepHead).toEqual({ style: 'dashed', width: 0.6, gap: 2 })
  })

  it('repairs invalid line-spec fields without discarding valid ones', () => {
    saveRaw({ sepQ: { style: 'zigzag', width: 99, gap: -3 } })
    const o = loadExportOpts()
    expect(o.sepQ.style).toBe('dashed')
    expect(o.sepQ.width).toBe(4)
    expect(o.sepQ.gap).toBe(0.5)
  })

  it('clamps margins into range and restores defaults for NaN', () => {
    saveRaw({ marginT: 999, marginB: -50, marginL: 'x', marginR: 20 })
    const o = loadExportOpts()
    expect(o.marginT).toBe(40)
    expect(o.marginB).toBe(5)
    expect(o.marginL).toBe(16) // NaN → default
    expect(o.marginR).toBe(20)
  })

  it('clamps bindingExtra and repairs invalid enum-ish numbers', () => {
    saveRaw({ bindingExtra: 999, perPage: 9, optsPerRow: 3, fontScale: 0.5 })
    const o = loadExportOpts()
    expect(o.bindingExtra).toBe(30)
    expect(o.perPage).toBe(2)
    expect(o.optsPerRow).toBe(4)
    expect(o.fontScale).toBe(1)
  })

  it('survives corrupted JSON', () => {
    localStorage.setItem(StorageKeys.exportOpts, '{oops')
    const o = loadExportOpts()
    expect(o.paper).toBe('A4')
  })
})
