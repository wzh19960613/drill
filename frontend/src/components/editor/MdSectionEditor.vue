<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { Bold, ChevronRight, Code2, DollarSign, Heading3, ImagePlus, ListOrdered, Quote } from 'lucide-vue-next'
import { paraHtml } from '../../math'
import { clampHeadings, joinBlocks, nextOptionLetter, splitBlocks } from '../../mdoc'
import { uploadImage } from '../../api'
import { openImageViewer } from '../../composables/useImageViewer'


const props = withDefaults(
  defineProps<{
    modelValue: string
    placeholder?: string
    
    temps?: Set<string>
    allowOptions?: boolean
    allowImage?: boolean
    minHeight?: string
    
    clamp?: boolean
  }>(),
  { placeholder: '', temps: () => new Set<string>(), allowOptions: false, allowImage: true, minHeight: '6rem', clamp: true },
)

const emit = defineEmits<{
  (e: 'update:modelValue', v: string): void
  (e: 'temp-added', id: string, name: string): void
}>()

const mode = ref<'live' | 'source'>('live')
const blocks = ref<string[]>([])
const activeIdx = ref<number | null>(null)

let activeTa: HTMLTextAreaElement | null = null
const sourceTa = ref<HTMLTextAreaElement | null>(null)
const fileInput = ref<HTMLInputElement | null>(null)
const uploading = ref(false)
const uploadErr = ref('')

function setActiveTa(el: unknown) {
  activeTa = (el as HTMLTextAreaElement | null) ?? null
}

function syncFromModel() {
  blocks.value = splitBlocks(props.modelValue)
  activeIdx.value = null
}

watch(
  () => props.modelValue,
  (v) => {
    if (v !== joinBlocks(blocks.value)) syncFromModel()
  },
  { immediate: true },
)

const activeText = computed({
  get: () => (activeIdx.value == null ? '' : blocks.value[activeIdx.value] ?? ''),
  set: (v: string) => {
    if (activeIdx.value == null) return
    setBlockText(activeIdx.value, v)
  },
})

function maybeClamp(text: string, caret: number): { text: string; caret: number } {
  return props.clamp ? clampHeadings(text, caret) : { text, caret }
}

function setBlockText(i: number, raw: string) {
  const ta = activeTa
  const caret = ta?.selectionStart ?? raw.length
  const { text, caret: newCaret } = maybeClamp(raw, caret)
  blocks.value[i] = text
  emit('update:modelValue', joinBlocks(blocks.value))
  void nextTick(() => {
    if (ta && document.activeElement === ta && newCaret !== caret) {
      ta.setSelectionRange(newCaret, newCaret)
    }
    resize(ta)
  })
}

function blockHtml(b: string): string {
  return paraHtml(b, {
    optionLetters: props.allowOptions,
    srcFor: (name) =>
      props.temps.has(name) ? `/api/tmp/${encodeURIComponent(name)}` : undefined,
  })
}


function onBlockMouseDown(i: number, e: MouseEvent) {
  const img = (e.target as HTMLElement).closest?.('img.q-svg')
  if (img) {
    e.preventDefault()
    openImageViewer((img as HTMLImageElement).getAttribute('src') ?? '', img)
    return
  }
  activate(i, e)
}

function activate(i: number, evt?: MouseEvent) {
  const el = evt?.currentTarget as HTMLElement | undefined
  let caret = (blocks.value[i] ?? '').length
  if (el && evt) {
    const r = el.getBoundingClientRect()
    const frac = Math.min(1, Math.max(0, (evt.clientY - r.top) / Math.max(1, r.height)))
    caret = Math.round(frac * (blocks.value[i]?.length ?? 0))
  }
  activeIdx.value = i
  void nextTick(() => {
    const ta = activeTa
    if (!ta) return
    ta.focus()
    ta.setSelectionRange(caret, caret)
    resize(ta)
  })
}

function activateEnd() {
  if (!blocks.value.length) blocks.value = ['']
  activate(blocks.value.length - 1)
}

function onBlur() {
  
  
  void nextTick(() => {
    const focused = [activeTa, sourceTa.value].some(
      (el) => el && document.activeElement === el,
    )
    if (!focused) {
      activeIdx.value = null
      blocks.value = splitBlocks(joinBlocks(blocks.value))
    }
  })
}

