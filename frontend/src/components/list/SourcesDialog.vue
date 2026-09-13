<script setup lang="ts">
import { onMounted, ref } from 'vue'
import {
  ChevronDown,
  ChevronRight,
  Ellipsis,
  Eye,
  FilePlus,
  FileX2,
  Folder,
  FolderInput,
  FolderOpen,
  FolderPlus,
  PencilLine,
  RefreshCw,
  Trash2,
} from 'lucide-vue-next'
import {
  deleteSourceFile,
  fetchBooks,
  fetchSources,
  markSourceFile,
  markSourceFolder,
  checkSourceFolderMark,
  type SourceInfo,
} from '../../api'
import type { Question } from '../../types'
import { loadAll, questionByKey } from '../../store'
import { useAnchoredMenu } from '../../composables/useAnchoredMenu'
import { useDialogShell } from '../../composables/useDialogShell'
import {
  dirOf,
  fileRel,
  fileNameOf,
  useSourceTree,
} from '../../composables/useSourceTree'
import { useSourceManage } from '../../composables/useSourceManage'
import DialogHeader from '../DialogHeader.vue'
import ConfirmDialog from '../ConfirmDialog.vue'
import MenuPop, { type MenuPopItem } from '../MenuPop.vue'
import FilePreviewDialog from './FilePreviewDialog.vue'
import SourceTreeFolder from './SourceTreeFolder.vue'
import CollapseBox from './CollapseBox.vue'
import FileEditDialog from './FileEditDialog.vue'
import SourcePathDialog from './SourcePathDialog.vue'


const emit = defineEmits<{
  (e: 'close'): void
  (e: 'changed'): void
  (e: 'open-question', q: Question, mode: 'view' | 'edit'): void
  (e: 'create-in', source: string, dir: string): void
}>()

const sources = ref<SourceInfo[]>([])
const srcMsg = ref('')

const srcMsgBad = ref(false)

function msg(text: string, bad = false) {
  srcMsg.value = text
  srcMsgBad.value = bad
}

const tree = useSourceTree()
const {
  retryBrowse,
  toggleExpand,
  refreshFolder,
  refreshSource,
  dropSource,
  subtree,
  rowBrowse,
  rowLoading,
  isExpanded,
  isLoading,
  browseOf,
  hasBrowse,
  hasFailed,
  dirInfoOf,
} = tree


const previewFile = ref<{ source: string; path: string } | null>(null)
const editFile = ref<{ source: string; path: string } | null>(null)
const pathDialog = ref<{ mode: 'add' | 'relocate'; source?: SourceInfo } | null>(null)
const confirmDelFile = ref<{ source: string; path: string; dir: string } | null>(null)

async function refresh() {
  try {
    sources.value = await fetchSources()
    sourcesFailed.value = false
  } catch {
    sourcesFailed.value = true
  }
}

const sourcesFailed = ref(false)

function isDirMarked(source: string, dir: string): boolean {
  return sources.value.find((s) => s.id === source)?.excluded.includes(dir) ?? false
}

async function reloadEverywhere() {
  await refresh()
  emit('changed')
}

const {
  confirmDelSource,
  delImpact,
  askRemoveSource,
  removeConfirmed,
  confirmRecursive,
  toggleRecursive,
  confirmRecursiveOff,
} = useSourceManage({ tree, msg, reload: reloadEverywhere })


async function onPathDone() {
  const mode = pathDialog.value?.mode
  const s = pathDialog.value?.source
  if (s) dropSource(s.id)
  pathDialog.value = null
  await reloadEverywhere()
  msg(mode === 'add' ? '已添加题源' : '已重定位')
}




interface MenuCtx {
  kind: 'q' | 'other' | 'source' | 'dir'
  source: string
  dir: string
  file: string
  id?: string
  src?: SourceInfo
  marked?: boolean
}

const menuAnchor = useAnchoredMenu('right')
const menuCtx = ref<MenuCtx | null>(null)

