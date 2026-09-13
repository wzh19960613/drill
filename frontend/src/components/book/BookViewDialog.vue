<script setup lang="ts">
import { computed, ref } from 'vue'
import { Check, ClipboardCheck, Download, Eraser, FolderOpen, Pencil, Play, Trash2 } from 'lucide-vue-next'
import DialogHeader from '../DialogHeader.vue'
import ConfirmDialog from '../ConfirmDialog.vue'
import QuestionList from '../QuestionList.vue'
import { sessionFromDef } from '../../book'
import { store } from '../../store'
import { dateZh } from '../../format'
import type { BookDef, Question } from '../../types'

const props = defineProps<{ def: BookDef; isFavorite?: boolean }>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'start', review: boolean): void
  (e: 'export', def: BookDef): void
  (e: 'delete', def: BookDef): void
  (e: 'study-from', q: Question): void
  (e: 'open-detail', q: Question): void
  (e: 'rename', name: string): void
  (e: 'set-favorite', def: BookDef): void
  (e: 'open-sources'): void
  (e: 'clean-stale'): void
}>()

const questions = computed(() => sessionFromDef(props.def, store.questions).items)


const staleCount = computed(
  () => props.def.items.length - questions.value.length,
)

const confirmClean = ref(false)

function confirmCleanStale() {
  confirmClean.value = false
  emit('clean-stale')
}

const renaming = ref(false)
const renameDraft = ref('')

function startRename() {
  renameDraft.value = props.def.name
  renaming.value = true
}

async function commitRename() {
  const def = props.def
  const name = renameDraft.value.trim()
  renaming.value = false
  if (!name || name === def.name) return
  emit('rename', name)
}
</script>

<template>
  <Teleport to="body">
  <div class="modal-mask fs" @click.self="emit('close')">
    <div class="view-modal dialog fs">
      <DialogHeader
        flush
        :subtitle="`${def.items.length} 题 · ${dateZh(def.date)}${def.seed ? ` · 种子 ${def.seed}` : ''}`"
        @close="emit('close')"
      >
        <template #title>
          <span v-if="!renaming" class="vm-title">
            <span class="vm-name">{{ def.name }}</span>
            <button class="icon-btn" aria-label="重命名" title="重命名" @click="startRename">
              <Pencil style="width: 0.8125rem; height: 0.8125rem" />
            </button>
          </span>
          <span v-else class="vm-title">
            <input
              v-model="renameDraft"
              class="rename-input"
              maxlength="40"
              @keydown.escape.stop
              @keyup.enter="commitRename"
              @keyup.escape="renaming = false"
              @blur="commitRename"
            />
            <button class="btn sm primary" aria-label="保存名称" @click="commitRename">
              <Check style="width: 0.875rem; height: 0.875rem" />
            </button>
          </span>
        </template>
      </DialogHeader>
      <div v-if="staleCount" class="vm-stale">
        <div class="vm-stale-text">
          有 {{ staleCount }} 道题目已失效（题源被移除、文件夹被移动，或题源的「子文件夹」开关被关闭）。恢复题源或重新打开开关后，这些题目会自动回到题本。
        </div>
        <div class="vm-stale-actions">
          <button class="btn sm" @click="emit('open-sources')">
            <FolderOpen style="width: 0.875rem; height: 0.875rem" /> 题源管理
          </button>
          <button class="btn bad sm" @click="confirmClean = true">
            <Eraser style="width: 0.875rem; height: 0.875rem" /> 清理失效题目
          </button>
        </div>
      </div>
      <QuestionList :questions="questions" mode="view" @open="emit('open-detail', $event)" @study-from="emit('study-from', $event)" />
      <footer class="v-foot">
        <button class="btn primary" @click="emit('start', false)"><Play style="width: 0.9375rem; height: 0.9375rem" /> 开始刷题</button>
        <button class="btn" @click="emit('start', true)"><ClipboardCheck style="width: 0.9375rem; height: 0.9375rem" /> 对答案</button>
        <button class="btn" :class="{ on: isFavorite }" @click="emit('set-favorite', def)">
          {{ isFavorite ? '停止在此收藏' : '设为收藏题本' }}
        </button>
        <button class="btn" @click="emit('export', def)"><Download style="width: 0.9375rem; height: 0.9375rem" /> 导出</button>
        <button class="btn ghost danger" @click="emit('delete', def)"><Trash2 style="width: 0.9375rem; height: 0.9375rem" /> 删除</button>
      </footer>
    </div>

    <ConfirmDialog
      v-if="confirmClean"
      nested
      width="25rem"
      confirm-text="清理"
      @confirm="confirmCleanStale"
      @cancel="confirmClean = false"
    >
      确定从「{{ def.name }}」中移除 {{ staleCount }} 道失效题目？不会删除任何文件；但清理后即使题源恢复，这些题目也不会再回到题本。
    </ConfirmDialog>
  </div>
  </Teleport>
</template>

<style scoped>
.view-modal {
  width: min(62.5rem, 96vw);
  max-height: 88vh;
}

.vm-stale {
  flex: none;
  margin: 0.5rem 1rem 0;
  padding: 0.5rem 0.75rem;
  font-size: 0.8438rem;
  line-height: 1.55;
  color: var(--bad);
  background: var(--bad-weak);
  border-radius: 0.5rem;
}

.vm-stale-actions {
  display: flex;
  gap: 0.5rem;
  margin-top: 0.5rem;
  flex-wrap: wrap;
}

.v-foot {
  display: flex;
  gap: 0.5rem;
  padding:0.75rem 1rem calc(0.75rem + env(safe-area-inset-bottom));
  border-top:1px solid var(--line);
  flex: none;
}

.v-foot .btn {
  flex: 1;
  justify-content: center;
}

@container (max-width: 44.9375rem) {
  .v-foot {
    padding-left: 0.625rem;
    padding-right: 0.625rem;
    flex-wrap: wrap;
    row-gap: 0.5rem;
  }

  .v-foot .btn {
    padding: 0.5625rem 0.5rem;
    font-size: 0.875rem;
  }

  .v-foot .btn:nth-child(-n + 2) {
    flex: 1 1 calc(50% - 0.25rem);
  }

  .v-foot .btn:nth-child(n + 3) {
    flex: 1 1 calc(33.333% - 0.3333rem);
  }
}

@container (min-width: 45rem) {
  .v-foot {
    justify-content: flex-end;
  }

  .v-foot .btn {
    flex: none;
  }
}

.view-modal :deep(.ql) {
  flex: 1;
  overflow-y: auto; /* the dialog scrolls vertically by itself */
  overscroll-behavior: contain;
}

/* Inside the view dialog the list spans edge to edge (no outer margins), so the panel no longer needs a border or rounding */
.view-modal :deep(.ql-panel) {
  border: none;
  border-radius: 0;
}

/* View dialog title + inline rename; the name itself truncates with an
   ellipsis so the rename button always stays visible */
.vm-title {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  min-width: 0;
  max-width: 100%;
}

.vm-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.rename-input {
  width: 12.5rem;
  font-size: 0.9062rem;
  font-weight: 700;
  padding: 0.375rem 0.625rem;
}
</style>
