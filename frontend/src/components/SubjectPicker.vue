<script setup lang="ts">
import { computed } from 'vue'
import { currentSubject, subjects, UNKNOWN } from '../store'

/** A lone subject has nothing to filter → plain label instead of a picker */
const named = computed(() => subjects.value.filter((s) => s !== UNKNOWN))
</script>

<template>
  <select
    v-if="named.length >= 2"
    v-model="currentSubject"
    class="subj-sel"
    aria-label="学科筛选"
    title="学科筛选"
  >
    <option value="">[综合]</option>
    <option v-for="s in subjects" :key="s" :value="s">{{ s }}</option>
  </select>
  <span v-else-if="named.length === 1" class="subj-sel subj-txt" :title="named[0]">{{ named[0] }}</span>
</template>

<style scoped>
.subj-sel {
  height: 2.25rem;
  padding:0 4px;
  border: none;
  border-radius:0.5rem;
  color: var(--ink);
  font-size: 0.8438rem;
  cursor: pointer;
  max-width: 9rem;
}

.subj-sel:focus,
.subj-sel:focus-visible {
  outline: none;
  border: none;
}

.subj-txt {
  height: auto;
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: right;
  cursor: default;
}
</style>
