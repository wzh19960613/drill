<script setup lang="ts">
import { ref } from 'vue'
import { Download, FileText, Printer } from 'lucide-vue-next'
import type { BookSession } from '../book'
import { exportMarkdown, type MdOrg } from '../book'
import { useDialogShell } from '../composables/useDialogShell'
import DialogHeader from './DialogHeader.vue'
import ExportPdfDialog from './export/ExportPdfDialog.vue'

const props = defineProps<{ session: BookSession }>()

const mdOpen = ref(false)
const mdOrg = ref<MdOrg>('workbook')
const mdMeta = ref(true)
const mdAnswers = ref(true)
const mdAnsStem = ref(true)

const busy = ref(false)
const status = ref('')

const pdfDialogDoc = ref<'workbook' | 'answers'>('workbook')
const pdfDialogOpen = ref(false)
const suppressed = ref(false)

const emit = defineEmits<{ (e: 'close'): void }>()

async function genMd() {
  status.value = ''
  busy.value = true
  try {
    status.value = await exportMarkdown(props.session, {
      org: mdOrg.value,
      meta: mdMeta.value,
      answers: mdAnswers.value,
      answersWithQuestion: mdAnsStem.value,
    })
  } catch (e) {
    status.value = `导出失败（${e instanceof Error ? e.message : '未知错误'}）`
  } finally {
    busy.value = false
  }
}

function openPdfDialog(doc: 'workbook' | 'answers') {
  pdfDialogDoc.value = doc
  pdfDialogOpen.value = true
  suppressed.value = true
  status.value = ''
}

function onPdfExported(message: string) {
  status.value = message
  pdfDialogOpen.value = false
  suppressed.value = false
  emit('close')
}

function onPdfClosed() {
  pdfDialogOpen.value = false
  suppressed.value = false
  emit('close')
}

function openMdDialog() {
  mdOpen.value = true
  suppressed.value = true
}

function closeMd() {
  mdOpen.value = false
  if (suppressed.value) {
    suppressed.value = false
    emit('close')
  }
}

useDialogShell((e) => {
  if (e.code === 'Escape') {
    e.stopPropagation()
    if (pdfDialogOpen.value || mdOpen.value) return 
    emit('close')
  }
})
</script>

<template>
  <Teleport to="body">
  <div v-show="!suppressed" class="modal-mask" @click.self="emit('close')">
    <div class="modal dialog pop">
      <DialogHeader :title="`导出 · ${session.title}`" @close="emit('close')" />

      <div class="m-body">
        <button class="big-btn" @click="openPdfDialog('workbook')">
          <Printer style="width: 1.375rem; height: 1.375rem" class="big-ico" />
          <span class="big-txt">
            <span class="big-main">导出 PDF</span>
            <span class="big-sub">按页排版，便于打印、分发</span>
          </span>
        </button>
        <button class="big-btn" @click="openMdDialog()">
          <FileText style="width: 1.375rem; height: 1.375rem" class="big-ico" />
          <span class="big-txt">
            <span class="big-main">导出 Markdown</span>
            <span class="big-sub">可二次编辑，与 Obsidian、AI 等工具天然适配</span>
          </span>
        </button>
      </div>

      <div v-if="status" class="status">{{ status }}</div>
    </div>
  </div>

  <ExportPdfDialog
    v-if="pdfDialogOpen"
    :session="session"
    :doc="pdfDialogDoc"
    @close="onPdfClosed"
    @exported="onPdfExported"
  />

    <div v-if="mdOpen" class="modal-mask" @click.self="closeMd">
      <div class="modal md dialog pop">
        <DialogHeader title="导出 Markdown" @close="closeMd" />
        <div class="m-body">
          <div class="field">
            <span class="lbl">文件组织</span>
            <select class="fill" v-model="mdOrg">
              <option value="workbook">仅题目本</option>
              <option value="answers">仅答案本</option>
              <option value="both">题目本 + 答案本</option>
              <option value="per">每题一个文件</option>
            </select>
          </div>
          <button class="toggle-row md-toggle" type="button" @click="mdMeta = !mdMeta">
            <input type="checkbox" :checked="mdMeta" tabindex="-1" />
            <span>包含题目元信息</span>
          </button>
          <button v-if="mdOrg === 'per'" class="toggle-row md-toggle" type="button" @click="mdAnswers = !mdAnswers">
            <input type="checkbox" :checked="mdAnswers" tabindex="-1" />
            <span>包含答案与解析</span>
          </button>
          <button
            v-if="mdOrg === 'answers' || mdOrg === 'both'"
            class="toggle-row md-toggle"
            type="button"
            @click="mdAnsStem = !mdAnsStem"
          >
            <input type="checkbox" :checked="mdAnsStem" tabindex="-1" />
            <span>答案本中包含题目</span>
          </button>
          <div class="md-tip">
            按源文件原样拼接：除题号外不添加任何额外内容；引用的 SVG 图片随文件一并导出；多文件时打包为 zip。
          </div>
          <div v-if="status" class="md-status">{{ status }}</div>
        </div>
        <div class="m-actions">
          <button class="btn primary" :disabled="busy" @click="genMd">
            <Download style="width: 0.9375rem; height: 0.9375rem" /> 导出 Markdown
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal {
  width: 31.25rem;
  max-width: 100%;
  max-height: 90vh;
  overflow-y: auto;
  overflow-x: clip;
  overscroll-behavior: contain;
}

.m-body {
  padding: 0.875rem 1.375rem 1.125rem;
  display: flex;
  flex-direction: column;
  gap: 0.875rem;
}

.big-btn {
  display: flex;
  align-items: center;
  gap: 0.875rem;
  border: 1px solid var(--line);
  background: var(--panel);
  border-radius:0.875rem;
  padding: 1.125rem 1.25rem;
  cursor: pointer;
  font-family: inherit;
  text-align: left;
  color: var(--ink);
}

.big-btn:hover {
  border-color: var(--brand);
  background: var(--brand-weak);
}

.big-ico {
  flex: none;
  color: var(--brand);
}

.big-txt {
  display: flex;
  flex-direction: column;
  gap:4px;
  min-width: 0;
}

.big-main {
  font-size: 1.0312rem;
  font-weight: 700;
}

.big-sub {
  font-size: 0.7812rem;
  color: var(--muted);
  line-height: 1.5;
  margin:4px;
}

.md-toggle {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  border: none;
  background: transparent;
  padding: 0;
  font: inherit;
  cursor: pointer;
}

.md-status {
  color: var(--good);
  font-size: 0.8125rem;
  line-height: 1.7;
  word-break: break-all;
}

.md-tip {
  font-size: 0.7812rem;
  color: var(--muted);
  line-height: 1.7;
}

.m-actions {
  display: flex;
  gap: 0.5rem;
  margin-top: 1.125rem;
  padding: 0 1.375rem 1.125rem;
  flex-wrap: wrap;
}

.m-actions .btn {
  flex: 1;
  min-width: 9.375rem;
  min-height: 3rem;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.375rem;
}

.status {
  margin: 0.625rem 1.375rem 1.125rem;
  color: var(--good);
  font-size: 0.8438rem;
  line-height: 1.7;
  white-space: normal;
  word-break: break-word;
  overflow-wrap: anywhere;
}
</style>
