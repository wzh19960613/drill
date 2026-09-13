import type { Question } from './types'
import {
  chapterOf,
  isMastered,
  questionKey,
  recentOf,
  statsOf,
  subjectOf,
  type QuestionLike,
} from './store'

export type Op = '>=' | '>' | '<=' | '<' | '='

export interface Cond {
  id: string
  kind: 'subject' | 'origin' | 'chapter' | 'qtype' | 'count' | 'recentWrong'
  what?: 'attempts' | 'wrong'
  op?: Op
  n?: number
  m?: number
  values?: string[]
}

export type SortKey = 'id' | 'random' | 'wrong' | 'attempts' | 'recentWrong' | 'lastAt'

export interface Filter {
  conds: Cond[]
  match: 'all' | 'any'
  sort: SortKey
  includeMastered?: boolean

  search?: string

  searchRegex?: boolean
}

export const SORT_OPTIONS: { value: SortKey; label: string }[] = [
  { value: 'random', label: '随机' },
  { value: 'id', label: '按定位' },
  { value: 'wrong', label: '错误多优先' },
  { value: 'attempts', label: '做题多优先' },
  { value: 'recentWrong', label: '最近做错优先' },
  { value: 'lastAt', label: '最近做过优先' },
]

export function defaultFilter(): Filter {
  return { conds: [], match: 'all', sort: 'id', includeMastered: false }
}

let condSeq = 0
export function newCond(partial: Omit<Cond, 'id'>): Cond {
  return { id: `c${Date.now().toString(36)}${condSeq++}`, ...partial }
}

const OP_LABELS: Record<Op, string> = { '>=': '⩾', '>': '>', '<=': '≤', '<': '<', '=': '=' }

export function opLabel(op: Op): string {
  return OP_LABELS[op] ?? '='
}

export function condLabel(c: Cond): string {
  switch (c.kind) {
    case 'subject':
      return `学科：${c.values?.length ? c.values.join('、') : '不限'}`
    case 'origin':
      if (!c.values?.length) return '来源：不限'
      return c.values.length === 1 ? `来源：${c.values[0]}` : `来源 ×${c.values.length}`
    case 'chapter':
      if (!c.values?.length) return '章节：不限'
      return c.values.length === 1 ? `章节：${c.values[0]}` : `章节 ×${c.values.length}`
    case 'qtype':
      return `题型：${c.values?.length ? c.values.join('、') : '不限'}`
    case 'count':
      return `${c.what === 'wrong' ? '做错' : '做过'} ${opLabel(c.op ?? '>=')} ${c.n ?? 0} 次`
    case 'recentWrong': {
      const n = c.n ?? 1
      const m = c.m ?? 1
      return n <= 1 && m <= 1 ? '最近一次做错' : `最近 ${n} 次内错 ⩾ ${m}`
    }
  }
}

function cmp(a: number, op: Op, b: number): boolean {
  switch (op) {
    case '>':
      return a > b
    case '<':
      return a < b
    case '<=':
      return a <= b
    case '=':
      return a === b
    default:
      return a >= b
  }
}

function evalCond(c: Cond, q: Question): boolean {
  const s = statsOf(q)
  switch (c.kind) {
    case 'subject':
      return !c.values?.length || c.values.includes(subjectOf(q))
    case 'origin':
      return !c.values?.length || c.values.includes(q.origin ?? '')
    case 'chapter':
      return !c.values?.length || c.values.includes(chapterOf(q))
    case 'qtype':
      return !c.values?.length || c.values.includes(q.qtype)
    case 'count': {
      const v = c.what === 'wrong' ? s.wrong : s.attempts
      return cmp(v, c.op ?? '>=', c.n ?? 0)
    }
    case 'recentWrong': {
      const recs = recentOf(q, Math.max(1, c.n ?? 1))
      return recs.length > 0 && recs.filter((r) => !r.correct).length >= Math.max(1, c.m ?? 1)
    }
  }
}