function openMenu(evt: MouseEvent, ctx: MenuCtx) {
  menuCtx.value = ctx
  menuAnchor.toggle(evt)
}

function menuItems(): MenuPopItem[] {
  const ctx = menuCtx.value
  if (!ctx) return []
  if (ctx.kind === 'source') {
    const s = ctx.src!
    const items: MenuPopItem[] = [
      { key: 'recursive', label: '包含子文件夹', checked: s.recursive },
      { key: 'createIn', label: '在此新建题目', icon: FilePlus },
    ]
    if (!s.exists) items.push({ key: 'relocate', label: '重定位…', icon: FolderInput })
    items.push({ key: 'remove', label: '移除题源', icon: Trash2, danger: true })
    return items
  }
  if (ctx.kind === 'dir') {
    if (ctx.marked) return [{ key: 'unmarkDir', label: '取消「非题目」标记', icon: RefreshCw }]
    return [
      { key: 'markDir', label: '标记为非题目', icon: FileX2, danger: true },
      { key: 'createIn', label: '在此新建题目', icon: FilePlus },
    ]
  }
  if (ctx.kind === 'q') {
    return [
      { key: 'view', label: '查看', icon: Eye },
      { key: 'edit', label: '编辑', icon: PencilLine },
      { key: 'unmark', label: '标记为非题目', icon: FileX2 },
      { key: 'delete', label: '删除文件', icon: Trash2, danger: true },
    ]
  }
  return [
    { key: 'preview', label: '查看', icon: Eye },
    { key: 'edit', label: '编辑', icon: PencilLine },
    { key: 'identify', label: '尝试识别为题目', icon: RefreshCw },
    { key: 'delete', label: '删除文件', icon: Trash2, danger: true },
  ]
}

async function onMenuSelect(item: MenuPopItem) {
  const ctx = menuCtx.value
  if (!ctx) return
  if (ctx.kind === 'source') {
    await onSourceMenu(ctx, item)
    return
  }
  if (ctx.kind === 'dir') {
    await onDirMenu(ctx, item)
    return
  }
  if (ctx.kind === 'q') {
    await onQuestionMenu(ctx, item)
    return
  }
  await onOtherMenu(ctx, item)
}

async function onSourceMenu(ctx: MenuCtx, item: MenuPopItem) {
  const s = ctx.src!
  if (item.key === 'recursive') await toggleRecursive(s)
  else if (item.key === 'createIn') emit('create-in', ctx.source, '')
  else if (item.key === 'relocate') pathDialog.value = { mode: 'relocate', source: s }
  else if (item.key === 'remove') void askRemoveSource(s)
}

async function onDirMenu(ctx: MenuCtx, item: MenuPopItem) {
  if (item.key === 'createIn') {
    emit('create-in', ctx.source, ctx.dir)
    return
  }
  if (item.key === 'markDir') {
    await askFolderMark(ctx.source, ctx.dir)
    return
  }
  await applyFolderMark(ctx.source, ctx.dir, false)
}

/** Folder-level mark with the same impact confirmation as the recursive
 *  toggle: books citing questions under the subtree are listed first. */
const confirmDirMark = ref<{ source: string; dir: string; count: number; impact: { name: string; count: number }[] } | null>(null)

async function askFolderMark(source: string, dir: string) {
  msg('')
  try {
    const r = await checkSourceFolderMark(source, dir)
    if (r.count > 0) {
      confirmDirMark.value = { source, dir, count: r.count, impact: r.impact }
      return
    }
  } catch {
    // impact preview failure should not block the confirmation-less path
  }
  await applyFolderMark(source, dir, true)
}

async function onConfirmDirMark() {
  const c = confirmDirMark.value
  confirmDirMark.value = null
  if (c) await applyFolderMark(c.source, c.dir, true)
}

/** Apply the folder mark; only the marked folder itself collapses, the rest
 *  of the tree keeps its expansion state (caches refresh in place). */
