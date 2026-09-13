import { ref, type Ref } from 'vue'
import { browseSource, type BrowseInfo, type SourceInfo } from '../api'

export interface TreeRow {
  source: string
  dir: string
  name: string
  depth: number
}

const key = (source: string, dir: string) => `${source}\u0000${dir}`

export function useSourceTree() {
  const browses: Ref<Map<string, BrowseInfo>> = ref(new Map())
  const expanded: Ref<Set<string>> = ref(new Set())
  const loadingKey = ref('')

  const browseFailed: Ref<Set<string>> = ref(new Set())


  const ensureBrowse = (source: string, dir: string) =>
    fetchInto(browseSource, source, dir, browses, loadingKey, browseFailed)
  const refreshFolder = (source: string, dir: string) =>
    forceBrowse(browseSource, source, dir, browses, browseFailed)
  const subtree = (s: SourceInfo) => subtreeRows(s, browses.value, expanded.value)

  const retryBrowse = (source: string, dir: string) => retryBrowseOf(source, dir, browseFailed, ensureBrowse)
  const toggleExpand = (source: string, dir: string) =>
    toggleExpandOf(source, dir, expanded, ensureBrowse)
  const dropSource = (source: string) => dropSourceOf(source, browses)

  /** Re-fetch every cached folder of the source; expansion stays untouched */
  async function refreshSource(source: string) {
    for (const k of [...browses.value.keys()]) {
      if (k.startsWith(`${source}\u0000`)) await refreshFolder(source, k.slice(source.length + 1))
    }
  }

  function dirInfoOf(row: TreeRow): { questions: number; other_md: number; questionsAll: number } | undefined {
    const parent = browses.value.get(key(row.source, dirOf(row.dir)))
    return parent?.dirInfo?.find((d) => d.name === row.name)
  }

  const k = (source: string, dir: string) => key(source, dir)
  return {
    browses, expanded, ensureBrowse, retryBrowse, toggleExpand,
    refreshFolder, dropSource, subtree, dirInfoOf, refreshSource,
    rowBrowse: (row: TreeRow) => browses.value.get(k(row.source, row.dir)),
    rowLoading: (row: TreeRow) => loadingKey.value === k(row.source, row.dir),
    isExpanded: (s: string, d: string) => expanded.value.has(k(s, d)),
    isLoading: (s: string, d: string) => loadingKey.value === k(s, d),
    browseOf: (s: string, d: string) => browses.value.get(k(s, d)),
    hasBrowse: (s: string, d: string) => browses.value.has(k(s, d)),
    hasFailed: (s: string, d: string) => browseFailed.value.has(k(s, d)),
    collapseRoot: (source: string) => expanded.value.delete(k(source, '')),
    collapseDir: (source: string, dir: string) => expanded.value.delete(k(source, dir)),
  }
}

async function retryBrowseOf(
  source: string,
  dir: string,
  browseFailed: Ref<Set<string>>,
  ensureBrowse: (source: string, dir: string) => Promise<BrowseInfo | null>,
) {
  browseFailed.value.delete(key(source, dir))
  await ensureBrowse(source, dir)
}

async function toggleExpandOf(
  source: string,
  dir: string,
  expanded: Ref<Set<string>>,
  ensureBrowse: (source: string, dir: string) => Promise<BrowseInfo | null>,
) {
  const k = key(source, dir)
  if (expanded.value.has(k)) expanded.value.delete(k)
  else {
    expanded.value.add(k)
    await ensureBrowse(source, dir)
  }
}

function dropSourceOf(source: string, browses: Ref<Map<string, BrowseInfo>>) {
  for (const k of [...browses.value.keys()]) {
    if (k.startsWith(`${source}\u0000`)) browses.value.delete(k)
  }
}

export function fileRel(dir: string, file: string): string {
  return dir ? `${dir}/${file}` : file
}

export function fileNameOf(path: string): string {
  return path.slice(path.lastIndexOf('/') + 1)
}

export function dirOf(path: string): string {
  const i = path.lastIndexOf('/')
  return i < 0 ? '' : path.slice(0, i)
}

async function fetchInto(
  fetch: (source: string, dir: string) => Promise<BrowseInfo>,
  source: string,
  dir: string,
  browses: Ref<Map<string, BrowseInfo>>,
  loadingKey: Ref<string>,
  browseFailed: Ref<Set<string>>,
): Promise<BrowseInfo | null> {
  const k = key(source, dir)
  const hit = browses.value.get(k)
  if (hit) return hit
  loadingKey.value = k
  try {
    const b = await fetch(source, dir)
    browses.value.set(k, b)
    browseFailed.value.delete(k)
    return b
  } catch {
    browseFailed.value.add(k)
    return null
  } finally {
    loadingKey.value = ''
  }
}

async function forceBrowse(
  fetch: (source: string, dir: string) => Promise<BrowseInfo>,
  source: string,
  dir: string,
  browses: Ref<Map<string, BrowseInfo>>,
  browseFailed: Ref<Set<string>>,
): Promise<void> {
  const k = key(source, dir)
  try {
    browses.value.set(k, await fetch(source, dir))
    browseFailed.value.delete(k)
  } catch {
    browseFailed.value.add(k)
  }
}

function subtreeRows(
  s: SourceInfo,
  browses: Map<string, BrowseInfo>,
  expanded: Set<string>,
): TreeRow[] {
  const out: TreeRow[] = []
  if (!s.recursive || !expanded.has(key(s.id, ''))) return out
  const walk = (dir: string, depth: number) => {
    const b = browses.get(key(s.id, dir))
    if (!b) return
    for (const sub of b.dirs) {
      const child = dir ? `${dir}/${sub}` : sub
      out.push({ source: s.id, dir: child, name: sub, depth })
      if (expanded.has(key(s.id, child))) walk(child, depth + 1)
    }
  }
  walk('', 1)
  return out
}
