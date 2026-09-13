import { ref } from 'vue'
import { fetchBooks, removeSource, updateSource, type SourceInfo } from '../api'
import { store } from '../store'
import type { useSourceTree } from './useSourceTree'

function useRecursiveToggle(opts: {
  tree: ReturnType<typeof useSourceTree>
  msg: (t: string, bad?: boolean) => void
  reload: () => Promise<void>
}) {
  const { ensureBrowse } = opts.tree
  const subQuestionIds = (s: SourceInfo) => subIdsOf(s, ensureBrowse)
  const subImpact = (s: SourceInfo, ids: Set<string>) => impactedBooks(s, ids)

  const confirmRecursive = ref<{
    src: SourceInfo
    subCount: number
    impact: { name: string; count: number }[]
  } | null>(null)

  async function toggleRecursive(s: SourceInfo) {
    opts.msg('')
    if (s.recursive) {
      const ids = await subQuestionIds(s)
      if (ids === null) {
        opts.msg('读取文件夹失败，未更改子文件夹设置', true)
        return
      }
      if (ids.size > 0) {
        confirmRecursive.value = { src: s, subCount: ids.size, impact: await subImpact(s, ids) }
        return
      }
    }
    await applyRecursive(s, opts)
  }

  async function confirmRecursiveOff() {
    const c = confirmRecursive.value
    confirmRecursive.value = null
    if (c) await applyRecursive(c.src, opts)
  }

  return { confirmRecursive, toggleRecursive, confirmRecursiveOff }
}

async function applyRecursive(
  s: SourceInfo,
  opts: { tree: ReturnType<typeof useSourceTree>; msg: (t: string, bad?: boolean) => void; reload: () => Promise<void> },
) {
  const { dropSource, collapseRoot } = opts.tree
  try {
    await updateSource(s.id, { recursive: !s.recursive })
    dropSource(s.id)
    collapseRoot(s.id)
    await opts.reload()
  } catch (e) {
    opts.msg(e instanceof Error ? e.message : '设置子文件夹失败', true)
  }
}

function useSourceRemoval(opts: { tree: ReturnType<typeof useSourceTree>; msg: (t: string, bad?: boolean) => void; reload: () => Promise<void> }) {
  const { dropSource } = opts.tree
  const confirmDelSource = ref<SourceInfo | null>(null)

  const delImpact = ref<{ name: string; count: number }[]>([])

  async function askRemoveSource(s: SourceInfo) {
    confirmDelSource.value = s
    delImpact.value = []
    try {
      const books = await fetchBooks()
      delImpact.value = books
        .map((b) => ({ name: b.name, count: b.items.filter((it) => it.source === s.id).length }))
        .filter((x) => x.count > 0)
    } catch {

    }
  }

  async function removeConfirmed(s: SourceInfo) {
    opts.msg('')
    try {
      await removeSource(s.id)
      dropSource(s.id)
      confirmDelSource.value = null
      await opts.reload()
    } catch (e) {
      opts.msg(e instanceof Error ? e.message : '移除题源失败', true)
    }
  }

  return { confirmDelSource, delImpact, askRemoveSource, removeConfirmed }
}

async function subIdsOf(
  s: SourceInfo,
  ensureBrowse: (source: string, dir: string) => Promise<{ questions: { id: string }[] } | null>,
): Promise<Set<string> | null> {
  const root = await ensureBrowse(s.id, '')
  if (!root) return null
  const rootIds = new Set(root.questions.map((q) => q.id))
  return new Set(
    store.questions
      .filter((q) => q.source === s.id && !rootIds.has(q.id))
      .map((q) => q.id),
  )
}

async function impactedBooks(
  s: SourceInfo,
  ids: Set<string>,
): Promise<{ name: string; count: number }[]> {
  try {
    return (await fetchBooks())
      .map((b) => ({
        name: b.name,
        count: b.items.filter((it) => it.source === s.id && ids.has(it.id)).length,
      }))
      .filter((x) => x.count > 0)
  } catch {
    return []
  }
}

export function useSourceManage(opts: {
  tree: ReturnType<typeof useSourceTree>
  msg: (text: string, bad?: boolean) => void
  reload: () => Promise<void>
}) {
  const removal = useSourceRemoval(opts)
  const switching = useRecursiveToggle(opts)

  return {
    confirmDelSource: removal.confirmDelSource,
    delImpact: removal.delImpact,
    askRemoveSource: removal.askRemoveSource,
    removeConfirmed: removal.removeConfirmed,
    confirmRecursive: switching.confirmRecursive,
    toggleRecursive: switching.toggleRecursive,
    confirmRecursiveOff: switching.confirmRecursiveOff,
  }
}