async function applyFolderMark(source: string, dir: string, on: boolean) {
  msg('')
  try {
    await markSourceFolder(source, dir, on)
    if (on) tree.collapseDir(source, dir)
    await refreshSource(source)
    await reloadEverywhere()
    msg(on ? `已将「${dir}」标记为非题目` : `已取消「${dir}」的非题目标记`)
  } catch (e) {
    msg(e instanceof Error ? e.message : '标记失败', true)
  }
}

async function onQuestionMenu(ctx: MenuCtx, item: MenuPopItem) {
  if (item.key === 'view' || item.key === 'edit') {
    await openQuestion(ctx, item.key === 'view' ? 'view' : 'edit')
    return
  }
  if (item.key === 'unmark') await askMark(ctx)
  else if (item.key === 'delete') await askDeleteFile(ctx)
}

async function onOtherMenu(ctx: MenuCtx, item: MenuPopItem) {
  const rel = fileRel(ctx.dir, ctx.file)
  if (item.key === 'preview') previewFile.value = { source: ctx.source, path: rel }
  else if (item.key === 'edit') editFile.value = { source: ctx.source, path: rel }
  else if (item.key === 'identify') await tryIdentify(ctx.source, ctx.dir, ctx.file)
  else if (item.key === 'delete') await askDeleteFile(ctx)
}

async function openQuestion(ctx: MenuCtx, mode: 'view' | 'edit') {
  const k = `${ctx.source}\u0000${ctx.id ?? ''}`
  const found = questionByKey.value.get(k) ?? (await refreshAndFind(k))
  if (!found) {
    msg('刷新后仍未找到这道题，请稍后再试', true)
    return
  }
  emit('open-question', found, mode)
}

async function refreshAndFind(k: string) {
  msg('正在刷新题目列表…')
  await reloadEverywhere()
  await loadAll(true)
  return questionByKey.value.get(k)
}


async function fileImpact(ctx: MenuCtx): Promise<string[]> {
  if (ctx.kind !== 'q') return []
  try {
    const books = await fetchBooks()
    return books
      .filter((b) => b.items.some((it) => it.source === ctx.source && it.id === ctx.id))
      .map((b) => b.name)
  } catch {
    return []
  }
}


const confirmMark = ref<MenuCtx | null>(null)
const markImpact = ref<string[]>([])

async function askMark(ctx: MenuCtx) {
  markImpact.value = await fileImpact(ctx)
  confirmMark.value = ctx
}

async function applyMark() {
  const ctx = confirmMark.value
  confirmMark.value = null
  if (!ctx) return
  msg('')
  try {
    await markSourceFile(ctx.source, fileRel(ctx.dir, ctx.file), true)
    msg(`已标记为非题目：${ctx.file}（文件保留）`)
    await refreshFolder(ctx.source, ctx.dir)
    await reloadEverywhere()
  } catch (e) {
    msg(e instanceof Error ? e.message : '标记失败', true)
  }
}


const delFileImpact = ref<string[]>([])

async function askDeleteFile(ctx: MenuCtx) {
  delFileImpact.value = await fileImpact(ctx)
  confirmDelFile.value = { source: ctx.source, path: fileRel(ctx.dir, ctx.file), dir: ctx.dir }
}

async function tryIdentify(source: string, dir: string, file: string) {
  msg('')
  try {
    const info = await markSourceFile(source, fileRel(dir, file), false)
    if (info.question) {
      msg(`已识别为题目：${file}`)
      await reloadEverywhere()
    } else {
      msg(`${file} 仍不是题目：${info.reason ?? '原因未知'}`, true)
    }
    await refreshFolder(source, dir)
  } catch (e) {
    msg(e instanceof Error ? e.message : '识别失败', true)
  }
}

async function deleteFile(source: string, dir: string, file: string) {
  msg('')
  try {
    await deleteSourceFile(source, fileRel(dir, file))
    confirmDelFile.value = null
    msg(`已删除文件：${file}`)
    await refreshFolder(source, dir)
    await reloadEverywhere()
  } catch (e) {
    msg(e instanceof Error ? e.message : '删除文件失败', true)
  }
}

