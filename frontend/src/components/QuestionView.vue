<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { ChevronDown } from 'lucide-vue-next'
import type { QCore } from '../types'
import { chapterOf, subjectOf } from '../store'
import { displayLength, paraHtml, richText } from '../math'
import {
  answerIds,
  displayOptions,
  letterOf,
  remapIds,
} from '../qutil'
import ContextMenu from './question/ContextMenu.vue'
import { useQuestionContextMenu } from '../composables/useQuestionContextMenu'
import { useMathFit } from '../composables/useMathFit'
import { openImageViewer } from '../composables/useImageViewer'

const props = withDefaults(
  defineProps<{
    q: QCore
    optionOrder?: number[] | null
    showAnswer?: boolean
    showMeta?: boolean
    hideOptions?: boolean
    foldAnswer?: boolean
  }>(),
  { optionOrder: null, showAnswer: false, showMeta: true, hideOptions: false, foldAnswer: false },
)

const ansOpen = defineModel<boolean>('answerOpen', { default: false })
watch(
  () => props.q.id,
  () => (ansOpen.value = false),
)

const optionsDisplay = computed(() => displayOptions(props.q, props.optionOrder))
const isChoice = computed(() => props.q.options.length > 0)

const metaTags = computed(() =>
  [subjectOf(props.q), chapterOf(props.q), props.q.qtype].filter(Boolean).join(' · '),
)
const metaTitle = computed(() =>
  [subjectOf(props.q), props.q.origin, chapterOf(props.q), props.q.qtype]
    .filter(Boolean)
    .join(' · '),
)


const headId = computed(() => {
  const q = props.q
  if (q.locate) return q.locate
  return metaTags.value ? '' : (q.file || q.id)
})

function onImageClick(e: MouseEvent) {
  const img = (e.target as HTMLElement).closest?.('img.q-svg')
  if (img) openImageViewer((img as HTMLImageElement).getAttribute('src') ?? '', img)
}

function stripTrailingPeriod(html: string): string {
  return html.replace(/(?:。|\.)\s*$/, '')
}

const answerHtml = computed(() => {
  const q = props.q
  if (isChoice.value) {
    const ids = remapIds(q, props.optionOrder) ?? answerIds(q)
    if (ids.length === 1) {
      const opt = optionsDisplay.value.find((o) => o.letter === letterOf(ids[0]))
      const body = opt ? ' ' + richText(opt.text) : ''
      return `<strong class="ans-letter">（${letterOf(ids[0])}）</strong>${stripTrailingPeriod(body)}`
    }
    if (ids.length > 1) {
      return `<strong class="ans-letter">${ids.map((i) => letterOf(i)).join('')}</strong>`
    }
  }
  return stripTrailingPeriod(richText(q.answer?.[0] ?? ''))
})


const answerRest = computed(() => (props.q.answer ?? []).slice(1))


