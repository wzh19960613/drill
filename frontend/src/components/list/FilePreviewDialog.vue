<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { getSourceFile } from '../../api'
import { paraHtml } from '../../math'
import { splitBlocks } from '../../mdoc'
import { openImageViewer } from '../../composables/useImageViewer'
import DialogHeader from '../DialogHeader.vue'
import { useDialogShell } from '../../composables/useDialogShell'


const props = defineProps<{
  source: string
  path: string
}>()

const emit = defineEmits<{ (e: 'close'): void }>()

const md = ref('')
const err = ref('')
const blocks = ref<string[]>([])

function onImageClick(e: MouseEvent) {
  const img = (e.target as HTMLElement).closest?.('img.q-svg')
  if (img) openImageViewer((img as HTMLImageElement).getAttribute('src') ?? '', img)
}

onMounted(async () => {
  try {
    const r = await getSourceFile(props.source, props.path)
    md.value = r.markdown
    blocks.value = splitBlocks(r.markdown)
  } catch (e) {
    err.value = e instanceof Error ? e.message : '读取失败'
  }
})

useDialogShell((e) => {
  if (e.code === 'Escape') {
    e.stopPropagation()
    emit('close')
  }
})
</script>

<template>
  <Teleport to="body">
  <div class="modal-mask fpv-mask fs" @click.self="emit('close')">
    <div class="fpv dialog fs">
      <DialogHeader no-border :title="`预览 · ${path}`" @close="emit('close')" />
      <div class="fpv-body" @click="onImageClick">
        <div v-if="err" class="fpv-err">{{ err }}</div>
        <template v-else>
          <div v-for="(b, i) in blocks" :key="i" class="fpv-block" v-html="paraHtml(b)"></div>
          <div v-if="!blocks.length" class="fpv-err">（空文件）</div>
        </template>
      </div>
    </div>
  </div>
  </Teleport>
</template>

<style scoped>
.fpv-mask {
  z-index: var(--z-nested);
}

.fpv {
  width: min(46rem, 94vw);
  max-height: min(84vh, 48rem);
}

.fpv-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: clip;
  overscroll-behavior: contain;
  padding: 0.875rem 1.25rem 1.25rem;
  font-size: 0.9375rem;
  line-height: 1.8;
  color: var(--ink);
}

.fpv-block {
  margin-bottom: 0.375rem;
}

.fpv-err {
  color: var(--muted);
  padding: 0.5rem 0;
}
</style>
