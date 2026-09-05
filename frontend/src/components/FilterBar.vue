<script setup lang="ts">
import { computed, ref } from 'vue'
import { Plus, X } from 'lucide-vue-next'
import {
  QUICK_PRESETS,
  condLabel,
  defaultFilter,
  newCond,
  type Cond,
  type Filter,
} from '../filter'
import FilterCondAdd from './filter/FilterCondAdd.vue'

const props = defineProps<{ modelValue: Filter }>()
const emit = defineEmits<{ (e: 'update:modelValue', f: Filter): void }>()

function patch(p: Partial<Filter>) {
  emit('update:modelValue', { ...props.modelValue, ...p })
}

function normalize(cs: Cond[]) {
  return cs.map((c) => ({
    kind: c.kind,
    what: c.what,
    op: c.op,
    n: c.n,
    m: c.m,
    values: c.values,
  }))
}

const activePreset = computed(() => {
  const f = props.modelValue
  if (f.conds.length === 0) return '全部'
  for (const p of QUICK_PRESETS.slice(1)) {
    const cs = p.conds()
    if (cs.length === f.conds.length && JSON.stringify(normalize(cs)) === JSON.stringify(normalize(f.conds))) {
      return p.label
    }
  }
  return ''
})

function applyPreset(conds: Cond[]) {
  patch({ conds })
}

function removeCond(id: string) {
  patch({ conds: props.modelValue.conds.filter((c) => c.id !== id) })
}

const addOpen = ref(false)

function addCond(cond: Omit<Cond, 'id'>) {
  patch({ conds: [...props.modelValue.conds, newCond(cond)] })
}

function clearAll() {
  emit('update:modelValue', defaultFilter())
}
</script>

<template>
  <div class="fb">
    <div class="fb-main">
      <div class="fb-chips">
        <button
          v-for="p in QUICK_PRESETS"
          :key="p.label"
          class="chip-btn"
          :class="{ on: activePreset === p.label }"
          @click="applyPreset(p.conds())"
        >{{ p.label }}</button>
      </div>
      <div class="fb-tools">
        <button class="more-btn" :class="{ on: addOpen }" @click="addOpen = !addOpen">
          <Plus style="width: 0.9375rem; height: 0.9375rem" /> 条件
        </button>
        <button
          class="more-btn"
          :class="{ on: modelValue.includeMastered }"
          :title="modelValue.includeMastered ? '当前：包含已熟练题目' : '当前：默认排除已熟练题目'"
          @click="patch({ includeMastered: !modelValue.includeMastered })"
        >
          {{ modelValue.includeMastered ? '含已熟练' : '排除已熟练' }}
        </button>
        <div v-if="modelValue.conds.length > 1" class="match-seg">
          <button :class="{ on: modelValue.match === 'all' }" @click="patch({ match: 'all' })">满足全部</button>
          <button :class="{ on: modelValue.match === 'any' }" @click="patch({ match: 'any' })">满足任一</button>
        </div>
      </div>
    </div>

    <div v-if="modelValue.conds.length" class="fb-conds">
      <span
        v-for="c in modelValue.conds"
        :key="c.id"
        class="cond-chip"
        :title="condLabel(c)"
      >
        <span class="cond-txt">{{ condLabel(c) }}</span>
        <button class="cond-x" aria-label="删除该条件" @click="removeCond(c.id)">
          <X style="width: 0.8125rem; height: 0.8125rem" />
        </button>
      </span>
    </div>

    <FilterCondAdd v-show="addOpen" @add="addCond" @clear="clearAll" @close="addOpen = false" />
  </div>
</template>

<style scoped>
.fb {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.fb-main {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  flex-wrap: wrap;
}

.fb-tools {
  display: flex;
  align-items: center;
  gap: 0.625rem;
}

.fb-chips {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
  flex: 1;
}

.more-btn {
  display: inline-flex;
  align-items: center;
  gap:4px;
  border: 1px dashed var(--line);
  background: transparent;
  color: var(--muted);
  font-size: 0.875rem;
  min-height: 2.75rem;
  padding: 0 0.875rem;
  cursor: pointer;
  border-radius: var(--radius-pill);
  white-space: nowrap;
}

.more-btn:hover,
.more-btn.on {
  border-color: var(--brand);
  color: var(--brand);
}

.match-seg {
  display: inline-flex;
  border: 1px solid var(--line);
  border-radius: var(--radius-pill);
  overflow: hidden;
}

.match-seg button {
  border: none;
  background: transparent;
  color: var(--muted);
  font-size: 0.8125rem;
  padding: 0.625rem 0.75rem;
  min-height: 2.75rem;
  cursor: pointer;
  white-space: nowrap;
}

.match-seg button.on {
  background: var(--brand-weak);
  color: var(--brand);
  font-weight: 600;
}

@container (max-width: 40rem) {
  .fb-main {
    flex-direction: column;
    align-items: stretch;
  }

  .fb-chips {
    flex: none;
  }

  .fb-chips .chip-btn {
    flex: 1 1 auto;
  }

  .fb-tools .more-btn {
    flex: 1;
    justify-content: center;
  }

  .fb-tools .match-seg {
    flex: 1;
    display: flex;
  }

  .fb-tools .match-seg button {
    flex: 1;
  }
}

.fb-conds {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.cond-chip {
  display: inline-flex;
  align-items: center;
  gap:4px;
  max-width: 20rem;
  border: 1px solid var(--brand);
  background: var(--brand-weak);
  color: var(--brand);
  border-radius:0.5rem;
  padding: 0.375rem 0.5rem 0.375rem 0.75rem;
  font-size: 0.8438rem;
}

.cond-txt {
  min-width: 0;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.cond-x {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
  height: 1.5rem;
  border: none;
  background: transparent;
  color: var(--brand);
  border-radius:0.375rem;
  cursor: pointer;
  flex: none;
}

.cond-x:hover {
  background: rgba(0, 0, 0, 0.08);
}
</style>
