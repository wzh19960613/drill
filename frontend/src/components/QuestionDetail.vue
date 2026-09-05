<script setup lang="ts">
import { computed, ref } from 'vue'
import {
  BadgeCheck,
  Check,
  ChevronLeft,
  ChevronRight,
  MoreHorizontal,
  Pencil,
  Play,
  Trash2,
  XCircle,
} from 'lucide-vue-next'
import type { Question } from '../types'
import { acc5Percent, addRecord, chapterOf, currentSubject, isMastered, statsOf, toggleMastered } from '../store'
import { useFavorites } from '../composables/useFavorites'
import { deleteQuestionApi } from '../api'
import { codeIndex, loadHotkeys, type HotkeyAction } from '../hotkeys'
import { COPY_MENU_ITEMS, copyQuestionPart } from '../composables/copyText'
import { useDialogShell } from '../composables/useDialogShell'
import { useAnchoredMenu } from '../composables/useAnchoredMenu'
import { fmtTime } from '../format'
import DialogHeader from './DialogHeader.vue'
import FavoriteStar from './FavoriteStar.vue'
import MenuPop, { type MenuPopItem } from './MenuPop.vue'
import QuestionView from './QuestionView.vue'
import QuestionHistoryList from './question/QuestionHistoryList.vue'

const props = withDefaults(
  defineProps<{
    q: Question
    editable?: boolean
    studyable?: boolean
    siblings?: Question[]
  }>(),
  { editable: false, studyable: false, siblings: () => [] },
)

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'edit', q: Question): void
  (e: 'deleted', q: Question): void
  (e: 'studyFrom', q: Question): void
  (e: 'navigate', q: Question): void
}>()

const favs = useFavorites()

async function toggleFav() {
  await favs.toggleQuestion(props.q, !favs.isFavorited(props.q.id))
}

const mastered = computed(() => isMastered(props.q))

async function toggleMasteredNow() {
  await toggleMastered(props.q)
}

const navIdx = computed(() => props.siblings.findIndex((s) => s.id === props.q.id))
const prevQ = computed(() => (navIdx.value > 0 ? props.siblings[navIdx.value - 1] : null))
const nextQ = computed(() =>
  navIdx.value >= 0 && navIdx.value < props.siblings.length - 1
    ? props.siblings[navIdx.value + 1]
    : null,
)

const { open: menuOpen, x: menuX, y: menuY, toggle: toggleMenu, close: closeMenu } =
  useAnchoredMenu()

const menuItems = computed<MenuPopItem[]>(() =>
  props.editable
    ? [
        ...COPY_MENU_ITEMS,
        { key: 'edit', label: '编辑', icon: Pencil, sep: true },
        { key: 'delete', label: '删除', icon: Trash2, danger: true },
      ]
    : COPY_MENU_ITEMS,
)

function onMenuSelect(item: MenuPopItem) {
  if (item.key === 'edit') emit('edit', props.q)
  else if (item.key === 'delete') delConfirm.value = true
  else void copyQuestionPart(props.q, item.key as 'stem' | 'answer' | 'full')
}

const stats = computed(() => statsOf(props.q))

const showSubject = computed(() => !!props.q.subject && currentSubject.value === '')

const acc5 = computed(() => acc5Percent(props.q))

async function mark(correct: boolean) {
  await addRecord(props.q, correct)
}

const histRef = ref<InstanceType<typeof QuestionHistoryList> | null>(null)

const delConfirm = ref(false)
const deleting = ref(false)

async function doDelete() {
  deleting.value = true
  try {
    await deleteQuestionApi(props.q.id, props.q.source)
    emit('deleted', props.q)
    emit('close')
  } finally {
    deleting.value = false
  }
}

const hotkeyIdx = computed(() => codeIndex(loadHotkeys()))

function runDialogAction(a: HotkeyAction): boolean {
  if (a === 'prev' && prevQ.value) {
    emit('navigate', prevQ.value)
    return true
  }
  if (a === 'next' && nextQ.value) {
    emit('navigate', nextQ.value)
    return true
  }
  if (a === 'markMastered') {
    void toggleMasteredNow()
    return true
  }
  if (a === 'toggleFavorite') {
    void toggleFav()
    return true
  }
  return false
}

useDialogShell((e) => {
  if (e.code === 'Escape') {
    e.stopPropagation()
    if (menuOpen.value) {
      closeMenu()
      return
    }
    if (histRef.value?.handleEscape()) return
    if (delConfirm.value) {
      delConfirm.value = false
      return
    }
    emit('close')
    return
  }
  const actions = hotkeyIdx.value.get(e.code)
  if (actions?.length && actions.some((a) => runDialogAction(a))) {
    e.preventDefault()
    e.stopPropagation()
  }
})
</script>

