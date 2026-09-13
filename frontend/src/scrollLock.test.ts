import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { lockBodyScroll, unlockBodyScroll } from './scrollLock'

function fakeEl() {
  return {
    style: { overflow: '', paddingRight: '' } as Record<string, string>,
    offsetWidth: 100,
    clientWidth: 92,
  }
}

let html: ReturnType<typeof fakeEl>
let app: ReturnType<typeof fakeEl> | null

beforeEach(() => {
  html = fakeEl()
  app = null
  vi.stubGlobal('document', {
    querySelector: (sel: string) => (sel === '.app' ? app : null),
    documentElement: html,
  })
})

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('scrollLock', () => {
  it('locks and restores the document scroller', () => {
    lockBodyScroll()
    expect(html.style.overflow).toBe('hidden')
    expect(html.style.paddingRight).toBe('8px')
    unlockBodyScroll()
    expect(html.style.overflow).toBe('')
    expect(html.style.paddingRight).toBe('')
  })

  it('prefers the .app scroller when present', () => {
    app = fakeEl()
    lockBodyScroll()
    expect(app.style.overflow).toBe('hidden')
    expect(html.style.overflow).toBe('')
    unlockBodyScroll()
    expect(app.style.overflow).toBe('')
  })

  it('restores on the element captured at lock time even if .app is rebuilt', () => {
    const originalApp = fakeEl()
    app = originalApp
    lockBodyScroll()

    app = fakeEl()
    unlockBodyScroll()
    expect(originalApp.style.overflow).toBe('')
    expect(app.style.overflow).toBe('')
  })

  it('nested locks keep the page locked until the last unlock', () => {
    lockBodyScroll()
    lockBodyScroll()
    unlockBodyScroll()
    expect(html.style.overflow).toBe('hidden')
    unlockBodyScroll()
    expect(html.style.overflow).toBe('')
  })

  it('extra unlocks never drive the count negative', () => {
    unlockBodyScroll()
    unlockBodyScroll()
    expect(html.style.overflow).toBe('')
    lockBodyScroll()
    expect(html.style.overflow).toBe('hidden')
    unlockBodyScroll()
    unlockBodyScroll()
    expect(html.style.overflow).toBe('')
  })
})
