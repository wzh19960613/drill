import { computed, reactive, ref, watch } from 'vue'
import type { Question, Rec, Stats } from './types'
import * as api from './api'
import { readString, writeString, StorageKeys } from './storage'

export const store = reactive({
  questions: [] as Question[],
  records: [] as Rec[],
  masteries: [] as string[],
  loaded: false,
  loading: false,
  error: '',
  lastRecord: null as Rec | null,
})

export const selection = reactive({ ids: [] as string[] })

export const currentSubject = ref<string>(readString(StorageKeys.subject) ?? '')
watch(currentSubject, (v) => writeString(StorageKeys.subject, v))

export const UNKNOWN = '未知'

/** Question identity for stats/mastery lookups: cross-source keying needs the
 *  source (question ids are only unique within one source) */
export interface QuestionLike {
  id: string
  source?: string
}

const NUL = '\u0000'

/** Composite record/stats key: source + NUL + question id */
export function questionKey(q: QuestionLike): string {
  return `${q.source ?? ''}${NUL}${q.id}`
}

/** Mastery entries are opaque strings on the backend; we store source:id */
function masteryKey(q: QuestionLike): string {
  return q.source ? `${q.source}:${q.id}` : q.id
}

export function subjectOf(q: { subject?: string }): string {
  return q.subject || UNKNOWN
}

export function chapterOf(q: { chapter?: string }): string {
  return q.chapter || UNKNOWN
}

export const subjectQuestions = computed(() =>
  currentSubject.value
    ? store.questions.filter((q) => subjectOf(q) === currentSubject.value)
    : store.questions,
)

const nonEmpty = (vs: (string | undefined)[]): string[] => vs.filter((v): v is string => !!v)

export const subjects = computed(() => {
  const list = Array.from(new Set(nonEmpty(store.questions.map((q) => q.subject)))).sort()
  if (store.questions.some((q) => !q.subject)) list.push(UNKNOWN)
  return list
})

export const origins = computed(() =>
  Array.from(new Set(nonEmpty(subjectQuestions.value.map((q) => q.origin)))).sort(),
)

export const chapters = computed(() =>
  Array.from(new Set(subjectQuestions.value.map((q) => chapterOf(q)))).sort(),
)

export const qtypes = computed(() =>
  Array.from(new Set(subjectQuestions.value.map((q) => q.qtype))).sort(),
)

watch(subjects, (list) => {
  if (currentSubject.value && !list.includes(currentSubject.value)) currentSubject.value = ''
})

export const questionById = computed(() => {
  const m = new Map<string, Question>()
  for (const q of store.questions) m.set(q.id, q)
  return m
})

/** Same bank indexed by the cross-source composite key (for book items) */
export const questionByKey = computed(() => {
  const m = new Map<string, Question>()
  for (const q of store.questions) m.set(questionKey(q), q)
  return m
})

const recordsByQ = computed(() => {
  const m = new Map<string, Rec[]>()
  for (const r of store.records) {
    const key = `${r.source ?? ''}${NUL}${r.questionId}`
    const list = m.get(key)
    if (list) list.push(r)
    else m.set(key, [r])
  }
  for (const list of m.values()) list.sort((a, b) => a.at - b.at)
  return m
})

function deriveStats(mine: Rec[]): Stats {
  const recent = mine.slice(-5)
  return {
    attempts: mine.length,
    wrong: mine.filter((r) => !r.correct).length,
    last_correct: mine.length ? mine[mine.length - 1].correct : null,
    last_at: mine.length ? mine[mine.length - 1].at : null,
    recent_wrong5: recent.filter((r) => !r.correct).length,
  }
}

const statsCache = computed(() => {
  const m = new Map<string, Stats>()
  for (const [key, mine] of recordsByQ.value) m.set(key, deriveStats(mine))
  return m
})

export function statsOf(q: QuestionLike): Stats {
  return statsCache.value.get(questionKey(q)) ?? deriveStats([])
}

export function recentOf(q: QuestionLike, k: number): Rec[] {
  return recordsByQ.value.get(questionKey(q))?.slice(-k) ?? []
}

export function recentAcc5(q: QuestionLike): { total: number; correct: number } {
  const last = recentOf(q, 5)
  return { total: last.length, correct: last.filter((r) => r.correct).length }
}

/** Accuracy over the last five attempts as a percentage (0 when unanswered) */
export function acc5Percent(q: QuestionLike): number {
  const { total, correct } = recentAcc5(q)
  return total ? Math.round((correct / total) * 100) : 0
}

let loadingPromise: Promise<void> | null = null

function startLoad(): Promise<void> {
  loadingPromise = (async () => {
    store.loading = true
    store.error = ''
    try {
      const [qs, rs, m] = await Promise.all([
        api.fetchQuestions(),
        api.fetchRecords(),
        api.fetchMastery(),
      ])
      store.questions = qs
      store.records = rs
      store.masteries = m.ids
      store.loaded = true
    } catch (e) {
      store.error = e instanceof Error ? e.message : '加载失败'
    } finally {
      store.loading = false
      loadingPromise = null
    }
  })()
  return loadingPromise
}

export async function loadAll(force = false): Promise<void> {
  if (store.loaded && !force) return
  if (loadingPromise && !force) return loadingPromise
  // force while a load is in flight: wait for it, then refetch fresh data
  if (loadingPromise) await loadingPromise
  return startLoad()
}

export function isMastered(q: QuestionLike): boolean {
  return store.masteries.includes(masteryKey(q))
}

export async function toggleMastered(q: QuestionLike): Promise<boolean> {
  const key = masteryKey(q)
  const set = new Set(store.masteries)
  const on = !set.has(key)
  if (on) set.add(key)
  else set.delete(key)
  store.masteries = [...set]
  await api.setMastery(store.masteries)
  return on
}

export async function addRecord(q: QuestionLike, correct: boolean, ms?: number): Promise<Rec> {
  const { record } = await api.postRecord(q.id, q.source ?? '', correct, api.normalizeMs(ms))
  store.records.push(record)
  store.lastRecord = record
  return record
}

export async function updateRecord(id: number, correct: boolean, ms?: number) {
  const rec = store.records.find((r) => r.id === id)
  const ms1 = api.normalizeMs(ms)
  await api.updateRecord(id, correct, ms1)
  if (rec) {
    rec.correct = correct
    // keep the previous local duration when the caller passes none
    if (ms1 !== undefined) rec.ms = ms1
  }
}

export async function removeRecord(id: number) {
  await api.deleteRecord(id)
  store.records = store.records.filter((r) => r.id !== id)
  if (store.lastRecord?.id === id) store.lastRecord = null
}

export async function undoLast(): Promise<number | null> {
  if (!store.lastRecord) return null
  const id = store.lastRecord.id
  await removeRecord(id)
  return id
}
