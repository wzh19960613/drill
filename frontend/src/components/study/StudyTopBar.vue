<script setup lang="ts">
import { ref } from 'vue'
import { BadgeCheck, BookOpen, Keyboard, MoreHorizontal, X } from 'lucide-vue-next'
import MenuPop, { type MenuPopItem } from '../MenuPop.vue'
import StudyTimer from './StudyTimer.vue'
import { fmtKey, primaryCode } from '../../hotkeys'
import { COPY_MENU_ITEMS } from '../../composables/copyText'
import FavoriteStar from '../FavoriteStar.vue'

const props = defineProps<{
  title: string
  idx: number
  total: number
  review: boolean
  mastered: boolean
  favorited: boolean
  done: boolean
  hasItem: boolean
  showAnswer: boolean
  revisitAnswered: boolean
  sessionMs: number
  questionMs: number
  timerPaused: boolean
  menuOpen: boolean
  hotkeys: Record<string, string>
}>()

const emit = defineEmits<{
  (e: 'leave'): void
  (e: 'toggle-timer'): void
  (e: 'reset-timer'): void
  (e: 'toggle-mastered'): void
  (e: 'toggle-favorite'): void
  (e: 'update:menuOpen', v: boolean): void
  (e: 'copy', kind: 'stem' | 'answer' | 'full'): void
  (e: 'manage-books'): void
  (e: 'open-hotkeys'): void
}>()

const menuBtn = ref<HTMLElement | null>(null)
const menuPos = ref({ x: 0, y: 0 })

const MENU_ITEMS: MenuPopItem[] = [
  ...COPY_MENU_ITEMS,
  { key: 'books', label: '管理题本', icon: BookOpen, sep: true },
  { key: 'hotkeys', label: '快捷键设置…', icon: Keyboard },
]

function toggleMenu() {
  if (props.menuOpen) {
    emit('update:menuOpen', false)
    return
  }
  const rect = menuBtn.value?.getBoundingClientRect()
  if (!rect) return
  menuPos.value = { x: rect.right, y: rect.bottom + 4 }
  emit('update:menuOpen', true)
}

function onMenuSelect(item: MenuPopItem) {
  if (item.key === 'books') emit('manage-books')
  else if (item.key === 'hotkeys') emit('open-hotkeys')
  else emit('copy', item.key as 'stem' | 'answer' | 'full')
}
</script>

<template>
  <header class="s-top" :class="{ review }">
    <button class="s-icon s-exit" aria-label="退出刷题" title="退出（Esc）" @click="emit('leave')">
      <X style="width: 1.375rem; height: 1.375rem" />
    </button>
    <div class="s-mid">
      <div class="s-title">
        <span v-if="mastered" class="chip good">已熟练</span>
        <span class="t">{{ title }}</span>
        <span class="seq num" v-if="total">{{ idx + 1 }} / {{ total }}</span>
      </div>
      <div class="s-bar-track">
        <div class="s-bar-fill" :style="{ width: (total ? ((idx + 1) / total) * 100 : 0) + '%' }"></div>
      </div>
    </div>

    <StudyTimer
      v-if="!review"
      :session-ms="sessionMs"
      :question-ms="questionMs"
      :paused="timerPaused"
      :frozen="timerPaused || revisitAnswered || showAnswer"
      :toggle-hint="fmtKey(primaryCode(hotkeys.timerToggle))"
      :reset-hint="fmtKey(primaryCode(hotkeys.timerReset))"
      @toggle="emit('toggle-timer')"
      @reset="emit('reset-timer')"
    />
    <FavoriteStar
      v-if="!done && hasItem"
      :on="favorited"
      style="width: 1.25rem; height: 1.25rem"
      class="s-icon fav-star-top"
      :title="`收藏 / 取消收藏（${fmtKey(primaryCode(hotkeys.toggleFavorite))}）`"
      @toggle="emit('toggle-favorite')"
    />
    <button
      v-if="!done && hasItem"
      class="s-icon mastered-btn"
      :class="{ on: mastered }"
      :aria-label="mastered ? '取消熟练' : '标记为熟练'"
      :title="`标记/取消熟练（${fmtKey(primaryCode(hotkeys.markMastered))}）`"
      @click="emit('toggle-mastered')"
    >
      <BadgeCheck style="width: 1.25rem; height: 1.25rem" />
    </button>
    <button ref="menuBtn" class="s-icon s-menu-btn" aria-label="菜单" @click="toggleMenu">
      <MoreHorizontal style="width: 1.375rem; height: 1.375rem" />
    </button>

    <MenuPop
      :open="menuOpen"
      :x="menuPos.x"
      :y="menuPos.y"
      align="right"
      :items="MENU_ITEMS"
      @select="onMenuSelect"
      @close="emit('update:menuOpen', false)"
    />
  </header>
</template>

<style scoped>
.s-top {
  position: relative;
  display: flex;
  align-items: center;
  gap: 0.875rem;
  padding:calc(1rem + env(safe-area-inset-top)) 1rem 1rem;
  background: var(--card);
  border-bottom:1px solid var(--line);
}

.s-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap:2px;
  border: none;
  background: transparent;
  font-size: 0.875rem;
  color: var(--muted);
  cursor: pointer;
  min-width: 2.75rem;
  min-height: 2.75rem;
  padding: 0.5rem 0.625rem;
  border-radius:0.625rem;
  flex: none;
}

.s-icon:hover {
  background: var(--hover);
  color: var(--ink);
}

.mastered-btn.on {
  color: var(--good);
}

.fav-star-top {
  min-width: 2.75rem;
  min-height: 2.75rem;
  margin-left: -0.875rem;
}

/* the star keeps its semantic color on hover (the generic .s-icon:hover
   would repaint it ink at equal specificity and later source order) */
.fav-star-top.on:hover {
  color: var(--warn);
}

/* The mastered/menu pair: buttons keep their 44px touch width, only the side
   gaps shrink (including the one towards the timer) */
.mastered-btn {
  margin-left: -0.5rem;
  margin-right: -0.75rem;
}

.s-mid {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.3125rem;
}

.s-title {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  font-size: 0.75rem;
  color: var(--muted);
  min-width: 0;
}

.s-title .t {
  color: var(--muted);
  font-weight: 600;
}

.s-title .seq {
  font-weight: 600;
  color: var(--muted);
}

.s-bar-track {
  height: 0.1875rem;
  border-radius: var(--radius-pill);
  background: var(--panel);
  overflow: hidden;
}

.s-bar-fill {
  height: 0.1875rem;
  background: var(--line);
  border-radius: var(--radius-pill);
  transition: width 0.25s;
}

@container (max-width: 40rem) {
  .s-top {
    flex-wrap: wrap;
    row-gap: 0.5rem;
  }

  .s-mid {
    /* 10px slack: an exact 100%-58px makes some engines wrap the whole column
       onto its own line due to subpixel rounding (the X takes a full row) */
    flex: 1 1 calc(100% - 4.25rem);
    min-width: 0;
  }

  .s-title .t {
    display: inline;
    max-width: 40vw;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .s-timer {
    margin-right: auto;
  }

  .s-top.review {
    flex-wrap: nowrap;
  }

  .s-top.review .s-mid {
    flex: 1 1 auto;
    min-width: 0;
  }
}
</style>