async function onFileSaved(source: string, dir: string, info: { question: boolean }) {
  await refreshFolder(source, dir)
  await reloadEverywhere()
  if (info.question) msg('已识别为题目，文件重新进入题库')
}




useDialogShell((e) => {
  if (e.code !== 'Escape') return
  if (
    pathDialog.value ||
    previewFile.value ||
    editFile.value ||
    confirmDelFile.value ||
    confirmDelSource.value ||
    confirmRecursive.value ||
    confirmDirMark.value ||
    confirmMark.value ||
    menuAnchor.open.value
  ) {
    return
  }
  e.stopPropagation()
  emit('close')
})

onMounted(refresh)
</script>

<template>
  <Teleport to="body">
  <div class="modal-mask fs" @click.self="emit('close')">
    <div class="src-pop dialog fs">
      <DialogHeader no-border title="题源" @close="emit('close')" />
      <div class="src-body">
      <div v-if="sourcesFailed" class="tree-hint err">
        题源列表读取失败
        <button type="button" class="tree-retry" @click="refresh()">重试</button>
      </div>
      <div v-for="s in sources" :key="s.id" class="src-item">
        <div class="src-row">
          <button
            class="row-btn src-chev"
            :aria-label="isExpanded(s.id, '') ? '收起' : '展开浏览'"
            @click="toggleExpand(s.id, '')"
          >
            <component
              :is="isExpanded(s.id, '') ? ChevronDown : ChevronRight"
              style="width: 1.0625rem; height: 1.0625rem"
            />
          </button>
          <div class="src-info" @click="toggleExpand(s.id, '')">
            <span class="src-name">{{ s.name }}<span v-if="!s.exists" class="src-miss"> · 文件夹不存在，请重定位</span></span>
            <span class="src-dir">{{ s.path }}</span>
          </div>
          <span class="chip" :title="s.recursive ? '包含子文件夹' : '不含子文件夹'">{{ s.count }} 题</span>
          <span class="src-wide">
            <button
              class="row-btn"
              aria-label="在此新建题目（保存到该题源根目录）"
              title="在此新建题目（保存到该题源根目录）"
              @click="emit('create-in', s.id, '')"
            >
              <FilePlus style="width: 1rem; height: 1rem" />
            </button>
            <button
              class="rec-switch"
              :class="{ on: s.recursive }"
              role="switch"
              :aria-checked="s.recursive"
              aria-label="包含子文件夹"
              @click="toggleRecursive(s)"
            >
              <span class="rec-knob"></span>
            </button>
            <span class="rec-label" @click="toggleRecursive(s)">子文件夹</span>
            <button
              v-if="!s.exists"
              class="row-btn"
              aria-label="重定位该题源（文件夹移动后使用）"
              @click="pathDialog = { mode: 'relocate', source: s }"
            >
              <FolderInput style="width: 1rem; height: 1rem" />
            </button>
            <button class="row-btn src-x" aria-label="移除该题源" @click="askRemoveSource(s)">
              <Trash2 style="width: 1rem; height: 1rem" />
            </button>
          </span>
          <button
            class="row-btn dots src-dots"
            aria-label="题源操作"
            @click="openMenu($event, { kind: 'source', source: s.id, dir: '', file: '', src: s })"
          >
            <Ellipsis style="width: 1.125rem; height: 1.125rem" />
          </button>
        </div>

        <div v-if="isExpanded(s.id, '')" class="src-tree">
          <div v-if="isLoading(s.id, '')" class="tree-hint">读取中…</div>
          <template v-else>
            <SourceTreeFolder
              :source="s.id"
              dir=""
              :browse="browseOf(s.id, '')"
              :failed="hasFailed(s.id, '')"
              @menu="(evt, c) => openMenu(evt, { ...c, source: s.id, dir: '' })"
              @retry="retryBrowse(s.id, '')"
            />

            <template v-if="s.recursive">
              <template v-for="row in subtree(s)" :key="row.dir">
                <div
                  class="folder-l1"
                  :style="{ paddingInlineStart: `${(row.depth - 1) * 2}rem` }"
                >
                  <button class="chevbtn" aria-label="展开/收起" @click="toggleExpand(row.source, row.dir)">
                    <component
                      :is="isExpanded(row.source, row.dir) ? ChevronDown : ChevronRight"
                      class="chev"
                    />
                  </button>
                  <component
                    :is="isExpanded(row.source, row.dir) ? FolderOpen : Folder"
                    style="width: 1rem; height: 1rem; flex: none; color: var(--muted)"
                  />
                  <span class="tf-name">{{ row.name }}</span>
                  <span v-if="rowLoading(row)" class="tf-why">读取中…</span>
                  <span v-else-if="dirInfoOf(row)" class="tf-why">{{ dirInfoOf(row)!.questionsAll }} 题</span>
                  <span v-if="isDirMarked(row.source, row.dir)" class="dir-mark">非题目</span>
                  <button
                    class="dots"
                    aria-label="文件夹操作"
                    @click.stop="openMenu($event, { kind: 'dir', source: row.source, dir: row.dir, file: '', marked: isDirMarked(row.source, row.dir) })"
                  >
                    <Ellipsis style="width: 1.0625rem; height: 1.0625rem" />
                  </button>
                </div>
                <CollapseBox :open="isExpanded(row.source, row.dir)">
                  <SourceTreeFolder
                    :source="row.source"
                    :dir="row.dir"
                    :depth="row.depth"
                    :browse="rowBrowse(row)"
                    :failed="hasFailed(row.source, row.dir)"
                    @menu="(evt, c) => openMenu(evt, { ...c, source: row.source, dir: row.dir })"
                    @retry="retryBrowse(row.source, row.dir)"
                  />
                </CollapseBox>
              </template>
            </template>

            <div v-if="hasFailed(s.id, '')" class="tree-hint err">
              文件夹读取失败
              <button type="button" class="tree-retry" @click="retryBrowse(s.id, '')">重试</button>
            </div>
            <div
              v-else-if="hasBrowse(s.id, '') && !(browseOf(s.id, '')?.questions.length) && !(browseOf(s.id, '')?.other_md.length) && !(s.recursive && browseOf(s.id, '')?.dirs.length)"
              class="tree-hint"
            >
              此文件夹为空
            </div>
          </template>
        </div>
      </div>

      <button class="btn block src-add" @click="pathDialog = { mode: 'add' }">
        <FolderPlus style="width: 1.0625rem; height: 1.0625rem" /> 添加题源
      </button>
      <div v-if="srcMsg" class="src-msg" :class="{ bad: srcMsgBad }">{{ srcMsg }}</div>
      </div>
    </div>

    <MenuPop
      :open="menuAnchor.open.value"
      :x="menuAnchor.x.value"
      :y="menuAnchor.y.value"
      :items="menuItems()"
      @select="onMenuSelect"
      @close="menuAnchor.close"
    />

    <SourcePathDialog
      v-if="pathDialog"
      :mode="pathDialog.mode"
      :source="pathDialog.source"
      @done="onPathDone"
      @close="pathDialog = null"
    />

    <FilePreviewDialog
      v-if="previewFile"
      :source="previewFile.source"
      :path="previewFile.path"
      @close="previewFile = null"
    />
    <FileEditDialog
      v-if="editFile"
      :source="editFile.source"
      :path="editFile.path"
      @close="editFile = null"
      @saved="(info) => onFileSaved(editFile!.source, dirOf(editFile!.path), info)"
    />

    <ConfirmDialog
      v-if="confirmDelFile"
      nested
      width="25rem"
      @confirm="deleteFile(confirmDelFile!.source, confirmDelFile!.dir, fileNameOf(confirmDelFile!.path))"
      @cancel="confirmDelFile = null"
    >
      确定删除文件「<b>{{ fileNameOf(confirmDelFile!.path) }}</b>」？删除后无法恢复。
      <template v-if="delFileImpact.length">
        <br />
        它被 {{ delFileImpact.length }} 个题本（{{ delFileImpact.join('、') }}）引用，删除后对应题目会失效。
      </template>
    </ConfirmDialog>

    <ConfirmDialog
      v-if="confirmMark"
      nested
      width="25rem"
      confirm-text="标记"
      @confirm="applyMark"
      @cancel="confirmMark = null"
    >
      确定将「<b>{{ confirmMark!.file }}</b>」标记为非题目？文件保留、可随时恢复。
      <template v-if="markImpact.length">
        <br />
        它被 {{ markImpact.length }} 个题本（{{ markImpact.join('、') }}）引用，标记后对应题目会失效。
      </template>
    </ConfirmDialog>

    <ConfirmDialog
      v-if="confirmDelSource"
      nested
      width="25rem"
      confirm-text="移除"
      @confirm="removeConfirmed(confirmDelSource!)"
      @cancel="confirmDelSource = null"
    >
      确定移除题源「<b>{{ confirmDelSource!.name }}</b>」？只是不再读取该文件夹，不会删除任何文件。
      <template v-if="delImpact.length">
        <br /><br />
        以下题本的题目来自此题源，移除后会失效（重新添加或重定位题源可恢复）：
        <ul class="del-impact">
          <li v-for="b in delImpact" :key="b.name">{{ b.name }} · {{ b.count }} 题</li>
        </ul>
      </template>
    </ConfirmDialog>

    <ConfirmDialog
      v-if="confirmDirMark"
      nested
      width="25rem"
      confirm-text="标记"
      @confirm="onConfirmDirMark"
      @cancel="confirmDirMark = null"
    >
      将「<b>{{ confirmDirMark!.dir || '根目录' }}</b>」标记为非题目？该文件夹（含子文件夹）下
      <b>{{ confirmDirMark!.count }}</b> 道题目不再显示。
      <template v-if="confirmDirMark!.impact.length">
        <br />
        以下题本中的题目会失效（取消标记即可恢复）：
        <ul class="del-impact">
          <li v-for="b in confirmDirMark!.impact" :key="b.name">{{ b.name }} · {{ b.count }} 题</li>
        </ul>
      </template>
      <template v-else>取消标记即可恢复。</template>
    </ConfirmDialog>

    <ConfirmDialog
      v-if="confirmRecursive"
      nested
      width="25rem"
      confirm-text="关闭"
      @confirm="confirmRecursiveOff"
      @cancel="confirmRecursive = null"
    >
      关闭「子文件夹」后，<b>{{ confirmRecursive!.subCount }}</b> 道子文件夹里的题目不再显示。
      <template v-if="confirmRecursive!.impact.length">
        <br />
        以下题本中的题目会失效（重新打开开关即可恢复）：
        <ul class="del-impact">
          <li v-for="b in confirmRecursive!.impact" :key="b.name">{{ b.name }} · {{ b.count }} 题</li>
        </ul>
      </template>
      <template v-else>重新打开开关即可恢复。</template>
    </ConfirmDialog>
  </div>
  </Teleport>
