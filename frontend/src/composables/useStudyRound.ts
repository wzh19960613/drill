import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { addRecord, currentSubject, store, subjectQuestions, updateRecord } from '../store'
import { clearPaused, getPaused, savePaused } from '../api'
import {
  buildSession,
  currentSession,
  fetchBooks,
  getActiveBookId,
  pausedFrom,
  sessionFromDef,
  sessionFromPaused,
  type BookSession,
} from '../book'
import { fmtDur } from '../format'
import { useStudyTimer } from './useStudyTimer'

type RoundRecord = { id: number; correct: boolean; ms: number }
type Toast = (msg: string) => void

export function useStudyRound(toast: Toast) {
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

  // per-instance round state (module-level state was shared by every caller)
  let marking = false
  let advanceTimer: ReturnType<typeof setTimeout> | undefined
  let buildSeq = 0
  let exiting = false

  const timer = useStudyTimer(
    () => !review.value && !showAnswer.value && !revisitAnswered.value && !done.value,
  )
  const { sessionMs, questionMs, paused: timerPaused, start: startClock, resetQuestion, resetAll } = timer

  const item = computed(() => session.value?.items[idx.value] ?? null)
  const total = computed(() => session.value?.items.length ?? 0)
  /** The current question's verdict in this round (null = not yet marked) */
  const marked = computed(() => {
    const r = item.value ? roundRecords.value.get(item.value.id) : undefined
    return r ? r.correct : null
  })
  const counts = computed(() => {
    let right = 0
    let wrong = 0
    for (const v of roundRecords.value.values()) {
      if (v.correct) right++
      else wrong++
    }
    return { right, wrong }
  })
  const right = computed(() => counts.value.right)
  const wrong = computed(() => counts.value.wrong)
  const unanswered = computed(() => total.value - roundRecords.value.size)

  function resetCounters() {
    roundRecords.value = new Map()
    revisitAnswered.value = false
  }

  function cancelAdvance() {
    clearTimeout(advanceTimer)
    advanceTimer = undefined
  }

  function enterQuestion() {
    const r = item.value ? roundRecords.value.get(item.value.id) : undefined
    revisitAnswered.value = !!r
    showAnswer.value = !!r
    questionMs.value = r ? r.ms : 0
  }

  async function tryResume(): Promise<boolean> {
    const p = await getPaused().catch(() => null)
    const rebuilt = p ? sessionFromPaused(p, store.questions) : null
    if (!p || !rebuilt) return false
    session.value = rebuilt.session
    idx.value = rebuilt.idx
    const ids = new Set(session.value.items.map((it) => it.id))
    for (const r of p.results) {
      if (ids.has(r.qid)) roundRecords.value.set(r.qid, { id: r.id, correct: r.correct, ms: r.ms ?? 0 })
    }
    sessionMs.value = p.sessionMs
    await clearPaused().catch(() => {})
    enterQuestion()
    toast(`已恢复进度：已做 ${roundRecords.value.size} / ${session.value.items.length} 题`)
    return true
  }

  /** Fallback when no book is active: a quick run over the whole bank. */
  function quickSession(): BookSession {
    return buildSession(subjectQuestions.value, {
      shuffleQ: false,
      shuffleO: false,
      seed: '',
      title: '快速刷题',
    })
  }

  async function activeBookSession(): Promise<BookSession | null> {
    try {
      const [books, activeId] = await Promise.all([fetchBooks(), getActiveBookId(currentSubject.value)])
      const def = books.find((b) => b.id === activeId)
      const s = def ? sessionFromDef(def, store.questions) : null
      return s && s.items.length ? s : null
    } catch {
      return null
    }
  }

  async function build() {
    // A stale build (route watcher + onMounted both fire) must not overwrite a
    // newer one; every await re-checks the sequence number.
    const seq = ++buildSeq
    cancelAdvance()
    done.value = false
    interrupted.value = false
    idx.value = 0
    showAnswer.value = false
    resetCounters()
    if (route.query.resume && (await tryResume())) return
    if (seq !== buildSeq) return
    if (route.path === '/book/study' && currentSession.value) {
      session.value = currentSession.value
    } else {
      const s = await activeBookSession()
      if (seq !== buildSeq) return
      session.value = s ?? quickSession()
    }
    enterQuestion()
  }

  function next() {
    if (!session.value) return
    cancelAdvance()
    if (idx.value < session.value.items.length - 1) {
      idx.value++
      enterQuestion()
    } else {
      done.value = true
    }
  }

  function prev() {
    if (idx.value > 0) {
      cancelAdvance()
      idx.value--
      enterQuestion()
    }
  }

  async function mark(correct: boolean, advance = true) {
    if (!item.value || done.value || marking) return
    marking = true
    cancelAdvance()
    try {
      const q = item.value
      const qid = q.id
      const ms = review.value ? undefined : Math.round(questionMs.value)
      const prevRecord = roundRecords.value.get(qid)
      if (prevRecord) {
        await updateRecord(prevRecord.id, correct, ms)
        prevRecord.correct = correct
        prevRecord.ms = ms ?? 0
        toast(`已改为「${correct ? '做对了' : '做错了'}」`)
      } else {
        const rec = await addRecord(q, correct, ms)
        roundRecords.value.set(qid, { id: rec.id, correct, ms: ms ?? 0 })
        const label = correct ? '做对了' : '做错了'
        toast(review.value ? `已记「${label}」` : `已记「${label}」（本题 ${fmtDur(questionMs.value)}）`)
      }
      if (advance) advanceTimer = setTimeout(next, 450)
    } finally {
      marking = false
    }
  }
  function toggleTimer() {
    if (showAnswer.value) {
      toast('显示答案时计时自动暂停')
      return
    }
    timerPaused.value = !timerPaused.value
    toast(timerPaused.value ? '计时已暂停' : '计时已恢复')
  }

  function resetTimer() {
    resetQuestion()
    toast('本题计时已重置')
  }

  function retryAll() {
    idx.value = 0
    done.value = false
    resetCounters()
    resetAll()
    enterQuestion()
  }

  function retryWrongOnly() {
    if (!session.value) return
    if (!wrong.value) {
      toast('本轮没有做错的题')
      done.value = false
      return
    }
    session.value = {
      ...session.value,
      items: session.value.items.filter((it) => roundRecords.value.get(it.id)?.correct === false),
    }
    idx.value = 0
    done.value = false
    resetCounters()
    enterQuestion()
  }

  async function leaveWithSummary() {
    if (!review.value && !done.value && roundRecords.value.size >= 1 && item.value) {
      if (exiting) return
      exiting = true
      try {
        await savePaused(pausedFrom(session.value!, idx.value, sessionMs.value, roundRecords.value)).catch(() => {})
      } finally {
        exiting = false
      }
      interrupted.value = true
      done.value = true
      return
    }
    router.push('/')
  }

  async function continueInterrupted() {
    await clearPaused().catch(() => {})
    interrupted.value = false
    done.value = false
    enterQuestion()
  }

  function leave() {
    if (done.value) {
      router.push('/')
      return
    }
    void leaveWithSummary()
  }

  return {
    review,
    session,
    idx,
    showAnswer,
    done,
    interrupted,
    roundRecords,
    revisitAnswered,
    sessionMs,
    questionMs,
    timerPaused,
    item,
    total,
    marked,
    right,
    wrong,
    unanswered,
    build,
    startClock,
    next,
    prev,
    mark,
    toggleTimer,
    resetTimer,
    retryAll,
    retryWrongOnly,
    continueInterrupted,
    leave,
    leaveWithSummary,
  }
}
