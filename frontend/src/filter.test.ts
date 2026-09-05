import { beforeEach, describe, expect, it } from 'vitest'
import { applyFilter, condLabel, defaultFilter, type Cond } from './filter'
import { store } from './store'
import type { Question, Rec } from './types'

function mkQuestion(id: string, partial: Partial<Question> = {}): Question {
  return {
    id,
    source: 's1',
    subject: '数学',
    origin: '书A',
    chapter: '第1章',
    qtype: '选择题',
    stem: [],
    options: [],
    correct_id: null,
    correct_ids: [],
    answer_line: '',
    solution: [],
    ...partial,
  }
}

let seq = 0
function rec(questionId: string, correct: boolean, at: number, source = 's1'): Rec {
  return { id: ++seq, questionId, source, correct, at, ms: undefined }
}

beforeEach(() => {
  seq = 0
  store.questions = [
    mkQuestion('P1-1'),
    mkQuestion('P1-2', { subject: '物理', chapter: '第2章' }),
    mkQuestion('P1-3', { qtype: '填空题' }),
  ]
  store.masteries = ['s1:P1-3']
  store.records = [
    rec('P1-1', false, 100),
    rec('P1-1', false, 200),
    rec('P1-1', true, 300),
    rec('P1-2', true, 150),
  ]
})

describe('applyFilter conditions', () => {
  it('excludes mastered by default and includes them when asked', () => {
    const ids = applyFilter(store.questions, defaultFilter()).map((x) => x.id)
    expect(ids).toEqual(['P1-1', 'P1-2'])
    const withMastered = applyFilter(store.questions, { ...defaultFilter(), includeMastered: true })
    expect(withMastered).toHaveLength(3)
  })

  it('count conditions compare with the given operator', () => {
    const cond = (c: Partial<Cond>): Cond[] => [Object.assign({ id: 'c', kind: 'count' }, c) as Cond]
    const f = (c: Partial<Cond>) => applyFilter(store.questions, { ...defaultFilter(), conds: cond(c), includeMastered: true })
    expect(f({ what: 'attempts', op: '=', n: 0 }).map((x) => x.id)).toEqual(['P1-3'])
    expect(f({ what: 'attempts', op: '>=', n: 2 }).map((x) => x.id)).toEqual(['P1-1'])
    expect(f({ what: 'wrong', op: '>', n: 0 }).map((x) => x.id)).toEqual(['P1-1'])
    expect(f({ what: 'wrong', op: '<', n: 1 }).map((x) => x.id)).toEqual(['P1-2', 'P1-3'])
    expect(f({ what: 'attempts', op: '<=', n: 1 }).map((x) => x.id)).toEqual(['P1-2', 'P1-3'])
  })

  it('recentWrong defaults to 1/1 and honors n/m', () => {
    const f = (n?: number, m?: number) =>
      applyFilter(store.questions, {
        ...defaultFilter(),
        includeMastered: true,
        conds: [Object.assign({ id: 'c', kind: 'recentWrong' }, { n, m }) as Cond],
      })
    // P1-1's most recent attempt is correct → default condition misses
    expect(f().map((x) => x.id)).toEqual([])
    // ≥2 wrong within the last 3 → P1-1
    expect(f(3, 2).map((x) => x.id)).toEqual(['P1-1'])
    // never-attempted questions never satisfy recentWrong
    expect(f(5, 1).map((x) => x.id)).toEqual(['P1-1'])
  })

  it('multi-value conditions match any selected value; empty = unrestricted', () => {
    const f = (kind: Cond['kind'], values?: string[]) =>
      applyFilter(store.questions, {
        ...defaultFilter(),
        includeMastered: true,
        conds: [Object.assign({ id: 'c', kind }, { values }) as Cond],
      })
    expect(f('subject', ['物理']).map((x) => x.id)).toEqual(['P1-2'])
    expect(f('chapter', ['第1章', '第2章'])).toHaveLength(3)
    expect(f('qtype', []).map((x) => x.id)).toEqual(['P1-1', 'P1-2', 'P1-3'])
  })

  it('combines with all / any', () => {
    const a: Cond = { id: 'a', kind: 'subject', values: ['物理'] }
    const b: Cond = { id: 'b', kind: 'qtype', values: ['填空题'] }
    // No question is both physics and fill-in
    const all = applyFilter(store.questions, { ...defaultFilter(), includeMastered: true, conds: [a, b], match: 'all' })
    expect(all).toEqual([])
    // any: physics P1-2 or fill-in P1-3
    const any = applyFilter(store.questions, { ...defaultFilter(), includeMastered: true, conds: [a, b], match: 'any' })
    expect(any.map((x) => x.id)).toEqual(['P1-2', 'P1-3'])
  })

  it('isolates stats across sources: the same id in two sources counts separately', () => {
    const bank = [
      mkQuestion('P1-1', { source: 's1' }),
      mkQuestion('P1-1', { source: 's2' }),
    ]
    store.questions = bank
    store.masteries = []
    // wrong records only under s1/P1-1
    store.records = [rec('P1-1', false, 100), rec('P1-1', false, 200)]
    const f = applyFilter(bank, {
      ...defaultFilter(),
      conds: [{ id: 'c', kind: 'count', what: 'wrong', op: '=', n: 2 }],
    })
    expect(f).toHaveLength(1)
    expect(f[0].source).toBe('s1')
    // the s2 twin stays untouched (0 wrong)
    const none = applyFilter(bank, {
      ...defaultFilter(),
      conds: [{ id: 'c', kind: 'count', what: 'wrong', op: '=', n: 0 }],
    })
    expect(none.map((x) => x.source)).toEqual(['s2'])
  })
})

