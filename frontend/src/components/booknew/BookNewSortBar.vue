<script setup lang="ts">
import { ArrowUpDown, Dices } from 'lucide-vue-next'
import { SORT_OPTIONS, type SortKey } from '../../filter'

defineProps<{
  sort: SortKey
  seed: string
}>()

const emit = defineEmits<{
  (e: 'update:sort', v: SortKey): void
  (e: 'update:seed', v: string): void
  (e: 'dice'): void
}>()

function onSortChange(e: Event) {
  emit('update:sort', (e.target as HTMLSelectElement).value as SortKey)
}

function onSeedInput(e: Event) {
  emit('update:seed', (e.target as HTMLInputElement).value)
}
</script>

<template>
  <div class="bn-sort" :class="{ random: sort === 'random' }">
    <label class="sort-wrap">
      <ArrowUpDown style="width: 0.875rem; height: 0.875rem" />
      <select :value="sort" class="sort-sel" @change="onSortChange">
        <option v-for="so in SORT_OPTIONS" :key="so.value" :value="so.value">{{ so.label }}</option>
      </select>
    </label>
    <div v-if="sort === 'random'" class="seed-line">
      <span class="seed-label">种子</span>
      <span class="seed-box">
        <input type="text" :value="seed" placeholder="如 7A2F" @input="onSeedInput" />
        <button class="seed-dice" aria-label="换一个种子" title="换一个种子" @click="emit('dice')">
          <Dices style="width: 0.9375rem; height: 0.9375rem" />
        </button>
      </span>
    </div>
  </div>
</template>

<style scoped>
.bn-sort {
  grid-column: 1;
  grid-row: 2;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
}

.bn-sort.random {
  justify-content: space-between;
}

.bn-sort .sort-wrap {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--muted);
  font-size: 0.875rem;
}

.bn-sort .sort-sel {
  height: 2.75rem;
  max-width: 9rem;
  padding: 0 0.5rem;
  border: 1px solid var(--line);
  border-radius:0.5rem;
  background: var(--card);
  color: var(--ink);
  font-size: 0.8438rem;
  text-overflow: ellipsis;
}

.seed-line {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.seed-line .seed-label {
  font-size: 0.8438rem;
  color: var(--muted);
  white-space: nowrap;
}

.seed-box {
  position: relative;
  display: inline-flex;
  align-items: center;
}

.seed-box input {
  width: 6rem;
  height: 2.75rem;
  padding: 0 2.375rem 0 0.625rem;
  border: 1px solid var(--line);
  border-radius:0.5rem;
  background: var(--card);
  color: var(--ink);
}

.seed-dice {
  position: absolute;
  right: 0.25rem;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.875rem;
  height: 1.875rem;
  border: none;
  background: transparent;
  border-radius:0.375rem;
  color: var(--muted);
  cursor: pointer;
}

.seed-dice:hover {
  background: var(--hover);
  color: var(--ink);
}

@container (max-width: 40rem) {
  .bn-sort {
    padding-inline: 0.75rem;
  }
}
</style>
