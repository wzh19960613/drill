import { ref } from 'vue'
import type { BookDef, BookItemDef, PausedSession, Question } from '../types'
import type { PrintItem } from '../print'
import { randomToken, shuffleWithSeed } from '../rng'
import { isoToday } from '../format'

export interface BookSession {
  bookId?: string
  title: string
  seed: string
  date: string
  items: PrintItem[]
}

export const currentSession = ref<BookSession | null>(null)

export const askUseAsCurrent = ref<BookDef | null>(null)

export interface SubjectSplit {
  main: BookDef[]
  mixed: BookDef[]
}

export function splitBooksBySubject(
  books: BookDef[],
  subject: string,
  subjectOfItem: (it: BookItemDef) => string | undefined,
): SubjectSplit {
  if (!subject) return { main: books, mixed: [] }
  const main: BookDef[] = []
  const mixed: BookDef[] = []
  for (const b of books) {
    const subs = new Set(b.items.map(subjectOfItem).filter((s): s is string => !!s))
    if (!subs.has(subject)) continue
    ;(subs.size <= 1 ? main : mixed).push(b)
  }
  return { main, mixed }
}

export interface BuildOpts {
  shuffleQ: boolean
  shuffleO: boolean
  seed: string
  title?: string
}

function itemKey(it: { id: string; source?: string }): string {
  return `${it.source ?? ''}\u0000${it.id}`
}

export function buildSession(qs: Question[], opts: BuildOpts): BookSession {
  const indices = qs.map((_, i) => i)
  const order = opts.shuffleQ ? shuffleWithSeed(indices, opts.seed + '|q') : indices
  const items: PrintItem[] = order.map((qi, pos) => {
    const q = qs[qi]
    const optionOrder =
      opts.shuffleO && q.options.length > 0
        ? shuffleWithSeed(
            q.options.map((_, i) => i),
            opts.seed + '|o|' + q.id,
          )
        : null
    return { ...q, seq: pos + 1, optionOrder }
  })
  return {
    title: opts.title ?? (opts.seed ? `题本 ${opts.seed}` : '题本'),
    seed: opts.shuffleQ || opts.shuffleO ? opts.seed : '',
    date: isoToday(),
    items,
  }
}

export function defFromSession(s: BookSession): BookDef {
  return {
    id: s.bookId ?? randomToken(),
    name: s.title,
    seed: s.seed,
    date: s.date,
    createdAt: Date.now(),
    items: s.items.map((it) => ({ id: it.id, source: it.source, optionOrder: it.optionOrder })),
  }
}

export function sessionFromDef(def: BookDef, questions: Question[]): BookSession {
  const byId = new Map(questions.map((q) => [itemKey(q), q]))
  const items: PrintItem[] = []
  for (const d of def.items) {
    const q = byId.get(itemKey(d))
    if (q) items.push({ ...q, seq: items.length + 1, optionOrder: d.optionOrder })
  }
  return { bookId: def.id, title: def.name, seed: def.seed, date: def.date, items }
}

export function pausedFrom(
  s: BookSession,
  idx: number,
  sessionMs: number,
  results: Map<string, { id: number; correct: boolean; ms: number }>,
): PausedSession {
  return {
    title: s.title,
    seed: s.seed,
    date: s.date,
    bookId: s.bookId ?? null,
    idx,
    sessionMs: Math.round(sessionMs),
    items: s.items.map((it) => ({ id: it.id, source: it.source, optionOrder: it.optionOrder })),
    results: [...results.entries()].map(([qid, r]) => ({ qid, ...r })),
  }
}

export function sessionFromPaused(
  p: PausedSession,
  questions: Question[],
): { session: BookSession; idx: number } | null {
  const byId = new Map(questions.map((q) => [itemKey(q), q]))
  const items: PrintItem[] = []
  for (const d of p.items) {
    const q = byId.get(itemKey(d))
    if (q) items.push({ ...q, seq: items.length + 1, optionOrder: d.optionOrder })
  }
  if (!items.length) return null
  const session: BookSession = {
    bookId: p.bookId ?? undefined,
    title: p.title,
    seed: p.seed,
    date: p.date,
    items,
  }
  const curItem = p.items[p.idx]
  let idx = curItem ? items.findIndex((it) => itemKey(it) === itemKey(curItem)) : -1
  if (idx < 0) {
    const answered = new Set(p.results.map((r) => r.qid))
    idx = items.findIndex((it) => !answered.has(it.id))
    if (idx < 0) idx = 0
  }
  return { session, idx }
}
