import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { clearPaused } from '../api'
import { currentSession, sessionFromDef, setActiveBookId } from '../book'
import { currentSubject, store } from '../store'
import type { BookDef, PausedSession, Question } from '../types'
import type { BookSession } from '../book'

type Toast = (msg: string) => void

export interface StartFlowContext {
  viewing: { value: BookDef | null }
  flash: Toast
  activeId: { value: string | null }
  paused: { value: PausedSession | null }
  closeViewing: () => void
}

function useResumeGuard(ctx: StartFlowContext, router: ReturnType<typeof useRouter>) {
  const resumeAsk = ref(false)
  let pendingStart: (() => void) | null = null

  function guardStart(action: () => void) {
    if (!ctx.paused.value) {
      action()
      return
    }
    pendingStart = action
    resumeAsk.value = true
  }

  function doResume() {
    resumeAsk.value = false
    ctx.paused.value = null
    router.push('/study?resume=1')
  }

  async function doDiscard() {
    resumeAsk.value = false
    const action = pendingStart
    pendingStart = null
    ctx.paused.value = null
    await clearPaused().catch(() => {})
    action?.()
  }

  function closeResumeAsk() {
    resumeAsk.value = false
    pendingStart = null
  }

  return { resumeAsk, guardStart, doResume, doDiscard, closeResumeAsk }
}

function useLauncher(ctx: StartFlowContext, router: ReturnType<typeof useRouter>, guardStart: (a: () => void) => void) {
  function launch(def: BookDef, review = false) {
    const session = sessionFromDef(def, store.questions)
    if (!session.items.length) {
      ctx.flash('该题本中的题目已不在题库里')
      return
    }
    const start = () => {
      currentSession.value = session
      router.push(review ? '/book/study?mode=review' : '/book/study')
    }
    if (review) start()
    else guardStart(start)
  }

  function studyFrom(q: Question) {
    const def = ctx.viewing.value
    if (!def) return
    const base = sessionFromDef(def, store.questions)
    const at = base.items.findIndex((it) => it.id === q.id)
    if (at < 0) return
    const session: BookSession = { ...base, title: `${def.name} · 从第${at + 1}题起`, items: base.items.slice(at) }
    ctx.closeViewing()
    guardStart(() => {
      currentSession.value = session
      router.push('/book/study')
    })
  }

  return { launch, studyFrom }
}

function useBookSwitching(
  ctx: StartFlowContext,
  launch: (def: BookDef, review?: boolean) => void,
) {
  const switchAsk = ref<{ def: BookDef; review: boolean } | null>(null)

  function startFromViewing(review = false) {
    const def = ctx.viewing.value
    if (!def) return
    if (def.id === ctx.activeId.value) {
      ctx.closeViewing()
      launch(def, review)
      return
    }
    if (!ctx.activeId.value) {
      void doSwitch(def, review)
      return
    }
    switchAsk.value = { def, review }
  }

  async function doSwitch(def: BookDef, review: boolean) {
    ctx.closeViewing()
    await setActiveBookId(currentSubject.value, def.id)
    ctx.activeId.value = def.id
    ctx.flash(`已将「${def.name}」设为当前题本`)
    launch(def, review)
  }

  async function confirmSwitch() {
    const a = switchAsk.value
    switchAsk.value = null
    if (!a) return
    await doSwitch(a.def, a.review)
  }

  function withoutSwitch() {
    const a = switchAsk.value
    switchAsk.value = null
    if (!a) return
    ctx.closeViewing()
    launch(a.def, a.review)
  }

  return { switchAsk, startFromViewing, confirmSwitch, withoutSwitch }
}

export function useBookStartFlow(ctx: StartFlowContext) {
  const router = useRouter()
  const guard = useResumeGuard(ctx, router)
  const { launch, studyFrom } = useLauncher(ctx, router, guard.guardStart)
  const switching = useBookSwitching(ctx, launch)

  const viewBook = (def: BookDef) => {
    ctx.viewing.value = def
  }

  return {
    resumeAsk: guard.resumeAsk,
    switchAsk: switching.switchAsk,
    viewBook,
    launch,
    startFromViewing: switching.startFromViewing,
    confirmSwitch: switching.confirmSwitch,
    withoutSwitch: switching.withoutSwitch,
    studyFrom,
    doResume: guard.doResume,
    doDiscard: guard.doDiscard,
    closeResumeAsk: guard.closeResumeAsk,
  }
}
