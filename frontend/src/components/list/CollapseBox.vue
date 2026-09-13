<script setup lang="ts">
defineProps<{ open: boolean }>()
</script>

<template>
  <div class="collapse-box" :class="{ closed: !open }">
    <div class="collapse-in">
      <slot />
    </div>
  </div>
</template>

<style scoped>
/* grid-rows 0fr/1fr keeps the height animation purely CSS; the inner wrapper
   must clip while animating (nothing sticky lives inside) */
.collapse-box {
  display: grid;
  grid-template-rows: 1fr;
  transition: grid-template-rows 0.24s ease;
}

.collapse-in {
  min-height: 0;
  overflow: hidden;
}

.collapse-box.closed {
  grid-template-rows: 0fr;
}
</style>
