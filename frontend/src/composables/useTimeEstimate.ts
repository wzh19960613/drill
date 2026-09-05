import { computed } from 'vue'
import { store, subjectQuestions } from '../store'

const MINUTES_BY_TYPE: Record<string, number> = {
  选择题: 1.5,
  填空题: 2.5,
  证明题: 8,
  解答题: 8,
}

export function useTimeEstimate() {
  const estimate = computed(() => {
    if (!store.loaded) return ''
    const perType = measuredMinutesPerType()
    const minutes = subjectQuestions.value.reduce(
      (n, q) => n + (perType.get(q.qtype) ?? MINUTES_BY_TYPE[q.qtype] ?? 3),
      0,
    )
    return formatDuration(minutes)
  })

  const typeSummary = computed(() => {
    const counts = new Map<string, number>()
    for (const q of subjectQuestions.value) counts.set(q.qtype, (counts.get(q.qtype) ?? 0) + 1)
    return [...counts.entries()].map(([t, n]) => `${t} ${n}`).join(' · ')
  })

  const total = computed(() => subjectQuestions.value.length)

  return { estimate, typeSummary, total }
}

function measuredMinutesPerType(): Map<string, number> {
  const qTypeById = new Map(store.questions.map((q) => [q.id, q.qtype]))
  const samples = new Map<string, number[]>()
  for (const r of store.records) {
    if (!r.ms || r.ms <= 0) continue
    const t = qTypeById.get(r.questionId)
    if (!t) continue
    const list = samples.get(t) ?? []
    list.push(r.ms)
    samples.set(t, list)
  }
  const perType = new Map<string, number>()
  for (const [t, list] of samples) {
    if (list.length >= 3) {
      const mean = list.reduce((a, b) => a + b, 0) / list.length
      perType.set(t, mean / 60000)
    }
  }
  return perType
}

function formatDuration(minutes: number): string {
  if (minutes < 60) return `约 ${Math.max(5, Math.round(minutes / 5) * 5)} 分钟`
  const hours = minutes / 60
  const rounded = Math.round(hours * 2) / 2
  return `约 ${Number.isInteger(rounded) ? rounded : rounded.toFixed(1)} 小时`
}
