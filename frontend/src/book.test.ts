import { describe, expect, it } from 'vitest'
import {
  buildSession,
  defFromSession,
  pausedFrom,
  sessionFromDef,
  sessionFromPaused,
  type BookSession,
  type ExportOpts,
  payloadFor,
} from './book'
import type { Question } from './types'

function mkQuestion(id: string, source = 's1'): Question {
  return {
    id,
    source,
    subject: '数学',
    chapter: 'c',
    qtype: '选择题',
    stem: [`${id} 题干`],
    options: [
      { id: 1, text: '甲' },
      { id: 2, text: '乙' },
    ],
    correct_id: 1,
    correct_ids: [1],
    answer_line: '(A)',
    solution: [],
  }
}

const OPTS: BuildOptsLike = { shuffleQ: false, shuffleO: false, seed: '' }
type BuildOptsLike = { shuffleQ: boolean; shuffleO: boolean; seed: string }

describe('buildSession', () => {
  it('assigns sequential numbers and keeps order when not shuffling', () => {
    const s = buildSession([mkQuestion('A'), mkQuestion('B')], OPTS)
    expect(s.items.map((it) => it.seq)).toEqual([1, 2])
    expect(s.items.map((it) => it.id)).toEqual(['A', 'B'])
    expect(s.items[0].optionOrder).toBeNull()
  })

  it('is deterministic for the same seed and shuffles options per question', () => {
    const qs = Array.from({ length: 8 }, (_, i) => mkQuestion(`Q${i}`))
    const a = buildSession(qs, { shuffleQ: true, shuffleO: true, seed: 'S1' })
    const b = buildSession(qs, { shuffleQ: true, shuffleO: true, seed: 'S1' })
    expect(a.items.map((it) => it.id)).toEqual(b.items.map((it) => it.id))
    expect(a.items.map((it) => it.optionOrder)).toEqual(b.items.map((it) => it.optionOrder))
    // option order is a permutation of the option indices
    for (const it of a.items) expect([...it.optionOrder!].sort()).toEqual([0, 1])
  })

  it('clears the seed when nothing is shuffled', () => {
    const s = buildSession([mkQuestion('A')], { ...OPTS, seed: 'XYZ' })
    expect(s.seed).toBe('')
  })

  it('dates the session with a local-timezone ISO date', () => {
    const s = buildSession([mkQuestion('A')], OPTS)
    expect(s.date).toMatch(/^\d{4}-\d{2}-\d{2}$/)
  })
})

describe('sessionFromDef', () => {
  it('skips questions no longer in the bank and renumbers', () => {
    const def = defFromSession(buildSession([mkQuestion('A'), mkQuestion('B'), mkQuestion('C')], OPTS))
    const s = sessionFromDef(def, [mkQuestion('C'), mkQuestion('A')])
    expect(s.items.map((it) => it.id)).toEqual(['A', 'C'])
    expect(s.items.map((it) => it.seq)).toEqual([1, 2])
    expect(s.bookId).toBe(def.id)
  })

  it('keys items by source+id: the same id in two sources never crosses', () => {
    const def = defFromSession(
      buildSession([mkQuestion('A', 'src1'), mkQuestion('A', 'src2'), mkQuestion('B', 'src1')], OPTS),
    )
    // Bank has only the src2 variant of A: exactly that one is restored
    const s = sessionFromDef(def, [mkQuestion('A', 'src2'), mkQuestion('B', 'src1')])
    expect(s.items.map((it) => `${it.source}/${it.id}`)).toEqual(['src2/A', 'src1/B'])
  })

  it('defFromSession persists each item with its source', () => {
    const def = defFromSession(buildSession([mkQuestion('A', 'src1')], OPTS))
    expect(def.items).toEqual([{ id: 'A', source: 'src1', optionOrder: null }])
  })
})

