<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { currentSubject, loadAll, store } from '../store'
import { lockBodyScroll, unlockBodyScroll } from '../scrollLock'
import {
  askUseAsCurrent,
  currentSession,
  sessionFromDef,
  setActiveBookId,
  type BookSession,
} from '../book'
import type { BookDef, Question } from '../types'
import { getPaused } from '../api'
import { useBookLibrary } from '../composables/useBookLibrary'
import { useFavorites } from '../composables/useFavorites'
import { useBookStartFlow } from '../composables/useBookStartFlow'
import ExportDialog from '../components/ExportDialog.vue'
import QuestionDetail from '../components/QuestionDetail.vue'
import BookNewDialog from '../components/BookNewDialog.vue'
import ActiveBookCard from '../components/book/ActiveBookCard.vue'
import FavoriteBookCard from '../components/book/FavoriteBookCard.vue'
import BookListCard from '../components/book/BookListCard.vue'
import BookViewDialog from '../components/book/BookViewDialog.vue'
import BookHomeDialogs from '../components/book/BookHomeDialogs.vue'
import NewBookPill from '../components/book/NewBookPill.vue'
import SourcesDialog from '../components/list/SourcesDialog.vue'

const router = useRouter()

const viewing = ref<BookDef | null>(null)
const favAsk = ref<{ id: string; name: string } | null>(null)
const detailQ = ref<Question | null>(null)
const newOpen = ref(false)
const exportSession = ref<BookSession | null>(null)
const flash = ref('')

const asking = ref<BookDef | null>(null)
const confirmDel = ref<BookDef | null>(null)
const sourcesOpen = ref(false)
const favs = useFavorites()
const lib = useBookLibrary({ viewing, flash: (msg: string) => (flash.value = msg) })


async function onSourcesChanged() {
  await loadAll(true)
  await lib.refresh({ quiet: true })
}

function cleanStaleViewing() {
  const def = viewing.value
  if (def) void lib.removeStale(def)
}

const flow = useBookStartFlow({
  viewing,
  flash: (msg) => (flash.value = msg),
  activeId: lib.activeId,
  paused: lib.paused,
  closeViewing: () => (viewing.value = null),
})
const { startFromViewing, studyFrom } = flow

const activeName = computed(() => lib.activeBook.value?.name ?? '未设置')
const resumeActive = lib.resumeActive

const viewSiblings = computed(() =>
  viewing.value ? sessionFromDef(viewing.value, store.questions).items : [],
)

const favoriteDef = computed(
  () => lib.books.value.find((b) => b.id === favs.favoriteBookId.value) ?? null,
)

let flashTimer: number | undefined
watch(flash, (v) => {
  window.clearTimeout(flashTimer)
  if (v) flashTimer = window.setTimeout(() => (flash.value = ''), 2800)
})

watch(
  () => favs.favoritedIds.value.size,
  () => {
    void lib.refresh({ quiet: true })
  },
)

const NO_BOOKS_HINT = '还没有题本。点「新建题本」创建，或直接开始刷题。'
const mainEmptyHint = computed(() =>
  currentSubject.value ? `目前没有「${currentSubject.value}」的题本` : NO_BOOKS_HINT,
)

async function acceptCurrent(start: boolean) {
  const def = asking.value
  asking.value = null
  askUseAsCurrent.value = null
  if (!def) return
  await setActiveBookId(currentSubject.value, def.id)
  lib.activeId.value = def.id
  flash.value = `已将「${def.name}」设为当前题本`
  if (start) {
    flow.launch(def)
  } else {
    await lib.refresh()
  }
}

function declineCurrent() {
  asking.value = null
  askUseAsCurrent.value = null
}

onMounted(async () => {
  await loadAll()
  await lib.refresh()
  favs.ensureLoaded()
  getPaused().then((p) => (lib.paused.value = p)).catch(() => {})
  if (askUseAsCurrent.value) asking.value = askUseAsCurrent.value
  window.addEventListener('keydown', onKey)
})
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKey)
  if (anyModalOpen.value) unlockBodyScroll()
})
watch(askUseAsCurrent, async (v) => {
  if (v) {
    asking.value = v
    newOpen.value = false
    await lib.refresh()
  }
})

