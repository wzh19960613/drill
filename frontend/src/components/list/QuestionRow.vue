<script setup lang="ts">
import { computed } from 'vue'
import { Clock } from 'lucide-vue-next'
import type { Question } from '../../types'
import { acc5Percent, isMastered, recentAcc5, statsOf } from '../../store'
import { fmtTime } from '../../format'
import { stemExcerptHtml } from '../../qutil'

const props = defineProps<{
  q: Question
  mode: 'select' | 'manage' | 'view'
  checked: boolean
  seq?: number
  overflow: boolean
}>()

const emit = defineEmits<{
  (e: 'toggle', on: boolean): void
  (e: 'open', q: Question): void
}>()

const recent5 = computed(() => recentAcc5(props.q))

const acc5 = computed(() => acc5Percent(props.q))

const sigBars = computed(() => {
  const a = acc5.value
  return a <= 0 ? 0 : Math.min(4, Math.ceil(a / 25))
})
</script>

<template>
  <div
    class="ql-row"
    :class="{ checked, clickable: true, 'x-overflow': overflow }"
    @click="emit('open', q)"
  >
    <label v-if="mode !== 'view'" class="ql-check" @click.stop>
      <input
        type="checkbox"
        :checked="checked"
        @change="emit('toggle', ($event.target as HTMLInputElement).checked)"
      />
    </label>
    <span v-if="mode === 'view'" class="qseq num">{{ seq }}.</span>
    <span class="qid" :title="q.id">{{ q.id }}</span>
    <span v-if="isMastered(q)" class="chip good">已熟练</span>
    <span class="excerpt" v-html="stemExcerptHtml(q)"></span>
    <span class="row-stats">
      <span class="chip brand">{{ q.qtype }}</span>
      <span class="stat mark" :title="recent5.total ? `近五次正确率 ${acc5}%` : '没做过'">
        <span v-if="recent5.total" class="sig">
          <i v-for="b in 4" :key="b" :class="{ on: b <= sigBars }"></i>
        </span>
        <Clock v-else style="width: 0.9375rem; height: 0.9375rem" class="clock" />
      </span>
      <span class="stat num time">
        {{ recent5.total ? fmtTime(statsOf(q).last_at) : '尚未做过' }}
      </span>
    </span>
  </div>
</template>

<style scoped>
.ql-row {
  display: flex;
  align-items: center;
  gap: 0.875rem;
  padding: 0.5rem 13rem 0.5rem 0.875rem;
  border-top:1px dashed var(--line);
  font-size: 0.875rem;
  position: relative;
}

.row-stats {
  position: absolute;
  right: 0.875rem;
  top: 50%;
  transform: translateY(-50%);
  display: flex;
  align-items: center;
  gap: 0.875rem;
}

.ql-row.clickable {
  cursor: pointer;
}

.ql-row:hover {
  background: var(--hover);
}

.ql-row.checked {
  background: var(--brand-weak);
}

.ql-check {
  display: flex;
  align-items: center;
  flex: none;
  /* Pinned left during horizontal scroll. The covering strip starts at x=0
     (negative margin cancels the row padding, then padding restores it),
     otherwise the sticky offset leaks sliding content on the far left;
     align-self: stretch fills the row height against top/bottom leaks */
  position: sticky;
  left: 0;
  align-self: stretch;
  margin: -0.5rem -0.875rem -0.5rem -0.875rem;
  padding: 0.5rem 0.875rem 0.5rem 0.875rem;
  z-index: 1;
}

.x-overflow .ql-check {
  background: var(--glass-bg);
  -webkit-backdrop-filter: blur(0.4375rem);
  backdrop-filter: blur(0.4375rem);
}

.x-overflow.checked .ql-check {
  background: color-mix(in srgb, var(--brand-weak) 66%, transparent);
}

.qid {
  font-family: var(--font-mono);
  font-weight: 700;
  /* definite cap only: a percentage max-width is ignored by intrinsic sizing,
     so a long locate/id would stretch the row's min-content past every
     container and the list could never shrink (overflowing the page) */
  max-width: 12rem;
  /* 定位回退成文件名主干时可能很长：允许收缩并截断（悬停可见全名），
     否则窄屏下它会把题干预览挤没、行的渐隐截断也跟着错位 */
  flex: 0 1 auto;
  min-width: 0;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.qseq {
  flex: none;
  min-width: 3ch;
  text-align: right;
  font-size: 0.8125rem;
  color: var(--muted);
  position: sticky;
  left: 0;
  align-self: stretch;
  margin: -0.5rem -0.875rem -0.5rem -0.875rem;
  padding: 0.5rem 0.875rem 0.5rem 0.875rem;
  z-index: 1;
  display: inline-flex;
  align-items: center;
}

.x-overflow .qseq {
  background: var(--glass-bg);
  -webkit-backdrop-filter: blur(0.4375rem);
  backdrop-filter: blur(0.4375rem);
}

.x-overflow.checked .qseq {
  background: color-mix(in srgb, var(--brand-weak) 66%, transparent);
}

.excerpt {
  flex: 1;
  min-width: 6.3rem;
  overflow: hidden;
  white-space: nowrap;
  color: var(--ink);
  line-height: 1.9;
  /* The excerpt's own content must not join the row's min-width calculation:
     otherwise a long nowrap formula stretches min-content to the full row
     content width and the row never shrinks; truncation is done by the
     min-width floor + flex shrinking */
  contain: inline-size;
  /* No text-overflow: ellipsis: Safari 26 pops a native tooltip with the full
     text for truncated content; use a right-edge fade mask to mark truncation */
  -webkit-mask-image: linear-gradient(90deg, #000 calc(100% - 2.5em), transparent);
  mask-image: linear-gradient(90deg, #000 calc(100% - 2.5em), transparent);
}

.excerpt :deep(.katex) {
  font-size: 0.95em;
}

.chip.brand {
  flex: none;
}

.stat {
  flex: none;
  color: var(--muted);
  font-size: 0.8438rem;
  white-space: nowrap;
}

.stat.mark {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
}

.stat.mark .clock {
  color: var(--muted);
  opacity: 0.65;
}

.sig {
  display: inline-flex;
  align-items: flex-end;
  gap: 0.0938rem;
}

.sig i {
  display: inline-block;
  width: 0.1875rem;
  border-radius:1px;
  background: var(--line);
}

.sig i:nth-child(1) { height:0.25rem; }
.sig i:nth-child(2) { height:0.4375rem; }
.sig i:nth-child(3) { height:0.625rem; }
.sig i:nth-child(4) { height:0.8125rem; }

.sig i.on {
  background: var(--good);
}

.stat.mark.low .sig i.on {
  background: var(--bad);
}

/* Stats text (time / never done): fixed-width slot, right aligned. Fixed width is
   a hard requirement — with a variable width, when the row is compressed
   (horizontal scroll area), font metric rounding pushes a few rows' content past
   the row width and the right edge drifts; slot width fits the longest format
   08-28 14:30 (11ch; cross-year now shows "X years ago" and no longer overflows) */
.stat.time {
  display: inline-flex;
  justify-content: flex-end;
  font-size: 0.7812rem;
  width: 11.5ch;
  white-space: nowrap;
}
</style>