describe('pausedFrom / sessionFromPaused', () => {
  const session: BookSession = buildSession([mkQuestion('A'), mkQuestion('B'), mkQuestion('C')], OPTS)
  const paused = pausedFrom(session, 1, 65_000, new Map([['A', { id: 9, correct: true, ms: 12_000 }]]))

  it('maps the round results into the snapshot', () => {
    expect(paused.idx).toBe(1)
    expect(paused.sessionMs).toBe(65_000)
    expect(paused.results).toEqual([{ qid: 'A', id: 9, correct: true, ms: 12_000 }])
    expect(paused.items.map((it) => it.id)).toEqual(['A', 'B', 'C'])
    expect(paused.items.map((it) => it.source)).toEqual(['s1', 's1', 's1'])
  })

  it('resumes at the interrupted question when it still exists', () => {
    const r = sessionFromPaused(paused, [mkQuestion('A'), mkQuestion('B'), mkQuestion('C')])!
    expect(r.idx).toBe(1)
    expect(r.session.items.map((it) => it.id)).toEqual(['A', 'B', 'C'])
  })

  it('drops deleted questions and their results', () => {
    const r = sessionFromPaused(paused, [mkQuestion('B'), mkQuestion('C')])!
    expect(r.session.items.map((it) => it.id)).toEqual(['B', 'C'])
    // The interrupted question B still exists → resume at it
    expect(r.idx).toBe(0)
  })

  it('resumes at the first unanswered question when the interrupted one was deleted', () => {
    // Interrupted at C (idx=2), C deleted → stop at the first unanswered
    // question (A already answered → B)
    const p2 = pausedFrom(session, 2, 0, new Map([['A', { id: 9, correct: true, ms: 1 }]]))
    const r = sessionFromPaused(p2, [mkQuestion('A'), mkQuestion('B')])!
    expect(r.idx).toBe(1)
  })

  it('returns null when every question is gone', () => {
    expect(sessionFromPaused(paused, [])).toBeNull()
  })

  it('falls back to index 0 when everything is answered and the interrupted question is gone', () => {
    const p3 = pausedFrom(session, 2, 0, new Map([['A', { id: 1, correct: true, ms: 1 }], ['B', { id: 2, correct: false, ms: 1 }], ['C', { id: 3, correct: true, ms: 1 }]]))
    // Interrupted question C no longer in the bank → first unanswered; all
    // answered → 0
    const r = sessionFromPaused(p3, [mkQuestion('A'), mkQuestion('B')])!
    expect(r.idx).toBe(0)
  })

  it('keys the resume index by source+id: same id in another source does not hijack it', () => {
    const mixed = buildSession([mkQuestion('A', 'src1'), mkQuestion('A', 'src2'), mkQuestion('B', 'src1')], OPTS)
    // Interrupted at the second item (src2/A); a bank containing both variants
    // must resume at src2/A, not at the first same-id match
    const p = pausedFrom(mixed, 1, 0, new Map())
    const r = sessionFromPaused(p, [mkQuestion('A', 'src1'), mkQuestion('A', 'src2'), mkQuestion('B', 'src1')])!
    expect(r.idx).toBe(1)
    expect(r.session.items[r.idx].source).toBe('src2')

    // Bank where only the src1 variant survives: src2/A is treated as gone;
    // nothing was answered, so the resume falls back to the first item (0)
    const r2 = sessionFromPaused(p, [mkQuestion('A', 'src1'), mkQuestion('B', 'src1')])!
    expect(r2.session.items.map((it) => `${it.source}/${it.id}`)).toEqual(['src1/A', 'src1/B'])
    expect(r2.idx).toBe(0)
  })
})

describe('payloadFor', () => {
  const opts = { paper: 'A4', perPage: 2 } as unknown as ExportOpts
  const s: BookSession = { title: '题本 1', seed: 'AB', date: '2026-09-04', items: [] }

  it('passes the stored ISO date through untouched', () => {
    expect(payloadFor(s, 'workbook', opts).date).toBe('2026-09-04')
  })

  it('keeps unparseable dates as-is (legacy stored formats)', () => {
    expect(payloadFor({ ...s, date: '2026/9/4' }, 'workbook', opts).date).toBe('2026/9/4')
    expect(payloadFor({ ...s, date: '某天' }, 'workbook', opts).date).toBe('某天')
  })
})
