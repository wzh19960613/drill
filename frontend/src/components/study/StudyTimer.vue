<script setup lang="ts">
import { Pause, Play, RotateCcw } from 'lucide-vue-next'
import { fmtDur } from '../../format'

defineProps<{
  sessionMs: number
  questionMs: number
  paused: boolean
  frozen: boolean
  toggleHint: string
  resetHint: string
}>()

const emit = defineEmits<{ (e: 'toggle'): void; (e: 'reset'): void }>()
</script>

<template>
  <div class="s-timer" :class="{ paused: frozen }">
    <div class="t-times">
      <div class="t-row">
        <span class="t-lab">总计</span><span class="num t-total">{{ fmtDur(sessionMs) }}</span>
      </div>
      <div class="t-row">
        <span class="t-lab">本题</span><span class="num t-cur">{{ fmtDur(questionMs) }}</span>
      </div>
    </div>
    <button
      class="t-btn"
      :aria-label="paused ? '恢复计时' : '暂停计时'"
      :title="`暂停/恢复计时（${toggleHint}）`"
      @click="emit('toggle')"
    >
      <Play v-if="paused" style="width: 0.875rem; height: 0.875rem" />
      <Pause v-else style="width: 0.875rem; height: 0.875rem" />
    </button>
    <button class="t-btn" aria-label="重置本题计时" :title="`重置本题计时（${resetHint}）`" @click="emit('reset')">
      <RotateCcw style="width: 0.8125rem; height: 0.8125rem" />
    </button>
  </div>
</template>

<style scoped>
.s-timer {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  flex: none;
  font-size: 0.75rem;
  color: var(--muted);
  padding:4px 0.375rem 4px 0.625rem;
  border-radius: var(--radius-pill);
  background: var(--panel);
  white-space: nowrap;
}

.t-times {
  display: flex;
  flex-direction: column;
  gap:1px;
}

.t-row {
  display: flex;
  align-items: baseline;
  gap: 0.375rem;
  line-height: 1.4;
}

.t-lab {
  flex: none;
  width: 1.5rem;
  font-size: 0.6875rem;
}

.s-timer .num {
  display: inline-block;
  width: 3.25rem;
  text-align: center;
  font-variant-numeric: tabular-nums;
}

.s-timer .t-total {
  font-weight: 600;
  color: var(--ink);
}

.s-timer .t-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.875rem;
  height: 1.875rem;
  border: none;
  background: transparent;
  color: var(--muted);
  border-radius:0.5rem;
  cursor: pointer;
}

.s-timer .t-btn:hover {
  background: var(--hover);
  color: var(--ink);
}

.s-timer.paused {
  background: var(--warn-weak);
  color: var(--warn);
}

@container (max-width: 40rem) {
  .s-timer {
    padding-left: 0.5rem;
  }

  .t-lab {
    display: none;
  }
}
</style>
