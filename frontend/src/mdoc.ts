export interface DocMeta {
  subject: string
  origin: string
  chapter: string
  locate: string
  qtype: string
}

export interface DocSections {
  stem: string
  answer: string
  solution: string
  notes: string
}

export const EMPTY_META: DocMeta = {
  subject: '',
  origin: '',
  chapter: '',
  locate: '',
  qtype: '',
}

export function splitBlocks(md: string): string[] {
  const out: string[] = []
  let cur: string[] = []
  let inFence = false
  for (const line of md.split('\n')) {
    const t = line.trim()
    if (t.startsWith('```')) {
      inFence = !inFence
      cur.push(t)
    } else if (!t && !inFence) {
      if (cur.length) out.push(cur.join('\n'))
      cur = []
    } else {
      cur.push(t)
    }
  }
  if (cur.length) out.push(cur.join('\n'))
  return out
}

export function joinBlocks(blocks: string[]): string {
  return blocks.filter((b) => b.trim()).join('\n\n')
}

export function splitDoc(md: string): { meta: DocMeta; sections: DocSections } {
  const lines = md.split('\n')
  let i = lines.findIndex((l) => l.trim() !== '')
  if (i < 0) i = lines.length
  let header = ''
  let bodyStart = i
  if (i < lines.length && lines[i].trimStart().startsWith('>')) {
    header = lines[i].trim().replace(/^>+/, '').trim()
    bodyStart = i + 1
  }

  const find = (title: string) => lines.findIndex((l) => l.trim() === title)
  const ans = find('## 答案')
  const sol = find('## 解析')
  const note = find('## 备注')

  const section = (start: number | null, ends: (number | null)[]): string => {
    if (start == null || start < 0) return ''
    const end = Math.min(
      ...ends.filter((e): e is number => e != null && e > start).concat(lines.length),
    )
    return joinBlocks(splitBlocks(lines.slice(start + 1, end).join('\n')))
  }

  const bodyEnd = Math.max(
    [ans, sol, note].filter((v): v is number => v != null && v >= 0).reduce((a, b) => Math.min(a, b), lines.length),
    bodyStart,
  )
  const sections: DocSections = {
    stem: joinBlocks(splitBlocks(lines.slice(bodyStart, bodyEnd).join('\n'))),
    answer: section(ans, [sol, note]),
    solution: section(sol, [note]),
    notes: section(note, []),
  }
  return { meta: parseHeader(header), sections }
}

export function parseHeader(header: string): DocMeta {
  const h = header.trim()
  let subject = ''
  let rest = h
  if (h.startsWith('[')) {
    const close = h.indexOf(']')
    if (close >= 0) {
      subject = h.slice(1, close).trim()
      rest = h.slice(close + 1).trimStart()
    }
  }
  const seps = ['|', '@', '>']
  const fields = ['', '', '', '']
  let field = 0
  let tail = rest
  for (;;) {
    let pos = -1
    let sepIdx = -1
    for (let si = field; si < seps.length; si++) {
      const p = tail.indexOf(seps[si])
      if (p >= 0 && (pos < 0 || p < pos)) {
        pos = p
        sepIdx = si
      }
    }
    if (pos < 0) break
    fields[field] = tail.slice(0, pos).trim()
    field = sepIdx + 1
    tail = tail.slice(pos + 1)
    if (field >= fields.length) break
  }
  fields[Math.min(field, fields.length - 1)] = tail.trim()
  return { subject, origin: fields[0], chapter: fields[1], locate: fields[2], qtype: fields[3] }
}

export function buildHeaderLine(meta: DocMeta): string {
  const head = meta.subject.trim() ? `[${meta.subject.trim()}]` : ''
  const parts: string[] = []
  if (meta.origin.trim()) parts.push(meta.origin.trim())
  if (meta.chapter.trim()) parts.push(`| ${meta.chapter.trim()}`)
  if (meta.locate.trim()) parts.push(`@ ${meta.locate.trim()}`)
  if (meta.qtype.trim()) parts.push(`> ${meta.qtype.trim()}`)
  return [head, parts.join(' ')].filter(Boolean).join(' ').trim()
}

