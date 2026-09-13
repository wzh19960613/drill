import { computed, ref, watch, type Ref } from 'vue'
import { currentSubject, questionByKey, store, subjects, UNKNOWN } from '../store'
import { splitBooksBySubject } from '../book'
import type { BookItemDef, Question } from '../types'
import { deleteBook, fetchBooks, getActiveBookId, sessionFromDef, upsertBook } from '../book'
import type { BookSession } from '../book'
import type { BookDef, PausedSession } from '../types'

type Flash = (msg: string) => void

export interface BookLibraryContext {
  viewing: Ref<BookDef | null>
  flash: Flash
}

function subjectOfItemOf(byKey: Map<string, Question>, it: BookItemDef): string | undefined {
  const q = byKey.get(`${it.source ?? ''}\u0000${it.id}`)
  return q ? q.subject || UNKNOWN : undefined
}

function subjectsOfBook(byKey: Map<string, Question>, b: BookDef): string[] {
  const subs: string[] = []
  for (const it of b.items) {
    const s = subjectOfItemOf(byKey, it)
    if (s && !subs.includes(s)) subs.push(s)
  }
  return subs
}

function staleCountOfBook(byKey: Map<string, Question>, b: BookDef): number {
  return b.items.filter((it) => !byKey.has(`${it.source ?? ''}\u0000${it.id}`)).length
}

async function refreshLibrary(
  ctx: BookLibraryContext,
  books: Ref<BookDef[]>,
  activeId: Ref<string | null>,
  error: Ref<string>,
  loading: Ref<boolean>,
  opts?: { quiet?: boolean },
): Promise<void> {
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

async function removeStaleItems(
  def: BookDef,
  byKey: Map<string, Question>,
  refresh: () => Promise<void>,
  flash: Flash,
) {
  const keep = def.items.filter((it) => byKey.has(`${it.source ?? ''}\u0000${it.id}`))
  if (keep.length === def.items.length) return
  try {
    await upsertBook({ ...def, items: keep })
    await refresh()
    flash(`已清理 ${def.items.length - keep.length} 道失效题目`)
  } catch (e) {
    flash(e instanceof Error ? e.message : '清理失败')
  }
}

function useBookMutations(
  ctx: BookLibraryContext,
  error: Ref<string>,
  refresh: () => Promise<void>,
) {
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

  return { commitViewRename, removeConfirmed, sessionForExport }
}

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

  const subjectOfItem = (it: BookItemDef) => subjectOfItemOf(questionByKey.value, it)

  const subjectSplit = computed(() =>
    splitBooksBySubject(
      books.value.filter((b) => b.items.length > 0),
      currentSubject.value,
      subjectOfItem,
    ),
  )

  const emptyBooks = computed(() => books.value.filter((b) => b.items.length === 0))

  const subjectCountOf = (b: BookDef) =>
    b.items.filter((it) => subjectOfItem(it) === currentSubject.value).length
  const subjectsOf = (b: BookDef) => subjectsOfBook(questionByKey.value, b)

  const staleCountOf = (b: BookDef) => staleCountOfBook(questionByKey.value, b)

  const removeStale = (def: BookDef) =>
    removeStaleItems(def, questionByKey.value, refresh, ctx.flash)

  const refresh = (opts?: { quiet?: boolean }) => refreshLibrary(ctx, books, activeId, error, loading, opts)

  const { commitViewRename, removeConfirmed, sessionForExport } = useBookMutations(
    ctx, error, refresh,
  )

  watch(currentSubject, async () => {
    activeId.value = await getActiveBookId(currentSubject.value).catch(() => null)
  })

  return {
    books, activeId, loading, paused, error, activeBook, resumeActive,
    isComposite, subjectSplit, emptyBooks, subjectCountOf, subjectsOf,
    staleCountOf, removeStale, refresh, commitViewRename, removeConfirmed,
    sessionForExport,
  }
}
