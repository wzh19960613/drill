<script setup lang="ts">
import { ref } from 'vue'
import { ChevronDown, ChevronRight, Ellipsis, FileText } from 'lucide-vue-next'
import type { BrowseInfo } from '../../api'
import CollapseBox from './CollapseBox.vue'

const props = withDefaults(
  defineProps<{
    source: string
    dir: string
    browse: BrowseInfo | undefined
    failed?: boolean
    /** nesting depth of this folder's content (source root = 0); each level
     *  indents one 2rem step, files one step below their group header */
    depth?: number
  }>(),
  { failed: false, depth: 0 },
)

const emit = defineEmits<{
  (e: 'menu', evt: MouseEvent, ctx: { kind: 'q' | 'other'; file: string; id?: string }): void
  (e: 'retry'): void
}>()

/** the question group starts expanded; the other-md group starts folded */
const qOpen = ref(true)
const otherOpen = ref(false)
</script>

<template>
  <div class="stf" :style="{ '--lvl': depth }">
    <template v-if="browse?.questions.length">
      <button type="button" class="tree-l1" @click="qOpen = !qOpen">
        <component :is="qOpen ? ChevronDown : ChevronRight" class="chev" />
        <FileText class="ticon" />
        <span class="tname">题目（{{ browse.questions.length }}）</span>
      </button>
      <CollapseBox :open="qOpen">
        <div v-for="q in browse.questions" :key="q.file" class="tree-l2">
          <span class="tf-name">{{ q.file }}</span>
          <button
            class="dots"
            aria-label="题目操作"
            @click.stop="emit('menu', $event, { kind: 'q', file: q.file, id: q.id })"
          >
            <Ellipsis style="width: 1.0625rem; height: 1.0625rem" />
          </button>
        </div>
      </CollapseBox>
    </template>

    <template v-if="browse?.other_md.length">
      <button type="button" class="tree-l1" @click="otherOpen = !otherOpen">
        <component :is="otherOpen ? ChevronDown : ChevronRight" class="chev" />
        <FileText class="ticon" />
        <span class="tname">其它 Markdown 文件（{{ browse.other_md.length }}）</span>
      </button>
      <CollapseBox :open="otherOpen">
        <div
          v-for="o in browse.other_md"
          :key="o.file"
          class="tree-l2 dim"
          :title="`未识别为题目：${o.reason}`"
        >
          <span class="tf-name">{{ o.file }}</span>
          <span class="why-wrap">
            <span class="tf-why">{{ o.reason }}</span>
          </span>
          <button
            class="dots"
            aria-label="文件操作"
            @click.stop="emit('menu', $event, { kind: 'other', file: o.file })"
          >
            <Ellipsis style="width: 1.0625rem; height: 1.0625rem" />
          </button>
        </div>
      </CollapseBox>
    </template>

    <div v-if="failed" class="tree-l2 dim fail">
      <span class="tf-why">读取失败</span>
      <button type="button" class="tree-retry" @click.stop="emit('retry')">重试</button>
    </div>
  </div>
</template>

<style scoped>
.stf {
  display: flex;
  flex-direction: column;
}

.tree-l1 {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  min-height: 2.25rem;
  padding: 0;
  padding-inline-start: calc(var(--lvl, 0) * 2rem);
  border: none;
  background: transparent;
  color: var(--muted);
  font: inherit;
  font-size: 0.8125rem;
  font-weight: 600;
  text-align: left;
  cursor: pointer;
  border-radius: 0.375rem;
}

.tree-l1:hover {
  background: var(--hover);
  color: var(--ink);
}

/* chevron column (1.75rem) + icon column (1rem): one shared text edge per level */
.tree-l1 .chev {
  flex: none;
  width: 1.75rem;
  height: 1.75rem;
  padding: 0.375rem;
  box-sizing: border-box;
  color: var(--muted);
}

.tree-l1 .ticon {
  flex: none;
  width: 1rem;
  height: 1rem;
  color: var(--muted);
}

.tree-l1 .tname {
  min-width: 0;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

/* file rows: one 2rem step below their group header's level */
.tree-l2 {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-height: 2.375rem;
  padding-inline-start: calc((var(--lvl, 0) + 1) * 2rem);
  padding-inline-end: 0.25rem;
  font-size: 0.875rem;
  border-radius: 0.375rem;
}

.tree-l2:hover {
  background: var(--hover);
}

.tree-l2.dim {
  color: var(--muted);
}

.tree-l2 .dots {
  margin-left: auto;
}

.dots {
  flex: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2.25rem;
  height: 2.25rem;
  padding: 0;
  border: none;
  background: none;
  color: var(--muted);
  border-radius: 0.5rem;
  cursor: pointer;
}

.dots:hover {
  background: var(--hover);
  color: var(--ink);
}

.tf-name {
  font-family: var(--font-mono);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.why-wrap {
  /* the reason gives way first (down to a 5rem floor); only then does the
     file name start shrinking */
  flex: 1 1 5rem;
  min-width: 5rem;
  display: flex;
  justify-content: flex-start;
}

.tf-why {
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 0.7188rem;
  color: var(--muted);
  background: var(--panel);
  border-radius: var(--radius-pill);
  padding: 0.125rem 0.5rem;
}

.tree-l2.fail .tf-why {
  margin-left: auto;
  background: none;
  padding: 0;
}

.tree-retry {
  border: none;
  background: none;
  color: var(--brand);
  font-size: inherit;
  padding: 0 0.25rem;
  cursor: pointer;
  text-decoration: underline;
}
</style>
