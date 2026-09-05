<script setup lang="ts">
import { Trash2 } from 'lucide-vue-next'

defineProps<{
  selected: number
  total: number
}>()

const emit = defineEmits<{
  (e: 'selectAll'): void
  (e: 'invert'): void
  (e: 'requestDelete'): void
}>()
</script>

<template>
  <div class="foot-bar glass">
    <span class="fb-count">已选 <b class="num">{{ selected }}</b> / {{ total }}</span>
    <div class="btn-pair">
      <button class="btn ghost sm" @click="emit('selectAll')">全选</button>
      <button class="btn ghost sm" @click="emit('invert')">反选</button>
    </div>
    <button
      class="fb-del"
      :disabled="!selected"
      :aria-label="selected ? `删除选中的 ${selected} 题` : '删除'"
      :title="`删除选中（已选 ${selected}）`"
      @click="emit('requestDelete')"
    >
      <Trash2 style="width: 1rem; height: 1rem" />
    </button>
  </div>
</template>

<style scoped>
.foot-bar {
  position: sticky;
  bottom:calc(var(--bnav-h) + var(--bnav-gap) + 0.875rem);
  z-index: var(--z-float); /* above the sticky chapter headers */
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 0.875rem;
  border: 1px solid var(--line);
  border-radius: var(--radius-pill);
  margin: 0 0.75rem;
  margin-top: 2rem;
}

@container (min-width: 49.125rem) {
  .foot-bar {
    width: fit-content;
    max-width: 20rem;
    margin-left: auto;
    margin-right: auto;
    bottom: var(--bnav-gap);
  }
}

@container (max-width: 22.5rem) {
  .foot-bar {
    display: flex;
    flex-wrap: wrap;
    row-gap: 0.5rem;
    justify-content: center;
  }

  .fb-count {
    flex: 1 1 100%;
    text-align: center;
  }

  .btn-pair,
  .fb-del {
    flex: none;
  }
}

.fb-count {
  justify-self: start;
  min-width: 0;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  font-size: 0.8438rem;
  color: var(--muted);
}

.btn-pair {
  justify-self: center;
}

.fb-del {
  justify-self: end;

  flex: none;
  width: 2.75rem;
  height: 2.75rem;
  border: none;
  border-radius: 50%;
  background: var(--bad);
  color: #fff;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.fb-del:hover {
  opacity: 0.9;
}

.fb-del:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

@container (max-width: 40rem) {
  .foot-bar {
    margin: 0 var(--bnav-mx);
    margin-top: 2rem;
  }
}
</style>
