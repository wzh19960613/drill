import { ref } from 'vue'
import { fetchBooks, getFavoriteBookApi, saveBookApi, setFavoriteBookApi } from '../api'
import type { BookDef } from '../types'
import type { QuestionLike } from '../store'
import { randomToken } from '../rng'
import { isoToday } from '../format'

export const FAVORITE_BOOK_NAME = '收藏题本'

const favoriteBookId = ref<string | null>(null)
const loaded = ref(false)

const favoritedIds = ref(new Set<string>())

let writeQueue: Promise<unknown> = Promise.resolve()

export function useFavorites() {
  async function ensureLoaded() {
    if (loaded.value) return
    loaded.value = true
    favoriteBookId.value = await getFavoriteBookApi().catch(() => null)
    if (favoriteBookId.value) await syncFromBooks()
  }

  async function syncFromBooks() {
    if (!favoriteBookId.value) return
    const books = await fetchBooks().catch(() => [] as BookDef[])
    const book = books.find((b) => b.id === favoriteBookId.value)
    favoritedIds.value = new Set(book?.items.map((it) => it.id) ?? [])
  }

  /** Re-resolve (and if needed create/reuse) the favorite book using the book
   *  list the caller already fetched — no extra fetchBooks roundtrip */
  async function resolveFavoriteBook(books: BookDef[]): Promise<string> {
    favoriteBookId.value = null
    favoritedIds.value = new Set()
    const existing = books.find((b) => b.name === FAVORITE_BOOK_NAME)
    const id = existing?.id ?? `fav-${randomToken()}`
    if (!existing) {
      await saveBookApi({
        id,
        name: FAVORITE_BOOK_NAME,
        seed: '',
        date: isoToday(),
        createdAt: Date.now(),
        items: [],
      })
    } else {
      favoritedIds.value = new Set(existing.items.map((it) => it.id))
    }
    favoriteBookId.value = id
    await setFavoriteBookApi(id)
    return id
  }

  async function ensureFavoriteBook(): Promise<string> {
    await ensureLoaded()
    const books = await fetchBooks().catch(() => [] as BookDef[])
    if (favoriteBookId.value && books.some((b) => b.id === favoriteBookId.value)) {
      return favoriteBookId.value
    }
    return resolveFavoriteBook(books)
  }

  function isFavorited(questionId: string): boolean {
    return favoritedIds.value.has(questionId)
  }

  function setFavorited(questionId: string, on: boolean) {
    if (on) favoritedIds.value.add(questionId)
    else favoritedIds.value.delete(questionId)
  }

  function toggleQuestion(q: QuestionLike, on: boolean): Promise<boolean> {
    const run = writeQueue.then(() => toggleQuestionNow(q, on))
    writeQueue = run.catch(() => {})
    return run
  }

  /** One fetchBooks per pass; the book created mid-operation only shows up
   *  on a second pass (self-heal retry), and success is reported back */
  async function toggleQuestionNow(q: QuestionLike, on: boolean): Promise<boolean> {
    await ensureLoaded()
    for (let attempt = 0; attempt < 2; attempt++) {
      const books = await fetchBooks().catch(() => [] as BookDef[])
      let bookId = favoriteBookId.value
      if (on && (!bookId || !books.some((b) => b.id === bookId))) {
        bookId = await resolveFavoriteBook(books).catch(() => null)
      }
      if (!bookId) return false
      const book = books.find((b) => b.id === bookId)
      if (!book) {
        // the pointed-to book vanished (deleted elsewhere): reset; only a
        // favorite (on) self-heals with a second pass
        favoriteBookId.value = null
        favoritedIds.value = new Set()
        if (!on) return false
        continue
      }
      try {
        const items = on
          ? [...book.items, { id: q.id, source: q.source ?? '', optionOrder: null }]
          : book.items.filter((it) => it.id !== q.id)
        await saveBookApi({ ...book, items })
      } catch {
        return false
      }
      setFavorited(q.id, on)
      return true
    }
    return false
  }

  async function setFavoriteBook(id: string | null): Promise<void> {
    await setFavoriteBookApi(id)
    favoriteBookId.value = id
    if (id) await syncFromBooks()
    else favoritedIds.value = new Set()
  }

  return {
    favoriteBookId,
    favoritedIds,
    ensureLoaded,
    ensureFavoriteBook,
    setFavoriteBook,
    isFavorited,
    setFavorited,
    toggleQuestion,
  }
}
