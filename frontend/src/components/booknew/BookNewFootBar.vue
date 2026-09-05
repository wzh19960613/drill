<script setup lang="ts">
import { Plus } from 'lucide-vue-next'

defineProps<{
  selected: number
  total: number
}>()

const emit = defineEmits<{
  (e: 'selectAll'): void
  (e: 'invert'): void
  (e: 'start'): void
}>()
</script>

<template>
  <div class="bnd-foot glass">
    <span class="bf-count">已选 <b class="num">{{ selected }}</b> / {{ total }}</span>
    <div class="btn-pair">
      <button class="btn ghost sm" @click="emit('selectAll')">全选</button>
      <button class="btn ghost sm" @click="emit('invert')">反选</button>
    </div>
    <button class="btn primary sm bnd-go" @click="emit('start')">
      <Plus style="width: 0.875rem; height: 0.875rem" /> 新建
    </button>
  </div>
</template>

<style scoped>
.bnd-foot {
  position: sticky;
  bottom:calc(0.625rem + env(safe-area-inset-bottom));
  z-index: var(--z-float);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  padding: 0.75rem 0.875rem;
  border: 1px solid var(--line);
  border-radius: var(--radius-pill);
  margin-block: 2rem;
  margin-inline: var(--bnav-mx);
}

.bf-count {
  font-size: 0.8438rem;
  color: var(--muted);
}

.bnd-go {
  flex: none;
  border-radius: var(--radius-pill);
  padding-inline: 0.875rem;
  min-width: 3.75rem;
}

@container (max-width: 22.5rem) {
  .bnd-foot {
    flex-wrap: wrap;
    row-gap: 0.5rem;
    justify-content: center;
  }

  .bf-count {
    flex: 1 1 100%;
    text-align: center;
  }

  .btn-pair,
  .bnd-go {
    flex: none;
  }
}
</style>
