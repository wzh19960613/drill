import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { Question } from '../types'

const mockRoute = { query: {} as Record<string, unknown>, path: '/study' }
const mockRouterPush = vi.fn()

vi.mock('vue-router', () => ({
  useRoute: () => mockRoute,
  useRouter: () => ({ push: mockRouterPush }),
}))

const state = { questions: [] as Question[] }
const addRecord = vi.fn()
const updateRecord = vi.fn()

vi.mock('../store', async () => {
  const { computed, reactive, ref } = await import('vue')
  const store = reactive({ questions: state.questions })
  return {
    store,
    subjectQuestions: computed(() => store.questions),
    currentSubject: ref(''),
    addRecord: (...args: unknown[]) => addRecord(...(args as [never])),
    updateRecord: (...args: unknown[]) => updateRecord(...(args as [never])),
  }
})

const getPaused = vi.fn()
const clearPaused = vi.fn()
const savePaused = vi.fn()
const fetchBooks = vi.fn()
const getActiveBookId = vi.fn()

vi.mock('../api', () => ({
  getPaused: (...a: unknown[]) => getPaused(...(a as [never])),
  clearPaused: (...a: unknown[]) => clearPaused(...(a as [never])),
  savePaused: (...a: unknown[]) => savePaused(...(a as [never])),
  fetchBooks: (...a: unknown[]) => fetchBooks(...(a as [never])),
  getActiveBookId: (...a: unknown[]) => getActiveBookId(...(a as [never])),
  fetchQuestionRaw: vi.fn(),
  saveBookApi: vi.fn(),
  deleteBookApi: vi.fn(),
  setActiveBookApi: vi.fn(),
}))

function mkQuestion(id: string, source = 's1'): Question {
  return {
    id,
    source,
    chapter: 'c',
    qtype: '选择题',
    stem: [id],
    options: [],
    correct_id: null,
    correct_ids: [],
    answer: [],
    solution: [],
  }
}

const toast = vi.fn()

async function freshRound() {
  const { useStudyRound } = await import('./useStudyRound')
  return useStudyRound(toast)
}

beforeEach(() => {
  vi.clearAllMocks()
  vi.resetModules()
  mockRoute.query = {}
  mockRoute.path = '/study'
  state.questions = [mkQuestion('A'), mkQuestion('B'), mkQuestion('C')]
  getPaused.mockResolvedValue(null)
  fetchBooks.mockResolvedValue([])
  getActiveBookId.mockResolvedValue(null)
  savePaused.mockResolvedValue(undefined)
  clearPaused.mockResolvedValue(undefined)
})

afterEach(() => {
  vi.useRealTimers()
})

describe('mark', () => {
  it('a fast double-click produces exactly one record', async () => {
    vi.useFakeTimers()
    const round = await freshRound()
    await round.build()
    expect(round.item.value?.id).toBe('A')
    addRecord.mockResolvedValue({ id: 11, questionId: 'A', source: 's1', correct: true, at: 1 })
    const first = round.mark(true)
    const second = round.mark(false)
    await Promise.all([first, second])
    expect(addRecord).toHaveBeenCalledTimes(1)
    expect(addRecord).toHaveBeenCalledWith(
      expect.objectContaining({ id: 'A', source: 's1' }),
      true,
      0,
    )
    expect(updateRecord).not.toHaveBeenCalled()
  })

  it('auto-advances after 450ms', async () => {
    vi.useFakeTimers()
    const round = await freshRound()
    await round.build()
    addRecord.mockResolvedValue({ id: 11, questionId: 'A', source: 's1', correct: true, at: 1 })
    await round.mark(true)
    expect(round.idx.value).toBe(0)
    vi.advanceTimersByTime(450)
    expect(round.idx.value).toBe(1)
    expect(round.item.value?.id).toBe('B')
  })

  it('a manual next cancels the pending auto-advance (no double skip)', async () => {
    vi.useFakeTimers()
    const round = await freshRound()
    await round.build()
    addRecord.mockResolvedValue({ id: 11, questionId: 'A', source: 's1', correct: true, at: 1 })
    await round.mark(true)
    round.next()
    expect(round.idx.value).toBe(1)
    vi.advanceTimersByTime(450)
    expect(round.idx.value).toBe(1)
  })

  it('re-judging an answered question goes through updateRecord', async () => {
    vi.useFakeTimers()
    const round = await freshRound()
    await round.build()
    addRecord.mockResolvedValue({ id: 11, questionId: 'A', source: 's1', correct: true, at: 1 })
    updateRecord.mockResolvedValue(undefined)
    await round.mark(true)
    vi.advanceTimersByTime(450)
    round.prev()
    expect(round.idx.value).toBe(0)
    await round.mark(false)
    expect(updateRecord).toHaveBeenCalledWith(11, false, expect.anything())
    expect(addRecord).toHaveBeenCalledTimes(1)
    expect(round.roundRecords.value.get('A')).toMatchObject({ id: 11, correct: false })
  })
})