export function searchHaystack(q: Question): string {
  return [
    q.file ?? '',
    q.id,
    q.subject ?? '',
    q.origin ?? '',
    q.chapter,
    q.locate ?? '',
    q.qtype,
    ...q.stem,
    ...q.options.map((o) => o.text),
    ...(q.answer ?? []),
    ...q.solution,
    ...(q.notes ?? []),
  ].join('\n')
}

export function matchesSearch(q: Question, search: string, regex: boolean): boolean {
  const hay = searchHaystack(q)
  if (!regex) return hay.toLowerCase().includes(search.toLowerCase())
  try {
    return new RegExp(search, 'i').test(hay)
  } catch {
    return hay.toLowerCase().includes(search.toLowerCase())
  }
}

function searchMatcher(f: Filter): (q: Question) => boolean {
  const needle = f.search?.trim() ?? ''
  const needleLower = needle.toLowerCase()
  const re = (() => {
    if (!needle || !f.searchRegex) return null
    try {
      return new RegExp(needle, 'i')
    } catch {
      return null
    }
  })()
  return (q) => {
    if (!needle) return true
    const hay = searchHaystack(q)
    return re ? re.test(hay) : hay.toLowerCase().includes(needleLower)
  }
}

function passes(q: Question, f: Filter, match: (q: Question) => boolean): boolean {
  if (!match(q)) return false
  if (!f.includeMastered && isMastered(q)) return false
  if (!f.conds.length) return true
  return f.match === 'all' ? f.conds.every((c) => evalCond(c, q)) : f.conds.some((c) => evalCond(c, q))
}

const byId = (a: Question, b: Question) => a.id.localeCompare(b.id, undefined, { numeric: true })

function shuffle<T>(arr: T[]): T[] {
  for (let i = arr.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1))
    ;[arr[i], arr[j]] = [arr[j], arr[i]]
  }
  return arr
}

function sortedBy(list: Question[], sort: Filter['sort']): Question[] {
  switch (sort) {
    case 'random':
      return shuffle([...list])
    case 'wrong':
      return [...list].sort((a, b) => statsOf(b).wrong - statsOf(a).wrong || byId(a, b))
    case 'attempts':
      return [...list].sort((a, b) => statsOf(b).attempts - statsOf(a).attempts || byId(a, b))
    case 'recentWrong': {
      const lastWrong = new Map(list.map((q) => [questionKey(q), lastWrongAt(q)]))
      return [...list].sort(
        (a, b) =>
          (lastWrong.get(questionKey(b)) ?? 0) - (lastWrong.get(questionKey(a)) ?? 0) || byId(a, b),
      )
    }
    case 'lastAt':
      return [...list].sort(
        (a, b) => (statsOf(b).last_at ?? -1) - (statsOf(a).last_at ?? -1) || byId(a, b),
      )
    default:
      return [...list].sort(byId)
  }
}

export function applyFilter(qs: Question[], f: Filter): Question[] {
  const match = searchMatcher(f)
  return sortedBy(qs.filter((q) => passes(q, f, match)), f.sort)
}

function lastWrongAt(q: QuestionLike): number {
  let t = 0
  for (const r of recentOf(q, Number.MAX_SAFE_INTEGER)) {
    if (!r.correct && r.at > t) t = r.at
  }
  return t
}

export const QUICK_PRESETS: { label: string; conds: () => Cond[] }[] = [
  { label: '全部', conds: () => [] },
  { label: '没做过', conds: () => [newCond({ kind: 'count', what: 'attempts', op: '=', n: 0 })] },
  { label: '做错⩾5', conds: () => [newCond({ kind: 'count', what: 'wrong', op: '>=', n: 5 })] },
  { label: '近 5 次内错过', conds: () => [newCond({ kind: 'recentWrong', n: 5, m: 1 })] },
]
