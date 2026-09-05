<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowUpDown, FolderOpen, Plus } from 'lucide-vue-next'
import { loadAll, selection, subjectQuestions } from '../store'
import { applyFilter, defaultFilter, SORT_OPTIONS, type Filter, type SortKey } from '../filter'
import { invertSelection } from '../qutil'
import { fetchSources, type SourceInfo } from '../api'
import FilterBar from '../components/FilterBar.vue'
import QuestionList from '../components/QuestionList.vue'
import QuestionDetail from '../components/QuestionDetail.vue'
import QuestionEditor from '../components/QuestionEditor.vue'
import SourcesDialog from '../components/list/SourcesDialog.vue'
import DeleteConfirmDialog from '../components/list/DeleteConfirmDialog.vue'
import ListFootBar from '../components/list/ListFootBar.vue'
import type { Question } from '../types'

const router = useRouter()
const filter = ref<Filter>(defaultFilter())
const detailQ = ref<Question | null>(null)
const confirmDelQ = ref(false)

const list = computed(() => applyFilter(subjectQuestions.value, filter.value))

/** The bank sort selector hides the random entry (it belongs to book building) */
const SORTS = SORT_OPTIONS.filter((o) => o.value !== 'random')

function pickSort(e: Event) {
  filter.value = { ...filter.value, sort: (e.target as HTMLSelectElement).value as SortKey }
}

const sourcesOpen = ref(false)
const sources = ref<SourceInfo[]>([])

async function refreshSources() {
  try {
    sources.value = await fetchSources()
  } catch {
  }
}

async function onSourcesChanged() {
  await refreshSources()
  await loadAll(true)
}

const editing = ref<Question | null | 'new'>(null)

function openNew() {
  editing.value = 'new'
}

function openEdit(q: Question) {
  editing.value = q
  detailQ.value = null
}

async function onDeleted(q: Question) {
  selection.ids = selection.ids.filter((id) => id !== q.id)
  if (detailQ.value?.id === q.id) detailQ.value = null
  await loadAll(true)
  await refreshSources()
}

async function onSaved() {
  await loadAll(true)
  await refreshSources()
  editing.value = null
}

async function onConfirmDeleted() {
  confirmDelQ.value = false
  await loadAll(true)
  await refreshSources()
}

onMounted(() => {
  loadAll()
  refreshSources()
  window.addEventListener('keydown', onKey)
})
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKey)
})

function onKey(e: KeyboardEvent) {
  if (editing.value) return
  if (detailQ.value) return // the dialog handles Esc itself in the capture phase
  if (e.code === 'Escape' && confirmDelQ.value) {
    confirmDelQ.value = false
    return
  }
  if (e.code === 'Escape' && sourcesOpen.value) {
    sourcesOpen.value = false
    return
  }
  if (e.code === 'Escape') router.push('/')
}

function selectAllFiltered() {
  selection.ids = list.value.map((q) => q.id)
}

function invert() {
  selection.ids = invertSelection(list.value, selection.ids)
}
</script>

<template>
  <div class="list">
    <div class="card tools-card">
      <FilterBar v-model="filter" />
    </div>

    <div class="between-bar">
      <label class="sort-wrap">
        <ArrowUpDown style="width: 0.875rem; height: 0.875rem" />
        <select class="sort-sel" :value="filter.sort" @change="pickSort">
          <option v-for="o in SORTS" :key="o.value" :value="o.value">{{ o.label }}</option>
        </select>
      </label>

      <div class="src-wrap">
        <button class="btn ghost sm" @click="openNew">
          <Plus style="width: 0.9375rem; height: 0.9375rem" /> 新增
        </button>
        <button class="btn ghost sm" @click="sourcesOpen = true">
          <FolderOpen style="width: 0.9375rem; height: 0.9375rem" /> 题源
        </button>
      </div>
    </div>

    <QuestionList v-model="selection.ids" mode="manage" :questions="list" @open="detailQ = $event" />

    <SourcesDialog v-if="sourcesOpen" @close="sourcesOpen = false" @changed="onSourcesChanged" />

    <QuestionDetail
      v-if="detailQ"
      :key="detailQ.id"
      :q="detailQ"
      :siblings="list"
      editable
      @close="detailQ = null"
      @edit="openEdit"
      @deleted="onDeleted"
      @navigate="detailQ = $event"
    />

    <QuestionEditor
      v-if="editing"
      :question="editing === 'new' ? null : editing"
      :sources="sources"
      @close="editing = null"
      @saved="onSaved"
    />

    <ListFootBar
      :selected="selection.ids.length"
      :total="list.length"
      @select-all="selectAllFiltered"
      @invert="invert"
      @request-delete="confirmDelQ = true"
    />

    <DeleteConfirmDialog v-if="confirmDelQ" @cancel="confirmDelQ = false" @done="onConfirmDeleted" />
  </div>
</template>

<style scoped>
.list {
  display: flex;
  flex-direction: column;
  gap: 0.875rem;
}

/* Wide screens (content never overflows, no horizontal scrolling possible): remove
   .ql's overflow so it is no longer a scroll container — vertical sticky returns to
   page scrolling and chapter headers stick to the viewport top (works in
   Chrome/Safari); narrow screens keep the horizontal scroll container
   (sticky is captured there and gives up sticking), page vertical scroll as usual */
@container (min-width: 40.0625rem) {
  .list :deep(.ql) {
    overflow: visible;
    --ghead-top:calc(env(safe-area-inset-top) + 0.125rem);
  }
}

.tools-card {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.between-bar {
  display: flex;
  align-items: flex-start;
  gap: 0.625rem;
  padding: 0 0.375rem;
}

.sort-wrap {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  color: var(--muted);
}

.sort-sel {
  min-width: 9.375rem;
  font-size: 0.875rem;
  padding: 0 0.625rem;
}

.src-wrap {
  margin-left: auto;
  display: flex;
  gap: 0.5rem;
}

@container (max-width: 40rem) {
  .between-bar {
    padding-left: 0.875rem;
  }

  .tools-card,
  .list :deep(.ql) {
    border-radius: 0;
    border-left: none;
    border-right: none;
  }
}
</style>