function onKey(e: KeyboardEvent) {
  if (exportSession.value) return
  if (newOpen.value) return 
  if (detailQ.value) return 
  if (flow.resumeAsk.value) {
    if (e.code === 'Escape') flow.closeResumeAsk()
    return
  }
  if (flow.switchAsk.value) {
    if (e.code === 'Escape') flow.switchAsk.value = null
    return
  }
  if (asking.value) {
    if (e.code === 'Escape') declineCurrent()
    return
  }
  if (favAsk.value) {
    if (e.code === 'Escape') favAsk.value = null
    return
  }
  if (viewing.value) {
    if (e.code === 'Escape') viewing.value = null
    return
  }
  if (e.code === 'Escape') {
    if (confirmDel.value) {
      confirmDel.value = null
      return
    }
    router.push('/')
  }
}


async function onBlankCreated(b: { id: string; name: string }) {
  favAsk.value = b
  await lib.refresh({ quiet: true })
}

async function acceptFavorite() {
  const b = favAsk.value
  favAsk.value = null
  if (!b) return
  await favs.setFavoriteBook(b.id)
  flash.value = `已将「${b.name}」设为收藏题本\n刷题时收藏的题目会加入这里`
}

async function doDelete() {
  const def = confirmDel.value
  confirmDel.value = null
  if (!def) return
  await lib.removeConfirmed(def)
  if (favs.favoriteBookId.value === def.id) {
    await favs.setFavoriteBook(null)
  }
}

function exportAndCloseView(def: BookDef) {
  const session = lib.sessionForExport(def)
  if (session) {
    currentSession.value = session
    exportSession.value = session
  }
  viewing.value = null
}

function deleteAndCloseView(def: BookDef) {
  confirmDel.value = def
  viewing.value = null
}

async function setFavorite(def: BookDef) {
  if (favs.favoriteBookId.value === def.id) {
    await favs.setFavoriteBook(null)
    flash.value = `已停止在「${def.name}」收藏\n已收藏的题目仍保留在题本中`
  } else {
    await favs.setFavoriteBook(def.id)
    flash.value = `已将「${def.name}」设为收藏题本\n刷题时收藏的题目会加入这里`
  }
}

function goNew() {
  newOpen.value = true
}

const anyModalOpen = computed(
  () => !!(asking.value || viewing.value || confirmDel.value || flow.resumeAsk.value || flow.switchAsk.value || favAsk.value),
)
watch(anyModalOpen, (on) => (on ? lockBodyScroll() : unlockBodyScroll()))
</script>

