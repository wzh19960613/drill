<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { Save, Trash2 } from 'lucide-vue-next'
import type { Question } from '../types'
import type { SourceInfo } from '../api'
import { discardTemp, fetchQuestionRaw } from '../api'
import { EMPTY_META, docDefects, extractImageRefs, splitDoc, type DocMeta } from '../mdoc'
import { currentSubject, store } from '../store'
import DialogHeader from './DialogHeader.vue'
import QuestionDeleteDialog from './QuestionDeleteDialog.vue'
import MdSectionEditor from './editor/MdSectionEditor.vue'
import SaveQuestionDialog from './editor/SaveQuestionDialog.vue'
import { useDialogShell } from '../composables/useDialogShell'

const props = defineProps<{
  question: Question | null
  sources: SourceInfo[]
  createIn?: { source: string; dir: string } | null
}>()

const emit = defineEmits<{ (e: 'close'): void; (e: 'saved'): void }>()

const meta = reactive<DocMeta>({ ...EMPTY_META })
const sections = reactive({ stem: '', answer: '', solution: '', notes: '' })
const temps = ref(new Map<string, string>())
const tempsSet = computed(() => new Set(temps.value.keys()))

const loaded = ref(false)
const loadErr = ref('')
const original = ref<{ source: string; dir: string; file: string } | null>(null)
const saveOpen = ref(false)

const sourceName = computed(() => {
  const id = original.value?.source ?? props.question?.source
  return props.sources.find((s) => s.id === id)?.name ?? id ?? ''
})

const defects = computed(() => docDefects(sections))
const canSave = computed(() => loaded.value && !defects.value.stemBlank && !defects.value.noAnswer)


function uniq(vals: (string | undefined)[], cap = 300): string[] {
  const seen = new Set<string>()
  const out: string[] = []
  for (const v of vals) {
    if (v && !seen.has(v)) {
      seen.add(v)
      out.push(v)
      if (out.length >= cap) break
    }
  }
  return out
}
const dlSubject = computed(() => uniq(store.questions.map((q) => q.subject)))
const dlOrigin = computed(() => uniq(store.questions.map((q) => q.origin)))
const dlChapter = computed(() => uniq(store.questions.map((q) => q.chapter)))
const dlQtype = computed(() => uniq(store.questions.map((q) => q.qtype)))
const dlLocate = computed(() => uniq(store.questions.map((q) => q.locate || q.id)))

const delConfirm = ref(false)

function onDeleted() {
  delConfirm.value = false
  emit('saved')
}

async function loadInitial() {
  if (props.question) {
    try {
      const r = await fetchQuestionRaw(props.question.id, props.question.source)
      const doc = splitDoc(r.markdown)
      Object.assign(meta, doc.meta)
      Object.assign(sections, doc.sections)
      original.value = { source: r.source, dir: r.dir, file: r.file }
    } catch (e) {
      loadErr.value = e instanceof Error ? e.message : '读取原文失败'
    }
  } else {
    meta.subject = currentSubject.value && currentSubject.value !== '未知' ? currentSubject.value : ''
    sections.stem = '题干（　）。\n\n(A) \n\n(B) \n\n(C) \n\n(D) '
    original.value = null
  }
  loaded.value = true
}

function tempAdded(id: string, name: string) {
  temps.value.set(id, name)
}


let cleanupTimer: ReturnType<typeof setTimeout> | undefined
watch(
  () => [sections.stem, sections.answer, sections.solution, sections.notes],
  () => {
    if (!temps.value.size) return
    clearTimeout(cleanupTimer)
    cleanupTimer = setTimeout(() => {
      const all = [sections.stem, sections.answer, sections.solution, sections.notes].join('\n')
      const refs = new Set(extractImageRefs(all))
      for (const id of [...temps.value.keys()]) {
        if (!refs.has(id)) {
          temps.value.delete(id)
          void discardTemp(id).catch(() => {})
        }
      }
    }, 400)
  },
)

async function discardAllTemps() {
  for (const id of [...temps.value.keys()]) {
    temps.value.delete(id)
    void discardTemp(id).catch(() => {})
  }
}

function close() {
  void discardAllTemps()
  emit('close')
}

function onSaved() {
  temps.value.clear()
  emit('saved')
}

useDialogShell((e) => {
  if (saveOpen.value) return
  
  if (delConfirm.value) return
  if (e.code === 'Escape') {
    e.stopPropagation()
    close()
  } else if ((e.metaKey || e.ctrlKey) && e.code === 'KeyS') {
    e.preventDefault()
    if (canSave.value) saveOpen.value = true
  }
})

onMounted(() => void loadInitial())
</script>

