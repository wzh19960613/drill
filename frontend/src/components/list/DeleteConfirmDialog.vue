<script setup lang="ts">
import { ref } from 'vue'
import { deleteQuestionApi } from '../../api'
import { selection, store } from '../../store'
import ConfirmDialog from '../ConfirmDialog.vue'

const emit = defineEmits<{ (e: 'cancel'): void; (e: 'done'): void }>()

const deleting = ref(false)

async function deleteSelected() {
  if (!selection.ids.length) return
  deleting.value = true
  try {
    const ids = new Set(selection.ids)
    for (const q of store.questions) {
      if (ids.has(q.id)) await deleteQuestionApi(q.id, q.source)
    }
    selection.ids = []
    emit('done')
  } finally {
    deleting.value = false
  }
}
</script>

<template>
  <ConfirmDialog
    title="删除题目"
    :busy="deleting"
    width="25rem"
    @confirm="deleteSelected"
    @cancel="emit('cancel')"
  >
    确定删除选中的 <b>{{ selection.ids.length }}</b> 个题目文件？<br />
    文件会移除（保留 .bak 备份），记录统计将不再包含它们。
  </ConfirmDialog>
</template>
