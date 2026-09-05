import { computed, ref, watch, type Ref } from 'vue'
import { currentSubject, questionByKey, store, subjects, UNKNOWN } from '../store'
import { deleteBook, fetchBooks, getActiveBookId, sessionFromDef, upsertBook } from '../book'
import type { BookSession } from '../book'
import type { BookDef, PausedSession } from '../types'

type Flash = (msg: string) => void

export interface BookLibraryContext {
  viewing: Ref<BookDef | null>
  flash: Flash
}

/**
 * Server-persisted book list for the book home page: loading, the active
 * book per subject, and the subject-based split into pure vs mixed books.
 */
export function useBookLibrary(ctx: BookLibraryContext) {
  const books = ref<BookDef[]>([])
  const activeId = ref<string | null>(null)
  const loading = ref(true)
  const paused = ref<PausedSession | null>(null)
  const error = ref('')

  const activeBook = computed(() => books.value.find((b) => b.id === activeId.value) ?? null)

  const resumeActive = computed(() => !!paused.value && paused.value.bookId === activeBook.value?.id)

  const isComposite = computed(
    () => !currentSubject.value && subjects.value.filter((s) => s !== UNKNOWN).length >= 2,
  )

  const subjectSplit = computed(() => {
    const subj = currentSubject.value
    if (!subj) return { main: books.value, mixed: [] as BookDef[] }
    const main: BookDef[] = []
    const mixed: BookDef[] = []
    for (const b of books.value) {
      const subs = new Set(
        b.items
          .map((it) => questionByKey.value.get(`${it.source ?? ''}\u0000${it.id}`)?.subject)
          .filter((s): s is string => !!s),
      )
      if (subs.size && !subs.has(subj)) continue
      ;(subs.size <= 1 ? main : mixed).push(b)
    }
    return { main, mixed }
  })

  function subjectCountOf(b: BookDef): number {
    return b.items.filter(
      (it) =>
        questionByKey.value.get(`${it.source ?? ''}\u0000${it.id}`)?.subject ===
        currentSubject.value,
    ).length
  }

  function subjectsOf(b: BookDef): string[] {
    const subs: string[] = []
    for (const it of b.items) {
      const s = questionByKey.value.get(`${it.source ?? ''}\u0000${it.id}`)?.subject
      if (s && !subs.includes(s)) subs.push(s)
    }
    return subs
  }

  async function refresh(opts?: { quiet?: boolean }) {
    if (!opts?.quiet) loading.value = true
    try {
      ;[books.value, activeId.value] = await Promise.all([
        fetchBooks(),
        getActiveBookId(currentSubject.value),
      ])
      error.value = ''
    } catch (e) {
      error.value = e instanceof Error ? e.message : '题本加载失败'
    } finally {
      if (!opts?.quiet) loading.value = false
    }
    if (ctx.viewing.value) {
      ctx.viewing.value = books.value.find((b) => b.id === ctx.viewing.value!.id) ?? null
    }
  }

  async function commitViewRename(name: string) {
    const def = ctx.viewing.value
    if (!def || !name || name === def.name) return
    try {
      await upsertBook({ ...def, name })
      await refresh()
      ctx.flash(`已重命名为 ${name}`)
    } catch (e) {
      error.value = e instanceof Error ? e.message : '重命名失败'
    }
  }

  async function removeConfirmed(def: BookDef) {
    try {
      await deleteBook(def.id)
      await refresh()
      ctx.flash(`已删除 ${def.name}`)
    } catch (e) {
      error.value = e instanceof Error ? e.message : '删除失败'
    }
  }

  function sessionForExport(def: BookDef): BookSession | null {
    const session = sessionFromDef(def, store.questions)
    if (!session.items.length) {
      ctx.flash('该题本中的题目已不在题库里')
      return null
    }
    return session
  }

  /** After a subject switch only the active book id changes (remembered per subject); no need to refetch the book list */
  watch(currentSubject, async () => {
    activeId.value = await getActiveBookId(currentSubject.value).catch(() => null)
  })

  return {
    books,
    activeId,
    loading,
    paused,
    error,
    activeBook,
    resumeActive,
    isComposite,
    subjectSplit,
    subjectCountOf,
    subjectsOf,
    refresh,
    commitViewRename,
    removeConfirmed,
    sessionForExport,
  }
}
