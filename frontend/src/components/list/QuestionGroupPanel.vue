<script setup lang="ts">
import { computed } from 'vue'
import { ChevronDown } from 'lucide-vue-next'
import type { Question } from '../../types'
import type { QuestionGroup } from '../../composables/useQuestionGroups'
import QuestionRow from './QuestionRow.vue'

const props = defineProps<{
  group: QuestionGroup
  mode: 'select' | 'manage' | 'view'
  checkedIds: string[]
  collapsed: boolean
  overflow: boolean
  seqOf: Map<string, number>
}>()

const emit = defineEmits<{
  (e: 'toggleCollapse'): void
  (e: 'toggleGroup', on: boolean): void
  (e: 'toggleOne', q: Question, on: boolean): void
  (e: 'open', q: Question): void
}>()

const checkedSet = computed(() => new Set(props.checkedIds))

const groupState = computed(() => {
  const n = props.group.questions.filter((q) => checkedSet.value.has(q.id)).length
  if (n === 0) return false
  if (n === props.group.questions.length) return true
  return 'indeterminate'
})
</script>

<template>
  <div class="ql-group" :class="{ 'x-overflow': overflow }">
    <div v-if="group.chapter" class="ql-ghead" @click="emit('toggleCollapse')">
      <label v-if="mode !== 'view'" class="ql-gcheck" @click.stop>
        <input
          type="checkbox"
          :checked="groupState === true"
          :indeterminate="groupState === 'indeterminate'"
          @change="emit('toggleGroup', ($event.target as HTMLInputElement).checked)"
        />
        <ChevronDown style="width: 1rem; height: 1rem" class="chev" :class="{ closed: collapsed }" @click.stop />
      </label>
      <span v-else class="ql-gcheck">
        <ChevronDown style="width: 1rem; height: 1rem" class="chev" :class="{ closed: collapsed }" />
      </span>
      <span class="ql-gtitle">
        <span class="gname">{{ group.chapter }}</span>
      </span>
      <span class="chip ql-gcount">{{ group.questions.length }} 题</span>
    </div>

    <div class="ql-collapse" :class="{ closed: !!group.chapter && collapsed }">
      <div class="ql-rows">
        <QuestionRow
          v-for="q in group.questions"
          :key="q.id"
          :q="q"
          :mode="mode"
          :checked="checkedSet.has(q.id)"
          :seq="seqOf.get(q.id)"
          :overflow="overflow"
          @toggle="(on) => emit('toggleOne', q, on)"
          @open="emit('open', q)"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.ql-group {
  /* Same width as .ql-rows (shrink floor): the sticky chapter header must not
     exceed its parent, otherwise the header background is cut off halfway
     during horizontal scrolling */
  min-width: min-content;
}

.ql-group + .ql-group {
  border-top:1px solid var(--line);
}

/* No overflow clipping on the panel (hidden/clip would capture or clip the
   header's page-level sticky); the rounding comes from the chapter header
   touching the panel's top edge */
.ql-group:first-child .ql-ghead {
  border-top-left-radius: 0.75rem;
  border-top-right-radius: 0.75rem;
}

@container (max-width: 40rem) {
  .ql-group:first-child .ql-ghead {
    border-top-left-radius: 0;
    border-top-right-radius: 0;
  }
}

/* Chapter header checkbox: pinned left during horizontal scroll (same covering
   strip technique as .ql-check); nests with the outer chapter header's vertical
   sticky — each axis manages itself, supported by browsers */
.ql-gcheck {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  flex: none;
  position: sticky;
  left: 0;
  margin: -0.375rem -0.375rem -0.375rem -0.875rem;
  padding: 0.375rem 0.375rem 0.375rem 0.875rem;
  z-index: 1;
}

.x-overflow .ql-gcheck {
  background: var(--glass-bg);
  -webkit-backdrop-filter: blur(0.4375rem);
  backdrop-filter: blur(0.4375rem);
}

/* Chapter header sticks to the top; also pinned left during horizontal scroll.
   Background matches the panel; the vertical sticky offset is controlled by the
   host via --ghead-top (page scenario sticks below the top bar) */
.ql-ghead {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.5rem 0.875rem;
  min-height: 2.8125rem;
  box-sizing: border-box;
  background: var(--card);
  position: sticky;
  top: var(--ghead-top, 0);
  left: 0;
  z-index: 2;
  cursor: pointer;
  user-select: none;
}

.ql-ghead:hover .gname {
  color: var(--brand);
}

.ql-gcount {
  margin-left: auto;
}

.ql-gtitle {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex: 1;
  min-width: 0;
  color: var(--ink);
  font-size: 0.875rem;
  font-weight: 600;
}

.chev {
  flex: none;
  color: var(--muted);
}

.ql-collapse {
  display: grid;
  grid-template-rows: 1fr;
  transition: grid-template-rows 0.24s ease;
}

.ql-collapse > .ql-rows {
  min-width: min-content;
  min-height: 0;
  overflow: hidden;
  transform-origin: top center;
  transition: transform 0.24s ease, opacity 0.24s ease;
}

/* Expanded state releases clipping: overflow: hidden is also a scroll container
   and would trap the in-row sticky (checkbox/number) and break it; the collapse
   animation relies on the closed state's hidden to clip the content */
.ql-collapse:not(.closed) > .ql-rows {
  overflow: visible;
}

.ql-collapse.closed {
  grid-template-rows: 0fr;
}

.ql-collapse.closed > .ql-rows {
  transform: scaleY(0.72);
  opacity: 0;
}
</style>
