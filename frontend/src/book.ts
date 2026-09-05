import { ref } from 'vue'
import type { BookDef, PausedSession, Paper, Question } from './types'
import type { PrintItem, PrintPayload } from './print'
import { randomToken, shuffleWithSeed } from './rng'
import { isoToday } from './format'
import {
  deleteBookApi,
  fetchBooks,
  fetchQuestionRaw,
  getActiveBookApi,
  saveBookApi,
  setActiveBookApi,
} from './api'

export interface BookSession {
  bookId?: string
  title: string
  seed: string
  date: string
  items: PrintItem[]
}

export const currentSession = ref<BookSession | null>(null)

export const askUseAsCurrent = ref<BookDef | null>(null)

export {
  fetchBooks,
  saveBookApi as upsertBook,
  deleteBookApi as deleteBook,
  getActiveBookApi as getActiveBookId,
  setActiveBookApi as setActiveBookId,
}

export interface BuildOpts {
  shuffleQ: boolean
  shuffleO: boolean
  seed: string
  title?: string
}

/** Cross-source item key: question ids are only unique within a source */
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

export interface LineSpec {
  style: 'none' | 'dashed' | 'dotted' | 'solid' | 'double' | 'thickThin'
  width: number
  gap: number
}

export const LINE_STYLES: { value: LineSpec['style']; label: string }[] = [
  { value: 'none', label: '无' },
  { value: 'dashed', label: '虚线' },
  { value: 'dotted', label: '点线' },
  { value: 'solid', label: '实线' },
  { value: 'double', label: '双实线' },
  { value: 'thickThin', label: '粗细线' },
]

export interface ExportOpts {
  paper: Paper
  perPage: number

  fontScale: number
  optsPerRow: 1 | 2 | 4
  bindingExtra: number
  marginT: number
  marginB: number
  marginL: number
  marginR: number
  sepQ: LineSpec
  sepHead: LineSpec
  sepFoot: LineSpec
  showMeta: boolean
  bindingLong: boolean
  firstPageLeft: boolean
  workbookMeta: string
  answersMeta: string
  headerBinding: string
  headerCenter: string
  headerFlip: string
  footerBinding: string
  footerCenter: string
  footerFlip: string
  customStyle: string
}

export function payloadFor(
  session: BookSession,
  doc: 'workbook' | 'answers',
  o: ExportOpts,
): PrintPayload {
  return {
    doc,
    items: session.items,
    paper: o.paper,
    perPage: o.perPage,
    date: session.date,
    title: session.title,
    bookId: session.bookId ?? null,
    meta: { workbook: o.workbookMeta, answers: o.answersMeta },
    sepQ: o.sepQ,
    sepHead: o.sepHead,
    sepFoot: o.sepFoot,
    showMeta: o.showMeta,
    bindingExtra: o.bindingExtra,
    marginT: o.marginT,
    marginB: o.marginB,
    marginL: o.marginL,
    marginR: o.marginR,
    fontScale: o.fontScale,
    optsPerRow: o.optsPerRow,
    bindingLong: o.bindingLong,
    firstPageLeft: o.firstPageLeft,
    header: { binding: o.headerBinding, center: o.headerCenter, flip: o.headerFlip },
    footer: { binding: o.footerBinding, center: o.footerCenter, flip: o.footerFlip },
    customStyle: o.customStyle,
  }
}

