import { afterEach, describe, expect, it, vi } from 'vitest'
import { deleteRecord, fetchMastery, normalizeMs, postRecord, updateRecord } from './api'

interface FetchCall {
  url: string
  init?: RequestInit
}

let calls: FetchCall[] = []
let responder: () => Response

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  })
}

afterEach(() => {
  vi.unstubAllGlobals()
  calls = []
})

function stubFetch(fn: () => Response) {
  responder = fn
  vi.stubGlobal(
    'fetch',
    vi.fn((url: string, init?: RequestInit) => {
      calls.push({ url, init })
      return responder()
    }),
  )
}

describe('req (shared error handling)', () => {
  it('returns parsed JSON for 2xx', async () => {
    stubFetch(() => jsonResponse({ ids: ['s1:P1'] }))
    await expect(fetchMastery()).resolves.toEqual({ ids: ['s1:P1'] })
    expect(calls[0].url).toBe('/api/mastery')
  })

  it('resolves 204 to undefined', async () => {
    stubFetch(() => new Response(null, { status: 204 }))
    await expect(deleteRecord(7)).resolves.toBeUndefined()
  })

  it('throws with method, url, status and backend text', async () => {
    stubFetch(() => new Response('boom', { status: 500 }))
    await expect(fetchMastery()).rejects.toThrow('GET /api/mastery 失败（500）：boom')
  })

  it('falls back to a status-only message when the body is unreadable', async () => {
    stubFetch(() => new Response('', { status: 502 }))
    await expect(fetchMastery()).rejects.toThrow('GET /api/mastery 失败（502）')
  })
})

describe('postRecord', () => {
  it('sends questionId, source, correct and normalized ms', async () => {
    stubFetch(() => jsonResponse({ record: { id: 1, questionId: 'P1', source: 's1', correct: true, at: 1 } }))
    await postRecord('P1', 's1', true, 1234.6)
    const body = JSON.parse(String(calls[0].init?.body))
    expect(body).toEqual({ questionId: 'P1', source: 's1', correct: true, ms: 1235 })
  })

  it('omits ms when it is zero or negative', async () => {
    stubFetch(() => jsonResponse({ record: { id: 1, questionId: 'P1', source: 's1', correct: false, at: 1 } }))
    await postRecord('P1', 's1', false, 0)
    await postRecord('P1', 's1', false, -5)
    await postRecord('P1', 's1', false, undefined)
    for (const c of calls) {
      expect(JSON.parse(String(c.init?.body)).ms).toBeUndefined()
    }
  })
})

describe('updateRecord', () => {
  it('normalizes ms the same way', async () => {
    stubFetch(() => new Response(null, { status: 204 }))
    await updateRecord(3, false, 99.9)
    expect(JSON.parse(String(calls[0].init?.body))).toEqual({ correct: false, ms: 100 })
  })
})

describe('normalizeMs', () => {
  it('rounds positive values and drops the rest', () => {
    expect(normalizeMs(1500.6)).toBe(1501)
    expect(normalizeMs(1)).toBe(1)
    expect(normalizeMs(0)).toBeUndefined()
    expect(normalizeMs(-100)).toBeUndefined()
    expect(normalizeMs(undefined)).toBeUndefined()
  })
})
