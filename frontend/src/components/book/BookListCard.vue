<script setup lang="ts">
import { Eye, Trash2 } from 'lucide-vue-next'
import type { BookDef } from '../../types'
import { dateZh } from '../../format'

defineProps<{
  title: string
  count: number
  books: BookDef[]
  activeId: string | null
  loading: boolean
  emptyHint: string
  mixed?: boolean
  subjectsOf?: (b: BookDef) => string[]
  showSubjects?: boolean
  subjectCountOf?: (b: BookDef) => number
  currentSubject?: string
}>()

const emit = defineEmits<{
  (e: 'view', def: BookDef): void
  (e: 'delete', def: BookDef): void
}>()
</script>

<template>
  <div class="bk-block">
    <header class="bk-head">
      <span class="bk-name">{{ title }}</span>
      <span class="chip">共 {{ count }} 本</span>
    </header>

    <section class="card bk-card" :class="{ 'bk-also': mixed }">
      <div v-if="loading" class="empty">加载中…</div>
      <div v-else-if="!books.length" class="empty">{{ emptyHint }}</div>

      <div v-else class="bk-list">
        <div v-for="b in books" :key="b.id" class="bk-row" :class="{ active: b.id === activeId }">
          <div class="bk-info">
            <span class="bk-title">{{ b.name }}</span>
            <span v-if="!mixed && showSubjects && subjectsOf?.(b).length" class="chip">{{ subjectsOf(b).join(' · ') }}</span>
            <span v-if="mixed && currentSubject" class="chip">
              含{{ currentSubject }} {{ subjectCountOf?.(b) }} 题 · 共 {{ b.items.length }} 题
            </span>
            <span v-if="!mixed" class="chip">{{ b.items.length }} 题</span>
            <span class="chip">{{ dateZh(b.date) }}</span>
            <span v-if="b.id === activeId" class="chip brand">当前使用</span>
          </div>
          <div class="bk-actions">
            <button class="btn sm" @click="emit('view', b)"><Eye style="width: 0.875rem; height: 0.875rem" /> 查看</button>
            <button class="btn ghost sm danger" aria-label="删除" @click="emit('delete', b)">
              <Trash2 style="width: 0.875rem; height: 0.875rem" />
            </button>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.bk-head {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding:0 4px 0.375rem;
  font-size: 0.8438rem;
  font-weight: 600;
  color: var(--brand);
}

.bk-name {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.bk-list {
  display: flex;
  flex-direction: column;
}

.bk-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  padding:0.75rem 4px;
  border-bottom:1px solid var(--line);
  flex-wrap: wrap;
}

.bk-row:last-child {
  border-bottom: none;
}

.bk-row.active .bk-title {
  color: var(--brand);
}

.bk-info {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.bk-title {
  font-weight: 700;
  font-size: 0.9062rem;
  min-width: 3.75rem;
}

.bk-actions {
  display: flex;
  gap: 0.375rem;
  align-items: center;
  flex-wrap: wrap;
}

.bk-actions .btn {
  display: inline-flex;
  align-items: center;
  gap: 0.3125rem;
}

.bk-also .bk-title {
  color: var(--ink);
}

@container (max-width: 40rem) {
  .bk-head {
    margin-top: 1rem;
    padding-left: 1rem;
    padding-right: 1rem;
  }

  .bk-card {
    border-radius: 0;
    border-left: none;
    border-right: none;
  }

  .bk-info .icon-btn {
    display: none;
  }

  .bk-info .chip:not(.brand) {
    display: none;
  }

  .bk-actions .btn:not(:first-child) {
    display: none;
  }
}
</style>
