<script setup lang="ts">
import { Star } from 'lucide-vue-next'

withDefaults(defineProps<{ on: boolean; size?: string }>(), { size: '1rem' })

const emit = defineEmits<{ (e: 'toggle'): void }>()
</script>

<template>
  <button
    class="fav-star"
    :class="{ on }"
    :aria-label="on ? '取消收藏' : '收藏'"
    @click="emit('toggle')"
  >
    <Star :style="{ width: size, height: size }" :fill="on ? 'currentColor' : 'none'" />
  </button>
</template>

<style scoped>
.fav-star {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: inherit;
  padding:4px;
  border-radius:0.5rem;
  color: var(--ink);
  cursor: pointer;
}

.fav-star.on {
  color: var(--warn);
}

.fav-star:hover {
  background: var(--hover);
}
</style>
