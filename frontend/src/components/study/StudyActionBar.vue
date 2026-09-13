<script setup lang="ts">
import { Check, ChevronLeft, ChevronRight, Eye, EyeOff, X } from 'lucide-vue-next'
import { fmtKey, primaryCode } from '../../hotkeys'

defineProps<{
  review: boolean
  showAnswer: boolean
  idx: number
  total: number
  
  marked: boolean | null
  hotkeys: Record<string, string>
}>()

const emit = defineEmits<{
  (e: 'mark', correct: boolean): void
  (e: 'show-answer'): void
  (e: 'hide-answer'): void
  (e: 'prev'): void
  (e: 'next'): void
}>()
</script>

<template>
  <footer class="s-bar">
    <slot></slot>

    <!-- Drilling, answer still hidden -->
    <div v-if="!review && !showAnswer" class="s-actions">
      <button class="act side" :disabled="idx === 0" @click="emit('prev')">
        <ChevronLeft style="width: 0.875rem; height: 0.875rem" /> 上一题 <span class="kbd">{{ fmtKey(primaryCode(hotkeys.prev)) }}</span>
      </button>
      <button class="act primary" @click="emit('show-answer')">
        <Eye style="width: 1.0312rem; height: 1.0312rem" /> 显示答案 <span class="kbd">{{ fmtKey(primaryCode(hotkeys.showAnswer)) }}</span>
      </button>
      <button class="act side" aria-label="跳过" :title="`跳过（${fmtKey(primaryCode(hotkeys.next))}）`" @click="emit('next')">
        跳过 <span class="kbd">{{ fmtKey(primaryCode(hotkeys.next)) }}</span><ChevronRight style="width: 0.875rem; height: 0.875rem" />
      </button>
    </div>

    <!-- Review mode, or revisiting an answered question: verdict buttons plus
         free navigation (icon-only prev/next). A marked verdict disables its
         own button (tap the other one to overwrite); the last question's
         "next" finishes the round -->
    <div v-else class="s-actions">
      <button
        class="act icon"
        :disabled="idx === 0"
        aria-label="上一题"
        title="上一题"
        @click="emit('prev')"
      >
        <ChevronLeft style="width: 1.125rem; height: 1.125rem" />
      </button>
      <button class="act good" :disabled="marked === true" @click="emit('mark', true)">
        <Check style="width: 1.0312rem; height: 1.0312rem" /> {{ marked === true ? '已做对' : '做对了' }} <span class="kbd">{{ fmtKey(primaryCode(hotkeys.markRight)) }}</span>
      </button>
      <button v-if="!review" class="act side mid" @click="emit('hide-answer')">
        <EyeOff style="width: 0.875rem; height: 0.875rem" /> 隐藏答案 <span class="kbd">{{ fmtKey(primaryCode(hotkeys.hideAnswer)) }}</span>
      </button>
      <button class="act bad" :disabled="marked === false" @click="emit('mark', false)">
        <X style="width: 1.0312rem; height: 1.0312rem" /> {{ marked === false ? '已做错' : '做错了' }} <span class="kbd">{{ fmtKey(primaryCode(hotkeys.markWrong)) }}</span>
      </button>
      <button
        v-if="idx < total - 1"
        class="act icon"
        aria-label="下一题"
        title="下一题"
        @click="emit('next')"
      >
        <ChevronRight style="width: 1.125rem; height: 1.125rem" />
      </button>
      <button v-else class="act side next finish" @click="emit('next')">
        完成 <ChevronRight style="width: 0.875rem; height: 0.875rem" />
      </button>
    </div>
  </footer>
</template>

<style scoped>
.s-bar {
  position: relative;
  background: var(--card);
  border-top:1px solid var(--line);
  padding:0.75rem 1.125rem calc(0.75rem + env(safe-area-inset-bottom));
}

.s-actions {
  display: flex;
  gap: 0.625rem;
  align-items: stretch;
}

.act {
  flex: 1;
  min-height: 3rem;
  font-size: 1.0312rem;
  border-radius:0.75rem;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  border: none;
  background: transparent;
  cursor: pointer;
  font-family: inherit;
  color: var(--muted);
  white-space: nowrap;
}

.act.side {
  flex: none;
  min-width: 5.375rem;
  font-size: 0.875rem;
}

/* icon-only navigation (review prev/next) */
.act.icon {
  flex: none;
  min-width: 3.25rem;
  padding: 0 0.5rem;
}

.act.side.mid {
  min-width: 9.375rem;
}

.act.primary {
  color: var(--brand);
}

.act.side.next.finish {
  color: var(--brand);
  font-weight: 600;
}

.act.good {
  color: var(--good);
}

.act.bad {
  color: var(--bad);
}

.act:hover:not(:disabled) {
  background: var(--hover);
}

.act:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

/* five buttons only coexist while revisiting: shrink the verdict labels'
   kbd hints out of the way first, then the hide-answer button */
@container (max-width: 62.4375rem) {
  .act .kbd {
    display: none;
  }

  .act.side.mid {
    min-width: 0;
  }
}
</style>
