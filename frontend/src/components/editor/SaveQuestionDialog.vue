<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { ChevronDown, ChevronRight, Folder, FolderOpen, Save } from 'lucide-vue-next'
import type { ResolvedImage, SourceInfo } from '../../api'
import { resolveImages, saveQuestion } from '../../api'
import type { Question } from '../../types'
import {
  applyImageMapping,
  autoImageNames,
  buildDoc,
  defaultFileName,
  extractImageRefs,
  sanitizeFileName,
  type DocMeta,
  type DocSections,
} from '../../mdoc'
import DialogHeader from '../DialogHeader.vue'
import { useDialogShell } from '../../composables/useDialogShell'
import { useFolderPicker, type TreeNode } from '../../composables/useFolderPicker'


const props = defineProps<{
  meta: DocMeta
  sections: DocSections
  temps: Map<string, string>
  sources: SourceInfo[]
  original: { source: string; dir: string; file: string } | null
  initialTarget?: { source: string; dir: string } | null
}>()

const emit = defineEmits<{ (e: 'close'): void; (e: 'saved', q: Question): void }>()

const picker = useFolderPicker(computed(() => props.sources))
const { sel, loading, loadErr, browseOf, loadBrowse, expandAncestors, toggleNode, treeNodes } =
  picker

const filename = ref('')
const filenameDirty = ref(false)

interface ImageRow {
  kind: 'temp' | 'existing'
  
  key: string
  origName: string
  ext: string
  value: string
  dirty: boolean
}
const rows = ref<ImageRow[]>([])
const resolved = ref(new Map<string, ResolvedImage>())
const resolvedLoading = ref(false)

const saving = ref(false)
const errMsg = ref('')


const allMd = computed(() =>
  [props.sections.stem, props.sections.answer, props.sections.solution, props.sections.notes].join('\n'),
)


const existingRefs = computed(() => extractImageRefs(allMd.value).filter((n) => !props.temps.has(n)))

function refreshAutoNames() {
  const files = browseOf(sel.value.source, sel.value.dir)?.files ?? []
  const tempIds = [...props.temps.keys()]
  const auto = autoImageNames(filename.value, tempIds.map((id) => ({ id })), [
    ...files,
    ...rows.value.filter((r) => r.kind === 'existing').map((r) => r.value),
  ])
  let i = 0
  for (const row of rows.value) {
    if (row.kind === 'temp' && !row.dirty) row.value = auto[i] ?? row.value
    if (row.kind === 'temp') i++
  }
}

function rebuildRows() {
  const files = browseOf(sel.value.source, sel.value.dir)?.files ?? []
  const keep = new Map(rows.value.map((r) => [r.kind + r.key, r]))
  const next: ImageRow[] = []
  for (const [id, orig] of props.temps) {
    const old = keep.get('temp' + id)
    next.push({
      kind: 'temp',
      key: id,
      origName: orig,
      ext: id.slice(id.lastIndexOf('.')),
      value: old?.dirty ? old.value : '',
      dirty: old?.dirty ?? false,
    })
  }
  for (const name of existingRefs.value) {
    const old = keep.get('existing' + name)
    next.push({
      kind: 'existing',
      key: name,
      origName: name,
      ext: name.slice(name.lastIndexOf('.')),
      value: old?.value ?? name,
      dirty: old?.dirty ?? false,
    })
  }
  rows.value = next
  if (!filenameDirty.value) filename.value = defaultFileName(props.meta, files)
  refreshAutoNames()
}

const cleanName = computed(() => sanitizeFileName(filename.value))


const isOriginalTarget = computed(
  () =>
    !!props.original &&
    !!cleanName.value &&
    sel.value.source === props.original.source &&
    sel.value.dir === props.original.dir &&
    props.original.file === `${cleanName.value}.md`,
)

const fileConflict = computed(
  () =>
    !!cleanName.value &&
    !isOriginalTarget.value &&
    (browseOf(sel.value.source, sel.value.dir)?.files ?? []).includes(`${cleanName.value}.md`),
)

const movingOrRenaming = computed(() => {
  if (!props.original) return false
  return (
    sel.value.source !== props.original.source ||
    sel.value.dir !== props.original.dir ||
    props.original.file !== `${cleanName.value ?? ''}.md`
  )
})

