<script setup lang="ts">
import { ClipboardCheck, Eye, Play } from 'lucide-vue-next'
import type { BookDef } from '../../types'
import { dateZh } from '../../format'

defineProps<{
  def: BookDef
  resumeActive: boolean
  subjectsLine: string
}>()

const emit = defineEmits<{
  (e: 'start', review: boolean): void
  (e: 'view'): void
}>()
</script>

<template>
  <section class="card cur">
    <div class="cur-info">
      <span class="cur-label">当前题本</span>
      <span class="cur-title">{{ def.name }}</span>
      <span v-if="subjectsLine" class="chip">{{ subjectsLine }}</span>
      <span class="chip">{{ def.items.length }} 题</span>
      <span class="chip">{{ dateZh(def.date) }}</span>
    </div>
    <div class="cur-actions">
      <button class="btn primary" @click="emit('start', false)">
        <Play style="width: 0.9375rem; height: 0.9375rem" />
        <span class="go-full">{{ resumeActive ? '继续刷题' : '开始刷题' }}</span>
        <span class="go-short">{{ resumeActive ? '继续' : '刷题' }}</span>
      </button>
      <button class="btn" @click="emit('start', true)">
        <ClipboardCheck style="width: 0.9375rem; height: 0.9375rem" /> 对答案
      </button>
      <button class="btn" @click="emit('view')">
        <Eye style="width: 0.9375rem; height: 0.9375rem" /> 查看
      </button>
    </div>
  </section>
</template>

<style scoped>
.cur {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.875rem;
  flex-wrap: wrap;
  background: var(--brand-weak);
  border-color: color-mix(in srgb, var(--brand) 45%, transparent);
  box-shadow: 0 8px 28px color-mix(in srgb, var(--brand) 20%, transparent);
  padding: 1.25rem 1.5rem;
}

.cur-info {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  flex-wrap: wrap;
}

.cur-label {
  font-size: 0.75rem;
  color: var(--brand);
  font-weight: 700;
  letter-spacing: 0.0625rem;
}

.cur-title {
  font-size: 1.25rem;
  font-weight: 800;
}

.cur-actions {
  display: flex;
  gap: 0.5rem;
}

.cur-actions .btn {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
}

.go-short {
  display: none;
}

@container (max-width: 40rem) {
  .go-full {
    display: none;
  }

  .go-short {
    display: inline;
  }
}
</style>
