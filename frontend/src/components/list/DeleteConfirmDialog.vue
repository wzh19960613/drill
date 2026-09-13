<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { FileX2, Trash2 } from 'lucide-vue-next'
import { deleteQuestionApi, fetchBooks, fetchQuestionRaw, markSourceFile } from '../../api'
import { selection, store } from '../../store'
import { useDialogShell } from '../../composables/useDialogShell'


const emit = defineEmits<{ (e: 'cancel'): void; (e: 'done'): void }>()

const deleting = ref(false)
const impact = ref<{ name: string; count: number }[]>([])

function selectedQuestions() {
  const ids = new Set(selection.ids)
  return store.questions.filter((q) => ids.has(q.id))
}

onMounted(async () => {
  try {
    const keys = new Set(selectedQuestions().map((q) => `${q.source}\u0000${q.id}`))
    impact.value = (await fetchBooks())
      .map((b) => ({
        name: b.name,
        count: b.items.filter((it) => keys.has(`${it.source ?? ''}\u0000${it.id}`)).length,
      }))
      .filter((x) => x.count > 0)
  } catch {
    
  }
})

async function deleteFiles() {
  deleting.value = true
  try {
    for (const q of selectedQuestions()) await deleteQuestionApi(q.id, q.source)
    selection.ids = []
    emit('done')
  } finally {
    deleting.value = false
  }
}

async function markAll() {
  deleting.value = true
  try {
    for (const q of selectedQuestions()) {
      const raw = await fetchQuestionRaw(q.id, q.source)
      const rel = raw.dir ? `${raw.dir}/${raw.file}` : raw.file
      await markSourceFile(raw.source, rel, true)
    }
    selection.ids = []
    emit('done')
  } finally {
    deleting.value = false
  }
}

useDialogShell((e) => {
  if (e.code === 'Escape') {
    e.stopPropagation()
    emit('cancel')
  }
})
</script>

<template>
  <Teleport to="body">
    <div class="modal-mask" @click.self="emit('cancel')">
      <div class="qdel-modal">
        <h3>删除题目</h3>
        <p>确定处理选中的 <b>{{ selection.ids.length }}</b> 个题目？</p>
        <template v-if="impact.length">
          <p>以下题本包含其中的题目，处理后对应题目会失效：</p>
          <ul class="del-impact">
            <li v-for="b in impact" :key="b.name">{{ b.name }} · {{ b.count }} 题</li>
          </ul>
        </template>
        <p class="qdel-warn">直接删除文件后无法恢复；标记为非题目保留文件，可随时在题源管理中恢复。</p>
        <div class="qdel-actions">
          <button class="btn ghost" :disabled="deleting" @click="emit('cancel')">取消</button>
          <button class="btn" :disabled="deleting" @click="markAll">
            <FileX2 style="width: 0.875rem; height: 0.875rem" /> 标记为非题目
          </button>
          <button class="btn bad" :disabled="deleting" @click="deleteFiles">
            <Trash2 style="width: 0.875rem; height: 0.875rem" /> 直接删除文件
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
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

.del-impact {
  margin: 0.375rem 0;
  padding-inline-start: 1.25rem;
  font-size: 0.875rem;
  color: var(--ink);
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