</template>

<style scoped>
.src-pop {
  width: min(52rem, 94vw);
  max-height: 86vh;
}

.src-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: clip;
  overscroll-behavior: contain;
  padding: 0.5rem 1.25rem 1.25rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.src-item {
  border-bottom: 1px dashed var(--line);
  padding-bottom: 0.625rem;
}

.src-row {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.3125rem 2px;
}

.row-btn {
  flex: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: var(--muted);
  width: 2.25rem;
  height: 2.25rem;
  border-radius: 0.5rem;
  cursor: pointer;
}

.row-btn:hover {
  background: var(--hover);
  color: var(--ink);
}

.src-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
  cursor: pointer;
}

.src-name {
  font-size: 0.9375rem;
  font-weight: 600;
}

.src-miss {
  color: var(--warn);
  font-weight: 400;
  font-size: 0.8125rem;
}

.src-dir {
  font-size: 0.8125rem;
  color: var(--muted);
  font-family: var(--font-mono);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.src-wide {
  flex: none;
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
}

.src-dots {
  display: none;
}

@container (max-width: 48rem) {
  .src-wide {
    display: none;
  }

  .src-dots {
    display: inline-flex;
  }
}

.rec-switch {
  flex: none;
  width: 2.25rem;
  height: 1.3125rem;
  border-radius: var(--radius-pill);
  border: 1px solid var(--line);
  background: var(--panel);
  position: relative;
  cursor: pointer;
  padding: 0;
  transition: background 0.15s ease;
}

.rec-switch .rec-knob {
  position: absolute;
  top: 1px;
  left: 1px;
  width: 1.125rem;
  height: 1.125rem;
  border-radius: var(--radius-pill);
  background: var(--card);
  box-shadow: var(--shadow-inset);
  transition: left 0.15s ease;
}

.rec-switch.on {
  background: var(--brand);
  border-color: var(--brand);
}

.rec-switch.on .rec-knob {
  left: calc(100% - 1.1875rem);
}

.rec-label {
  flex: none;
  font-size: 0.8125rem;
  color: var(--muted);
  cursor: pointer;
  user-select: none;
}

.src-x:hover {
  color: var(--bad);
}

.src-tree {
  margin: 0.25rem 0 0.375rem 1.375rem;
  border-left: 2px solid var(--line);
  padding-left: 0.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.0625rem;
}

/* Folder rows sit in the level-1 column: chevron slot (1.75rem) + folder icon
   (1rem), text sharing the exact edge the group headers use (SourceTreeFolder) */
.folder-l1 {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 0.875rem;
  min-height: 2.25rem;
  padding-inline-end: 0.25rem;
  border-radius: 0.375rem;
}

.folder-l1:hover {
  background: var(--hover);
}

.folder-l1 .chevbtn {
  flex: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  border: none;
  background: transparent;
  color: var(--muted);
  border-radius: 0.375rem;
  cursor: pointer;
  padding: 0;
}

.folder-l1 .chevbtn:hover {
  background: var(--hover);
  color: var(--ink);
}

.folder-l1 .chev {
  flex: none;
  width: 1rem;
  height: 1rem;
  color: var(--muted);
}

.tf-name {
  font-family: var(--font-mono);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.tf-why {
  font-size: 0.75rem;
  color: var(--muted);
  white-space: nowrap;
}

.folder-l1 .tf-why {
  margin-left: auto;
}

.folder-l1 .dots {
  flex: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2.25rem;
  height: 2.25rem;
  padding: 0;
  margin-left: 0.25rem;
  border: none;
  background: none;
  color: var(--muted);
  border-radius: 0.5rem;
  cursor: pointer;
}

.folder-l1 .dots:hover {
  background: var(--hover);
  color: var(--ink);
}

.dir-mark {
  flex: none;
  margin-left: 0.375rem;
  font-size: 0.6875rem;
  color: var(--warn);
  background: var(--panel);
  border-radius: var(--radius-pill);
  padding: 0.0938rem 0.5rem;
}

.tree-hint {
  font-size: 0.8125rem;
  color: var(--muted);
  padding: 0.5rem;
}

.tree-hint.err {
  color: var(--bad);
}

.tree-retry {
  border: none;
  background: none;
  color: var(--brand);
  font-size: inherit;
  padding: 0 0.25rem;
  cursor: pointer;
  text-decoration: underline;
}

.src-add {
  margin-top: 0.25rem;
}

.src-msg {
  font-size: 0.8125rem;
  color: var(--good);
}

.src-msg.bad {
  color: var(--bad);
}

.del-impact {
  margin: 0.375rem 0 0;
  padding-inline-start: 1.25rem;
  text-align: left;
}
</style>
