import { computed, ref, type Ref } from 'vue'
import { randomSeed, shuffleWithSeed } from '../rng'
import { applyFilter, type Filter, type SortKey } from '../filter'
import { subjectQuestions } from '../store'
import { readJSON, writeJSON, StorageKeys } from '../storage'
import type { Question } from '../types'

export function useBookNewDraft(filter: Ref<Filter>) {
  const sort = ref<SortKey>('random')
  const shuffleO = ref(true)
  const seed = ref(randomSeed())

  function loadCfg() {
    const c = readJSON<{ sort?: SortKey; shuffleO?: boolean }>(StorageKeys.bookConfig)
    if (c && typeof c === 'object') {
      sort.value = c.sort ?? 'random'
      shuffleO.value = c.shuffleO ?? true
    }
  }

  function persistCfg() {
    writeJSON(StorageKeys.bookConfig, { sort: sort.value, shuffleO: shuffleO.value })
  }

  function dice() {
    seed.value = randomSeed()
  }

  const filteredQuestions = computed<Question[]>(() => {
    if (sort.value === 'random') {
      const base = applyFilter(subjectQuestions.value, { ...filter.value, sort: 'id' })
      const order = shuffleWithSeed(base.map((_, i) => i), seed.value + '|q')
      return order.map((i) => base[i])
    }
    return applyFilter(subjectQuestions.value, { ...filter.value, sort: sort.value })
  })

  return { sort, shuffleO, seed, filteredQuestions, loadCfg, persistCfg, dice }
}
