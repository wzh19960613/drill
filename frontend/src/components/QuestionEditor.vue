<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { Eye, FileText, Save, Trash2 } from 'lucide-vue-next'
import type { Question } from '../types'
import type { SourceInfo } from '../api'
import DialogHeader from './DialogHeader.vue'
import QuestionView from './QuestionView.vue'
import ConfirmDialog from './ConfirmDialog.vue'
import { useDialogShell } from '../composables/useDialogShell'
import { useQuestionEditor } from '../composables/useQuestionEditor'

const props = defineProps<{
  question: Question | null
  sources: SourceInfo[]
}>()

const emit = defineEmits<{ (e: 'close'): void; (e: 'saved'): void }>()

const source = ref(props.question?.source ?? props.sources[0]?.id ?? '')
const sourceName = computed(
  () => props.sources.find((s) => s.id === source.value)?.name ?? source.value,
)
const confirmDel = ref(false)

const {
  md,
  previewQ,
  showPreview,
  busy,
  errMsg,
  info,
  loadInitial,
  togglePreview,
  save,
  doDelete,
} = useQuestionEditor({
  question: () => props.question,
  source,
  onSaved: () => emit('saved'),
})

useDialogShell((e) => {
  if (e.code === 'Escape') {
    if (confirmDel.value) {
      confirmDel.value = false
      return
    }
    e.stopPropagation()
    emit('close')
  } else if ((e.metaKey || e.ctrlKey) && e.code === 'KeyS') {
    e.preventDefault()
    void save()
  }
})

onMounted(() => void loadInitial())
</script>

<template>
  <Teleport to="body">
  <div class="modal-mask ed-mask">
    <div class="ed dialog">
      <DialogHeader flush @close="emit('close')">
        <template #title>
          {{ question ? `编辑题目 ${question.id}` : '新增题目' }}<span class="src">（{{ sourceName }}）</span>
        </template>
        <template #actions>
          <button class="btn ghost sm" @click="togglePreview">
            <Eye v-if="!showPreview" style="width: 0.9375rem; height: 0.9375rem" /> <FileText v-else style="width: 0.9375rem; height: 0.9375rem" />
            {{ showPreview ? '编辑源码' : '预览渲染' }}
          </button>
          <button v-if="question" class="btn ghost sm danger" @click="confirmDel = true">
            <Trash2 style="width: 0.875rem; height: 0.875rem" /> 删除
          </button>
        </template>
      </DialogHeader>

      <div class="ed-body">
        <textarea
          v-if="!showPreview"
          v-model="md"
          class="ed-text"
          spellcheck="false"
          placeholder="Markdown 源码"
        ></textarea>
        <div v-else class="ed-preview">
          <QuestionView v-if="previewQ" :q="previewQ" :show-answer="true" />
          <div v-else class="ed-err">{{ errMsg || '解析中…' }}</div>
        </div>
      </div>

      <div class="ed-foot">
        <select v-if="!question && sources.length > 1" v-model="source" class="ed-src" aria-label="保存到题源">
          <option v-for="s in sources" :key="s.id" :value="s.id">{{ s.name }}</option>
        </select>
        <span class="ed-msg err" v-if="errMsg">{{ errMsg }}</span>
        <span class="ed-msg ok" v-else-if="info">{{ info }}</span>
        <span v-else class="ed-hint">首行「> [学科] 来源 | 章节 @ 定位 > 题型」，各项可省略；⌘S 保存</span>
        <span class="ed-count num">{{ md.length }} 字</span>
        <button class="btn ghost" @click="emit('close')">取消</button>
        <button class="btn primary" :disabled="busy || !md.trim()" @click="save">
          <Save style="width: 0.9375rem; height: 0.9375rem" /> {{ question ? '保存' : '新增' }}
        </button>
      </div>

      <ConfirmDialog
        v-if="confirmDel"
        nested
        :busy="busy"
        width="23.75rem"
        @confirm="doDelete"
        @cancel="confirmDel = false"
      >
        确定删除「<b>{{ question?.id }}</b>」？文件会移除（保留 .bak 备份），不可在应用内恢复。
      </ConfirmDialog>
    </div>
  </div>
  </Teleport>
</template>

<style scoped>
.ed-mask {
  z-index: var(--z-detail);
  background: rgba(0, 0, 0, 0.4);
  padding: 1.5rem;
}

.ed {
  position: relative;
  width: min(60rem, 100%);
  height: min(86vh, 56.25rem);
}

.ed-body {
  flex: 1;
  display: flex;
  min-height: 0;
}

.ed-text {
  flex: 1;
  border: none;
  outline: none;
  resize: none;
  padding: 1rem 1.25rem;
  font-family: var(--font-mono);
  font-size: 0.875rem;
  line-height: 1.8;
  background: var(--card);
  color: var(--ink);
}

.ed-preview {
  flex: 1;
  overflow-y: auto;
  overflow-x: clip;
  padding: 1.25rem 1.625rem;
}

.ed-err {
  color: var(--bad);
  padding: 1.25rem;
}

.ed-foot {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.625rem 1.125rem;
  border-top:1px solid var(--line);
}

.ed-src {
  flex: none;
  max-width: 9rem;
  font-size: 0.8438rem;
  height: 2.25rem;
}

.ed-msg.err {
  color: var(--bad);
  font-size: 0.8125rem;
  flex: 1;
}

.ed-msg.ok {
  color: var(--good);
  font-size: 0.8125rem;
  flex: 1;
}

.ed-hint {
  flex: 1;
  color: var(--muted);
  font-size: 0.7812rem;
}

.ed-count {
  color: var(--muted);
  font-size: 0.7812rem;
}
</style>
