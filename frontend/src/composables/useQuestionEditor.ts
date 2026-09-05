import { ref, type Ref } from 'vue'
import type { Question } from '../types'
import {
  createQuestion,
  deleteQuestionApi,
  fetchQuestionRaw,
  previewQuestion,
  updateQuestionApi,
} from '../api'

const NEW_TEMPLATE = `> [学科] 来源 | 第x章 章名 @ P页码-题号 > 选择题

题干（　）。

(A) 

(B) 

(C) 

(D) 

## 答案

**（A）**。

## 解析

推理步骤。`

export interface UseQuestionEditorOptions {
  question: () => Question | null
  source: Ref<string>
  onSaved: () => void
}

export function useQuestionEditor(opts: UseQuestionEditorOptions) {
  const md = ref('')
  const previewQ = ref<Question | null>(null)
  const showPreview = ref(false)
  const busy = ref(false)
  const errMsg = ref('')
  const info = ref('')
  let previewSeq = 0

  async function loadInitial() {
    const question = opts.question()
    if (question) {
      try {
        const r = await fetchQuestionRaw(question.id, opts.source.value)
        md.value = r.markdown
      } catch (e) {
        errMsg.value = e instanceof Error ? e.message : '读取原文失败'
      }
    } else {
      md.value = NEW_TEMPLATE
    }
  }

  async function refreshPreview() {
    // A slow earlier response must not overwrite a newer one
    const seq = ++previewSeq
    try {
      const q = await previewQuestion(md.value, opts.source.value)
      if (seq !== previewSeq) return
      previewQ.value = q
      errMsg.value = ''
    } catch (e) {
      if (seq !== previewSeq) return
      previewQ.value = null
      errMsg.value = e instanceof Error ? e.message : '解析失败'
    }
  }

  function togglePreview() {
    showPreview.value = !showPreview.value
    if (showPreview.value) void refreshPreview()
  }

  async function save() {
    const question = opts.question()
    busy.value = true
    errMsg.value = ''
    info.value = ''
    try {
      if (question) {
        await updateQuestionApi(question.id, md.value, opts.source.value)
        info.value = `已保存 ${question.id}.md`
      } else {
        const q = await createQuestion(md.value, opts.source.value)
        info.value = `已新增 ${q.id}.md`
      }
      opts.onSaved()
    } catch (e) {
      errMsg.value = e instanceof Error ? e.message : '保存失败'
    } finally {
      busy.value = false
    }
  }

  async function doDelete() {
    const question = opts.question()
    if (!question) return
    busy.value = true
    try {
      await deleteQuestionApi(question.id, opts.source.value)
      opts.onSaved()
    } catch (e) {
      errMsg.value = e instanceof Error ? e.message : '删除失败'
    } finally {
      busy.value = false
    }
  }

  return { md, previewQ, showPreview, busy, errMsg, info, loadInitial, refreshPreview, togglePreview, save, doDelete }
}
