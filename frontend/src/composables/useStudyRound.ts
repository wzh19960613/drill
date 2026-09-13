import { computed, ref, type ComputedRef, type Ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { addRecord, updateRecord } from '../store'
import { clearPaused, savePaused } from '../api'
import { currentSession, pausedFrom, type BookSession } from '../book'
import { fmtDur } from '../format'
import { useStudyTimer } from './useStudyTimer'
import { activeBookSession, quickSession, resumePaused } from './roundSources'

export type RoundRecord = { id: number; correct: boolean; ms: number }
export type Toast = (msg: string) => void

export interface RoundCore {
  route: ReturnType<typeof useRoute>
  router: ReturnType<typeof useRouter>
  review: ComputedRef<boolean>
  session: Ref<BookSession | null>
  idx: Ref<number>
  showAnswer: Ref<boolean>
  done: Ref<boolean>
  interrupted: Ref<boolean>
  roundRecords: Ref<Map<string, RoundRecord>>
  revisitAnswered: Ref<boolean>
  sessionMs: Ref<number>
  questionMs: Ref<number>
  timerPaused: Ref<boolean>
  startClock: () => void
  resetQuestion: () => void
  resetAll: () => void
  item: ComputedRef<ReturnType<typeof currentItem> | null>
  total: ComputedRef<number>
  right: ComputedRef<number>
  wrong: ComputedRef<number>
  flags: { marking: boolean; buildSeq: number; exiting: boolean }
}

type CurrentItem = BookSession['items'][number]

function countVerdicts(records: Map<string, RoundRecord>): { right: number; wrong: number } {
  let right = 0
  let wrong = 0
  for (const v of records.values()) v.correct ? right++ : wrong++
  return { right, wrong }
}

function currentItem(core: RoundCore): CurrentItem | null {
  return core.session.value?.items[core.idx.value] ?? null
}

function roundCoreState() {
  const route = useRoute()
  const router = useRouter()
  const review = computed(() => route.query.mode === 'review')
  const session = ref<BookSession | null>(null)
  const idx = ref(0)
  const showAnswer = ref(false)
  const done = ref(false)
  const interrupted = ref(false)
  const roundRecords = ref(new Map<string, RoundRecord>())
  const revisitAnswered = ref(false)
  const timer = useStudyTimer(
    () => !review.value && !showAnswer.value && !revisitAnswered.value && !done.value,
  )
  return { route, router, review, session, idx, showAnswer, done, interrupted, roundRecords, revisitAnswered, timer }
}

function useRoundCore(): RoundCore {
  const st = roundCoreState()
  const item = computed(() => currentItem(core))
  const total = computed(() => st.session.value?.items.length ?? 0)
  const counts = computed(() => countVerdicts(st.roundRecords.value))
  const core: RoundCore = {
    ...st,
    sessionMs: st.timer.sessionMs,
    questionMs: st.timer.questionMs,
    timerPaused: st.timer.paused,
    startClock: st.timer.start,
    resetQuestion: st.timer.resetQuestion,
    resetAll: st.timer.resetAll,
    item,
    total,
    right: computed(() => counts.value.right),
    wrong: computed(() => counts.value.wrong),
    flags: { marking: false, buildSeq: 0, exiting: false },
  }
  return core
}

function useRoundNav(core: RoundCore) {
  let advanceTimer: ReturnType<typeof setTimeout> | undefined

  function cancelAdvance() {
    clearTimeout(advanceTimer)
    advanceTimer = undefined
  }

  function enterQuestion() {
    const r = core.item.value ? core.roundRecords.value.get(core.item.value.id) : undefined
    core.revisitAnswered.value = !!r
    core.showAnswer.value = !!r
    core.questionMs.value = r ? r.ms : 0
  }

  function next() {
    if (!core.session.value) return
    cancelAdvance()
    if (core.idx.value < core.session.value.items.length - 1) {
      core.idx.value++
      enterQuestion()
    } else {
      core.done.value = true
    }
  }

  function prev() {
    if (core.idx.value > 0) {
      cancelAdvance()
      core.idx.value--
      enterQuestion()
    }
  }

  function scheduleAdvance() {
    advanceTimer = setTimeout(next, 450)
  }

  return { cancelAdvance, enterQuestion, next, prev, scheduleAdvance }
}

function useRoundMarking(core: RoundCore, nav: ReturnType<typeof useRoundNav>, toast: Toast) {
  async function mark(correct: boolean, advance = true) {
    const q = core.item.value
    if (!q || core.done.value || core.flags.marking) return
    core.flags.marking = true
    nav.cancelAdvance()
    try {
      const ms = core.review.value ? undefined : Math.round(core.questionMs.value)
      const prevRecord = core.roundRecords.value.get(q.id)
      if (prevRecord) await remark(prevRecord, correct, ms)
      else await recordFresh(q.id, correct, ms)
      if (advance) nav.scheduleAdvance()
    } finally {
      core.flags.marking = false
    }
  }

  async function remark(prevRecord: RoundRecord, correct: boolean, ms?: number) {
    await updateRecord(prevRecord.id, correct, ms)
    prevRecord.correct = correct
    prevRecord.ms = ms ?? 0
    toast(`已改为「${correct ? '做对了' : '做错了'}」`)
  }

  async function recordFresh(qid: string, correct: boolean, ms?: number) {
    const rec = await addRecord(core.item.value!, correct, ms)
    core.roundRecords.value.set(qid, { id: rec.id, correct, ms: ms ?? 0 })
    const label = correct ? '做对了' : '做错了'
    toast(core.review.value ? `已记「${label}」` : `已记「${label}」（本题 ${fmtDur(core.questionMs.value)}）`)
  }

  return { mark }
}

function useRoundBuild(core: RoundCore, nav: ReturnType<typeof useRoundNav>, toast: Toast) {
  function resetRound() {
    nav.cancelAdvance()
    core.done.value = false
    core.interrupted.value = false
    core.idx.value = 0
    core.showAnswer.value = false
    resetCounters()
  }

  function resetCounters() {
    core.roundRecords.value = new Map()
    core.revisitAnswered.value = false
  }

  async function build() {
    const seq = ++core.flags.buildSeq
    resetRound()
    if (core.route.query.resume && (await resumePaused(core, nav, toast))) return
    if (seq !== core.flags.buildSeq) return
    if (core.route.path === '/book/study' && currentSession.value) {
      core.session.value = currentSession.value
    } else {
      const s = await activeBookSession()
      if (seq !== core.flags.buildSeq) return
      core.session.value = s ?? quickSession()
    }
    nav.enterQuestion()
  }

  return { build }
}

function useRoundRetry(core: RoundCore, nav: ReturnType<typeof useRoundNav>, toast: Toast) {
  function retryAll() {
    core.idx.value = 0
    core.done.value = false
    core.roundRecords.value = new Map()
    core.revisitAnswered.value = false
    core.resetAll()
    nav.enterQuestion()
  }

  function retryWrongOnly() {
    if (!core.session.value) return
    if (!core.wrong.value) {
      toast('本轮没有做错的题')
      core.done.value = false
      return
    }
    core.session.value = {
      ...core.session.value,
      items: core.session.value.items.filter((it) => core.roundRecords.value.get(it.id)?.correct === false),
    }
    core.idx.value = 0
    core.done.value = false
    core.roundRecords.value = new Map()
    core.revisitAnswered.value = false
    nav.enterQuestion()
  }

  return { retryAll, retryWrongOnly }
}

function useRoundEnding(core: RoundCore, nav: ReturnType<typeof useRoundNav>) {
  async function leaveWithSummary() {
    if (!core.review.value && !core.done.value && core.roundRecords.value.size >= 1 && core.item.value) {
      if (core.flags.exiting) return
      core.flags.exiting = true
      try {
        await savePaused(pausedFrom(core.session.value!, core.idx.value, core.sessionMs.value, core.roundRecords.value)).catch(() => {})
      } finally {
        core.flags.exiting = false
      }
      core.interrupted.value = true
      core.done.value = true
      return
    }
    core.router.push('/')
  }

  async function continueInterrupted() {
    await clearPaused().catch(() => {})
    core.interrupted.value = false
    core.done.value = false
    nav.enterQuestion()
  }

  function leave() {
    if (core.done.value) {
      core.router.push('/')
      return
    }
    void leaveWithSummary()
  }

  return { leaveWithSummary, continueInterrupted, leave }
}

export function useStudyRound(toast: Toast) {
  const core = useRoundCore()
  const nav = useRoundNav(core)
  const { mark } = useRoundMarking(core, nav, toast)
  const { build } = useRoundBuild(core, nav, toast)
  const { retryAll, retryWrongOnly } = useRoundRetry(core, nav, toast)
  const { leaveWithSummary, continueInterrupted, leave } = useRoundEnding(core, nav)

  function toggleTimer() {
    if (core.showAnswer.value) {
      toast('显示答案时计时自动暂停')
      return
    }
    core.timerPaused.value = !core.timerPaused.value
    toast(core.timerPaused.value ? '计时已暂停' : '计时已恢复')
  }

  function resetTimer() {
    core.resetQuestion()
    toast('本题计时已重置')
  }

  const marked = computed(() => {
    const r = core.item.value ? core.roundRecords.value.get(core.item.value.id) : undefined
    return r ? r.correct : null
  })
  const unanswered = computed(() => core.total.value - core.roundRecords.value.size)
  return {
    review: core.review, session: core.session, idx: core.idx, showAnswer: core.showAnswer,
    done: core.done, interrupted: core.interrupted, roundRecords: core.roundRecords,
    revisitAnswered: core.revisitAnswered, sessionMs: core.sessionMs,
    questionMs: core.questionMs, timerPaused: core.timerPaused,
    item: core.item, total: core.total, marked, right: core.right, wrong: core.wrong,
    unanswered, build, startClock: core.startClock, next: nav.next, prev: nav.prev,
    mark, toggleTimer, resetTimer, retryAll, retryWrongOnly, continueInterrupted,
    leave, leaveWithSummary,
  }
}
