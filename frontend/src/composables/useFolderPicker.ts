import { computed, ref, type ComputedRef, type Ref } from 'vue'
import { browseSource, type BrowseInfo, type SourceInfo } from '../api'

export interface TreeNode {
  source: string
  dir: string
  name: string
  depth: number
  expandable: boolean
  open: boolean
  loading: boolean
}

const key = (source: string, dir: string) => `${source}\u0000${dir}`

async function loadBrowseInto(
  fetch: (source: string, dir: string) => Promise<BrowseInfo>,
  source: string,
  dir: string,
  tree: Ref<Map<string, BrowseInfo>>,
  loading: Ref<boolean>,
  loadErr: Ref<string>,
) {
  if (tree.value.has(key(source, dir))) return
  loading.value = true
  loadErr.value = ''
  try {
    const b = await fetch(source, dir)
    tree.value.set(key(source, dir), b)
  } catch (e) {
    loadErr.value = e instanceof Error ? e.message : '读取文件夹失败'
  } finally {
    loading.value = false
  }
}

function visibleNodes(
  sources: SourceInfo[],
  tree: Map<string, BrowseInfo>,
  expanded: Set<string>,
): TreeNode[] {
  const nodes: TreeNode[] = []
  for (const s of sources) nodes.push(...sourceNodes(s, tree, expanded))
  return nodes
}

function sourceNodes(
  s: SourceInfo,
  tree: Map<string, BrowseInfo>,
  expanded: Set<string>,
): TreeNode[] {
  const nodes: TreeNode[] = []
  const rootKey = key(s.id, '')
  const rootBrowse = tree.get(rootKey)
  nodes.push({
    source: s.id,
    dir: '',
    name: s.name,
    depth: 0,
    expandable: s.recursive,
    open: expanded.has(rootKey),
    loading: s.recursive && expanded.has(rootKey) && !rootBrowse,
  })
  if (!s.recursive || !expanded.has(rootKey)) return nodes
  const walk = (dir: string, depth: number) => {
    const b = tree.get(key(s.id, dir))
    if (!b) return
    for (const sub of b.dirs) {
      const child = dir ? `${dir}/${sub}` : sub
      const k2 = key(s.id, child)
      const cb = tree.get(k2)
      nodes.push({
        source: s.id,
        dir: child,
        name: sub,
        depth,
        expandable: !!cb?.dirs.length,
        open: expanded.has(k2),
        loading: expanded.has(k2) && !cb,
      })
      if (cb && expanded.has(k2)) walk(child, depth + 1)
    }
  }
  walk('', 1)
  return nodes
}

export function useFolderPicker(sources: ComputedRef<SourceInfo[]> | Ref<SourceInfo[]>) {
  const sel = ref({ source: '', dir: '' })
  const tree = ref(new Map<string, BrowseInfo>())
  const expanded = ref(new Set<string>())
  const loading = ref(false)
  const loadErr = ref('')

  const browseOf = (source: string, dir: string) => tree.value.get(key(source, dir))

  const loadBrowse = (source: string, dir: string) =>
    loadBrowseInto(browseSource, source, dir, tree, loading, loadErr)

  function expandAncestors(source: string, dir: string) {
    expanded.value.add(key(source, ''))
    if (!dir) return
    let cur = ''
    for (const part of dir.split('/')) {
      cur = cur ? `${cur}/${part}` : part
      expanded.value.add(key(source, cur))
    }
  }

  async function toggleNode(n: TreeNode) {
    if (!n.expandable) return
    const k2 = key(n.source, n.dir)
    if (expanded.value.has(k2)) expanded.value.delete(k2)
    else {
      expanded.value.add(k2)
      await loadBrowse(n.source, n.dir)
    }
  }

  const treeNodes = computed<TreeNode[]>(() =>
    visibleNodes(sources.value, tree.value, expanded.value),
  )

  return { sel, tree, expanded, loading, loadErr, browseOf, loadBrowse, expandAncestors, toggleNode, treeNodes }
}