function rowConflict(row: ImageRow): boolean {
  if (row.kind === 'existing' && row.value === row.key) return false
  const files = browseOf(sel.value.source, sel.value.dir)?.files ?? []
  if (files.includes(row.value)) return true
  return rows.value.filter((r) => r.value === row.value).length > 1
}

function rowInvalid(row: ImageRow): boolean {
  const v = sanitizeFileName(row.value)
  return !v || v !== row.value || v.slice(v.lastIndexOf('.')) !== row.ext
}

const anyConflict = computed(
  () => fileConflict.value || rows.value.some((r) => rowConflict(r) || rowInvalid(r)),
)
const canSubmit = computed(
  () => !!cleanName.value && !fileConflict.value && !anyConflict.value && !saving.value,
)

async function selectNode(n: TreeNode) {
  if (sel.value.source === n.source && sel.value.dir === n.dir) return
  sel.value = { source: n.source, dir: n.dir }
  
  
  
  await loadBrowse(n.source, n.dir)
  if (!filenameDirty.value) {
    filename.value = defaultFileName(props.meta, browseOf(n.source, n.dir)?.files ?? [])
  }
  refreshAutoNames()
}

watch(sel, () => {
  if (!filenameDirty.value) {
    filename.value = defaultFileName(
      props.meta,
      browseOf(sel.value.source, sel.value.dir)?.files ?? [],
    )
  }
  refreshAutoNames()
})

watch(filename, () => refreshAutoNames())

function onFilenameInput(e: Event) {
  filename.value = (e.target as HTMLInputElement).value
  filenameDirty.value = true
}

function resolvedOf(row: ImageRow): ResolvedImage | undefined {
  return row.kind === 'existing' ? resolved.value.get(row.key) : undefined
}


function renameLocked(row: ImageRow): boolean {
  if (row.kind !== 'existing') return false
  const r = resolvedOf(row)
  return !r?.found || r.source !== sel.value.source
}

async function resolveAll() {
  const names = existingRefs.value
  if (!names.length) return
  resolvedLoading.value = true
  try {
    const list = await resolveImages(sel.value.source || props.sources[0]?.id || '', names)
    resolved.value = new Map(list.map((r) => [r.name, r]))
  } catch {
    resolved.value = new Map()
  } finally {
    resolvedLoading.value = false
  }
}

async function submit() {
  if (!canSubmit.value || !cleanName.value) return
  saving.value = true
  errMsg.value = ''
  try {
    const tempPairs: [string, string][] = rows.value
      .filter((r) => r.kind === 'temp')
      .map((r) => [r.key, r.value])
    const renamePairs: [string, string][] = rows.value
      .filter((r) => r.kind === 'existing' && r.value !== r.key)
      .map((r) => [r.key, r.value])
    const md = applyImageMapping(
      buildDoc({ ...props.meta, locate: cleanName.value }, props.sections),
      [...tempPairs, ...renamePairs],
    )
    const q = await saveQuestion({
      source: sel.value.source,
      dir: sel.value.dir,
      filename: cleanName.value,
      markdown: md,
      images: rows.value
        .filter((r) => r.kind === 'temp')
        .map((r) => ({ temp: r.key, name: r.value })),
      renames: renamePairs.map(([from, to]) => ({ from, to })),
      original: props.original ? `${props.original.dir}/${props.original.file}` : undefined,
    })
    emit('saved', q)
  } catch (e) {
    errMsg.value = e instanceof Error ? e.message : '保存失败'
  } finally {
    saving.value = false
  }
}

useDialogShell((e) => {
  if (e.code === 'Escape') {
    e.stopPropagation()
    emit('close')
  } else if ((e.metaKey || e.ctrlKey) && e.code === 'KeyS') {
    e.preventDefault()
    void submit()
  }
})

onMounted(async () => {
  const initial = props.initialTarget
  sel.value = {
    source: props.original?.source ?? initial?.source ?? props.sources[0]?.id ?? '',
    dir: props.original?.dir ?? initial?.dir ?? '',
  }
  if (props.original) expandAncestors(props.original.source, props.original.dir)
  else if (initial) expandAncestors(initial.source, initial.dir)
  
  await loadBrowse(sel.value.source, '')
  await loadBrowse(sel.value.source, sel.value.dir)
  rebuildRows()
  void resolveAll()
})
</script>

