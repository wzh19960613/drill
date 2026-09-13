<script lang="ts">
import type { Component } from 'vue'

export interface MenuPopItem {
  key: string
  label: string
  icon?: Component
  danger?: boolean
  sep?: boolean
  /** renders as a toggleable checkbox entry when present */
  checked?: boolean
}
</script>

<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import { placeMenu } from '../menupos'

const props = withDefaults(
  defineProps<{
    open: boolean
    x: number
    y: number
    items: MenuPopItem[]
    align?: 'left' | 'right'
    mask?: boolean
  }>(),
  { align: 'left', mask: true },
)

const emit = defineEmits<{
  (e: 'select', item: MenuPopItem): void
  (e: 'close'): void
}>()

const listEl = ref<HTMLElement | null>(null)
const left = ref('0px')
const top = ref('0px')

watch(
  () => props.open,
  async (open) => {
    if (!open) return
    left.value = '0px'
    top.value = '0px'
    await nextTick()
    const el = listEl.value
    if (!el) return
    const pos = placeMenu(
      props.x,
      props.y,
      el.offsetWidth,
      el.offsetHeight,
      window.innerWidth,
      window.innerHeight,
      props.align,
    )
    left.value = pos.left + 'px'
    top.value = pos.top + 'px'
  },
)

function pick(item: MenuPopItem) {
  emit('select', item)
  emit('close')
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open && mask" class="mpop-mask" @click="emit('close')"></div>
    <div v-if="open" ref="listEl" class="mpop" :style="{ left, top }">
      <template v-for="it in items" :key="it.key">
        <div v-if="it.sep" class="mpop-sep"></div>
        <button :class="{ danger: it.danger }" @click="pick(it)">
          <span v-if="it.checked !== undefined" class="mpop-check" :class="{ on: it.checked }">
            <svg v-if="it.checked" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3.2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M20 6 9 17l-5-5" />
            </svg>
          </span>
          <template v-else-if="items.some((i) => i.icon)">
            <component :is="it.icon" v-if="it.icon" style="width: 1rem; height: 1rem" />
            <span v-else class="mpop-ico"></span>
          </template>
          {{ it.label }}
        </button>
      </template>
    </div>
  </Teleport>
</template>

<style scoped>
.mpop-mask {
  position: absolute;
  inset: 0;
  z-index: var(--z-menu);
}

.mpop {
  position: absolute;
  z-index: calc(var(--z-menu) + 1);
  min-width: 12.5rem;
  background: var(--card);
  border: 1px solid var(--line);
  border-radius: 0.75rem;
  box-shadow: var(--shadow-pop);
  padding: 0.375rem;
  display: flex;
  flex-direction: column;
}

.mpop button {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  border: none;
  background: transparent;
  text-align: left;
  padding: 0.6875rem 0.75rem;
  min-height: 2.75rem;
  font-size: 0.9375rem;
  font-family: inherit;
  color: var(--ink);
  border-radius: 0.5rem;
  cursor: pointer;
}

.mpop button:hover {
  background: var(--brand-weak);
  color: var(--brand);
}

.mpop button.danger {
  color: var(--bad);
}

.mpop button.danger:hover {
  background: var(--bad-weak);
  color: var(--bad);
}

.mpop-ico {
  flex: none;
  width: 1rem;
  height: 1rem;
}

.mpop-check {
  flex: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.125rem;
  height: 1.125rem;
  border: 1.5px solid var(--line);
  border-radius: 0.3125rem;
  color: transparent;
}

.mpop-check.on {
  background: var(--brand);
  border-color: var(--brand);
  color: #fff;
}

.mpop-check svg {
  width: 0.75rem;
  height: 0.75rem;
}

.mpop-sep {
  height: 0.0625rem;
  background: var(--line);
  margin: 0.375rem 4px;
}
</style>
