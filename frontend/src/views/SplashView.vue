<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { BookOpen, ClipboardCheck, List, Play } from 'lucide-vue-next'
import { currentSubject, loadAll, store } from '../store'
import { fetchBooks, getActiveBookId } from '../book'
import { getPaused } from '../api'
import type { PausedSession } from '../types'
import { useTimeEstimate } from '../composables/useTimeEstimate'

const router = useRouter()

const paused = ref<PausedSession | null>(null)

const activeBookName = ref('')

async function refreshActiveBookName() {
  try {
    const [books, activeId] = await Promise.all([fetchBooks(), getActiveBookId(currentSubject.value)])
    activeBookName.value = books.find((b) => b.id === activeId)?.name ?? ''
  } catch {
  }
}

/** The active book is remembered per subject; switching swaps the name */
watch(currentSubject, refreshActiveBookName)

const { estimate, typeSummary, total } = useTimeEstimate()

function goStudy() {
  router.push(paused.value ? '/study?resume=1' : '/study')
}

function goList() {
  router.push('/list')
}

function goBook() {
  router.push('/book')
}

function goReview() {
  router.push('/study?mode=review')
}

function onKey(e: KeyboardEvent) {
  if (e.metaKey || e.ctrlKey || e.altKey) return
  if (e.code === 'Space') {
    e.preventDefault()
    goStudy()
  } else if (e.code === 'KeyL') {
    e.preventDefault()
    goList()
  } else if (e.code === 'KeyB') {
    e.preventDefault()
    goBook()
  } else if (e.code === 'KeyA') {
    e.preventDefault()
    goReview()
  }
}

onMounted(async () => {
  getPaused().then((p) => (paused.value = p)).catch(() => {})
  await refreshActiveBookName()
  await loadAll()
  window.addEventListener('keydown', onKey)
})
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <div class="splash">
    <div class="sp-center">
      <div class="sp-book">{{ activeBookName || '未选择题本' }}</div>
      <div class="sp-count num">
        <span class="sp-num">{{ store.loaded ? total : '…' }}</span>
        <span class="sp-unit">题</span>
      </div>
      <div class="sp-est">{{ store.loaded ? estimate : '' }}</div>
      <div class="sp-sub">{{ store.loaded ? typeSummary : '加载中' }}</div>
    </div>

    <div class="sp-actions">
      <div v-if="paused" class="sp-resume num">
        上次中断：{{ paused.idx + 1 }} / {{ paused.items.length }}，点击继续
      </div>
      <div class="sp-join">
        <button class="sp-start" @click="goStudy">
          <Play style="width: 1.25rem; height: 1.25rem" /> {{ paused ? '继续刷题' : '开始刷题' }} <span class="kbd">空格</span>
        </button>
        <button class="sp-review" @click="goReview">
          <ClipboardCheck style="width: 1.125rem; height: 1.125rem" /> 对答案 <span class="kbd">A</span>
        </button>
      </div>
      <div class="sp-row">
        <button class="btn sp-sec" @click="goList">
          <List style="width: 1.0625rem; height: 1.0625rem" /> 题库 <span class="kbd">L</span>
        </button>
        <button class="btn sp-sec" @click="goBook">
          <BookOpen style="width: 1.0625rem; height: 1.0625rem" /> 题本 <span class="kbd">B</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.splash {
  position: relative;
  /* fill main's box so the content centers when there is slack; taller
     content grows main itself (page-level scrolling with the bottom bar) */
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12vh;
  padding:calc(2.5rem + env(safe-area-inset-top)) 1.25rem 1.25rem;
}

/* Centering via auto margins: with slack they center, when the viewport is
   too short they collapse to 0 and the content keeps its natural height —
   flex-shrink: 0 keeps blocks from being squashed, the page scrolls instead */
.sp-center,
.sp-actions {
  flex-shrink: 0;
}

.sp-center {
  margin-top: auto;
  text-align: center;
}

.sp-actions {
  margin-bottom: auto;
  width: min(28.75rem, 100%);
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.sp-book {
  font-size: 0.875rem;
  color: var(--muted);
  margin-bottom: 0.625rem;
  font-weight: 600;
}

.sp-count {
  display: flex;
  align-items: baseline;
  justify-content: center;
  gap: 0.625rem;
}

.sp-num {
  font-size: 5.5rem;
  font-weight: 800;
  line-height: 1;
  color: var(--ink);
  letter-spacing: -0.125rem;
}

.sp-unit {
  font-size: 1.375rem;
  color: var(--muted);
}

.sp-est {
  margin-top: 0.875rem;
  font-size: 1.25rem;
  color: var(--brand);
  font-weight: 600;
}

.sp-sub {
  margin-top: 0.625rem;
  font-size: 0.8125rem;
  color: var(--muted);
}

/* Joined button group: shared borders, no rounding; hover glows as a whole */
.sp-join {
  display: flex;
  border-radius:0.875rem;
  transition:
    box-shadow 0.28s ease,
    transform 0.28s ease,
    filter 0.28s ease;
}

.sp-join:hover {
  filter: brightness(1.07);
  transform: translateY(-1.5px);
  box-shadow: 0 0 18px 2px rgba(80, 140, 255, 0.75);
  box-shadow:
    0 0 0 2px color-mix(in srgb, var(--brand) 85%, transparent),
    0 0 22px 3px color-mix(in srgb, var(--brand) 70%, transparent),
    0 10px 38px -4px color-mix(in srgb, var(--brand) 60%, transparent);
}

.sp-resume {
  text-align: center;
  font-size: 0.8125rem;
  color: var(--muted);
}

.sp-join:active {
  transform: translateY(0);
  transition-duration: 0.1s;
}

.sp-start,
.sp-review {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.625rem;
  border: none;
  background: var(--brand);
  color: #fff;
  cursor: pointer;
  font-family: inherit;
}

.sp-start {
  flex: 1.8;
  min-height: 3.75rem;
  font-size: 1.125rem;
  border-radius:0.875rem 0 0 0.875rem;
}

.sp-review {
  flex: 1;
  min-height: 3.75rem;
  font-size: 0.9375rem;
  border-radius:0 0.875rem 0.875rem 0;
  border-left:1px solid rgba(255, 255, 255, 0.28);
}

.sp-start:hover,
.sp-review:hover {
  opacity: 0.92;
}

.sp-start .kbd,
.sp-review .kbd {
  background: rgba(255, 255, 255, 0.22);
  border-color: rgba(255, 255, 255, 0.45);
  color: #fff;
}

.sp-row {
  display: flex;
  gap: 0.75rem;
}

.sp-sec {
  flex: 1;
  min-height: 3.25rem;
  font-size: 1rem;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
}
</style>
