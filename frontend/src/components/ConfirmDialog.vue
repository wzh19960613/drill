<script setup lang="ts">
import { useDialogShell } from '../composables/useDialogShell'

const props = withDefaults(
  defineProps<{
    title?: string
    confirmText?: string
    busy?: boolean
    width?: string
    
    nested?: boolean
  }>(),
  { title: '', confirmText: '删除', busy: false, width: '25rem', nested: false },
)

const emit = defineEmits<{ (e: 'confirm'): void; (e: 'cancel'): void }>()

useDialogShell(
  (e) => {
    if (!props.nested && e.code === 'Escape') {
      e.stopPropagation()
      emit('cancel')
    }
  },
  !props.nested,
)
</script>

<template>
  <Teleport to="body">
    <div class="modal-mask" :class="{ lift: nested }" @click.self="emit('cancel')">
      <div class="mini-modal" :style="{ width }">
        <h3 v-if="title">{{ title }}</h3>
        <p><slot /></p>
        <div class="m-actions">
          <button class="btn bad" :disabled="busy" @click="emit('confirm')">
            {{ confirmText }}
          </button>
          <button class="btn ghost" @click="emit('cancel')">取消</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-mask.lift {
  z-index: var(--z-nested);
}
</style>
