import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useStudyTimer } from './useStudyTimer'

let hidden = false

beforeEach(() => {
  vi.useFakeTimers({ toFake: ['performance', 'setInterval', 'clearInterval'] })
  hidden = false
  vi.stubGlobal('window', { setInterval, clearInterval })
  vi.stubGlobal('document', {
    get hidden() {
      return hidden
    },
  })
})

afterEach(() => {
  vi.useRealTimers()
  vi.unstubAllGlobals()
})

function setup(canRun = () => true) {
  return useStudyTimer(canRun)
}

describe('useStudyTimer', () => {
  it('accumulates session and question time while running', () => {
    const t = setup()
    t.start()
    vi.advanceTimersByTime(1000)
    expect(t.sessionMs.value).toBe(1000)
    expect(t.questionMs.value).toBe(1000)
  })

  it('starting twice does not double-count (interval replaced)', () => {
    const t = setup()
    t.start()
    t.start()
    vi.advanceTimersByTime(1000)
    expect(t.sessionMs.value).toBe(1000)
  })

  it('stop then start keeps a single interval and resets the tick base', () => {
    const t = setup()
    t.start()
    vi.advanceTimersByTime(500)
    t.stop()
    vi.advanceTimersByTime(500)
    expect(t.sessionMs.value).toBe(500)
    t.start()
    vi.advanceTimersByTime(250)
    expect(t.sessionMs.value).toBe(750)
  })

  it('stop is idempotent', () => {
    const t = setup()
    t.start()
    t.stop()
    t.stop()
    vi.advanceTimersByTime(1000)
    expect(t.sessionMs.value).toBe(0)
  })

  it('does not accumulate while document.hidden', () => {
    const t = setup()
    t.start()
    hidden = true
    vi.advanceTimersByTime(1000)
    expect(t.sessionMs.value).toBe(0)
    expect(t.questionMs.value).toBe(0)
  })

  it('does not accumulate when canRun() is false or paused', () => {
    let can = true
    const t = setup(() => can)
    t.start()
    can = false
    vi.advanceTimersByTime(1000)
    expect(t.sessionMs.value).toBe(0)
    can = true
    t.paused.value = true
    vi.advanceTimersByTime(1000)
    expect(t.sessionMs.value).toBe(0)
  })

  it('resetQuestion/resetAll clear the counters', () => {
    const t = setup()
    t.start()
    vi.advanceTimersByTime(1000)
    t.resetQuestion()
    expect(t.questionMs.value).toBe(0)
    expect(t.sessionMs.value).toBe(1000)
    t.resetAll()
    expect(t.sessionMs.value).toBe(0)
  })
})
