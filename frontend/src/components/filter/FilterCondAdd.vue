<script setup lang="ts">
import { computed, ref } from 'vue'
import { Check } from 'lucide-vue-next'
import { chapters, currentSubject, origins, qtypes, subjects, UNKNOWN } from '../../store'
import { type Cond, type Op } from '../../filter'

const emit = defineEmits<{
  (e: 'add', cond: Omit<Cond, 'id'>): void
  (e: 'clear'): void
  (e: 'close'): void
}>()

const kind = ref<Cond['kind']>('chapter')
const what = ref<'attempts' | 'wrong'>('attempts')
const op = ref<Op>('>=')
const n = ref(1)
const k = ref(1)
const m = ref(1)
const selected = ref<string[]>([])

const KIND_LABELS: { key: Cond['kind']; label: string; param: boolean }[] = [
  { key: 'subject', label: '学科', param: true },
  { key: 'origin', label: '来源', param: true },
  { key: 'chapter', label: '章节', param: true },
  { key: 'qtype', label: '题型', param: true },
  { key: 'count', label: '次数', param: true },
  { key: 'recentWrong', label: '最近做错', param: true },
]

const kindChoices = computed(() =>
  KIND_LABELS.filter(
    (k) =>
      !(
        k.key === 'subject' &&
        (currentSubject.value || subjects.value.filter((s) => s !== UNKNOWN).length < 2)
      ),
  ),
)

const PARAM_OPTIONS: Partial<Record<Cond['kind'], () => string[]>> = {
  subject: () => subjects.value,
  origin: () => origins.value,
  chapter: () => chapters.value,
  qtype: () => qtypes.value,
}

const isParamKind = computed(() => !!PARAM_OPTIONS[kind.value])
const options = computed(() => PARAM_OPTIONS[kind.value]?.() ?? [])

function pickKind(kd: Cond['kind']) {
  kind.value = kd
  selected.value = []
  const item = KIND_LABELS.find((x) => x.key === kd)
  if (item && !item.param) addNow()
}

function toggleOption(v: string) {
  const s = new Set(selected.value)
  if (s.has(v)) s.delete(v)
  else s.add(v)
  selected.value = [...s]
}

function addNow() {
  let cond: Omit<Cond, 'id'>
  switch (kind.value) {
    case 'subject':
    case 'origin':
    case 'chapter':
    case 'qtype':
      if (!selected.value.length) return
      cond = { kind: kind.value, values: selected.value }
      break
    case 'count':
      cond = { kind: 'count', what: what.value, op: op.value, n: Math.max(0, n.value || 0) }
      break
    case 'recentWrong':
      cond = { kind: 'recentWrong', n: Math.max(1, k.value || 1), m: Math.max(1, m.value || 1) }
      break
  }
  emit('add', cond)
  emit('close')
}
</script>

<template>
  <div class="fb-add">
    <div class="fb-kinds">
      <button
        v-for="kd in kindChoices"
        :key="kd.key"
        class="chip-btn"
        :class="{ on: kind === kd.key }"
        @click="pickKind(kd.key)"
      >{{ kd.label }}</button>
    </div>

    <div v-if="isParamKind" class="fb-opt-list">
      <label v-for="v in options" :key="v" class="fb-opt">
        <input
          type="checkbox"
          :checked="selected.includes(v)"
          @change="toggleOption(v)"
        />
        <span>{{ v }}</span>
      </label>
      <div v-if="!options.length" class="fb-hint">无可选项</div>
    </div>

    <div v-else-if="kind === 'count'" class="fb-params">
      <select v-model="what">
        <option value="attempts">做过</option>
        <option value="wrong">做错</option>
      </select>
      <select v-model="op">
        <option value=">=">⩾</option>
        <option value=">">&gt;</option>
        <option value="<=">≤</option>
        <option value="<">&lt;</option>
        <option value="=">=</option>
      </select>
      <input type="number" min="0" v-model.number="n" /> 次
    </div>

    <div v-else-if="kind === 'recentWrong'" class="fb-params">
      最近 <input type="number" min="1" v-model.number="k" /> 次内错 ⩾
      <input type="number" min="1" v-model.number="m" /> 次
    </div>

    <div class="fb-add-actions">
      <button
        v-if="KIND_LABELS.find((x) => x.key === kind)?.param"
        class="btn primary sm"
        :disabled="isParamKind && !selected.length"
        @click="addNow"
      >
        <Check style="width: 0.9375rem; height: 0.9375rem" /> 添加
      </button>
      <button class="btn ghost sm" @click="emit('close')">取消</button>
      <button class="btn ghost sm" style="margin:auto" @click="emit('clear')">清空全部条件</button>
    </div>
  </div>
</template>

<style scoped>
.fb-add {
  border: 1px dashed var(--line);
  border-radius:0.75rem;
  padding: 0.875rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  background: var(--panel);
}

.fb-kinds {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.fb-opt-list {
  max-height: 13.75rem;
  overflow-y: auto;
  overflow-x: clip;
  overscroll-behavior: contain;
  border: 1px solid var(--line);
  border-radius:0.625rem;
  background: var(--card);
  padding: 0.375rem;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(13.75rem, 1fr));
  gap:2px;
}

.fb-opt {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 0.5625rem 0.625rem;
  font-size: 0.875rem;
  border-radius:0.5rem;
  cursor: pointer;
  white-space: nowrap;
  overflow: hidden;
}

/* the label's own ellipsis never applies to the inner span's text — put
   the truncation on the span so long chapter names show an ellipsis */
.fb-opt span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.fb-opt:hover {
  background: var(--hover);
}

.fb-params {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  font-size: 0.9062rem;
  color: var(--muted);
  flex-wrap: wrap;
}

/* Operator dropdowns and number inputs unified at 44px, same height as the
   global dropdowns / primary buttons */
.fb-params select,
.fb-params input {
  height: 2.75rem;
  min-height: 2.75rem;
  padding: 0.375rem 0.625rem;
}

.fb-params select {
  min-width: 4.75rem;
}

.fb-add-actions {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.fb-hint {
  padding: 1rem;
  color: var(--muted);
  font-size: 0.8438rem;
}
</style>