function isBlockish(p: string): boolean {
  return /^(\s*)([-*+]\s|\d+[.)、]\s|```|\|)/.test(p)
}

/** Answers render as the big centered pill only when they are one simple
 *  single-line paragraph (or a choice letter line); anything multi-line,
 *  multi-paragraph or block-level renders as normal left-aligned blocks. */
const answerAsPill = computed(() => {
  const q = props.q
  if (isChoice.value) return true
  const first = q.answer?.[0] ?? ''
  return (
    (q.answer?.length ?? 0) === 1 &&
    !first.includes('\n') &&
    !isBlockish(first) &&
    !first.startsWith('>') &&
    !first.startsWith('$$') &&
    !first.includes('![[') &&
    displayLength(first) < 20
  )
})

/** Labeled paragraph blocks rendered under the answer line (解析 / 备注) */
const answerSections = computed(() => {
  const sections: { label: string; paras: string[]; notes?: boolean }[] = []
  if (props.q.solution.length) sections.push({ label: '解析', paras: props.q.solution })
  if (props.q.notes?.length) sections.push({ label: '备注', paras: props.q.notes, notes: true })
  return sections
})

const rootEl = ref<HTMLElement | null>(null)

const { ctx, closeCtx } = useQuestionContextMenu({
  rootEl,
  q: () => props.q,
  answerVisible: () => props.showAnswer && (!props.foldAnswer || ansOpen.value),
})
useMathFit(rootEl)
</script>

<template>
  <div ref="rootEl" class="q-view" @click="onImageClick">
    <div v-if="showMeta" class="q-meta">
      <span v-if="headId" class="qid">{{ headId }}</span>
      <span class="q-tags" :title="metaTitle">{{ metaTags }}</span>
    </div>

    <div class="q-stem">
      <template v-for="p in q.stem" :key="p">
        <div v-html="paraHtml(p)"></div>
      </template>
    </div>

    <div v-if="optionsDisplay.length && !hideOptions" class="q-options">
      <div v-for="o in optionsDisplay" :key="o.letter" class="q-opt">
        <span class="opt-letter">（{{ o.letter }}）</span><span class="opt-text" v-html="richText(o.text)"></span>
      </div>
    </div>

    <div v-if="showAnswer || foldAnswer" class="q-answer">
      <button v-if="foldAnswer" class="ans-fold" type="button" @click="ansOpen = !ansOpen">
        <ChevronDown style="width: 0.9375rem; height: 0.9375rem" class="chev" :class="{ closed: !ansOpen }" />
        答案与解析
        <span class="fold-hint">{{ ansOpen ? '收起' : '展开' }}</span>
      </button>
      <div v-if="showAnswer && (!foldAnswer || ansOpen)" class="ans-wrap">
        <template v-if="q.answer?.length">
          <div class="ans-label">答案</div>
          <template v-if="answerAsPill">
            <div class="ans-line" :class="{ multi: answerRest.length > 0 }" v-html="answerHtml"></div>
            <div v-if="answerRest.length" class="ans-rest">
              <div v-for="(p, i) in answerRest" :key="i" v-html="paraHtml(p)"></div>
            </div>
          </template>
          <div v-else class="ans-multi">
            <div v-for="(p, i) in q.answer" :key="i" v-html="paraHtml(p)"></div>
          </div>
        </template>
        <template v-for="sec in answerSections" :key="sec.label">
          <div class="ans-label">{{ sec.label }}</div>
          <div class="ans-body" :class="{ 'ans-notes': sec.notes }">
            <div v-for="p in sec.paras" :key="p" v-html="paraHtml(p)"></div>
          </div>
        </template>
      </div>
    </div>

    <ContextMenu :show="ctx.show" :x="ctx.x" :y="ctx.y" :items="ctx.items" @close="closeCtx" />
  </div>
</template>

<style scoped>
.q-view {
  font-size: 0.9375rem;
}

.q-meta {
  display: flex;
  align-items: baseline;
  gap: 0.625rem;
  margin-bottom: 0.625rem;
}

.qid {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--muted);
  flex: none;
  white-space: nowrap;
}

.q-tags {
  margin-left: auto;
  font-size: 0.75rem;
  color: var(--muted);
  white-space: nowrap;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.q-stem {
  line-height: 1.9;
}

.q-stem :deep(p) {
  margin: 0 0 0.375rem;
}

.q-options {
  margin-top: 0.625rem;
  display: grid;
  gap: 0.5rem;
}

.q-opt {
  display: flex;
  gap:2px;
  align-items: baseline;
  line-height: 1.8;
}

.opt-letter {
  flex: none;
  font-weight: 600;
}

.q-answer {
  margin-top: 0.875rem;
  border-top:1px dashed var(--line);
  padding-top: 0.625rem;
}

.ans-fold {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  border: none;
  background: var(--brand-weak);
  color: var(--brand);
  padding: 0.6875rem 0.875rem;
  border-radius:0.625rem;
  font-size: 0.9062rem;
  font-weight: 700;
  cursor: pointer;
  font-family: inherit;
  text-align: left;
}

.ans-fold:hover {
  filter: brightness(0.96);
}

.ans-fold .chev {
  flex: none;
}

.fold-hint {
  margin-left: auto;
  font-size: 0.75rem;
  font-weight: 400;
}

.ans-label {
  font-size: 0.75rem;
  color: var(--muted);
  font-weight: 600;
  letter-spacing: 0.125rem;
  margin:4px;
}

.ans-line {
  font-size: 1.1875rem;
  font-weight: 700;
  background: var(--brand-weak);
  border-radius:0.75rem;
  padding: 0.625rem 0.875rem;
  text-align: center;
  margin-bottom: 0.625rem;
}

/* 选择题带补充段落：胶囊缩小为常规字号 */
.ans-line.multi {
  font-size: 0.9375rem;
  display: inline-block;
  padding: 0.375rem 0.75rem;
}

/* 非单行答案：整块浅底、字号略大，常规居左（含列表/表格等） */
.ans-multi {
  margin: 0 0 0.625rem;
  padding: 0.5rem 0.875rem;
  background: var(--brand-weak);
  border-radius: 0.75rem;
  font-size: 1.0625rem;
  line-height: 1.8;
}

.ans-multi :deep(p) {
  margin: 0 0 0.375rem;
}

.ans-multi :deep(p:last-child),
.ans-multi > :deep(*:last-child) {
  margin-bottom: 0;
}

/* 选择题的答案行 + 补充段落：补充段落同样给底色 */
.ans-rest {
  background: var(--brand-weak);
  border-radius: 0.75rem;
  padding: 0.5rem 0.875rem;
  font-size: 1.0625rem;
  line-height: 1.8;
  margin: 0 0 0.625rem;
}

.ans-rest :deep(p) {
  margin: 0 0 0.375rem;
}

.ans-line :deep(.ans-letter) {
  color: var(--brand);
}

.ans-body {
  color: var(--ink);
  line-height: 1.85;
}

.ans-notes {
  color: var(--muted);
  font-size: 0.9375rem;
}

.ans-body :deep(p) {
  margin: 0 0 0.5rem;
}

.ans-body :deep(.q-quote),
.q-stem :deep(.q-quote) {
  border-left:3px solid var(--line);
  margin: 0.375rem 0;
  padding:2px 0.625rem;
  color: var(--muted);
}
</style>
