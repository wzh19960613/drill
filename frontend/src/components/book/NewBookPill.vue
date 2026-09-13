<script setup lang="ts">
import { Plus } from 'lucide-vue-next'


defineProps<{ label?: string }>()

const emit = defineEmits<{ (e: 'click'): void }>()
</script>

<template>
  <button class="bh-new" @click="emit('click')">
    <Plus style="width: 1.125rem; height: 1.125rem" /> {{ label ?? '新建题本' }}
  </button>
</template>

<style scoped>
.bh-new {
  position: sticky;
  bottom:calc(var(--bnav-h) + var(--bnav-gap) + 0.875rem);
  z-index: var(--z-float);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  height: var(--bnav-h);
  padding: 0 2.25rem;
  border: none;
  border-radius: var(--radius-pill);
  background: var(--brand);
  color: #fff;
  font-size: 1rem;
  font-weight: 600;
  font-family: inherit;
  cursor: pointer;
  margin: 0 0.75rem;
  margin-top: 2rem;
  box-shadow: var(--shadow-float);
}

.bh-new:hover {
  opacity: 0.92;
}

@container (max-width: 40rem) {
  .bh-new {
    margin: 0 var(--bnav-mx);
    margin-top: 2rem;
  }
}

@container (min-width: 49.125rem) {
  .bh-new {
    width: fit-content;
    max-width: 20rem;
    margin-inline: auto;
    bottom: var(--bnav-gap);
  }
}
</style>
