<script setup lang="ts">
import type { BookDef, PausedSession } from '../../types'
import ConfirmDialog from '../ConfirmDialog.vue'

defineProps<{
  asking: BookDef | null
  switchAsk: { def: BookDef; review: boolean } | null
  activeName: string
  resumeAsk: boolean
  paused: PausedSession | null
  confirmDel: BookDef | null
  favAsk: { id: string; name: string } | null
}>()

const emit = defineEmits<{
  (e: 'accept-current', start: boolean): void
  (e: 'decline-current'): void
  (e: 'confirm-switch'): void
  (e: 'switch-without'): void
  (e: 'cancel-switch'): void
  (e: 'resume'): void
  (e: 'discard-pause'): void
  (e: 'close-resume'): void
  (e: 'confirm-delete'): void
  (e: 'cancel-delete'): void
  (e: 'accept-favorite'): void
  (e: 'cancel-favorite'): void
}>()
</script>

<template>
  <Teleport to="body">
  <div v-if="asking" class="modal-mask" @click.self="emit('decline-current')">
    <div class="mini-modal ask-box">
      <h3>生成完成</h3>
      <p>
        题本「<b>{{ asking.name }}</b>」（{{ asking.items.length }} 题）已生成。<br />
        是否将其设为<b>当前题本</b>？
      </p>
      <div class="m-actions">
        <button class="btn primary" @click="emit('accept-current', true)">是，并开始刷题</button>
        <button class="btn" @click="emit('accept-current', false)">是</button>
        <button class="btn ghost" @click="emit('decline-current')">否</button>
      </div>
    </div>
  </div>

  <!-- View-dialog action on a non-active book: ask whether to switch (the active book is remembered per subject) -->
  <div v-if="switchAsk" class="modal-mask" @click.self="emit('cancel-switch')">
    <div class="mini-modal ask-box">
      <h3>切换当前题本</h3>
      <p>
        当前题本为「<b>{{ activeName }}</b>」，是否切换成「<b>{{ switchAsk.def.name }}</b>」
        {{ switchAsk.review ? '并对答案' : '并开始刷题' }}？
      </p>
      <div class="m-actions col">
        <button class="btn primary" @click="emit('confirm-switch')">
          切换并{{ switchAsk.review ? '对答案' : '开始刷题' }}
        </button>
        <button v-if="switchAsk.review" class="btn" @click="emit('switch-without')">不切换，开始对答案</button>
        <button class="btn ghost" @click="emit('cancel-switch')">取消</button>
      </div>
    </div>
  </div>

  <div v-if="resumeAsk && paused" class="modal-mask" @click.self="emit('close-resume')">
    <div class="mini-modal resume-ask">
      <h3>继续上次刷题？</h3>
      <p>
        有上次中断的刷题「<b>{{ paused.title }}</b>」：<br />
        已做 {{ paused.results.length }} / {{ paused.items.length }} 题。
      </p>
      <div class="m-actions">
        <button class="btn primary" @click="emit('resume')">继续上次</button>
        <button class="btn" @click="emit('discard-pause')">放弃并开始新的</button>
        <button class="btn ghost" @click="emit('close-resume')">取消</button>
      </div>
    </div>
  </div>

  <ConfirmDialog
    v-if="confirmDel"
    title="删除题本"
    width="23.75rem"
    @confirm="emit('confirm-delete')"
    @cancel="emit('cancel-delete')"
  >
    确定删除「<b>{{ confirmDel.name }}</b>」（{{ confirmDel.items.length }} 题）？<br />
    删除后不可恢复，已导出的文件不受影响。
  </ConfirmDialog>

  <div v-if="favAsk" class="modal-mask" @click.self="emit('cancel-favorite')">
    <div class="mini-modal ask-box">
      <h3>设为收藏题本</h3>
      <p>
        空白题本「<b>{{ favAsk.name }}</b>」已新建。<br />
        是否将其设为<b>收藏题本</b>？（刷题时点「收藏」即可加入）
      </p>
      <div class="m-actions">
        <button class="btn primary" @click="emit('accept-favorite')">是</button>
        <button class="btn ghost" @click="emit('cancel-favorite')">否</button>
      </div>
    </div>
  </div>
  </Teleport>
</template>

<style scoped>
.ask-box {
  width: 26.25rem;
}

.resume-ask {
  width: 23.75rem;
}

.m-actions.col {
  flex-direction: column;
  align-items: stretch;
}
</style>
