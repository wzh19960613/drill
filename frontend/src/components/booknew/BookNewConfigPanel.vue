<script setup lang="ts">
import { Download } from 'lucide-vue-next'

defineProps<{
  selectedCount: number
  bookName: string
  shuffleO: boolean
}>()

const emit = defineEmits<{
  (e: 'update:bookName', v: string): void
  (e: 'update:shuffleO', v: boolean): void
  (e: 'export'): void
}>()

function onNameInput(e: Event) {
  emit('update:bookName', (e.target as HTMLInputElement).value)
}
</script>

<template>
  <section class="card opts">
    <div class="opt-grid">
      <div class="opt">
        <div class="opt-label">题本名称</div>
        <input
          type="text"
          :value="bookName"
          class="name-input"
          maxlength="40"
          placeholder="如：第一轮"
          @input="onNameInput"
        />
      </div>
      <div class="opt">
        <div class="opt-label">选项顺序（选择、多选题）</div>
        <div class="seg">
          <button type="button" :class="{ on: !shuffleO }" @click="emit('update:shuffleO', false)">
            保持原顺序
          </button>
          <button type="button" :class="{ on: shuffleO }" @click="emit('update:shuffleO', true)">
            随机
          </button>
        </div>
      </div>
    </div>

    <div class="alt-export">
      <button class="btn ghost" :disabled="!selectedCount" @click="emit('export')">
        <Download style="width: 0.9375rem; height: 0.9375rem" /> 不生成新题本，直接导出文件
      </button>
    </div>
  </section>
</template>

<style scoped>
.bn .opts {
  grid-column: 2;
  grid-row: 1 / 4;
  position: sticky;
  top: 0;
}

.opt-grid {
  display: flex;
  flex-direction: column;
  gap: 0.875rem;
}

.opt-label {
  font-size: 0.8125rem;
  color: var(--muted);
  font-weight: 600;
  margin:4px;
}

.name-input {
  width: 100%;
  font-size: 0.875rem;
}

.alt-export {
  margin-top: 0.75rem;
}

.alt-export .btn {
  width: 100%;
}

@container (max-width: 61.25rem) {
  .bn .opts {
    grid-column: 1;
    grid-row: auto;
    position: static;
  }
}

@container (max-width: 40rem) {
  .bn .opts {
    border-radius: 0;
    border-left: none;
    border-right: none;
  }
}
</style>
