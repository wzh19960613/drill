import type { BookDef, PausedSession, Question, Rec } from './types'
import type { PrintPayload } from './print'

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
  recursive: boolean
  count: number
  excluded: string[]
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

export function updateSource(
  id: string,
  body: { path?: string; recursive?: boolean },
): Promise<{ id: string; name: string; path: string; recursive: boolean }> {
  return req(`/api/sources/${encodeURIComponent(id)}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
}

export function removeSource(id: string): Promise<void> {
  return req(`/api/sources/${encodeURIComponent(id)}`, { method: 'DELETE' })
}

export interface BrowseInfo {
  path: string
  dirs: string[]

  dirInfo: { name: string; questions: number; other_md: number; questionsAll: number }[]
  questions: { file: string; id: string }[]
  other_md: { file: string; reason: string }[]
  files: string[]
  total: number
}

export function browseSource(id: string, path: string): Promise<BrowseInfo> {
  return req(`/api/sources/${encodeURIComponent(id)}/browse?path=${encodeURIComponent(path)}`)
}

export interface ResolvedImage {
  name: string
  found: boolean
  source: string | null
  dir: string
}

export function resolveImages(source: string, names: string[]): Promise<ResolvedImage[]> {
  return req(`/api/sources/${encodeURIComponent(source)}/resolve-images`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ names }),
  })
}

export interface ClassifyInfo {
  path: string

  question: boolean

  marked: boolean

  reason: string | null
}

export interface FolderMarkImpact {
  path: string
  count: number
  impact: { name: string; count: number }[]
}

/** Dry run of the folder mark: affected questions and the books citing them. */
export function checkSourceFolderMark(source: string, path: string): Promise<FolderMarkImpact> {
  return req(`/api/sources/${encodeURIComponent(source)}/mark-folder`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ path, non_question: true, check: true }),
  })
}

/** Mark or unmark a whole folder as non-questions (covers its subtree). */
export function markSourceFolder(source: string, path: string, nonQuestion: boolean): Promise<void> {
  return req(`/api/sources/${encodeURIComponent(source)}/mark-folder`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ path, non_question: nonQuestion }),
  })
}

export function getSourceFile(source: string, path: string): Promise<{ markdown: string }> {
  return req(
    `/api/sources/${encodeURIComponent(source)}/file?path=${encodeURIComponent(path)}`,
  )
}

export function putSourceFile(
  source: string,
  path: string,
  markdown: string,
): Promise<ClassifyInfo> {
  return req(`/api/sources/${encodeURIComponent(source)}/file`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ path, markdown }),
  })
}

export function deleteSourceFile(source: string, path: string): Promise<void> {
  return req(
    `/api/sources/${encodeURIComponent(source)}/file?path=${encodeURIComponent(path)}`,
    { method: 'DELETE' },
  )
}

export function markSourceFile(
  source: string,
  path: string,
  nonQuestion: boolean,
): Promise<ClassifyInfo> {
  return req(`/api/sources/${encodeURIComponent(source)}/mark`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ path, non_question: nonQuestion }),
  })
}

export interface TempUpload {

  id: string

  name: string
}

export async function uploadImage(file: File): Promise<TempUpload> {
  const r = await fetch(`/api/uploads?name=${encodeURIComponent(file.name)}`, {
    method: 'POST',
    headers: { 'Content-Type': file.type || 'application/octet-stream' },
    body: await file.arrayBuffer(),
  })
  await assertOk(r, `上传图片失败（${r.status}）`)
  return r.json()
}

export function discardTemp(id: string): Promise<void> {
  return req(`/api/uploads/${encodeURIComponent(id)}`, { method: 'DELETE' })
}

export interface QuestionRaw {
  id: string
  source: string
  markdown: string

  dir: string

  file: string
}

export function fetchQuestionRaw(id: string, source: string): Promise<QuestionRaw> {
  return req(`/api/questions/${encodeURIComponent(id)}/raw?source=${encodeURIComponent(source)}`)
}

export function previewQuestion(markdown: string, source: string): Promise<Question> {
  return req('/api/questions/preview', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ markdown, source }),
  })
}

export interface SaveImagePayload {
  temp: string
  name: string
}

export interface SaveQuestionPayload {
  source: string
  dir: string
  filename: string
  markdown: string
  images?: SaveImagePayload[]
  renames?: { from: string; to: string }[]

  original?: string
}

export function saveQuestion(payload: SaveQuestionPayload): Promise<Question> {
  return req('/api/questions/save', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
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
