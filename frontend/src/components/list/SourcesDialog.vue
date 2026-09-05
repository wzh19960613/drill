<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { FolderInput, Trash2 } from 'lucide-vue-next'
import {
  addSource as addSourceApi,
  fetchSources,
  relocateSource as relocateSourceApi,
  removeSource as removeSourceApi,
  type SourceInfo,
} from '../../api'
import { lockBodyScroll, unlockBodyScroll } from '../../scrollLock'
import DialogHeader from '../DialogHeader.vue'

const emit = defineEmits<{ (e: 'close'): void; (e: 'changed'): void }>()

const sources = ref<SourceInfo[]>([])
const newDir = ref('')
const srcMsg = ref('')
const relocating = ref<SourceInfo | null>(null)
const relocatePath = ref('')

async function refresh() {
  try {
    sources.value = await fetchSources()
  } catch {
  }
}

async function reloadEverywhere() {
  await refresh()
  emit('changed')
}

async function addSource() {
  srcMsg.value = ''
  const dir = newDir.value.trim().replace(/\/+$/, '')
  if (!dir) return
  try {
    await addSourceApi(dir)
    newDir.value = ''
    await reloadEverywhere()
    srcMsg.value = `已添加来源：${dir}`
  } catch (e) {
    srcMsg.value = e instanceof Error ? e.message : '添加失败'
  }
}

async function removeSource(s: SourceInfo) {
  srcMsg.value = ''
  try {
    await removeSourceApi(s.id)
    await reloadEverywhere()
  } catch (e) {
    srcMsg.value = e instanceof Error ? e.message : '移除失败（至少保留一个）'
  }
}

function startRelocate(s: SourceInfo) {
  srcMsg.value = ''
  relocating.value = s
  relocatePath.value = s.path
}

async function confirmRelocate() {
  const s = relocating.value
  if (!s) return
  const dir = relocatePath.value.trim().replace(/\/+$/, '')
  if (!dir) return
  try {
    await relocateSourceApi(s.id, dir)
    relocating.value = null
    await reloadEverywhere()
    srcMsg.value = `已重定位：${dir}`
  } catch (e) {
    srcMsg.value = e instanceof Error ? e.message : '重定位失败'
  }
}

onMounted(async () => {
  lockBodyScroll()
  await refresh()
})
onBeforeUnmount(() => unlockBodyScroll())
</script>

<template>
  <Teleport to="body">
  <div class="modal-mask" @click.self="emit('close')">
    <div class="src-pop dialog">
      <DialogHeader no-border title="题源 · 题目来源文件夹" @close="emit('close')" />
      <div class="src-body">
      <div v-for="s in sources" :key="s.id" class="src-row">
        <div class="src-info">
          <span class="src-name">{{ s.name }}<span v-if="!s.exists" class="src-miss"> · 文件夹不存在，请重定位</span></span>
          <span class="src-dir">{{ s.path }}</span>
        </div>
        <span class="chip">{{ s.count }} 题</span>
        <button class="icon-btn" aria-label="重定位该来源（文件夹移动后使用）" @click="startRelocate(s)">
          <FolderInput style="width: 0.875rem; height: 0.875rem" />
        </button>
        <button class="icon-btn src-x" aria-label="移除该来源" @click="removeSource(s)">
          <Trash2 style="width: 0.875rem; height: 0.875rem" />
        </button>
      </div>
      <div v-if="relocating" class="src-add src-relocate">
        <input
          v-model="relocatePath"
          type="text"
          :placeholder="`「${relocating.name}」的新路径`"
          @keyup.enter="confirmRelocate"
        />
        <button class="btn primary sm" @click="confirmRelocate">重定位</button>
        <button class="btn ghost sm" @click="relocating = null">取消</button>
      </div>
      <div class="src-add src-new">
        <input
          v-model="newDir"
          type="text"
          placeholder="题目文件夹路径（绝对或仓库内相对路径）"
          @keyup.enter="addSource"
        />
        <button class="btn primary sm" @click="addSource">添加</button>
      </div>
      <div v-if="srcMsg" class="src-msg">{{ srcMsg }}</div>
      <div class="src-tip">来源以绝对路径保存在服务端；移除来源只是不再读取该文件夹，不会删除任何文件。文件夹移动后可用「重定位」指回新位置。</div>
      </div>
    </div>
  </div>
  </Teleport>
</template>

<style scoped>
.src-pop {
  width: min(37.5rem, 94vw);
  max-height: 86vh;
}

.src-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: clip;
  overscroll-behavior: contain;
  padding: 0.5rem 1.25rem 1.125rem;
  display: flex;
  flex-direction: column;
  gap: 0.625rem;
}

.src-row {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding:0.375rem 2px;
  border-bottom:1px dashed var(--line);
}

.src-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap:2px;
}

.src-name {
  font-size: 0.875rem;
  font-weight: 600;
}

.src-miss {
  color: var(--warn);
  font-weight: 400;
  font-size: 0.8125rem;
}

.src-dir {
  font-size: 0.8125rem;
  color: var(--muted);
  font-family: var(--font-mono);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.src-x {
  width: 1.875rem;
  height: 1.875rem;
}

.src-x:hover {
  color: var(--bad);
}

.src-add {
  display: flex;
  gap: 0.5rem;
}

.src-add input {
  flex: 1;
}

.src-msg {
  font-size: 0.8125rem;
  color: var(--good);
}

.src-tip {
  font-size: 0.7812rem;
  color: var(--muted);
  line-height: 1.6;
}
</style>