<template>
  <Teleport to="body">
  <div class="modal-mask save-mask fs">
    <div class="save dialog fs">
      <DialogHeader no-border title="保存题目" @close="emit('close')" />
      <div class="save-body">
        <div class="save-tree">
          <div v-if="loading" class="tree-hint">读取文件夹…</div>
          <div v-else-if="loadErr" class="tree-hint err">{{ loadErr }}</div>
          <div
            v-for="n in treeNodes"
            :key="n.source + '/' + n.dir"
            class="tree-row"
            :class="{ sel: n.source === sel.source && n.dir === sel.dir }"
            :style="{ paddingInlineStart: `${n.depth * 0.875 + 0.5}rem` }"
            @click="selectNode(n)"
          >
            <button
              v-if="n.expandable || n.open"
              type="button"
              class="tree-chev"
              :title="n.open ? '收起' : '展开'"
              @click.stop="toggleNode(n)"
            >
              <ChevronDown v-if="n.open" style="width: 0.875rem; height: 0.875rem" />
              <ChevronRight v-else style="width: 0.875rem; height: 0.875rem" />
            </button>
            <span v-else class="tree-chev-off"></span>
            <component
              :is="n.source === sel.source && n.dir === sel.dir ? FolderOpen : Folder"
              style="width: 0.9375rem; height: 0.9375rem; flex: none"
            />
            <span class="tree-name">{{ n.name }}</span>
            <span v-if="n.depth === 0 && !n.expandable" class="tree-note">不含子文件夹</span>
          </div>
          <div v-if="!treeNodes.length" class="tree-hint">没有可用题源</div>
        </div>

        <div class="save-main">
          <label class="save-field">
            <span>文件名（不含 .md）</span>
            <div class="save-file">
              <input
                :value="filename"
                type="text"
                placeholder="定位 / 来源 / 题目 01"
                @input="onFilenameInput"
              />
              <span class="ext">.md</span>
            </div>
            <i v-if="!cleanName" class="bad">文件名不能为空，也不能包含 / 等路径字符</i>
            <i v-else-if="fileConflict" class="bad">该文件夹已存在同名文件，无法保存</i>
            <i v-else-if="movingOrRenaming" class="warn">保存时会把原文件移动到新位置；历史做题记录按题目 id 关联，文件移走后旧记录不会跟随</i>
          </label>

          <div v-if="rows.length" class="save-images">
            <div class="img-h">图片（保存到同一文件夹）</div>
            <div v-for="row in rows" :key="row.kind + row.key" class="img-row">
              <span class="img-kind" :class="row.kind">{{ row.kind === 'temp' ? '上传' : '已有' }}</span>
              <input
                v-model="row.value"
                type="text"
                class="img-name"
                :disabled="renameLocked(row)"
                @input="row.dirty = true"
              />
              <span v-if="row.kind === 'temp'" class="img-orig" :title="row.origName">{{
                row.origName
              }}</span>
              <template v-else>
                <span v-if="renameLocked(row)" class="img-loc lock">
                  <template v-if="resolvedOf(row)?.found">位于其他题源，不能在此重命名</template>
                  <template v-else>{{ resolvedLoading ? '正在核对文件位置…' : '文件已缺失' }}</template>
                </span>
                <span v-else class="img-loc">
                  {{ resolvedOf(row)?.dir ? `当前位于 ${resolvedOf(row)!.dir}/` : '就在根目录，保持原名' }}
                </span>
              </template>
              <i v-if="rowInvalid(row)" class="bad img-bad">名称不可用</i>
              <i v-else-if="rowConflict(row)" class="bad img-bad">与现有文件重名</i>
            </div>
          </div>

          <div v-if="errMsg" class="save-err">{{ errMsg }}</div>
        </div>
      </div>

      <div class="save-foot">
        <span class="save-hint">重名文件无法保存；上传的图片会一起存进所选文件夹</span>
        <button class="btn ghost" :disabled="saving" @click="emit('close')">返回编辑</button>
        <button class="btn primary" :disabled="!canSubmit" @click="submit">
          <Save style="width: 0.9375rem; height: 0.9375rem" /> {{ saving ? '保存中…' : '保存' }}
        </button>
      </div>
    </div>
  </div>
  </Teleport>
