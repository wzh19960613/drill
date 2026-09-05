<script setup lang="ts">
import { Eye } from 'lucide-vue-next'
import type { BookDef } from '../../types'

defineProps<{
  def: BookDef | null
}>()

const emit = defineEmits<{
  (e: 'view', def: BookDef): void
}>()
</script>

<template>
  <section class="card fav">
    <div class="fav-info">
      <span class="fav-label">收藏到题本</span>
      <template v-if="def">
        <span class="fav-title">{{ def.name }}</span>
        <span class="chip">{{ def.items.length }} 题</span>
      </template>
      <span v-else class="fav-hint">未设置 · 刷题时收藏题目会自动创建</span>
    </div>
    <button v-if="def" class="btn" @click="emit('view', def)">
      <Eye style="width: 0.9375rem; height: 0.9375rem" /> 查看
    </button>
  </section>
</template>

<style scoped>
.fav {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.875rem;
  flex-wrap: wrap;
  background: color-mix(in srgb, var(--warn) 10%, transparent);
  border-color: color-mix(in srgb, var(--warn) 40%, transparent);
  padding: 0.875rem 1.25rem;
}

.fav-info {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  flex-wrap: wrap;
}

.fav-label {
  font-size: 0.75rem;
  color: var(--warn);
  font-weight: 700;
  letter-spacing: 0.0625rem;
}

.fav-title {
  font-size: 1.0625rem;
  font-weight: 700;
}

.fav-hint {
  font-size: 0.8125rem;
  color: var(--muted);
}

.fav .btn {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
}
</style>