describe('retryWrongOnly', () => {
  it('leaves the session untouched when everything was right', async () => {
    vi.useFakeTimers()
    const round = await freshRound()
    await round.build()
    addRecord.mockResolvedValue({ id: 11, questionId: 'A', source: 's1', correct: true, at: 1 })
    await round.mark(true)
    vi.advanceTimersByTime(450)
    await round.mark(true)
    vi.advanceTimersByTime(450)
    const before = round.session.value
    round.retryWrongOnly()
    expect(round.session.value).toBe(before)
    expect(round.total.value).toBe(3)
    expect(toast).toHaveBeenCalledWith('本轮没有做错的题')
  })

  it('keeps only the wrongly answered questions otherwise', async () => {
    vi.useFakeTimers()
    const round = await freshRound()
    await round.build()
    addRecord.mockImplementation(
      async (_q: unknown, correct: boolean) =>
        ({ id: 11, questionId: 'A', source: 's1', correct, at: 1 }) as never,
    )
    await round.mark(false)
    vi.advanceTimersByTime(450)
    await round.mark(true)
    vi.advanceTimersByTime(450)
    round.retryWrongOnly()
    expect(round.session.value?.items.map((it) => it.id)).toEqual(['A'])
    expect(round.idx.value).toBe(0)
    expect(round.done.value).toBe(false)
  })
})

describe('leaveWithSummary', () => {
  it('routes home directly in review mode without pausing', async () => {
    mockRoute.query = { mode: 'review' }
    const round = await freshRound()
    await round.build()
    await round.leaveWithSummary()
    expect(mockRouterPush).toHaveBeenCalledWith('/')
    expect(savePaused).not.toHaveBeenCalled()
  })

  it('routes home when nothing was answered', async () => {
    const round = await freshRound()
    await round.build()
    await round.leaveWithSummary()
    expect(mockRouterPush).toHaveBeenCalledWith('/')
    expect(savePaused).not.toHaveBeenCalled()
  })

  it('routes home when the round is already done', async () => {
    const round = await freshRound()
    await round.build()
    addRecord.mockResolvedValue({ id: 11, questionId: 'A', source: 's1', correct: true, at: 1 })
    await round.mark(true)
    round.done.value = true
    await round.leaveWithSummary()
    expect(mockRouterPush).toHaveBeenCalledWith('/')
    expect(savePaused).not.toHaveBeenCalled()
  })

  it('saves a paused snapshot (items carry source) and shows the interruption', async () => {
    vi.useFakeTimers()
    const round = await freshRound()
    await round.build()
    addRecord.mockResolvedValue({ id: 11, questionId: 'A', source: 's1', correct: true, at: 1 })
    await round.mark(true)
    vi.advanceTimersByTime(450)
    await round.leaveWithSummary()
    expect(savePaused).toHaveBeenCalledTimes(1)
    const saved = savePaused.mock.calls[0][0]
    expect(saved.items.map((it: { id: string; source: string }) => [it.id, it.source])).toEqual([
      ['A', 's1'],
      ['B', 's1'],
      ['C', 's1'],
    ])
    expect(saved.results).toEqual([{ qid: 'A', id: 11, correct: true, ms: 0 }])
    expect(saved.idx).toBe(1)
    expect(round.interrupted.value).toBe(true)
    expect(round.done.value).toBe(true)
    expect(mockRouterPush).not.toHaveBeenCalled()
  })

  it('a second leave while saving is a no-op (single snapshot)', async () => {
    vi.useFakeTimers()
    let release!: () => void
    savePaused.mockReturnValue(new Promise<void>((r) => (release = r)))
    const round = await freshRound()
    await round.build()
    addRecord.mockResolvedValue({ id: 11, questionId: 'A', source: 's1', correct: true, at: 1 })
    await round.mark(true)
    const first = round.leaveWithSummary()
    await round.leaveWithSummary()
    release()
    await first
    expect(savePaused).toHaveBeenCalledTimes(1)
  })
})

describe('build', () => {
  it('a stale build never overwrites the newer session', async () => {
    let releaseStale!: (books: unknown[]) => void
    const stale = new Promise<unknown[]>((r) => (releaseStale = r))
    fetchBooks.mockImplementationOnce(() => stale)
    getActiveBookId.mockResolvedValue('bk9')
    const round = await freshRound()
    const first = round.build()
    const second = round.build()
    await second
    expect(round.session.value?.title).toBe('快速刷题')
    releaseStale([
      {
        id: 'bk9',
        name: '陈年题本',
        seed: '',
        date: '2026-01-01',
        createdAt: 1,
        items: [{ id: 'C', source: 's1', optionOrder: null }],
      },
    ])
    await first

    expect(round.session.value?.title).toBe('快速刷题')
    expect(round.session.value?.items.map((it) => it.id)).toEqual(['A', 'B', 'C'])
  })
})
