<script setup lang="ts">
import { computed, ref } from 'vue'
import { FolderInput, FolderPlus } from 'lucide-vue-next'
import { addSource, updateSource, type SourceInfo } from '../../api'
import DialogHeader from '../DialogHeader.vue'
import { useDialogShell } from '../../composables/useDialogShell'


const props = defineProps<{
  mode: 'add' | 'relocate'
  
  source?: SourceInfo
}>()

const emit = defineEmits<{ (e: 'close'): void; (e: 'done'): void }>()

const path = ref(props.source?.path ?? '')
const busy = ref(false)
const errMsg = ref('')


const isLocal = computed(() =>
  ['localhost', '127.0.0.1', '::1', '[::1]'].includes(location.hostname),
)

const title = props.mode === 'add' ? '添加题源' : `重定位「${props.source?.name ?? ''}」`

async function submit() {
  const p = path.value.trim().replace(/\/+$/, '')
  if (!p || busy.value) return
  busy.value = true
  errMsg.value = ''
  try {
    if (props.mode === 'add') await addSource(p)
    else await updateSource(props.source!.id, { path: p })
    emit('done')
  } catch (e) {
    errMsg.value = e instanceof Error ? e.message : '操作失败'
  } finally {
    busy.value = false
  }
}

useDialogShell((e) => {
  if (e.code === 'Escape') {
    e.stopPropagation()
    emit('close')
  } else if (e.code === 'Enter' && !busy.value) {
    e.preventDefault()
    void submit()
  }
})
</script>

<template>
  <Teleport to="body">
  <div class="modal-mask spd-mask fs" @click.self="emit('close')">
    <div class="spd dialog fs">
      <DialogHeader no-border :title="title" @close="emit('close')" />
      <div class="spd-body">
        <p class="spd-note">
          <template v-if="isLocal">服务运行在本机，填写本机的文件夹路径（绝对路径，或后端仓库内相对路径）。</template>
          <template v-else>
            注意：这里填写的是<b>服务器上</b>的文件夹路径（绝对路径，或后端仓库内相对路径），可能并不是你当前正在使用的设备上的路径。
          </template>
        </p>
        <input
          v-model="path"
          type="text"
          placeholder="文件夹路径"
          @keyup.enter="submit"
        />
        <p v-if="errMsg" class="spd-err">{{ errMsg }}</p>
      </div>
      <div class="spd-foot">
        <span class="flex"></span>
        <button class="btn ghost" :disabled="busy" @click="emit('close')">取消</button>
        <button class="btn primary" :disabled="!path.trim() || busy" @click="submit">
          <FolderPlus v-if="mode === 'add'" style="width: 0.9375rem; height: 0.9375rem" />
          <FolderInput v-else style="width: 0.9375rem; height: 0.9375rem" />
          {{ mode === 'add' ? '添加' : '重定位' }}
        </button>
      </div>
    </div>
  </div>
  </Teleport>
</template>

<style scoped>
.spd-mask {
  z-index: var(--z-nested);
}

.spd {
  width: min(30rem, 94vw);
}

.spd-body {
  padding: 0.75rem 1.25rem 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.spd-note {
  margin: 0;
  font-size: 0.8438rem;
  color: var(--muted);
  line-height: 1.7;
}

.spd-err {
  margin: 0;
  font-size: 0.8438rem;
  color: var(--bad);
}

.spd-foot {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.625rem 1.25rem;
  border-top: 1px solid var(--line);
}

.flex {
  flex: 1;
}
</style>
