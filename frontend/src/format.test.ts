import { describe, expect, it } from 'vitest'
import { dateZh, fmtDur, fmtTime, isoToday } from './format'

describe('fmtDur', () => {
  it('formats zero and sub-second values as 0:00', () => {
    expect(fmtDur(0)).toBe('0:00')
    expect(fmtDur(999)).toBe('0:00')
  })

  it('clamps negative values to 0:00', () => {
    expect(fmtDur(-1)).toBe('0:00')
    expect(fmtDur(-95_000)).toBe('0:00')
  })

  it('formats minutes and seconds', () => {
    expect(fmtDur(59_000)).toBe('0:59')
    expect(fmtDur(60_000)).toBe('1:00')
    expect(fmtDur(95_000)).toBe('1:35')
  })

  it('switches to h:mm:ss once hours are involved', () => {
    expect(fmtDur(3_600_000)).toBe('1:00:00')
    expect(fmtDur(3_723_000)).toBe('1:02:03')
  })
})

describe('fmtTime', () => {
  const now = new Date()
  const at = (dayOffset: number, h = 10, m = 30) =>
    new Date(now.getFullYear(), now.getMonth(), now.getDate() + dayOffset, h, m).getTime()
  const hm = (d: Date) =>
    `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`

  it('returns a dash for null', () => {
    expect(fmtTime(null)).toBe('—')
    expect(fmtTime(0)).toBe('—')
  })

  it('uses relative wording for the last three days', () => {
    expect(fmtTime(at(0, 8, 5))).toBe(`今天 08:05`)
    expect(fmtTime(at(-1))).toBe(`昨天 ${hm(new Date(at(-1)))}`)
    expect(fmtTime(at(-2))).toBe(`前天 ${hm(new Date(at(-2)))}`)
  })

  it('uses month-day within the current year', () => {
    const t = at(-10)
    const d = new Date(t)
    const p = (n: number) => String(n).padStart(2, '0')
    expect(fmtTime(t)).toBe(`${p(d.getMonth() + 1)}-${p(d.getDate())} ${hm(d)}`)
  })

  it('rounds to months beyond 30 days and years beyond 365', () => {
    expect(fmtTime(at(-40))).toContain('个月前')
    expect(fmtTime(at(-400))).toContain('年前')
  })
})

describe('isoToday', () => {
  it('yields a zero-padded local ISO date matching the local calendar day', () => {
    expect(isoToday()).toMatch(/^\d{4}-\d{2}-\d{2}$/)
    const d = new Date()
    const p = (n: number) => String(n).padStart(2, '0')
    expect(isoToday()).toBe(`${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`)
  })
})

describe('dateZh', () => {
  it('renders ISO dates without leading zeros', () => {
    expect(dateZh('2026-09-05')).toBe('2026/9/5')
    expect(dateZh('2026-11-01')).toBe('2026/11/1')
  })

  it('passes non-ISO values (legacy stored formats) through unchanged', () => {
    expect(dateZh('2026/9/5')).toBe('2026/9/5')
    expect(dateZh('某天')).toBe('某天')
  })
})
