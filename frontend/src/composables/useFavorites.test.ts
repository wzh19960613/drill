import { beforeEach, describe, expect, it, vi } from 'vitest'

/**
 * State-machine tests for useFavorites (api layer mocked, pure frontend
 * logic): favorite-book creation/reuse, self-healing after deletion, state
 * resync after switching.
 */

const booksStore = new Map<string, any>()
let favoriteId: string | null = null

vi.mock('../api', () => ({
  fetchBooks: vi.fn(async () => [...booksStore.values()].map((b: unknown) => JSON.parse(JSON.stringify(b)))),
  getFavoriteBookApi: vi.fn(async () => favoriteId),
  saveBookApi: vi.fn(async (def: any) => {
    booksStore.set(def.id, JSON.parse(JSON.stringify(def)))
  }),
  setFavoriteBookApi: vi.fn(async (id: string | null) => {
    favoriteId = id
  }),
}))

import type { useFavorites as useFavoritesFn } from './useFavorites'

let useFavorites: typeof useFavoritesFn
let FAVORITE_BOOK_NAME: string

function seedBook(id: string, name: string, items: string[] = []) {
  booksStore.set(id, {
    id,
    name,
    seed: '',
    date: '2026-09-05',
    createdAt: 1,
    items: items.map((q) => ({ id: q, source: 's1', optionOrder: null })),
  })
}

/** The module holds singleton state: each case resets the module for a clean copy */
async function freshFavs() {
  const mod = await import('./useFavorites')
  useFavorites = mod.useFavorites
  FAVORITE_BOOK_NAME = mod.FAVORITE_BOOK_NAME
  const favs = useFavorites()
  await favs.ensureLoaded()
  return favs
}

beforeEach(() => {
  booksStore.clear()
  favoriteId = null
  vi.clearAllMocks()
  vi.resetModules()
})

describe('ensureFavoriteBook', () => {
  it('creates the favorite book when none exists', async () => {
    seedBook('bk1', '第一批题本', ['P1'])
    const favs = await freshFavs()
    const id = await favs.ensureFavoriteBook()
    expect(booksStore.get(id)?.name).toBe(FAVORITE_BOOK_NAME)
    expect(favoriteId).toBe(id)
  })

  it('reuses an existing book named 收藏题本 instead of creating a duplicate', async () => {
    seedBook('old', FAVORITE_BOOK_NAME, ['P9'])
    const favs = await freshFavs()
    const id = await favs.ensureFavoriteBook()
    expect(id).toBe('old')
    expect([...booksStore.values()].filter((b) => b.name === FAVORITE_BOOK_NAME)).toHaveLength(1)
    // After reuse the state set resyncs: old favorites show as favorited
    expect(favs.isFavorited('P9')).toBe(true)
  })

  it('recovers when the pointed-to book was deleted elsewhere', async () => {
    seedBook('gone', FAVORITE_BOOK_NAME, ['P1'])
    favoriteId = 'gone' // 已设置但书随后被删
    booksStore.clear()
    const favs = await freshFavs()
    // Stale state pointing at a deleted book: favoriting self-heals —
    // rebuild/reuse and write successfully
    await favs.toggleQuestion({ id: 'P2' }, true)
    expect(favoriteId).not.toBe('gone')
    const book = booksStore.get(favoriteId!)
    expect(book.items.map((i: any) => i.id)).toContain('P2')
    expect(favs.isFavorited('P2')).toBe(true)
  })
})

describe('toggleQuestion', () => {
  it('adds and removes a question in the favorite book', async () => {
    seedBook('bk1', '第一批题本', ['P1'])
    const favs = await freshFavs()
    await favs.toggleQuestion({ id: 'P1' }, true)
    const id1 = favoriteId!
    expect(booksStore.get(id1).items.map((i: any) => i.id)).toEqual(['P1'])

    await favs.toggleQuestion({ id: 'P1' }, false)
    expect(booksStore.get(id1).items).toEqual([])
    expect(favs.isFavorited('P1')).toBe(false)
  })

  it('never creates a book when un-favoriting with no favorite book set', async () => {
    seedBook('bk1', '第一批题本', ['P1'])
    const favs = await freshFavs()
    await favs.toggleQuestion({ id: 'P1' }, false)
    expect(favoriteId).toBeNull()
    expect([...booksStore.values()]).toHaveLength(1) // 没有新书
    expect(favs.isFavorited('P1')).toBe(false)
  })

  it('un-favoriting is a no-op after the book was deleted (backend cleared the pointer)', async () => {
    seedBook('bk1', '第一批题本', ['P1'])
    // Book deletion already cleared the backend pointer and the frontend
    // synced: state is clean
    const favs = await freshFavs()
    await favs.toggleQuestion({ id: 'P1' }, false)
    expect(favoriteId).toBeNull()
    expect([...booksStore.values()]).toHaveLength(1)
    expect(favs.isFavorited('P1')).toBe(false)
  })

  it('persists the question source on newly favorited items', async () => {
    seedBook('bk1', '第一批题本', ['P1'])
    const favs = await freshFavs()
    await favs.toggleQuestion({ id: 'P9', source: 'src7' }, true)
    const book = booksStore.get(favoriteId!)
    expect(book.items).toContainEqual({ id: 'P9', source: 'src7', optionOrder: null })
  })

  it('reports failure when the book save fails', async () => {
    const { saveBookApi } = await import('../api')
    seedBook('bk1', '第一批题本', ['P1'])
    favoriteId = 'bk1'
    const favs = await freshFavs()
    vi.mocked(saveBookApi).mockRejectedValueOnce(new Error('network'))
    const ok = await favs.toggleQuestion({ id: 'P2' }, true)
    expect(ok).toBe(false)
    expect(favs.isFavorited('P2')).toBe(false)
    vi.mocked(saveBookApi).mockClear()
  })
})

describe('setFavoriteBook', () => {
  it('switching the favorite book resyncs the favorited set', async () => {
    seedBook('a', '题本A', ['P1', 'P2'])
    seedBook('b', '题本B', ['P3'])
    favoriteId = 'a'
    const favs = await freshFavs()
    expect(favs.isFavorited('P1')).toBe(true)
    expect(favs.isFavorited('P3')).toBe(false)

    await favs.setFavoriteBook('b')
    expect(favs.favoriteBookId.value).toBe('b')
    expect(favs.isFavorited('P1')).toBe(false)
    expect(favs.isFavorited('P3')).toBe(true)
  })

  it('clearing empties the favorited set', async () => {
    seedBook('a', '题本A', ['P1'])
    favoriteId = 'a'
    const favs = await freshFavs()
    expect(favs.isFavorited('P1')).toBe(true)
    await favs.setFavoriteBook(null)
    expect(favs.favoriteBookId.value).toBeNull()
    expect(favs.isFavorited('P1')).toBe(false)
  })
})
