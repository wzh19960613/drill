import type { BookDef, PausedSession, Question, Rec } from './types'
import type { PrintPayload } from './print'

/** Shared non-2xx handling: prefer the backend's message, fall back to a
 *  status-qualified label */
async function assertOk(r: Response, fallback: string): Promise<void> {
  if (r.ok) return
  const text = await r.text().catch(() => '')
  throw new Error(text ? `${fallback}：${text}` : fallback)
}

export async function exportPdf(
  payload: PrintPayload,
): Promise<{ url: string; name: string; bytes: number; cached: boolean }> {
  const r = await fetch('/api/export/pdf', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  })
  await assertOk(r, `后台导出失败（${r.status}）`)
  return r.json()
}

export async function previewPdf(payload: PrintPayload): Promise<string> {
  const r = await fetch('/api/preview/pdf', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  })
  await assertOk(r, `预览失败（${r.status}）`)
  return URL.createObjectURL(await r.blob())
}

async function req<T>(url: string, init?: RequestInit): Promise<T> {
  const r = await fetch(url, init)
  await assertOk(r, `${init?.method ?? 'GET'} ${url} 失败（${r.status}）`)
  if (r.status === 204) return undefined as T
  return (await r.json()) as T
}

export function fetchQuestions(): Promise<Question[]> {
  return req('/api/questions')
}

export function fetchRecords(): Promise<Rec[]> {
  return req('/api/records')
}

/** Normalize a duration for storage: round, drop missing/non-positive values */
export function normalizeMs(ms: number | undefined): number | undefined {
  return ms != null && ms > 0 ? Math.round(ms) : undefined
}

export async function postRecord(
  questionId: string,
  source: string,
  correct: boolean,
  ms?: number,
): Promise<{ record: Rec }> {
  return req('/api/records', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ questionId, source, correct, ms: normalizeMs(ms) }),
  })
}

export function deleteRecord(id: number): Promise<void> {
  return req(`/api/records/${id}`, { method: 'DELETE' })
}

export function updateRecord(id: number, correct: boolean, ms?: number): Promise<void> {
  return req(`/api/records/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ correct, ms: normalizeMs(ms) }),
  })
}

export interface SourceInfo {
  id: string
  name: string
  path: string
  exists: boolean
  count: number
}

export function fetchSources(): Promise<SourceInfo[]> {
  return req('/api/sources')
}

export function addSource(path: string): Promise<{ id: string; name: string; path: string }> {
  return req('/api/sources', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ path }),
  })
}

export function removeSource(id: string): Promise<void> {
  return req(`/api/sources/${encodeURIComponent(id)}`, { method: 'DELETE' })
}

export function relocateSource(
  id: string,
  path: string,
): Promise<{ id: string; name: string; path: string }> {
  return req(`/api/sources/${encodeURIComponent(id)}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ path }),
  })
}

export function fetchQuestionRaw(id: string, source: string): Promise<{ markdown: string }> {
  return req(`/api/questions/${encodeURIComponent(id)}/raw?source=${encodeURIComponent(source)}`)
}

export function previewQuestion(markdown: string, source: string): Promise<Question> {
  return req('/api/questions/preview', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ markdown, source }),
  })
}

export function createQuestion(markdown: string, source: string): Promise<Question> {
  return req('/api/questions', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ markdown, source }),
  })
}

export function updateQuestionApi(id: string, markdown: string, source: string): Promise<Question> {
  return req(`/api/questions/${encodeURIComponent(id)}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ markdown, source }),
  })
}

export function deleteQuestionApi(id: string, source: string): Promise<void> {
  return req(
    `/api/questions/${encodeURIComponent(id)}?source=${encodeURIComponent(source)}`,
    { method: 'DELETE' },
  )
}

export function fetchMastery(): Promise<{ ids: string[] }> {
  return req('/api/mastery')
}

export function setMastery(ids: string[]): Promise<void> {
  return req('/api/mastery', {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ ids }),
  })
}

export function fetchBooks(): Promise<BookDef[]> {
  return req('/api/books')
}

export function saveBookApi(def: BookDef): Promise<BookDef> {
  return req('/api/books', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(def),
  })
}

export function getFavoriteBookApi(): Promise<string | null> {
  return req<{ id: string | null }>('/api/books/favorite').then((r) => r.id)
}

export function setFavoriteBookApi(id: string | null): Promise<void> {
  return req('/api/books/favorite', {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ id }),
  })
}

export function deleteBookApi(id: string): Promise<void> {
  return req(`/api/books/${encodeURIComponent(id)}`, { method: 'DELETE' })
}

/** The active book is remembered per subject ('' = all subjects) */
export function getActiveBookApi(subject: string): Promise<string | null> {
  return req<{ id: string | null }>(
    `/api/books/active?subject=${encodeURIComponent(subject)}`,
  ).then((r) => r.id)
}

export function setActiveBookApi(subject: string, id: string | null): Promise<void> {
  return req('/api/books/active', {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ subject, id }),
  })
}

// ———————— Interrupted sessions (single slot snapshot) ————————

export function getPaused(): Promise<PausedSession | null> {
  return req<PausedSession | null>('/api/paused')
}

export function savePaused(p: PausedSession): Promise<void> {
  return req('/api/paused', {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(p),
  })
}

export function clearPaused(): Promise<void> {
  return req('/api/paused', { method: 'DELETE' })
}
