<script setup lang="ts">
import { ref } from 'vue'
import DialogHeader from './DialogHeader.vue'
import { useDialogShell } from '../composables/useDialogShell'

import {
  HOTKEY_ACTIONS,
  defaultHotkeys,
  fmtKey,
  keysOf,
  loadHotkeys,
  resetHotkeys,
  saveHotkeys,
  type HotkeyAction,
  type HotkeyMap,
} from '../hotkeys'

const emit = defineEmits<{ (e: 'close'): void }>()

const map = ref<HotkeyMap>(loadHotkeys())
const capturing = ref<{ action: HotkeyAction; slot: 0 | 1 } | null>(null)
const note = ref('')

function startCapture(action: HotkeyAction, slot: 0 | 1) {
  capturing.value = { action, slot }
  note.value = 'Esc 取消，⌫ 清除'
}

function slotKey(action: HotkeyAction, slot: 0 | 1): string {
  return keysOf(map.value[action])[slot] ?? ''
}

function setSlot(action: HotkeyAction, slot: 0 | 1, code: string) {
  const keys = keysOf(map.value[action])
  const old = keys[slot] ?? ''
  keys[slot] = code
  const seen = new Set<string>()
  for (let i = 0; i < keys.length; i++) {
    const k = keys[i]
    if (!k || seen.has(k)) keys[i] = ''
    else seen.add(k)
  }
  map.value[action] = keys.filter(Boolean).join('|')
  if (code) {
    for (const other of Object.keys(map.value) as HotkeyAction[]) {
      if (other === action) continue
      const okeys = keysOf(map.value[other])
      const idx = okeys.indexOf(code)
      if (idx >= 0) {
        okeys[idx] = old
        map.value[other] = okeys.filter(Boolean).join('|')
        note.value = `「${HOTKEY_ACTIONS.find((x) => x.action === other)?.label}」的该键已换绑`
        break
      }
    }
  }
}

function onKey(e: KeyboardEvent) {
  if (!capturing.value) {
    if (e.code === 'Escape') {
      e.stopPropagation()
      emit('close')
    }
    return
  }
  e.preventDefault()
  e.stopPropagation()
  const { action, slot } = capturing.value
  if (e.code === 'Escape') {
    capturing.value = null
    return
  }
  if (e.code === 'Backspace') {
    setSlot(action, slot, '')
    note.value = '已清除该键位'
  } else {
    setSlot(action, slot, e.code)
    note.value = note.value || `已绑定为 ${fmtKey(e.code)}`
  }
  saveHotkeys(map.value)
  capturing.value = null
}

useDialogShell(onKey)

function reset() {
  resetHotkeys()
  map.value = defaultHotkeys()
  note.value = '已恢复默认快捷键'
}
</script>

<template>
  <Teleport to="body">
  <div class="modal-mask" @click.self="emit('close')">
    <div class="modal dialog">
      <DialogHeader title="快捷键设置" @close="emit('close')" />

      <div class="m-list">
        <div v-for="a in HOTKEY_ACTIONS" :key="a.action" class="hk-row">
          <span class="hk-label">{{ a.label }}</span>
          <div class="hk-keys">
            <button
              v-for="slot in [0, 1] as const"
              :key="slot"
              class="hk-key"
              :class="{
                capturing: capturing?.action === a.action && capturing.slot === slot,
                unset: !slotKey(a.action, slot),
              }"
              @click="startCapture(a.action, slot)"
            >
              {{ capturing?.action === a.action && capturing.slot === slot ? '按新键…' : fmtKey(slotKey(a.action, slot)) }}
            </button>
          </div>
        </div>
      </div>

      <div class="m-foot">
        <span class="note">{{ note }}</span>
        <div class="m-actions-row">
          <button class="btn ghost sm" @click="reset">恢复默认</button>
          <button class="btn primary sm" @click="emit('close')">完成</button>
        </div>
      </div>
    </div>
  </div>
  </Teleport>
</template>

<style scoped>
.modal-mask {
  z-index: var(--z-modal-lift);
}

.modal {
  width: 28.75rem;
  max-width: 100%;
  max-height: 90vh;
}

.modal :deep(.dlg-head) {
  flex: none;
}

.m-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overscroll-behavior: contain;
  display: flex;
  flex-direction: column;
  padding: 0.25rem 1.375rem 0.5rem;
}

.hk-keys {
  display: flex;
  gap: 0.5rem;
}

.hk-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.875rem;
  padding:0.75rem 2px;
  border-bottom:1px dashed var(--line);
}

.hk-label {
  font-size: 0.875rem;
  white-space: nowrap;
  flex: none;
}

.hk-key {
  min-width: 7.5rem;
  min-height: 2.75rem;
  border: 1px solid var(--line);
  border-radius:0.5rem;
  background: var(--panel);
  padding: 0.5rem 0.75rem;
  font-family: var(--font-mono);
  font-size: 0.9375rem;
  cursor: pointer;
  color: var(--ink);
  white-space: nowrap;
}

.hk-key:hover {
  border-color: var(--brand);
  color: var(--brand);
}

.hk-key.capturing {
  background: var(--brand-weak);
  border-color: var(--brand);
  color: var(--brand);
  font-family: inherit;
}

.hk-key.unset {
  color: var(--muted);
}

.m-foot {
  flex: none;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.625rem;
  padding: 0.75rem 1.375rem;
  border-top: 1px solid var(--line);
}

.m-actions-row {
  display: flex;
  gap: 0.5rem;
}

.note {
  font-size: 0.7812rem;
  color: var(--good);
  flex: 1;
}

/* 窄窗口：行内 label + 双键位放不下，整行堆叠；键位平分宽度；
   底部按钮占整行 */
@container (max-width: 30rem) {
  .m-list {
    padding: 0.25rem 1rem 0.5rem;
  }

  .m-foot {
    flex-wrap: wrap;
    padding: 0.75rem 1rem;
  }

  .note {
    flex: 1 1 100%;
  }

  .m-actions-row {
    flex: 1 1 100%;
  }

  .m-actions-row .btn {
    flex: 1;
  }

  .hk-row {
    flex-direction: column;
    align-items: stretch;
    gap: 0.375rem;
  }

  .hk-keys {
    width: 100%;
  }

  .hk-key {
    flex: 1;
    min-width: 0;
  }
}
</style>
