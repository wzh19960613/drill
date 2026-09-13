<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { isMastered, loadAll, toggleMastered } from '../store'
import { applyAnswerActions, codeIndex, loadHotkeys, type HotkeyAction, type HotkeyMap } from '../hotkeys'
import { useFavorites } from '../composables/useFavorites'
import { useStudyRound } from '../composables/useStudyRound'
import { copyQuestionPart } from '../composables/copyText'
import { fmtDur } from '../format'
import QuestionView from '../components/QuestionView.vue'
import HotkeySettings from '../components/HotkeySettings.vue'
import StudyTopBar from '../components/study/StudyTopBar.vue'
import StudySummary from '../components/study/StudySummary.vue'
import StudyActionBar from '../components/study/StudyActionBar.vue'

const route = useRoute()
const router = useRouter()

const flash = ref('')
let flashTimer: number | undefined

function toast(msg: string) {
  flash.value = msg
  if (flashTimer) window.clearTimeout(flashTimer)
  flashTimer = window.setTimeout(() => (flash.value = ''), 1800)
}

const round = useStudyRound(toast)
const favs = useFavorites()
const menuOpen = ref(false)
const hotkeyOpen = ref(false)
const hotkeys = ref<HotkeyMap>(loadHotkeys())
const hotkeyIdx = computed(() => codeIndex(hotkeys.value))

function runAction(a: HotkeyAction, revealedAtPress: boolean) {
  if (round.review.value && ['showAnswer', 'hideAnswer', 'timerToggle', 'timerReset'].includes(a)) return
  switch (a) {
    case 'markRight':
      if (revealedAtPress) void round.mark(true)
      break
    case 'markWrong':
      if (revealedAtPress) void round.mark(false)
      break
    case 'toggleFavorite':
      void toggleFavoriteNow()
      break
    case 'prev':
      round.prev()
      break
    case 'next':
      round.next()
      break
    case 'timerToggle':
      round.toggleTimer()
      break
    case 'timerReset':
      round.resetTimer()
      break
    case 'markMastered':
      void toggleMasteredNow()
      break
  }
}

function applyAnswerKeys(actions: HotkeyAction[], revealedAtPress: boolean) {
  const answerActions = actions.filter((a) => a === 'showAnswer' || a === 'hideAnswer')
  if (answerActions.length && !round.review.value) {
    round.showAnswer.value = applyAnswerActions(revealedAtPress, answerActions)
  }
}

async function toggleFavoriteNow() {
  if (!round.item.value || round.done.value) return
  await favs.ensureLoaded()
  const q = round.item.value
  const on = !favs.isFavorited(q.id)
  const ok = await favs.toggleQuestion(q, on)
  toast(ok ? (on ? '已收藏到收藏题本' : '已取消收藏') : '操作失败')
}

async function toggleMasteredNow() {
  if (!round.item.value || round.done.value) return
  const on = await toggleMastered(round.item.value)
  toast(on ? '已标记为熟练' : '已取消熟练标记')
}

function onKey(e: KeyboardEvent) {
  const t = e.target as HTMLElement | null
  if (t && ['INPUT', 'SELECT', 'TEXTAREA'].includes(t.tagName)) return
  if (e.code === 'Escape') {
    if (menuOpen.value) {
      menuOpen.value = false
      return
    }
    if (hotkeyOpen.value) return
    round.leave()
    return
  }
  if (hotkeyOpen.value) return
  if (round.done.value) {
    if (round.interrupted.value && e.code === 'Space') {
      e.preventDefault()
      void round.continueInterrupted()
    }
    return
  }
  const actions = hotkeyIdx.value.get(e.code)
  if (!actions?.length) return
  e.preventDefault()
  const revealedAtPress = round.review.value || round.showAnswer.value
  applyAnswerKeys(actions, revealedAtPress)
  for (const a of actions) runAction(a, revealedAtPress)
}

async function menuCopy(kind: 'stem' | 'answer' | 'full') {
  menuOpen.value = false
  const q = round.item.value
  if (!q) return
  try {
    await copyQuestionPart(q, kind)
    toast('已复制到剪贴板')
  } catch {
    toast('复制失败')
  }
}

watch(hotkeyOpen, (v) => {
  if (!v) hotkeys.value = loadHotkeys()
})

watch(
  () => route.fullPath,
  () => {
    if (route.name === 'study' || route.name === 'bookStudy') {
      hotkeys.value = loadHotkeys()
      void round.build()
    }
  },
)