function resize(el: HTMLTextAreaElement | null) {
  if (!el) return
  el.style.height = 'auto'
  el.style.height = `${el.scrollHeight}px`
}

const sourceText = computed({
  get: () => props.modelValue,
  set: (v: string) => {
    const ta = sourceTa.value
    const caret = ta?.selectionStart ?? v.length
    const { text, caret: newCaret } = maybeClamp(v, caret)
    emit('update:modelValue', text)
    void nextTick(() => {
      if (ta && document.activeElement === ta && newCaret !== caret) {
        ta.setSelectionRange(newCaret, newCaret)
      }
    })
  },
})

function toggleMode() {
  if (mode.value === 'live') {
    mode.value = 'source'
    activeIdx.value = null
    void nextTick(() => {
      sourceTa.value?.focus()
      resize(sourceTa.value)
    })
  } else {
    mode.value = 'live'
    syncFromModel()
  }
}


function transformActive(fn: (text: string, caret: number) => { text: string; caret: number }) {
  if (mode.value === 'source') {
    const ta = sourceTa.value
    if (!ta) return
    const { text, caret } = fn(ta.value, ta.selectionStart)
    const r = maybeClamp(text, caret)
    emit('update:modelValue', r.text)
    void nextTick(() => {
      ta.focus()
      ta.setSelectionRange(r.caret, r.caret)
    })
    return
  }
  if (activeIdx.value == null) {
    activateEnd()
    void nextTick(() => transformActive(fn))
    return
  }
  const ta = activeTa
  if (!ta) return
  const { text, caret } = fn(ta.value, ta.selectionStart)
  setBlockText(activeIdx.value, text)
  void nextTick(() => {
    ta.focus()
    ta.setSelectionRange(caret, caret)
  })
}

function wrapSelection(before: string, after: string) {
  transformActive((text, caret) => {
    const end = text.indexOf('\n', caret)
    const lineEnd = end < 0 ? text.length : end
    const sel = text.slice(caret, lineEnd)
    if (!sel) {
      
      return {
        text: text.slice(0, caret) + before + after + text.slice(lineEnd),
        caret: caret + before.length,
      }
    }
    return {
      text: text.slice(0, caret) + before + sel + after + text.slice(lineEnd),
      caret: caret + before.length + sel.length,
    }
  })
}


function insertBlockAtCaret(insert: string) {
  transformActive((text, caret) => {
    let at = caret
    if (at > 0 && text[at - 1] !== '\n') {
      const nl = text.indexOf('\n', at)
      at = nl < 0 ? text.length : nl
    }
    return {
      text: text.slice(0, at) + insert + text.slice(at),
      caret: at + insert.length,
    }
  })
}

