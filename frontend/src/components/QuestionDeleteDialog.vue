<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { FileX2, Trash2 } from 'lucide-vue-next'
import { deleteQuestionApi, fetchBooks, fetchQuestionRaw, markSourceFile } from '../api'
import { useDialogShell } from '../composables/useDialogShell'
import type { Question } from '../types'


const props = defineProps<{ question: Question }>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'deleted'): void
}>()

const impact = ref<string[]>([])
const busy = ref(false)
const errMsg = ref('')

onMounted(async () => {
  try {
    const books = await fetchBooks()
    impact.value = books
      .filter((b) => b.items.some((it) => it.source === props.question.source && it.id === props.question.id))
      .map((b) => b.name)
  } catch {
    
  }
})

async function markNonQuestion() {
  busy.value = true
  errMsg.value = ''
  try {
    const raw = await fetchQuestionRaw(props.question.id, props.question.source)
    const rel = raw.dir ? `${raw.dir}/${raw.file}` : raw.file
    await markSourceFile(raw.source, rel, true)
    emit('deleted')
  } catch (e) {
    errMsg.value = e instanceof Error ? e.message : '标记失败'
  } finally {
    busy.value = false
  }
}

async function deleteFile() {
  busy.value = true
  errMsg.value = ''
  try {
    await deleteQuestionApi(props.question.id, props.question.source)
    emit('deleted')
  } catch (e) {
    errMsg.value = e instanceof Error ? e.message : '删除失败'
  } finally {
    busy.value = false
  }
}

useDialogShell((e) => {
  if (e.code === 'Escape') {
    e.stopPropagation()
    emit('close')
  }
})
</script>

<template>
  <Teleport to="body">
    <div class="modal-mask lift" @click.self="emit('close')">
      <div class="qdel-modal">
        <h3>删除题目「{{ question.id }}」？</h3>
        <p>
          <template v-if="impact.length">
            它在 {{ impact.length }} 个题本（{{ impact.join('、') }}）中，删除或标记后这些题本中的对应题目会失效。
          </template>
          <template v-else>它没有被任何题本引用。</template>
        </p>
        <p class="qdel-warn">直接删除文件后无法恢复；标记为非题目则保留文件，可随时在题源管理中恢复。</p>
        <p v-if="errMsg" class="qdel-err">{{ errMsg }}</p>
        <div class="qdel-actions">
          <button class="btn ghost" :disabled="busy" @click="emit('close')">取消</button>
          <button class="btn" :disabled="busy" @click="markNonQuestion">
            <FileX2 style="width: 0.875rem; height: 0.875rem" /> 标记为非题目
          </button>
          <button class="btn bad" :disabled="busy" @click="deleteFile">
            <Trash2 style="width: 0.875rem; height: 0.875rem" /> 直接删除文件
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-mask.lift {
  z-index: var(--z-nested);
}

.qdel-modal {
  width: min(27.5rem, 92vw);
  background: var(--card);
  border: 1px solid var(--line);
  border-radius: var(--radius-dlg);
  box-shadow: var(--shadow-modal);
  padding: 1.25rem 1.25rem 1rem;
}

.qdel-modal h3 {
  font-size: 1rem;
  margin-bottom: 0.625rem;
}

.qdel-modal p {
  font-size: 0.875rem;
  line-height: 1.6;
  color: var(--ink);
}

.qdel-warn {
  color: var(--muted);
}

.qdel-err {
  color: var(--bad);
}

.qdel-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  flex-wrap: wrap;
  margin-top: 1rem;
}

.qdel-actions .btn {
  display: inline-flex;
  align-items: center;
  gap: 0.3125rem;
}
</style>