</template>

<style scoped>
.save-mask {
  z-index: var(--z-nested);
  background: var(--scrim-nested);
}

.save {
  position: relative;
  width: min(52rem, 100%);
  max-height: min(86vh, 50rem);
}

.save-body {
  flex: 1;
  min-height: 0;
  display: flex;
  gap: 1rem;
  padding: 0.625rem 1.25rem 1rem;
}

.save-tree {
  flex: none;
  width: 15rem;
  overflow-y: auto;
  overflow-x: clip;
  border: 1px solid var(--line);
  border-radius: 0.625rem;
  padding: 0.375rem;
  background: var(--panel);
}

.tree-row {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  padding-block: 0.25rem;
  padding-inline-end: 0.375rem;
  border-radius: 0.375rem;
  cursor: pointer;
  min-width: 0;
  min-height: 2.375rem;
}

.tree-row:hover {
  background: var(--hover);
}

.tree-row.sel {
  background: var(--brand-weak);
  color: var(--brand);
}

/* the chevron's hit area is bigger than its icon: touch taps next to the
   folder name must not toggle expand instead of selecting the row */
.tree-chev {
  flex: none;
  display: inline-flex;
  border: none;
  background: transparent;
  color: var(--muted);
  padding: 0;
  width: 2rem;
  height: 2rem;
  margin-inline: -0.25rem;
  border-radius: 0.375rem;
  align-items: center;
  justify-content: center;
}

.tree-chev:hover {
  background: var(--hover);
}

.tree-chev-off {
  flex: none;
  width: 1.5rem;
}

.tree-name {
  font-size: 0.8438rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tree-note {
  margin-left: auto;
  font-size: 0.6875rem;
  color: var(--muted);
  white-space: nowrap;
}

.tree-hint {
  font-size: 0.8125rem;
  color: var(--muted);
  padding: 0.5rem;
}

.tree-hint.err {
  color: var(--bad);
}

.save-main {
  flex: 1;
  min-width: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 0.875rem;
}

.save-field {
  display: flex;
  flex-direction: column;
  gap: 0.3125rem;
}

.save-field > span {
  font-size: 0.8125rem;
  color: var(--muted);
}

.save-file {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.save-file input {
  flex: 1;
  min-width: 0;
}

.save-file .ext {
  color: var(--muted);
  font-family: var(--font-mono);
  font-size: 0.8438rem;
}

.save-field i,
.img-bad {
  font-style: normal;
  font-size: 0.7812rem;
}

.save-field .bad,
.img-bad {
  color: var(--bad);
}

.save-field .warn {
  color: var(--warn);
}

.save-images {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.img-h {
  font-size: 0.8125rem;
  color: var(--muted);
}

.img-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
}

.img-kind {
  flex: none;
  font-size: 0.7188rem;
  border-radius: var(--radius-pill);
  padding: 0.0625rem 0.5rem;
  background: var(--brand-weak);
  color: var(--brand);
}

.img-kind.existing {
  background: var(--panel);
  color: var(--muted);
}

.img-name {
  width: 13rem;
  flex: none;
  font-family: var(--font-mono);
  font-size: 0.8125rem;
}

.img-orig,
.img-loc {
  font-size: 0.75rem;
  color: var(--muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.img-loc.lock {
  color: var(--warn);
}

.save-err {
  color: var(--bad);
  font-size: 0.8438rem;
  white-space: pre-wrap;
}

.save-foot {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.625rem 1.25rem;
  border-top: 1px solid var(--line);
}

.save-hint {
  flex: 1;
  color: var(--muted);
  font-size: 0.7812rem;
}

@container (max-width: 40rem) {
  .save-body {
    flex-direction: column;
  }

  .save-tree {
    width: 100%;
    max-height: 14rem;
  }

  .img-name {
    width: 9rem;
  }
}
</style>
