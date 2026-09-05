<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { ChevronDown, Download, LoaderCircle } from 'lucide-vue-next'
import type { BookSession } from '../../book'
import { payloadFor } from '../../book'
import { exportPdf, previewPdf } from '../../api'
import { useDialogShell } from '../../composables/useDialogShell'
import { useExportPrefs } from '../../composables/useExportPrefs'
import ExportPdfOptions from './ExportPdfOptions.vue'
import DialogHeader from '../DialogHeader.vue'

const props = defineProps<{
  session: BookSession
  doc: 'workbook' | 'answers'
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'exported', message: string): void
}>()

const { opts, persist } = useExportPrefs()

const activeDoc = ref<'workbook' | 'answers'>(props.doc)

const previewOpen = ref(false)

const previewUrl = ref('')
const previewing = ref(false)
const previewError = ref('')
let previewTimer: number | undefined
let lastUrl = ''

const exporting = ref(false)
const exportStatus = ref('')

const payload = computed(() => payloadFor(props.session, activeDoc.value, opts))

let objectSeq = 0

async function refreshPreview() {
  const token = ++objectSeq
  previewing.value = true
  previewError.value = ''
  try {
    const url = await previewPdf(payload.value)
    if (token !== objectSeq) {
      URL.revokeObjectURL(url)
      return
    }
    if (lastUrl) URL.revokeObjectURL(lastUrl)
    lastUrl = url
    previewUrl.value = url
  } catch (e) {
    previewError.value = e instanceof Error ? e.message : String(e)
  } finally {
    if (token === objectSeq) previewing.value = false
  }
}

let debounceTimer: number | undefined
function schedulePreview() {
  window.clearTimeout(debounceTimer)
  window.clearTimeout(previewTimer)
  debounceTimer = window.setTimeout(refreshPreview, 500)
}

watch(
  () => [payload.value, activeDoc.value] as const,
  () => schedulePreview(),
  { deep: true },
)

onMounted(() => void refreshPreview())

useDialogShell((e) => {
  if (e.code === 'Escape') {
    e.stopPropagation()
    emit('close')
  }
}, false)

onBeforeUnmount(() => {
  window.clearTimeout(debounceTimer)
  if (lastUrl) URL.revokeObjectURL(lastUrl)
})

async function exportAndDownload(doc: 'workbook' | 'answers') {
  persist()
  exporting.value = true
  exportStatus.value = ''
  try {
    const res = await exportPdf(payloadFor(props.session, doc, opts))
    const a = document.createElement('a')
    a.href = res.url
    a.download = res.name
    a.click()
    exportStatus.value = res.cached
      ? `内容未变，已命中缓存，直接下载 ${res.name}`
      : `已生成 ${res.name}（${Math.max(1, Math.round(res.bytes / 1024))} KB），开始下载`
  } catch (e) {
    exportStatus.value = `导出失败（${e instanceof Error ? e.message : '未知错误'}）`
  } finally {
    exporting.value = false
  }
}

</script>

<template>
  <Teleport to="body">
  <div class="modal-mask pdf-mask fs" @click.self="emit('close')">
    <div class="big-modal dialog fs">
      <DialogHeader title="导出 PDF" @close="emit('close')" />

      <div class="b-body">
        <div class="b-options">
          <ExportPdfOptions />
        </div>

        <div class="b-preview" :class="{ collapsed: !previewOpen }">
          <div class="p-head">
            <button
              class="p-fold"
              type="button"
              :aria-label="previewOpen ? '收起预览' : '展开预览'"
              @click="previewOpen = !previewOpen"
            >
              <ChevronDown style="width: 1rem; height: 1rem" class="chev" :class="{ closed: !previewOpen }" />
              <span>预览</span>
            </button>
            <div class="seg sm b-doc-seg">
              <button :class="{ on: activeDoc === 'workbook' }" @click="activeDoc = 'workbook'">题本</button>
              <button :class="{ on: activeDoc === 'answers' }" @click="activeDoc = 'answers'">答案本</button>
            </div>
            <button v-if="previewError" class="btn sm" @click="refreshPreview">重试</button>
          </div>
          <div v-if="previewing" class="p-state"><LoaderCircle style="width: 1.125rem; height: 1.125rem" class="spin" /> 生成预览…</div>
          <div v-else-if="previewError" class="p-state bad">{{ previewError }}</div>
          <iframe v-else-if="previewUrl" :src="previewUrl" class="p-frame" title="PDF 预览"></iframe>
        </div>
      </div>

      <footer class="b-foot">
        <span v-if="exportStatus" class="status">{{ exportStatus }}</span>
        <span class="gap"></span>
        <button class="btn primary" :disabled="exporting" @click="exportAndDownload('workbook')">
          <Download style="width: 0.9375rem; height: 0.9375rem" /> 导出题本<span class="pdf-txt"> PDF</span>
        </button>
        <button class="btn primary" :disabled="exporting" @click="exportAndDownload('answers')">
          <Download style="width: 0.9375rem; height: 0.9375rem" /> 导出答案本<span class="pdf-txt"> PDF</span>
        </button>
      </footer>
    </div>
  </div>
  </Teleport>
</template>

<style scoped>
.pdf-mask {
  z-index: var(--z-modal-lift);
  padding: 1rem;
}

.big-modal {
  width: min(88rem, 97vw);
  height: min(56rem, 94vh);
}

/* narrow screens go fullscreen (global .fs): cancel the scoped margin here —
   a same-specificity scoped rule would otherwise beat the global one */
@container (max-width: 62.4375rem) {
  .pdf-mask {
    padding: 0;
  }
}

.b-body {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(19rem, 26rem) 1fr;
  gap: 1rem;
  padding: 1rem 1.25rem;
  overflow: hidden;
}

.b-options {
  overflow-y: auto;
  overscroll-behavior: contain;
  padding:4px;
}

.b-preview {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  background: var(--panel);
  border-radius:0.625rem;
  padding: 0.625rem;
}

.p-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  font-size: 0.7812rem;
  color: var(--muted);
  margin-bottom: 0.5rem;
  flex-wrap: wrap;
}

.p-fold {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  border: none;
  background: transparent;
  color: var(--muted);
  font: inherit;
  font-size: 0.7812rem;
  font-weight: 600;
  padding: 0;
  cursor: pointer;
}

.p-fold .chev {
  flex: none;
  color: var(--muted);
}

.p-frame {
  flex: 1;
  width: 100%;
  border: none;
  border-radius:0.5rem;
  background: #fff;
}

.p-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  color: var(--muted);
  font-size: 0.8125rem;
}

.p-state.bad {
  color: var(--bad);
}

@container (min-width: 62.5rem) {
  .p-fold {
    cursor: default;
  }

  .p-fold .chev {
    display: none;
  }
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.b-foot {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1.25rem;
  border-top:1px solid var(--line);
}

.b-foot .gap {
  flex: 1;
  pointer-events: none;
}

.status {
  font-size: 0.7812rem;
  color: var(--good);
}

@container (max-width: 62.4375rem) {
  .pdf-txt {
    display: none;
  }

  .b-foot .btn.primary {
    flex: 1;
    justify-content: center;
  }

  .b-body {
    grid-template-columns: 1fr;
    overflow-y: auto;
  }

  .b-preview.collapsed .p-frame,
  .b-preview.collapsed .p-state {
    display: none;
  }

  .b-preview.collapsed {
    background: transparent;
    padding: 0;
  }

  .b-preview:not(.collapsed) .p-frame {
    flex: none;
    height: calc(100dvh - 13rem);
  }
}
</style>
