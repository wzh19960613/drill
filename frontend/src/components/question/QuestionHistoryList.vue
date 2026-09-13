<script setup lang="ts">
import { computed, ref } from 'vue'
import { Trash2, X } from 'lucide-vue-next'
import { removeRecord, store, type QuestionLike } from '../../store'
import { fmtTime } from '../../format'
import type { Rec } from '../../types'
import ConfirmDialog from '../ConfirmDialog.vue'

const props = defineProps<{
  q: QuestionLike
}>()

const historyAll = computed(() =>
  store.records
    .filter((r) => r.source === props.q.source && r.questionId === props.q.id)
    .sort((a, b) => b.at - a.at),
)


const pendingDel = ref<Rec | null>(null)

async function doDelete() {
  const r = pendingDel.value
  if (!r) return
  pendingDel.value = null
  await removeRecord(r.id)
}

const clearConfirm = ref(false)
const clearing = ref(false)

async function doClear() {
  clearing.value = true
  try {
    for (const r of [...historyAll.value]) await removeRecord(r.id)
    clearConfirm.value = false
  } finally {
    clearing.value = false
  }
}


function handleEscape(): boolean {
  if (pendingDel.value) {
    pendingDel.value = null
    return true
  }
  if (clearConfirm.value) {
    clearConfirm.value = false
    return true
  }
  return false
}

defineExpose({ handleEscape })
</script>

<template>
  <div class="qm-hist">
    <div class="h-head">
      <span class="h-title">做题记录（{{ historyAll.length }} 条）</span>
      <template v-if="historyAll.length">
        <template v-if="clearConfirm">
          <span class="h-confirm">清空本题全部记录？</span>
          <button class="btn bad sm" :disabled="clearing" @click="doClear">确认清空</button>
          <button class="btn ghost sm" @click="clearConfirm = false">取消</button>
        </template>
        <button v-else class="btn ghost sm" @click="clearConfirm = true">
          <Trash2 style="width: 0.875rem; height: 0.875rem" /> 清空记录
        </button>
      </template>
    </div>
    <div v-if="!historyAll.length" class="h-empty">还没做过这道题</div>
    <div v-for="r in historyAll" :key="r.id" class="h-row">
      <span :class="r.correct ? 'chip good' : 'chip bad'">{{ r.correct ? '对' : '错' }}</span>
      <span class="num time">{{ fmtTime(r.at) }}</span>
      <span v-if="r.ms" class="num time">{{ Math.max(1, Math.round(r.ms / 1000)) }}s</span>
      <button
        class="h-del"
        aria-label="删除"
        title="删除"
        @click="pendingDel = r"
      >
        <X style="width: 0.875rem; height: 0.875rem" />
      </button>
    </div>

    <ConfirmDialog
      v-if="pendingDel"
      nested
      title="删除做题记录"
      @confirm="doDelete"
      @cancel="pendingDel = null"
    >
      删除「<b>{{ pendingDel.correct ? '对' : '错' }}</b> · {{ fmtTime(pendingDel.at) }}」这条记录？
    </ConfirmDialog>
  </div>
</template>

<style scoped>
.h-del {
  margin-left: auto;
  display: inline-flex;
  align-items: center;
  border: none;
  background: transparent;
  padding: 0.25rem;
  border-radius: 0.375rem;
  color: var(--muted);
  font: inherit;
  font-size: 0.7812rem;
  cursor: pointer;
}

.h-del:hover {
  background: var(--hover);
  color: var(--bad);
}

.qm-hist {
  margin-top: 0.75rem;
  border-top:1px dashed var(--line);
  padding-top: 0.625rem;
}

.h-head {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.375rem;
  min-height: 2.125rem;
}

.h-title {
  font-size: 0.8125rem;
  color: var(--muted);
  margin-right: auto;
}

.h-confirm {
  font-size: 0.8125rem;
  color: var(--bad);
}

.h-empty {
  font-size: 0.8438rem;
  color: var(--muted);
  padding:4px 0;
}

.h-row {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding:0.4375rem 2px;
  border-bottom:1px dashed var(--line);
  font-size: 0.8438rem;
}

.h-row:last-child {
  border-bottom: none;
}

.time {
  color: var(--muted);
}
</style>
