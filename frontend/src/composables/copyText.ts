import { ClipboardCopy } from 'lucide-vue-next'
import { fetchQuestionRaw } from '../api'
import type { QCore } from '../types'
import { answerMarkdown, questionMarkdown } from '../qutil'
import type { MenuPopItem } from '../components/MenuPop.vue'

/** Shared "copy stem / answer / everything" menu entries (keys are CopyKind) */
export const COPY_MENU_ITEMS: MenuPopItem[] = [
  { key: 'stem', label: '复制题目', icon: ClipboardCopy },
  { key: 'answer', label: '复制答案' },
  { key: 'full', label: '复制所有' },
]

/** Clipboard write with an execCommand fallback (non-secure contexts) */
export async function copyText(t: string) {
  try {
    await navigator.clipboard.writeText(t)
  } catch {
    const ta = document.createElement('textarea')
    ta.value = t
    document.body.appendChild(ta)
    ta.select()
    document.execCommand('copy')
    ta.remove()
  }
}

export type CopyKind = 'stem' | 'answer' | 'full'

export async function copyQuestionPart(q: QCore, kind: CopyKind) {
  if (kind === 'stem') return copyText(questionMarkdown(q))
  if (kind === 'answer') return copyText(answerMarkdown(q))
  try {
    const r = await fetchQuestionRaw(q.id, q.source)
    return copyText(r.markdown)
  } catch {
    return copyText(`${questionMarkdown(q)}\n\n## 答案\n\n${answerMarkdown(q)}`)
  }
}