<template>
  <div class="bh">
    <Teleport to="body">
      <Transition name="ft">
        <div v-if="flash" class="flash-toast">{{ flash }}</div>
      </Transition>
    </Teleport>

    <ActiveBookCard
      v-if="lib.activeBook.value"
      :def="lib.activeBook.value"
      :resume-active="resumeActive"
      :subjects-line="lib.isComposite.value ? lib.subjectsOf(lib.activeBook.value).join(' · ') : ''"
      :stale-count="lib.staleCountOf(lib.activeBook.value)"
      @start="(review) => flow.launch(lib.activeBook.value!, review)"
      @view="flow.viewBook(lib.activeBook.value!)"
    />

    <FavoriteBookCard :def="favoriteDef" @view="flow.viewBook" />

    <BookListCard
      :title="currentSubject ? `题本 · ${currentSubject}` : '题本'"
      :count="lib.subjectSplit.value.main.length"
      :books="lib.subjectSplit.value.main"
      :active-id="lib.activeId.value"
      :loading="lib.loading.value"
      :empty-hint="lib.books.value.length ? mainEmptyHint : NO_BOOKS_HINT"
      :show-subjects="lib.isComposite.value"
      :subjects-of="lib.subjectsOf"
      :stale-count-of="lib.staleCountOf"
      @view="flow.viewBook"
      @delete="(d) => (confirmDel = d)"
    />

    <BookListCard
      v-if="currentSubject && lib.subjectSplit.value.mixed.length"
      mixed
      :title="`也包含${currentSubject}的题本`"
      :count="lib.subjectSplit.value.mixed.length"
      :books="lib.subjectSplit.value.mixed"
      :active-id="lib.activeId.value"
      :loading="lib.loading.value"
      empty-hint=""
      :subject-count-of="lib.subjectCountOf"
      :current-subject="currentSubject"
      :stale-count-of="lib.staleCountOf"
      @view="flow.viewBook"
      @delete="(d) => (confirmDel = d)"
    />

    <BookListCard
      v-if="lib.emptyBooks.value.length"
      :title="'空题本'"
      :count="lib.emptyBooks.value.length"
      :books="lib.emptyBooks.value"
      :active-id="lib.activeId.value"
      :loading="lib.loading.value"
      empty-hint=""
      :show-subjects="false"
      @view="flow.viewBook"
      @delete="(d) => (confirmDel = d)"
    />

    <NewBookPill @click="goNew" />

    <ExportDialog
      v-if="exportSession"
      :session="exportSession"
      @close="exportSession = null"
    />

    <BookHomeDialogs
      :asking="asking"
      :switch-ask="flow.switchAsk.value"
      :active-name="activeName"
      :resume-ask="flow.resumeAsk.value"
      :paused="lib.paused.value"
      :confirm-del="confirmDel"
      :fav-ask="favAsk"
      @accept-current="acceptCurrent"
      @decline-current="declineCurrent"
      @confirm-switch="flow.confirmSwitch"
      @switch-without="flow.withoutSwitch"
      @cancel-switch="flow.switchAsk.value = null"
      @resume="flow.doResume"
      @discard-pause="flow.doDiscard"
      @close-resume="flow.closeResumeAsk"
      @confirm-delete="doDelete"
      @cancel-delete="confirmDel = null"
      @accept-favorite="acceptFavorite"
      @cancel-favorite="favAsk = null"
    />

    <BookViewDialog
      v-if="viewing"
      :def="viewing"
      :is-favorite="favs.favoriteBookId.value === viewing.id"
      @close="viewing = null"
      @start="startFromViewing"
      @export="exportAndCloseView"
      @delete="deleteAndCloseView"
      @study-from="studyFrom"
      @open-detail="detailQ = $event"
      @rename="lib.commitViewRename"
      @set-favorite="setFavorite"
      @open-sources="sourcesOpen = true"
      @clean-stale="cleanStaleViewing"
    />

    <SourcesDialog
      v-if="sourcesOpen"
      @close="sourcesOpen = false"
      @changed="onSourcesChanged"
    />

    <QuestionDetail
      v-if="detailQ"
      :key="detailQ.id"
      :q="detailQ"
      :siblings="viewSiblings"
      studyable
      @close="detailQ = null"
      @study-from="studyFrom"
      @navigate="detailQ = $event"
    />

    <BookNewDialog v-if="newOpen" @close="newOpen = false" @blank-created="onBlankCreated" />
  </div>
</template>

<style scoped>
.bh {
  display: flex;
  flex-direction: column;
  gap: 0.875rem;
}

.flash-toast {
  position: absolute;
  top: calc(1.25rem + env(safe-area-inset-top));
  left: 50%;
  transform: translateX(-50%);
  z-index: var(--z-toast);
  background: var(--card);
  border: 1px solid var(--line);
  border-radius:0.75rem;
  padding: 0.75rem 1.125rem;
  box-shadow: var(--shadow-pop);
  font-size: 0.875rem;
  font-weight: 600;
  white-space: pre-line;
  text-align: center;
  line-height: 1.7;
  max-width: min(26rem, calc(100vw - 2.5rem));
  pointer-events: none;
}

.ft-enter-active,
.ft-leave-active {
  transition:
    opacity 0.18s ease,
    transform 0.18s ease;
}

.ft-enter-from,
.ft-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-0.5rem);
}

/* Narrow screens: the book page switches to full-width flat sections — cards lose their border/
   rounding/color contrast, hierarchy is conveyed by dividers and light backgrounds, and the main
   button insets keep a sense of layering */
@container (max-width: 40rem) {
  .bh {
    gap: 0;
  }

  .bh .card {
    border: none;
    border-bottom:1px solid var(--line);
    border-radius: 0;
    padding: 1rem;
  }

  .bh .cur {
    box-shadow: none;
    padding: 1rem;
  }
}
</style>
