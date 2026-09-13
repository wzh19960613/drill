<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { Save } from 'lucide-vue-next'
import { getSourceFile, putSourceFile, type ClassifyInfo } from '../../api'
import MdSectionEditor from '../editor/MdSectionEditor.vue'
import DialogHeader from '../DialogHeader.vue'
import { useDialogShell } from '../../composables/useDialogShell'


const props = defineProps<{
  source: string
  path: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'saved', info: ClassifyInfo): void
}>()

const md = ref('')
const loaded = ref(false)
const busy = ref(false)
const errMsg = ref('')
const result = ref<ClassifyInfo | null>(null)

onMounted(async () => {
  try {
    const r = await getSourceFile(props.source, props.path)
    md.value = r.markdown
  } catch (e) {
    errMsg.value = e instanceof Error ? e.message : '读取失败'
  } finally {
    loaded.value = true
  }
})

async function save() {
  busy.value = true
  errMsg.value = ''
  result.value = null
  try {
    const info = await putSourceFile(props.source, props.path, md.value)
    result.value = info
    emit('saved', info)
  } catch (e) {
    errMsg.value = e instanceof Error ? e.message : '保存失败'
  } finally {
    busy.value = false
  }
}

useDialogShell((e) => {
  if (e.code === 'Escape') {
    e.stopPropagation()
    emit('close')
  } else if ((e.metaKey || e.ctrlKey) && e.code === 'KeyS') {
    e.preventDefault()
    if (loaded.value && !busy.value) void save()
  }
})
</script>

<template>
  <Teleport to="body">
  <div class="modal-mask fed-mask fs" @click.self="emit('close')">
    <div class="fed dialog fs">
      <DialogHeader no-border :title="`编辑文件 · ${path}`" @close="emit('close')" />
      <div class="fed-body">
        <div v-if="errMsg && !loaded" class="fed-msg err">{{ errMsg }}</div>
        <MdSectionEditor
          v-else
          v-model="md"
          :clamp="false"
          :allow-image="false"
          min-height="16rem"
          placeholder="Markdown 内容（标题层级不限）…"
        />
        <div v-if="result" class="fed-result" :class="{ ok: result.question }">
          <template v-if="result.question">✓ 已识别为题目，文件重新进入题库</template>
          <template v-else>仍不是题目：{{ result.reason ?? '原因未知' }}</template>
        </div>
      </div>
      <div class="fed-foot">
        <span class="fed-hint">保存后自动尝试识别为题目；⌘S 保存</span>
        <span v-if="errMsg && loaded" class="fed-msg err">{{ errMsg }}</span>
        <span class="flex"></span>
        <button class="btn ghost" :disabled="busy" @click="emit('close')">关闭</button>
        <button class="btn primary" :disabled="!loaded || busy" @click="save">
          <Save style="width: 0.9375rem; height: 0.9375rem" /> 保存
        </button>
      </div>
    </div>
  </div>
  </Teleport>
</template>

<style scoped>
.fed-mask {
  z-index: var(--z-nested);
}

.fed {
  width: min(56rem, 94vw);
  max-height: min(88vh, 52rem);
}

.fed-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overscroll-behavior: contain;
  padding: 0.75rem 1.25rem;
  display: flex;
  flex-direction: column;
  gap: 0.625rem;
}

.fed-foot {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.625rem 1.25rem;
  border-top: 1px solid var(--line);
}

.fed-hint {
  color: var(--muted);
  font-size: 0.7812rem;
}

.flex {
  flex: 1;
}

.fed-msg.err {
  color: var(--bad);
  font-size: 0.8438rem;
}

.fed-result {
  font-size: 0.8438rem;
  color: var(--warn);
}

.fed-result.ok {
  color: var(--good);
}
</style>
