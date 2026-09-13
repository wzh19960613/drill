import { onBeforeUnmount, onMounted, reactive, type Ref } from 'vue'
import type { QCore } from '../types'
import { questionMarkdown } from '../qutil'
import { copyQuestionPart, copyText } from './copyText'

export interface CtxItem {
  label: string
  run: () => void
}

export interface UseQuestionContextMenuOptions {
  rootEl: Ref<HTMLElement | null>
  q: () => QCore
  answerVisible: () => boolean
}

interface CtxState {
  show: boolean
  x: number
  y: number
  items: CtxItem[]
}

function menuItemsOf(
  opts: UseQuestionContextMenuOptions,
  katexEl: HTMLElement | null,
  zone: HTMLElement | null,
): CtxItem[] {
  const items: CtxItem[] = []
  if (katexEl) {
    const ann = katexEl.querySelector('annotation[encoding="application/x-tex"]')
    const tex = ann?.textContent?.trim() ?? ''
    if (tex) items.push({ label: '复制 LaTeX', run: () => void copyText(tex) })
  }
  if (zone) {
    items.push({ label: '复制题目', run: () => void copyText(questionMarkdown(opts.q())) })
    if (opts.answerVisible()) {
      items.push({ label: '复制答案', run: () => void copyQuestionPart(opts.q(), 'answer') })
      items.push({ label: '复制所有', run: () => void copyQuestionPart(opts.q(), 'full') })
    }
  }
  return items
}

function onContextOf(
  opts: UseQuestionContextMenuOptions,
  ctx: CtxState,
  markTarget: (el: HTMLElement | null) => void,
  e: MouseEvent,
) {
  const target = e.target as HTMLElement
  const katexEl = target.closest('.katex, .q-math') as HTMLElement | null
  const zone = target.closest('.q-stem, .q-options, .q-answer') as HTMLElement | null
  if (!katexEl && !zone) return
  e.preventDefault()
  window.getSelection()?.removeAllRanges()
  const items = menuItemsOf(opts, katexEl, zone)
  if (!items.length) return
  markTarget(katexEl)
  ctx.items = items
  ctx.x = e.clientX
  ctx.y = e.clientY
  ctx.show = true
}

export function useQuestionContextMenu(opts: UseQuestionContextMenuOptions) {
  const ctx = reactive<CtxState>({ show: false, x: 0, y: 0, items: [] })

  let lastTarget: HTMLElement | null = null

  function markTarget(el: HTMLElement | null) {
    lastTarget?.classList.remove('ctx-target')
    lastTarget = el
    el?.classList.add('ctx-target')
  }

  function closeCtx() {
    ctx.show = false
    markTarget(null)
  }

  const onContext = (e: MouseEvent) => onContextOf(opts, ctx, markTarget, e)

  function onWinMouseDown(e: MouseEvent) {
    if (!(e.target as HTMLElement).closest('.mpop')) closeCtx()
  }

  function onWinKey(e: KeyboardEvent) {
    if (e.code === 'Escape') closeCtx()
  }

  onMounted(() => {
    opts.rootEl.value?.addEventListener('contextmenu', onContext)
    window.addEventListener('mousedown', onWinMouseDown)
    window.addEventListener('blur', closeCtx)
    window.addEventListener('keydown', onWinKey)
  })
  onBeforeUnmount(() => {
    opts.rootEl.value?.removeEventListener('contextmenu', onContext)
    window.removeEventListener('mousedown', onWinMouseDown)
    window.removeEventListener('blur', closeCtx)
    window.removeEventListener('keydown', onWinKey)
    closeCtx()
  })

  return { ctx, closeCtx }
}
