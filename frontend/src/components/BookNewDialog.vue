<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import DialogHeader from './DialogHeader.vue'
import QuestionDetail from './QuestionDetail.vue'
import { loadAll, selection, store, subjectQuestions } from '../store'
import { defaultFilter, type Filter } from '../filter'
import type { Question } from '../types'
import { invertSelection } from '../qutil'
import {
  askUseAsCurrent,
  buildSession,
  currentSession,
  defFromSession,
  fetchBooks,
  upsertBook,
} from '../book'
import FilterBar from './FilterBar.vue'
import QuestionList from './QuestionList.vue'
import ExportDialog from './ExportDialog.vue'
import BookNewSortBar from './booknew/BookNewSortBar.vue'
import BookNewConfigPanel from './booknew/BookNewConfigPanel.vue'
import BookNewFootBar from './booknew/BookNewFootBar.vue'
import { useDialogShell } from '../composables/useDialogShell'
import { useBookNewDraft } from '../composables/useBookNewDraft'
import { randomToken } from '../rng'
import { isoToday } from '../format'

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'blank-created', book: { id: string; name: string }): void
}>()

const filter = ref<Filter>(defaultFilter())
const checkedIds = ref<string[]>([])
const bookName = ref('')
const exportOpen = ref(false)
const viewQ = ref<Question | null>(null)

const { sort, shuffleO, seed, filteredQuestions, loadCfg, persistCfg, dice } = useBookNewDraft(filter)

useDialogShell((e) => {
  if (exportOpen.value || viewQ.value) return
  const t = e.target as HTMLElement | null
  if (t && ['INPUT', 'SELECT', 'TEXTAREA'].includes(t.tagName)) return
  if (e.code === 'Escape') {
    e.stopPropagation()
    emit('close')
  }
})

onMounted(async () => {
  loadCfg()
  await loadAll()
  bookName.value = await defaultBookName()
  checkedIds.value = selection.ids.length ? [...selection.ids] : subjectQuestions.value.map((q) => q.id)
})

async function defaultBookName(): Promise<string> {
  let existing: string[] = []
  try {
    existing = (await fetchBooks()).map((b) => b.name)
  } catch {
  }
  let n = existing.length + 1
  while (existing.includes(`题本 ${n}`)) n++
  return `题本 ${n}`
}

const selectedCount = computed(() => checkedIds.value.length)
const filteredCount = computed(() => filteredQuestions.value.length)

function selectAllFiltered() {
  checkedIds.value = filteredQuestions.value.map((q) => q.id)
}

function invert() {
  checkedIds.value = invertSelection(filteredQuestions.value, checkedIds.value)
}

function toggleSelect(q: Question, on: boolean) {
  const s = new Set(checkedIds.value)
  if (on) s.add(q.id)
  else s.delete(q.id)
  checkedIds.value = [...s]
}

function makeSession() {
  const ids = new Set(checkedIds.value)
  const qs = store.questions.filter((q) => ids.has(q.id))
  persistCfg()
  return buildSession(qs, {
    shuffleQ: sort.value === 'random',
    shuffleO: shuffleO.value,
    seed: seed.value,
    title: bookName.value.trim() || '题本',
  })
}

async function startStudy() {
  if (!selectedCount.value) {
    const name = bookName.value.trim() || '题本'
    const id = `bk-${randomToken()}`
    await upsertBook({
      id,
      name,
      seed: '',
      date: isoToday(),
      createdAt: Date.now(),
      items: [],
    })
    emit('blank-created', { id, name })
    emit('close')
    return
  }
  const session = makeSession()
  const def = defFromSession(session)
  await upsertBook(def)
  currentSession.value = { ...session, bookId: def.id }
  selection.ids = []
  askUseAsCurrent.value = def
  emit('close')
}

function openExport() {
  if (!selectedCount.value) return
  currentSession.value = makeSession()
  exportOpen.value = true
}

</script>

<template>
  <Teleport to="body">
  <div class="modal-mask fs" @click.self="emit('close')">
    <div class="bnd dialog fs">
      <DialogHeader flush title="新建题本" @close="emit('close')" />

      <div class="bnd-body">
        <div class="bn">
          <section class="card bn-filter">
            <FilterBar v-model="filter" />
          </section>
          <BookNewSortBar v-model:sort="sort" v-model:seed="seed" @dice="dice" />

          <QuestionList
            v-model="checkedIds"
            :questions="filteredQuestions"
            :flat="sort === 'random'"
            @open="viewQ = $event"
          />

          <BookNewConfigPanel
            v-model:book-name="bookName"
            v-model:shuffle-o="shuffleO"
            :selected-count="selectedCount"
            @start="startStudy"
            @export="openExport"
          />
        </div>

        <BookNewFootBar
          :selected="selectedCount"
          :total="filteredCount"
          @select-all="selectAllFiltered"
          @invert="invert"
          @start="startStudy"
        />
      </div>

      <ExportDialog v-if="exportOpen && currentSession" :session="currentSession" @close="exportOpen = false" />

      <QuestionDetail
        v-if="viewQ"
        :key="viewQ.id"
        :q="viewQ"
        :siblings="filteredQuestions"
        select-mode
        :selected="checkedIds.includes(viewQ.id)"
        @toggle-select="toggleSelect"
        @navigate="viewQ = $event"
        @close="viewQ = null"
      />
    </div>
  </div>
  </Teleport>
</template>

<style scoped>
.bnd {
  background: var(--bg);
  width: min(73.75rem, 96vw);
  max-height: 90vh;
}

.bnd-body {
  flex: 1;
  min-height: 0;
  padding: 1rem;
  overflow-y: auto;
  overflow-x: clip;
  overscroll-behavior: contain;
}

.bn {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 22.5rem;
  grid-template-rows: auto auto auto;
  gap: 1rem;
  align-items: start;
}

/* Two-column wide layout (list never overflows): .ql drops its overflow and is
   no longer a scroll container — vertical sticky returns to body scrolling,
   chapter headers stick to the body top (same as the bank page wide screens) */
@container (min-width: 62.5rem) {
  .bn > .ql:not(.x-overflow) {
    overflow: visible;
  }
}

.bn-filter {
  grid-column: 1;
  grid-row: 1;
}

.bn > .ql {
  grid-column: 1;
  grid-row: 3;
  /* Narrow screens (fullscreen dialog) keep the base styles: the list is its own
     horizontal scroll container (the only place allowed to scroll sideways);
     the cost is the chapter header giving up sticky; the body clips horizontally,
     an over-wide list never drags the whole dialog into horizontal scrolling */
}

.bn > .card {
  min-width: 0;
}

@container (max-width: 61.25rem) {
  .bn {
    grid-template-columns: 1fr;
  }
}

@container (max-width: 40rem) {
  .bnd-body {
    padding: 0;
  }

  .bn {
    gap: 0.625rem;
  }

  .bn-filter {
    border-radius: 0;
    border-left: none;
    border-right: none;
  }
}
</style>
