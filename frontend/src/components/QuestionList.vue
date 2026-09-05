<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import type { Question } from '../types'
import { useQuestionGroups } from '../composables/useQuestionGroups'
import QuestionGroupPanel from './list/QuestionGroupPanel.vue'

const props = withDefaults(
  defineProps<{
    questions: Question[]
    mode?: 'select' | 'manage' | 'view'
    flat?: boolean
    modelValue?: string[]
  }>(),
  { mode: 'select', flat: false, modelValue: () => [] },
)

const emit = defineEmits<{
  (e: 'update:modelValue', ids: string[]): void
  (e: 'open', q: Question): void
}>()

const { collapsed, subjectGroups, seqOf, toggleCollapse } = useQuestionGroups({
  questions: computed(() => props.questions),
  flat: computed(() => props.mode === 'view' || props.flat),
})

const qlEl = ref<HTMLElement | null>(null)
const xOverflow = ref(false)

function measureOverflow() {
  const el = qlEl.value
  xOverflow.value = !!el && el.scrollWidth > el.clientWidth + 1
}

onMounted(() => {
  measureOverflow()
  window.addEventListener('resize', measureOverflow)
})
onBeforeUnmount(() => window.removeEventListener('resize', measureOverflow))
watch(
  () => props.questions,
  () => nextTick(measureOverflow),
)

const checkedSet = computed(() => new Set(props.modelValue))

function toggleOne(q: Question, on: boolean) {
  const s = new Set(checkedSet.value)
  if (on) s.add(q.id)
  else s.delete(q.id)
  emit('update:modelValue', [...s])
}

function toggleGroup(group: { questions: Question[] }, on: boolean) {
  const s = new Set(checkedSet.value)
  for (const q of group.questions) {
    if (on) s.add(q.id)
    else s.delete(q.id)
  }
  emit('update:modelValue', [...s])
}
</script>

<template>
  <div ref="qlEl" class="ql" :class="{ 'x-overflow': xOverflow }">
    <section v-for="sg in subjectGroups" :key="sg.subject || '__subject'" class="ql-subject">
      <header v-if="sg.showHead" class="ql-subhead">
        <span class="subname">{{ sg.subject }}</span>
        <span class="chip">{{ sg.count }} 题</span>
      </header>
      <div class="ql-panel">
        <QuestionGroupPanel
          v-for="g in sg.groups"
          :key="g.chapter || '__flat'"
          :group="g"
          :mode="mode"
          :checked-ids="modelValue"
          :collapsed="!!g.chapter && collapsed.has(g.chapter)"
          :overflow="xOverflow"
          :seq-of="seqOf"
          @toggle-collapse="toggleCollapse(g.chapter)"
          @toggle-group="toggleGroup(g, $event)"
          @toggle-one="toggleOne"
          @open="emit('open', $event)"
        />
      </div>
    </section>
    <div v-if="!subjectGroups.length" class="ql-empty">没有符合条件的题目</div>
  </div>
</template>

<style scoped>
/* Wide content scrolls horizontally inside the list, never stretching the page
   into a horizontal scrollbar */
.ql {
  overflow-x: auto;
  display: grid;
  grid-template-columns: minmax(min-content, 100%);
  align-content: start;
  gap: 0.875rem;
}

/* Subject panel: background/border/radius carried by the panel alone; a single
   subject makes it the whole list, "all subjects" gives one panel per subject.
   Clipping keeps row-level backgrounds from smearing the corners; panel width =
   track width, so pinned-left elements stay within the panel during horizontal
   scroll and are not clipped */
.ql-panel {
  border: 1px solid var(--line);
  border-radius:0.75rem;
  background: var(--card);
  /* Bottom padding: the last row's pinned-left elements' negative bottom margin
     lands inside the panel instead of leaking (must not go on .ql-rows — it
     would stretch the collapse animation's 0fr track) */
  padding-bottom: 0.5rem;
}

@container (max-width: 40rem) {
  .ql-panel {
    border: none;
    border-radius: 0;
  }
}

.ql-empty {
  padding: 2.25rem 0;
  text-align: center;
  color: var(--muted);
}

.ql-subject {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.ql-subhead {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding:0 4px;
  font-size: 0.8438rem;
  font-weight: 600;
  color: var(--brand);
  /* During horizontal list scrolling the subject head pins to the left edge
     (naturally inactive on wide screens where .ql is not a scroll container);
     fit-content shrinks the head to its content — a block filling its containing
     block leaves sticky no room to move */
  position: sticky;
  left: 0;
  width: fit-content;
}

@container (max-width: 40rem) {
  .ql-subhead {
    padding-left: 1rem;
  }
}
</style>
