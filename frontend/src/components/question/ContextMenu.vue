<script setup lang="ts">
import { computed } from 'vue'
import MenuPop, { type MenuPopItem } from '../MenuPop.vue'
import type { CtxItem } from '../../composables/useQuestionContextMenu'

const props = defineProps<{
  show: boolean
  x: number
  y: number
  items: CtxItem[]
}>()

const emit = defineEmits<{ (e: 'close'): void }>()

const popItems = computed<MenuPopItem[]>(() =>
  props.items.map((it, i) => ({ key: String(i), label: it.label })),
)

function onPick(item: MenuPopItem) {
  props.items[Number(item.key)]?.run()
}
</script>

<template>
  <MenuPop
    :open="show"
    :x="x"
    :y="y"
    :items="popItems"
    :mask="false"
    @select="onPick"
    @close="emit('close')"
  />
</template>

<style>
/* Right-click formula highlight box (applied to rendered content, must stay global) */
.katex.ctx-target {
  outline: 2px solid var(--brand);
  outline-offset: 2px;
  border-radius:4px;
  background: var(--brand-weak);
}
</style>