export function buildDoc(meta: DocMeta, sections: DocSections): string {
  const chunks: string[] = []
  const header = buildHeaderLine(meta)
  if (header) chunks.push(`> ${header}`)
  if (sections.stem.trim()) chunks.push(sections.stem.trim())
  if (sections.answer.trim()) chunks.push(`## 答案\n\n${sections.answer.trim()}`)
  if (sections.solution.trim()) chunks.push(`## 解析\n\n${sections.solution.trim()}`)
  if (sections.notes.trim()) chunks.push(`## 备注\n\n${sections.notes.trim()}`)
  return chunks.join('\n\n')
}

export function clampHeadings(text: string, caret: number): { text: string; caret: number } {
  let out = ''
  let pos = 0
  let shift = 0
  let inFence = false
  for (const line of text.split('\n')) {
    if (pos > 0) {
      out += '\n'
      pos += 1
    }
    const m = inFence ? null : /^(#{1,2})(\s[\s\S]*)?$/.exec(line)
    if (m) {
      out += '###' + (m[2] ?? '')
      if (caret > pos) shift += 3 - m[1].length
    } else {
      out += line
    }
    if (line.trim().startsWith('```')) inFence = !inFence
    pos += line.length
  }
  return { text: out, caret: caret + shift }
}

export function extractImageRefs(md: string): string[] {
  const out: string[] = []
  const seen = new Set<string>()
  for (const m of md.matchAll(/!\[\[([^\[\]]+?)\]\]/g)) {
    if (!seen.has(m[1])) {
      seen.add(m[1])
      out.push(m[1])
    }
  }
  return out
}

export function applyImageMapping(md: string, pairs: [string, string][]): string {
  const map = new Map<string, string>()
  for (const [from, to] of pairs) {
    if (from && from !== to) map.set(`![[${from}]]`, `![[${to}]]`)
  }
  if (!map.size) return md
  return md.replace(/!\[\[[^\[\]]+?\]\]/g, (ref) => map.get(ref) ?? ref)
}

export function pad2(n: number): string {
  return String(n).padStart(2, '0')
}

export function nextNumberedBase(
  base: string,
  usedFiles: string[],
): string {
  for (let n = 1; n < 10000; n++) {
    const name = `${base} ${pad2(n)}`
    if (!usedFiles.includes(`${name}.md`) && !usedFiles.includes(name)) return name
  }
  return `${base} 9999`
}

export function defaultFileName(meta: DocMeta, usedFiles: string[]): string {
  const locate = meta.locate.trim()
  if (locate) return locate
  const origin = meta.origin.trim()
  if (origin) return origin
  return nextNumberedBase('题目', usedFiles)
}

export function extOf(name: string): string {
  const i = name.lastIndexOf('.')
  return i >= 0 ? name.slice(i) : ''
}

export function autoImageNames(
  filename: string,
  temps: { id: string }[],
  usedFiles: string[],
): string[] {
  const names: string[] = []
  for (const _t of temps) {
    const ext = extOf(_t.id)
    for (let n = 1; n < 10000; n++) {
      const candidate = `${filename}-${pad2(n)}${ext}`
      if (!usedFiles.includes(candidate) && !names.includes(candidate)) {
        names.push(candidate)
        break
      }
    }
  }
  return names
}

export function sanitizeFileName(input: string): string | null {
  const name = input.trim().replace(/(?:\.md)+$/, '').trim()
  if (!name || name.startsWith('.') || name.includes('/') || name.includes('\\') || name.includes('\0')) {
    return null
  }
  return name
}

export function nextOptionLetter(md: string): string {
  let max = 0
  for (const m of md.matchAll(/(?:^|\n)\(([A-Z])\)\s/g)) {
    max = Math.max(max, m[1].charCodeAt(0) - 64)
  }
  return String.fromCharCode(64 + Math.min(max + 1, 26))
}

export function docDefects(sections: DocSections): { stemBlank: boolean; noAnswer: boolean } {
  return {
    stemBlank: !sections.stem.trim(),
    noAnswer: !sections.answer.trim() && !sections.solution.trim(),
  }
}
