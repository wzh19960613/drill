import { store, subjectQuestions, currentSubject } from '../store'
import { getPaused, clearPaused } from '../api'
import { buildSession, fetchBooks, getActiveBookId, sessionFromDef, sessionFromPaused, type BookSession } from '../book'
import type { RoundCore, Toast } from './useStudyRound'

export async function resumePaused(
  core: RoundCore,
  nav: { enterQuestion: () => void },
  toast: Toast,
): Promise<boolean> {
  const p = await getPaused().catch(() => null)
  const rebuilt = p ? sessionFromPaused(p, store.questions) : null
  if (!p || !rebuilt) return false
  core.session.value = rebuilt.session
  core.idx.value = rebuilt.idx
  const ids = new Set(core.session.value.items.map((it) => it.id))
  for (const r of p.results) {
    if (ids.has(r.qid)) core.roundRecords.value.set(r.qid, { id: r.id, correct: r.correct, ms: r.ms ?? 0 })
  }
  core.sessionMs.value = p.sessionMs
  await clearPaused().catch(() => {})
  nav.enterQuestion()
  toast(`已恢复进度：已做 ${core.roundRecords.value.size} / ${core.session.value.items.length} 题`)
  return true
}

export function quickSession(): BookSession {
  return buildSession(subjectQuestions.value, {
    shuffleQ: false,
    shuffleO: false,
    seed: '',
    title: '快速刷题',
  })
}

export async function activeBookSession(): Promise<BookSession | null> {
  try {
    const [books, activeId] = await Promise.all([fetchBooks(), getActiveBookId(currentSubject.value)])
    const def = books.find((b) => b.id === activeId)
    const s = def ? sessionFromDef(def, store.questions) : null
    return s && s.items.length ? s : null
  } catch {
    return null
  }
}
