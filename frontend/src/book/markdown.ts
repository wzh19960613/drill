import type { BookSession } from './session'
import { fetchQuestionRaw } from '../api'

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

type MdOfFile = (i: number) => string | null

/** One exported document: numbered items, per the workbook/answers shape. */
function docText(session: BookSession, doc: 'workbook' | 'answers', o: MdExportOpts, mdOf: MdOfFile): string {
  return session.items
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
}

function collectFiles(session: BookSession, o: MdExportOpts, mdOf: MdOfFile) {
  const files: { name: string; text: string }[] = []
  const addDoc = (doc: 'workbook' | 'answers') => {
    const name = `${safeName(session.title)}-${doc === 'workbook' ? '题目本' : '答案本'}.md`
    files.push({ name, text: docText(session, doc, o, mdOf) + '\n' })
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
  return files
}

async function collectImages(session: BookSession, mdOf: MdOfFile): Promise<Map<string, Blob>> {
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
  return blobs
}

export async function exportMarkdown(session: BookSession, o: MdExportOpts): Promise<string> {
  // Fetch all raws concurrently; a deleted question becomes a placeholder
  // instead of failing the whole export
  const raws = await Promise.allSettled(
    session.items.map((it) =>
      fetchQuestionRaw(it.id, it.source).then((r) => r.markdown.replace(/\r\n/g, '\n')),
    ),
  )
  const mdOf: MdOfFile = (i) => (raws[i].status === 'fulfilled' ? raws[i].value : null)

  const files = collectFiles(session, o, mdOf)
  const blobs = await collectImages(session, mdOf)

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
