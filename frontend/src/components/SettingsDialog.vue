<script setup lang="ts">
import { ref } from 'vue'
import { Github, Moon, Sun, SunMoon } from 'lucide-vue-next'
import { applyZoom, loadZoom, themeLabel, themeMode, ZOOM_STEPS, type ThemeMode } from '../theme'
import { APP_VERSION, REPO_URL } from '../about'
import { currentSubject, subjects, UNKNOWN } from '../store'
import { useDialogShell } from '../composables/useDialogShell'
import DialogHeader from './DialogHeader.vue'

const emit = defineEmits<{ (e: 'close'): void }>()

const THEME_MODES: ThemeMode[] = ['auto', 'light', 'dark']
const zoom = ref(loadZoom())

function pickZoom(v: number) {
  applyZoom(v)
  zoom.value = v
}

useDialogShell((e) => {
  if (e.code === 'Escape') {
    e.stopPropagation()
    emit('close')
  }
})
</script>

<template>
  <Teleport to="body">
  <div class="modal-mask" @click.self="emit('close')">
    <div class="modal dialog pop">
      <DialogHeader title="设置" @close="emit('close')" />
      <div class="set-row">
        <span class="lbl">样式</span>
        <div class="seg">
          <button
            v-for="m in THEME_MODES"
            :key="m"
            :class="{ on: themeMode === m }"
            @click="themeMode = m"
          >
            <Sun v-if="m === 'light'" style="width: 0.9375rem; height: 0.9375rem" />
            <Moon v-else-if="m === 'dark'" style="width: 0.9375rem; height: 0.9375rem" />
            <SunMoon v-else style="width: 0.9375rem; height: 0.9375rem" />
            {{ themeLabel(m) }}
          </button>
        </div>
      </div>
      <div class="set-row">
        <span class="lbl">字号</span>
        <select class="fill" :value="zoom" @change="pickZoom(Number(($event.target as HTMLSelectElement).value))">
          <option v-for="z in ZOOM_STEPS" :key="z.label" :value="z.v">{{ z.label }}</option>
        </select>
      </div>
      <div v-if="subjects.filter((s) => s !== UNKNOWN).length >= 2" class="set-row">
        <span class="lbl">学科</span>
        <select v-model="currentSubject" class="fill">
          <option value="">[综合]</option>
          <option v-for="s in subjects" :key="s" :value="s">{{ s }}</option>
        </select>
      </div>
      <div class="set-row">
        <span class="lbl">关于</span>
        <div class="about">
          <span class="ver">Drill v{{ APP_VERSION }}</span>
          <a class="btn ghost sm" :href="REPO_URL" target="_blank" rel="noopener">
            <Github style="width: 0.875rem; height: 0.875rem" /> GitHub
          </a>
        </div>
      </div>
    </div>
  </div>
  </Teleport>
</template>

<style scoped>
.modal {
  width: min(28.75rem, 94vw);
  padding: 0 1.25rem 1.125rem;
}

.modal :deep(.dlg-head) {
  margin: 0 -1.25rem 1.5rem;
  padding-left: 1.25rem;
  padding-right: 0.75rem;
}

.set-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 0.75rem;
}

.lbl {
  flex: none;
  width: 2.75rem;
  font-size: 0.8438rem;
  color: var(--muted);
  font-weight: 600;
}

.set-row .seg {
  flex: 1;
}

.about {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
}

.about .btn {
  text-decoration: none;
}

.ver {
  color: var(--muted);
  font-size: 0.8438rem;
}
</style>