<template>
  <Teleport to="body">
  <div class="modal-mask ed-mask fs">
    <div class="ed dialog fs">
      <DialogHeader flush @close="close">
        <template #title>
          {{ question ? `编辑题目 ${question.id}` : '新增题目'
          }}<span v-if="sourceName" class="src">（{{ sourceName }}）</span>
        </template>
        <template #actions>
          <button v-if="question" class="btn ghost sm danger" @click="delConfirm = true">
            <Trash2 style="width: 0.875rem; height: 0.875rem" /> 删除
          </button>
        </template>
      </DialogHeader>

      <div class="ed-body">
        <div v-if="loadErr" class="ed-err">{{ loadErr }}</div>
        <template v-else>
          <div class="ed-meta">
            <label class="ed-field">
              <span>学科</span>
              <input v-model="meta.subject" type="text" list="qe-dl-subject" placeholder="可空" />
              <datalist id="qe-dl-subject">
                <option v-for="v in dlSubject" :key="v" :value="v" />
              </datalist>
            </label>
            <label class="ed-field">
              <span>来源</span>
              <input v-model="meta.origin" type="text" list="qe-dl-origin" placeholder="可空" />
              <datalist id="qe-dl-origin">
                <option v-for="v in dlOrigin" :key="v" :value="v" />
              </datalist>
            </label>
            <label class="ed-field">
              <span>章节</span>
              <input v-model="meta.chapter" type="text" list="qe-dl-chapter" placeholder="可空" />
              <datalist id="qe-dl-chapter">
                <option v-for="v in dlChapter" :key="v" :value="v" />
              </datalist>
            </label>
            <label class="ed-field">
              <span>定位 <em>（默认作文件名）</em></span>
              <input v-model="meta.locate" type="text" list="qe-dl-locate" placeholder="可空" />
              <datalist id="qe-dl-locate">
                <option v-for="v in dlLocate" :key="v" :value="v" />
              </datalist>
            </label>
            <label class="ed-field">
              <span>题型</span>
              <input v-model="meta.qtype" type="text" list="qe-dl-qtype" placeholder="可空" />
              <datalist id="qe-dl-qtype">
                <option v-for="v in dlQtype" :key="v" :value="v" />
              </datalist>
            </label>
          </div>

          <section class="ed-sec">
            <header class="ed-sec-h">
              <b>题目</b><i v-if="defects.stemBlank" class="warn">不能为空</i>
            </header>
            <MdSectionEditor
              v-model="sections.stem"
              :temps="tempsSet"
              allow-options
              min-height="9rem"
              placeholder="题干…（选项用 (A) (B) 形式，每行一个）"
              @temp-added="tempAdded"
            />
          </section>

          <section class="ed-sec">
            <header class="ed-sec-h">
              <b>答案</b><i v-if="defects.noAnswer" class="warn">答案与解析至少填一项</i>
            </header>
            <MdSectionEditor
              v-model="sections.answer"
              :temps="tempsSet"
              min-height="4.5rem"
              placeholder="答案…（可空；答案和解析至少填一项）"
              @temp-added="tempAdded"
            />
          </section>

          <section class="ed-sec">
            <header class="ed-sec-h">
              <b>解析</b><i v-if="defects.noAnswer" class="warn">答案与解析至少填一项</i>
            </header>
            <MdSectionEditor
              v-model="sections.solution"
              :temps="tempsSet"
              min-height="4.5rem"
              placeholder="解析…（可空）"
              @temp-added="tempAdded"
            />
          </section>

          <section class="ed-sec">
            <header class="ed-sec-h"><b>备注</b></header>
            <MdSectionEditor
              v-model="sections.notes"
              :temps="tempsSet"
              min-height="4.5rem"
              placeholder="备注…（可空）"
              @temp-added="tempAdded"
            />
          </section>
        </template>
      </div>

      <div class="ed-foot">
        <span v-if="defects.stemBlank" class="ed-msg err">题目不能留空</span>
        <span v-else-if="defects.noAnswer" class="ed-msg err">请至少填写答案或解析</span>
        <span v-else class="ed-hint">标题最高三级；图片可上传、粘贴或拖入；⌘S 保存</span>
        <span class="flex"></span>
        <button class="btn ghost" @click="close">取消</button>
        <button class="btn primary" :disabled="!canSave" @click="saveOpen = true">
          <Save style="width: 0.9375rem; height: 0.9375rem" /> 保存
        </button>
      </div>

      <SaveQuestionDialog
        v-if="saveOpen"
        :meta="meta"
        :sections="sections"
        :temps="temps"
        :sources="sources"
        :original="original"
        :initial-target="question ? null : (createIn ?? null)"
        @close="saveOpen = false"
        @saved="onSaved"
      />

<QuestionDeleteDialog
  v-if="delConfirm && question"
  :question="question"
  @close="delConfirm = false"
  @deleted="onDeleted"
/>
    </div>
  </div>
  </Teleport>
</template>

<style scoped>
.ed-mask {
  z-index: var(--z-detail);
  background: var(--scrim-nested);
}

.ed {
  position: relative;
  width: min(62rem, 100%);
  height: min(88vh, 60rem);
}

.ed-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: clip;
  overscroll-behavior: contain;
  display: flex;
  flex-direction: column;
  gap: 0.875rem;
  padding: 0.875rem 1.25rem 1.25rem;
}

.ed-err {
  color: var(--bad);
  padding: 1.25rem;
}

.ed-meta {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(10.5rem, 1fr));
  gap: 0.625rem;
}

.ed-field {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  min-width: 0;
}

.ed-field > span {
  font-size: 0.7812rem;
  color: var(--muted);
}

.ed-field > span em {
  font-style: normal;
  opacity: 0.8;
}

.ed-field input {
  width: 100%;
}

.ed-sec {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.ed-sec-h {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
}

.ed-sec-h b {
  font-size: 0.875rem;
}

.ed-sec-h .warn {
  font-style: normal;
  font-size: 0.7812rem;
  color: var(--warn);
}

.ed-foot {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.625rem 1.125rem;
  border-top: 1px solid var(--line);
}

.flex {
  flex: 1;
}

.ed-msg.err {
  color: var(--bad);
  font-size: 0.8125rem;
}

.ed-hint {
  color: var(--muted);
  font-size: 0.7812rem;
}
</style>