describe('applyFilter sorting', () => {
  it('sorts ids numerically (P9-9 before P10-1)', () => {
    store.questions = [mkQuestion('P10-1'), mkQuestion('P9-9')]
    const ids = applyFilter(store.questions, defaultFilter()).map((x) => x.id)
    expect(ids).toEqual(['P9-9', 'P10-1'])
  })

  it('wrong/attempts descending with id tiebreak', () => {
    store.questions = [mkQuestion('A'), mkQuestion('B')]
    store.records = [rec('A', true, 1), rec('B', false, 2), rec('B', false, 3)]
    expect(applyFilter(store.questions, { ...defaultFilter(), sort: 'wrong' }).map((x) => x.id)).toEqual(['B', 'A'])
    expect(applyFilter(store.questions, { ...defaultFilter(), sort: 'attempts' }).map((x) => x.id)).toEqual(['B', 'A'])
  })

  it('lastAt descending, never-done last', () => {
    store.questions = [mkQuestion('A'), mkQuestion('B'), mkQuestion('C')]
    store.records = [rec('C', true, 10), rec('A', true, 20)]
    expect(applyFilter(store.questions, { ...defaultFilter(), sort: 'lastAt' }).map((x) => x.id)).toEqual(['A', 'C', 'B'])
  })

  it('random sort yields a permutation of the input', () => {
    const qs = Array.from({ length: 12 }, (_, i) => mkQuestion(`Q${i}`))
    const out = applyFilter(qs, { ...defaultFilter(), sort: 'random' })
    expect([...out.map((x) => x.id)].sort()).toEqual([...qs.map((x) => x.id)].sort())
    expect(out).toHaveLength(qs.length)
  })
})

describe('condLabel', () => {
  it('renders every kind without crashing', () => {
    for (const kind of ['subject', 'origin', 'chapter', 'qtype', 'count', 'recentWrong'] as const) {
      expect(typeof condLabel({ id: 'x', kind })).toBe('string')
    }
    expect(condLabel({ id: 'x', kind: 'count', what: 'wrong', op: '>=', n: 5 })).toContain('做错')
    expect(condLabel({ id: 'x', kind: 'recentWrong', n: 5, m: 1 })).toContain('5')
  })
})
