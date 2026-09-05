import type { QCore } from './types'
import { richTextSingle } from './math'

export const LETTERS = ['A', 'B', 'C', 'D', 'E', 'F']

export function letterOf(id: number): string {
  return LETTERS[id - 1] ?? String.fromCharCode(64 + id)
}

export interface DisplayOption {
  letter: string
  text: string
}

export function displayOptions(q: QCore, order: number[] | null | undefined): DisplayOption[] {
  if (order && order.length === q.options.length) {
    return order.map((oi, pos) => ({
      letter: LETTERS[pos] ?? '?',
      text: q.options[oi]?.text ?? '',
    }))
  }
  return q.options.map((o) => ({ letter: letterOf(o.id), text: o.text }))
}

export function answerIds(q: QCore): number[] {
  return q.correct_ids?.length ? q.correct_ids : q.correct_id != null ? [q.correct_id] : []
}

export function remapIds(q: QCore, order: number[] | null | undefined): number[] | null {
  const ids = answerIds(q)
  if (!order || !ids.length || order.length !== q.options.length) return null
  return ids
    .map((id) => {
      const origIdx = q.options.findIndex((o) => o.id === id)
      const pos = order.indexOf(origIdx)
      return pos >= 0 ? pos + 1 : id
    })
    .sort((a, b) => a - b)
}

/** Question excerpt (LaTeX rendered, overflow truncated by CSS). */
export function stemExcerptHtml(q: QCore): string {
  const paras = q.stem.filter((p) => !/^!\[\[/.test(p.trim())).slice(0, 3)
  return richTextSingle(paras.join(' '))
}

export function questionMarkdown(q: QCore): string {
  const parts = [...q.stem]
  if (q.options.length) {
    parts.push('', ...q.options.map((o) => `（${letterOf(o.id)}）${o.text}`))
  }
  return parts.join('\n\n')
}

export function answerMarkdown(q: QCore): string {
  const parts: string[] = []
  if (q.answer_line) parts.push(`**答案**：${q.answer_line}`)
  for (const block of [q.solution, q.notes]) {
    if (!block?.length) continue
    if (parts.length) parts.push('')
    parts.push(...block)
  }
  return parts.join('\n\n')
}

/** Invert the checked items among the shown list, keeping off-screen checks */
export function invertSelection(shown: { id: string }[], checkedIds: string[]): string[] {
  const shownIds = new Set(shown.map((q) => q.id))
  const checked = new Set(checkedIds)
  const out = shown.filter((q) => !checked.has(q.id)).map((q) => q.id)
  for (const id of checkedIds) {
    if (!shownIds.has(id) && !out.includes(id)) out.push(id)
  }
  return out
}