<template>
  <Teleport to="body">
  <div class="modal-mask qmodal-mask fs" @click.self="emit('close')">
    <div class="qmodal dialog fs">
      <div class="qm-headwrap">
        <DialogHeader no-border flush @close="emit('close')">
          <template #title>
            <span class="qm-qid">{{ q.locate || '未知' }}</span>
            <span v-if="q.origin" class="qm-src"> @ {{ q.origin }}</span>
            <span v-if="showSubject" class="qm-src"> [{{ q.subject }}]</span>
          </template>
          <template #subtitle>{{ [chapterOf(q), q.qtype].filter(Boolean).join(' · ') }}</template>
        </DialogHeader>

        <div class="qm-toolbar">
          <button
            class="qm-tool"
            :disabled="!prevQ"
            aria-label="上一题"
            title="上一题"
            @click="emit('navigate', prevQ!)"
          >
              <ChevronLeft style="width: 1.125rem; height: 1.125rem" />
            </button>
            <button
              class="qm-tool mastered"
              :class="{ on: mastered }"
              :aria-label="mastered ? '取消熟练' : '标记熟练'"
              :title="mastered ? '取消熟练' : '标记熟练'"
              @click="toggleMasteredNow"
            >
              <BadgeCheck style="width: 1.125rem; height: 1.125rem" />
            </button>
          <FavoriteStar :on="favs.isFavorited(q.id)" @toggle="toggleFav" />
          <button
            class="qm-tool"
            :class="{ on: menuOpen }"
            aria-label="更多操作"
            title="更多操作"
            @click="toggleMenu"
          >
            <MoreHorizontal style="width: 1.125rem; height: 1.125rem" />
          </button>
          <button
            class="qm-tool"
            :disabled="!nextQ"
            aria-label="下一题"
            title="下一题"
            @click="emit('navigate', nextQ!)"
          >
              <ChevronRight style="width: 1.125rem; height: 1.125rem" />
            </button>
        </div>
      </div>

      <MenuPop
        :open="menuOpen"
        :x="menuX"
        :y="menuY"
        align="right"
        :items="menuItems"
        @select="onMenuSelect"
        @close="closeMenu"
      />

      <div class="qm-body">
        <div class="qm-stats num" :class="{ none: !stats.attempts }">
          <template v-if="stats.attempts">
            做过 {{ stats.attempts }} 次 · 错 {{ stats.wrong }} 次 · 近五次正确率 {{ acc5 }}% · 最后做题
            {{ fmtTime(stats.last_at) }}
          </template>
          <template v-else>尚未做过本题</template>
        </div>

        <QuestionView :q="q" :show-answer="true" :show-meta="false" :fold-answer="true" />

        <QuestionHistoryList ref="histRef" :q="q" />
      </div>

      <div class="qm-foot">
        <template v-if="editable && delConfirm">
          <span class="del-confirm">删除题目文件？（保留 .bak 备份，记录统计不再包含它）</span>
          <button class="btn bad sm" :disabled="deleting" @click="doDelete">确认删除</button>
          <button class="btn ghost sm" @click="delConfirm = false">取消</button>
        </template>
        <template v-else>
          <div class="qm-foot-side">
            <button v-if="studyable" class="btn primary sm" @click="emit('studyFrom', q)">
              <Play style="width: 0.875rem; height: 0.875rem" /> 从此题处开始刷题
            </button>
          </div>
          <div class="qm-mark-row">
            <button class="btn good sm" @click="mark(true)">
              <Check style="width: 0.875rem; height: 0.875rem" /> 又对了一次
            </button>
            <button class="btn bad sm" @click="mark(false)">
              <XCircle style="width: 0.875rem; height: 0.875rem" /> 又错了一次
            </button>
          </div>
        </template>
      </div>
    </div>
  </div>
  </Teleport>
</template>

<style scoped>
.qmodal-mask {
  z-index: var(--z-detail);
}

.qmodal {
  width: min(48.75rem, 94vw);
  max-height: 86vh;
}

@container (max-width: 62.4375rem) {
  .qm-foot {
    padding-bottom: calc(0.75rem + env(safe-area-inset-bottom));
    flex-wrap: wrap;
    row-gap: 0.5rem;
  }

  .qm-foot-side,
  .qm-mark-row {
    flex: 1 1 100%;
    margin-left: 0;
  }

  .qm-foot-side .btn,
  .qm-mark-row .btn {
    flex: 1;
    justify-content: center;
  }
}

.qm-headwrap {
  position: relative;
  border-bottom:1px solid var(--line);
  flex: none;
}

.qm-toolbar {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 0 1rem 0.625rem;
}

.qm-toolbar .fav-star {
  width: 2.25rem;
  height: 2.25rem;
}

.qm-tool {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2.25rem;
  height: 2.25rem;
  border: none;
  background: transparent;
  border-radius:0.5rem;
  color: var(--muted);
  cursor: pointer;
}

.qm-tool:hover:not(:disabled) {
  background: var(--hover);
  color: var(--ink);
}

.qm-tool:disabled {
  opacity: 0.35;
  cursor: default;
}

.qm-tool.mastered.on {
  color: var(--good);
}

.qm-tool.on {
  color: var(--ink);
}

.qm-qid {
  font-family: var(--font-mono);
  font-weight: 700;
  font-size: 0.9375rem;
}

.qm-body {
  flex: 1;
  min-height: 0;
  padding: 0.875rem 1.25rem;
  overflow-y: auto;
  overflow-x: clip;
  overscroll-behavior: contain;
}

.qm-stats {
  font-size: 0.8125rem;
  color: var(--muted);
  background: var(--panel);
  border-radius:0.5rem;
  padding: 0.5rem 0.75rem;
  margin-bottom: 0.75rem;
}

.qm-stats.none {
  text-align: center;
  letter-spacing: 0.0625rem;
}

.qm-foot {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1.125rem;
  border-top:1px solid var(--line);
}

.qm-foot-side {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.qm-mark-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-left: auto;
}

.del-confirm {
  font-size: 0.8125rem;
  color: var(--bad);
  margin-right: auto;
}

.qm-src {
  color: var(--muted);
  font-size: 0.875rem;
  font-weight: 400;
}
</style>