function safeName(s: string): string {
  return s.replace(/[\\/:*?"<>|\s]+/g, '-')
}

/** Anchor-click download of a blob; revokes the object URL after a delay */
function triggerDownload(blob: Blob, filename: string, revokeDelay = 2000) {
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  a.click()
  setTimeout(() => URL.revokeObjectURL(url), revokeDelay)
}

export function downloadText(filename: string, text: string) {
  const blob = new Blob([text], { type: 'text/markdown;charset=utf-8' })
  triggerDownload(blob, filename)
}

export type MdOrg = 'workbook' | 'answers' | 'both' | 'per'

export interface MdExportOpts {
  org: MdOrg
  meta: boolean
  answers: boolean
  answersWithQuestion: boolean
}

function splitRaw(md: string): { meta: string; stem: string; answers: string } {
  let meta = ''
  let rest = md.replace(/^\s+/, '')
  const lines = rest.split('\n')
  if (lines[0]?.startsWith('>')) {
    meta = lines[0]
    rest = lines.slice(1).join('\n').replace(/^\s+/, '')
  }
  const idx = rest.search(/^##\s*答案\s*$/m)
  if (idx >= 0) {
    return { meta, stem: rest.slice(0, idx).trimEnd(), answers: rest.slice(idx).trim() }
  }
  return { meta, stem: rest.trimEnd(), answers: '' }
}

function imageRefs(md: string): string[] {
  return [...md.matchAll(/!?\[\[([^\]]+\.svg)\]\]/g)].map((m) => m[1])
}

function keptText(md: string, o: { meta: boolean; answers: boolean }): string {
  const { meta, stem, answers } = splitRaw(md)
  const head = o.meta && meta ? meta + '\n\n' : ''
  const tail = o.answers && answers ? '\n\n' + answers : ''
  return head + stem + tail
}

async function fetchAsset(name: string): Promise<Blob | null> {
  try {
    const r = await fetch('/api/assets/' + encodeURIComponent(name))
    return r.ok ? await r.blob() : null
  } catch {
    return null
  }
}

export async function exportMarkdown(session: BookSession, o: MdExportOpts): Promise<string> {
  // Fetch all raws concurrently; a deleted question becomes a placeholder
  // instead of failing the whole export
  const raws = await Promise.allSettled(
    session.items.map((it) =>
      fetchQuestionRaw(it.id, it.source).then((r) => r.markdown.replace(/\r\n/g, '\n')),
    ),
  )
  const mdOf = (i: number): string | null =>
    raws[i].status === 'fulfilled' ? raws[i].value : null

  const files: { name: string; text: string }[] = []
  const addDoc = (doc: 'workbook' | 'answers') => {
    const body = session.items
      .map((it, i) => {
        const n = i + 1
        const md = mdOf(i)
        if (md === null) return `${n}.\n\n<!-- 题目已删除：${it.id} -->`
        if (doc === 'workbook') {
          return `${n}.\n\n${keptText(md, { meta: o.meta, answers: false })}`
        }
        const parts = splitRaw(md)
        const meta = o.meta && parts.meta ? parts.meta + '\n\n' : ''
        const stem = o.answersWithQuestion ? parts.stem + '\n\n' : ''
        return `${n}.\n\n${meta}${stem}${parts.answers}`
      })
      .join('\n\n')
    files.push({ name: `${safeName(session.title)}-${doc === 'workbook' ? '题目本' : '答案本'}.md`, text: body + '\n' })
  }

  if (o.org === 'per') {
    session.items.forEach((it, i) => {
      const md = mdOf(i)
      files.push({
        name: `${i + 1}.md`,
        text: (md === null ? `<!-- 题目已删除：${it.id} -->` : keptText(md, o)) + '\n',
      })
    })
  } else if (o.org === 'both') {
    addDoc('workbook')
    addDoc('answers')
  } else {
    addDoc(o.org)
  }

  const names = [
    ...new Set(
      session.items.flatMap((_, i) => {
        const md = mdOf(i)
        return md === null ? [] : imageRefs(md)
      }),
    ),
  ]
  const blobs = new Map<string, Blob>()
  for (const name of names) {
    const b = await fetchAsset(name)
    if (b) blobs.set(name, b)
  }

  if (files.length === 1 && blobs.size === 0) {
    downloadText(files[0].name, files[0].text)
    return `已下载 ${files[0].name}`
  }

  const JSZip = (await import('jszip')).default
  const zip = new JSZip()
  const folderName = safeName(session.title)
  const folder = zip.folder(folderName)!
  for (const f of files) folder.file(f.name, f.text)
  for (const [name, b] of blobs) folder.file(name, b)
  const blob = await zip.generateAsync({ type: 'blob' })
  triggerDownload(blob, `${folderName}.zip`, 10_000)
  const imgs = blobs.size ? `，含 ${blobs.size} 张图片` : ''
  return `已下载 ${folderName}.zip（${files.length} 个文件${imgs}）`
}