function prefixLines(prefix: string) {
  transformActive((text, caret) => {
    const lineStart = text.lastIndexOf('\n', caret - 1) + 1
    const head = text.slice(0, lineStart)
    const line = text.slice(lineStart)
    const stripped = line.replace(/^>\s?/, '').replace(/^#+\s?/, '')
    return { text: head + prefix + stripped, caret: lineStart + prefix.length }
  })
}

function insertDisplayMath() {
  transformActive((text, caret) => {
    const insert = '$$\n\n$$'
    const at = text.slice(0, caret).endsWith('\n') || caret === 0 ? caret : caret + 1
    return {
      text: text.slice(0, at) + '\n\n' + insert + '\n\n' + text.slice(at),
      
      caret: at + 5,
    }
  })
}

function insertOption() {
  const letter = nextOptionLetter(joinBlocks(blocks.value))
  transformActive((text, caret) => ({
    text: text.slice(0, caret) + `\n\n(${letter}) ` + text.slice(caret),
    caret: caret + 6,
  }))
}



async function uploadFiles(files: FileList | File[] | null) {
  if (!files || !props.allowImage) return
  const list = Array.from(files).filter((f) => /\.(svg|png|jpe?g|webp|gif)$/i.test(f.name))
  if (!list.length) return
  uploading.value = true
  const failed: string[] = []
  try {
    for (const f of list) {
      try {
        const t = await uploadImage(f)
        emit('temp-added', t.id, t.name)
        insertBlockAtCaret(`\n\n![[${t.id}]]\n\n`)
      } catch (err) {
        
        failed.push(`${f.name}（${err instanceof Error && err.message ? err.message : '上传失败'}）`)
      }
    }
  } finally {
    uploading.value = false
  }
  if (!failed.length) {
    uploadErr.value = ''
    return
  }
  const msg = `有 ${failed.length} 张图片没有上传成功：${failed.join('；')}`
  uploadErr.value = msg
  setTimeout(() => {
    if (uploadErr.value === msg) uploadErr.value = ''
  }, 8000)
}

function onPaste(e: ClipboardEvent) {
  if (!props.allowImage) return
  const files = e.clipboardData?.files
  if (files?.length) {
    const images = Array.from(files).filter((f) => /\.(svg|png|jpe?g|webp|gif)$/i.test(f.name))
    if (images.length) {
      e.preventDefault()
      void uploadFiles(images)
    }
  }
}

function onFilePicked(e: Event) {
  const input = e.target as HTMLInputElement
  void uploadFiles(input.files)
  input.value = ''
}

function onDrop(e: DragEvent) {
  if (!props.allowImage) return
  if (e.dataTransfer?.files?.length) {
    e.preventDefault()
    if (mode.value === 'live' && activeIdx.value == null) activateEnd()
    void uploadFiles(e.dataTransfer.files)
  }
}

function onBlockKeydown(e: KeyboardEvent, i: number) {
  const ta = e.target as HTMLTextAreaElement
  const len = ta.value.length
  if (e.key === 'Escape') {
    ta.blur()
    return
  }
  if (e.key === 'ArrowUp' && ta.selectionStart === 0 && i > 0) {
    e.preventDefault()
    activate(i - 1)
  } else if (e.key === 'ArrowDown' && ta.selectionEnd === len && i < blocks.value.length - 1) {
    e.preventDefault()
    activate(i + 1)
  }
}
</script>

<template>
  <div
    class="md-ed"
    :class="{ source: mode === 'source' }"
    @paste="onPaste"
    @dragover.prevent
    @drop="onDrop"
  >
    <div class="md-toolbar">
      <button
        type="button"
        class="md-tb"
        title="加粗 **文字**"
        @mousedown.prevent
        @click="wrapSelection('**', '**')"
      >
        <Bold style="width: 0.875rem; height: 0.875rem" />
      </button>
      <button
        type="button"
        class="md-tb"
        title="行内公式 $…$"
        @mousedown.prevent
        @click="wrapSelection('$', '$')"
      >
        <DollarSign style="width: 0.875rem; height: 0.875rem" />
      </button>
      <button
        type="button"
        class="md-tb"
        title="独立公式块 $$…$$"
        @mousedown.prevent
        @click="insertDisplayMath"
      >
        <span class="md-tb-tex">$$</span>
      </button>
      <button
        type="button"
        class="md-tb"
        title="小标题（最高三级）"
        @mousedown.prevent
        @click="prefixLines('### ')"
      >
        <Heading3 style="width: 0.875rem; height: 0.875rem" />
      </button>
      <button
        type="button"
        class="md-tb"
        title="引用"
        @mousedown.prevent
        @click="prefixLines('> ')"
      >
        <Quote style="width: 0.875rem; height: 0.875rem" />
      </button>
      <button
        v-if="allowOptions"
        type="button"
        class="md-tb"
        title="插入选项 (A)"
        @mousedown.prevent
        @click="insertOption"
      >
        <ListOrdered style="width: 0.875rem; height: 0.875rem" />
      </button>
      <button
        v-if="allowImage"
        type="button"
        class="md-tb"
        :class="{ busy: uploading }"
        title="上传并插入图片（也可直接粘贴/拖入）"
        @mousedown.prevent
        @click="fileInput?.click()"
      >
        <ImagePlus style="width: 0.875rem; height: 0.875rem" />
      </button>
      <span class="md-tb-gap"></span>
      <button
        type="button"
        class="md-tb md-mode"
        :title="mode === 'live' ? '切换为源码模式' : '切换为预览'"
        @mousedown.prevent
        @click="toggleMode"
      >
        <!-- the label shows the target mode (what clicking switches to),
             not the current one -->
        <Code2 v-if="mode === 'live'" style="width: 0.875rem; height: 0.875rem" />
        <ChevronRight v-else style="width: 0.875rem; height: 0.875rem" />
        {{ mode === 'live' ? '源码' : '预览' }}
      </button>
      <input
        ref="fileInput"
        type="file"
        accept=".svg,.png,.jpg,.jpeg,.webp,.gif"
        multiple
        hidden
        @change="onFilePicked"
      />
    </div>

    <div v-if="uploadErr" class="md-upload-err" role="alert">{{ uploadErr }}</div>

    <div v-if="mode === 'live'" class="md-surface" :style="{ minHeight }">
      <template v-for="(b, i) in blocks" :key="i">
        <textarea
          v-if="i === activeIdx"
          :ref="setActiveTa"
          v-model="activeText"
          class="md-input"
          rows="1"
          spellcheck="false"
          @input="resize(activeTa)"
          @keydown="onBlockKeydown($event, i)"
          @blur="onBlur"
        ></textarea>
        <div
          v-else
          class="md-block"
          :title="'点击编辑此段'"
          @mousedown.prevent="onBlockMouseDown(i, $event)"
          v-html="blockHtml(b)"
        ></div>
      </template>
      <div v-if="!blocks.length" class="md-empty" @mousedown.prevent="activateEnd()">
        {{ placeholder || '点击开始输入…' }}
      </div>
    </div>

    <textarea
      v-else
      ref="sourceTa"
      v-model="sourceText"
      class="md-source"
      spellcheck="false"
      :style="{ minHeight }"
      @blur="onBlur"
    ></textarea>
  </div>
</template>

<style scoped>
.md-ed {
  border: 1px solid var(--line);
  border-radius: 0.625rem;
  background: var(--card);
  display: flex;
  flex-direction: column;
  overflow: clip;
}

.md-toolbar {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 0.25rem 0.375rem;
  border-bottom: 1px solid var(--line);
  background: var(--panel);
  flex: none;
}

.md-tb {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  border: none;
  background: transparent;
  color: var(--muted);
  height: 2.25rem;
  padding: 0 0.5rem;
  border-radius: 0.375rem;
  font-size: 0.8125rem;
  cursor: pointer;
}

.md-tb:hover {
  background: var(--hover);
  color: var(--ink);
}

.md-tb.busy {
  opacity: 0.5;
}

.md-upload-err {
  padding: 0.375rem 0.75rem;
  font-size: 0.8125rem;
  line-height: 1.45;
  color: var(--bad);
  background: var(--bad-weak);
  border-bottom: 1px solid var(--line);
  flex: none;
}

.md-tb-tex {
  font-family: var(--font-mono);
  font-weight: 700;
  font-size: 0.75rem;
  line-height: 1;
}

.md-tb-gap {
  flex: 1;
}

.md-mode {
  color: var(--muted);
  white-space: nowrap;
}

.md-surface {
  padding: 0.5rem 0.625rem;
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  font-size: 0.9375rem;
  line-height: 1.8;
  color: var(--ink);
  min-height: 4rem;
}

.md-block {
  border-radius: 0.375rem;
  padding: 0 0.375rem;
  cursor: text;
  min-height: 1.75rem;
}

.md-block:hover {
  background: var(--hover);
}

.md-block :deep(.md-h) {
  font-weight: 700;
  margin: 0.125rem 0;
}

.md-block :deep(.md-h3) {
  font-size: 1.0625rem;
}

.md-block :deep(.md-h4) {
  font-size: 1rem;
}

.md-block :deep(.md-h5),
.md-block :deep(.md-h6) {
  font-size: 0.9375rem;
}

.md-block :deep(.md-opt) {
  display: flex;
  gap: 2px;
  align-items: baseline;
}

.md-block :deep(.md-opt .opt-letter) {
  font-weight: 600;
  flex: none;
}

.md-block :deep(p) {
  margin: 0;
}

.md-input {
  width: 100%;
  border: 1px dashed var(--brand);
  border-radius: 0.375rem;
  outline: none;
  resize: none;
  padding: 0.0625rem 0.375rem;
  font: inherit;
  font-size: 0.9375rem;
  line-height: 1.8;
  background: var(--brand-weak);
  color: var(--ink);
  overflow: hidden;
}

.md-empty {
  color: var(--muted);
  padding: 0.25rem 0.375rem;
  cursor: text;
  border-radius: 0.375rem;
}

.md-empty:hover {
  background: var(--hover);
}

.md-source {
  border: none;
  outline: none;
  resize: vertical;
  padding: 0.5rem 0.625rem;
  font-family: var(--font-mono);
  font-size: 0.8438rem;
  line-height: 1.8;
  background: var(--card);
  color: var(--ink);
}
</style>