onMounted(async () => {
  await loadAll()
  void round.build()
  round.startClock()
  window.addEventListener('keydown', onKey)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKey)
})
</script>

<template>
  <div class="study">
    <StudyTopBar
      :title="round.session.value?.title ?? '刷题'"
      :idx="round.idx.value"
      :total="round.total.value"
      :review="round.review.value"
      :mastered="!!round.item.value && isMastered(round.item.value)"
      :favorited="!!round.item.value && favs.isFavorited(round.item.value.id)"
      :done="round.done.value"
      :has-item="!!round.item.value"
      :show-answer="round.showAnswer.value"
      :revisit-answered="round.revisitAnswered.value"
      :session-ms="round.sessionMs.value"
      :question-ms="round.questionMs.value"
      :timer-paused="round.timerPaused.value"
      v-model:menu-open="menuOpen"
      :hotkeys="hotkeys"
      @leave="round.leave()"
      @toggle-timer="round.toggleTimer()"
      @reset-timer="round.resetTimer()"
      @toggle-mastered="toggleMasteredNow"
      @toggle-favorite="toggleFavoriteNow"
      @copy="menuCopy"
      @manage-books="router.push('/book'); menuOpen = false"
      @open-hotkeys="hotkeyOpen = true; menuOpen = false"
    />

    <main class="s-body">
      <StudySummary
        v-if="round.done.value"
        :interrupted="round.interrupted.value"
        :judged="round.roundRecords.value.size"
        :total="round.total.value"
        :right="round.right.value"
        :wrong="round.wrong.value"
        :unanswered="round.unanswered.value"
        :session-ms="fmtDur(round.sessionMs.value)"
        @continue="round.continueInterrupted()"
        @retry-wrong="round.retryWrongOnly()"
        @retry-all="round.retryAll()"
        @home="round.leave()"
      />

      <div v-else-if="round.item.value" class="s-card card">
        <QuestionView
          :key="round.item.value.id"
          :q="round.item.value"
          :option-order="round.item.value.optionOrder"
          :show-answer="round.review.value || round.showAnswer.value"
        />
      </div>

      <div v-else class="s-card card empty">
        题库为空。
      </div>
    </main>

    <StudyActionBar
      v-if="!round.done.value && round.item.value"
      :review="round.review.value"
      :show-answer="round.showAnswer.value"
      :idx="round.idx.value"
      :total="round.total.value"
      :marked="round.marked.value"
      :hotkeys="hotkeys"
      @mark="(c) => round.mark(c)"
      @show-answer="round.showAnswer.value = true"
      @hide-answer="round.showAnswer.value = false"
      @prev="round.prev()"
      @next="round.next()"
    >
      <div class="s-flash" :class="{ show: flash }">{{ flash }}</div>
    </StudyActionBar>

    <HotkeySettings v-if="hotkeyOpen" @close="hotkeyOpen = false" />
  </div>
</template>

<style scoped>
.study {
  /* Full-screen view: filling the scroll container is enough (the .app scroll
     container itself is viewport-sized); sized by flex, never a height chain */
  position: relative;
  flex: 1;
  min-height: 0;
  z-index: var(--z-page);
  display: flex;
  flex-direction: column;
  background: var(--bg);
}

.s-body {
  flex: 1;
  /* allow the scroll area to shrink below its content height, otherwise a
     long question pushes the action bar out of the viewport */
  min-height: 0;
  overflow-y: auto;
  /* Vertical scrolling only: overwide content is handled by math scaling (fitmath); never scroll sideways */
  overflow-x: clip;
}

.s-card {
  max-width: 55rem;
  margin: 1.375rem auto 1.875rem;
  padding: 1.625rem 2rem;
}

.s-card :deep(.q-stem),
.s-card :deep(.q-opt) {
  font-size: 1rem;
}

.s-flash {
  position: absolute;
  top: -2.125rem;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(31, 35, 41, 0.82);
  color: #fff;
  font-size: 0.7812rem;
  border-radius: var(--radius-pill);
  padding:3px 0.875rem;
  opacity: 0;
  transition: opacity 0.2s;
  pointer-events: none;
  white-space: nowrap;
}

.s-flash.show {
  opacity: 1;
}

@container (max-width: 40rem) {
  .s-card {
    margin: 0.75rem 0.625rem 1.25rem;
    padding: 1.125rem 1rem;
  }
}
</style>
