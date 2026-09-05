import { computed, ref, type Ref } from 'vue'
import type { Question } from '../types'
import { chapterOf, subjectOf } from '../store'

export interface QuestionGroup {
  chapter: string
  questions: Question[]
}

export interface SubjectGroup {
  subject: string
  showHead: boolean
  count: number
  groups: QuestionGroup[]
}

export interface UseQuestionGroupsOptions {
  questions: Ref<Question[]>
  flat: Ref<boolean>
}

export function useQuestionGroups(opts: UseQuestionGroupsOptions) {
  const collapsed = ref(new Set<string>())

  function chapterGroups(qs: Question[]): QuestionGroup[] {
    const map = new Map<string, Question[]>()
    for (const q of qs) {
      const key = chapterOf(q)
      const list = map.get(key)
      if (list) list.push(q)
      else map.set(key, [q])
    }
    return [...map.entries()]
      .sort(([a], [b]) => a.localeCompare(b, 'zh'))
      .map(([chapter, questions]) => ({ chapter, questions }))
  }

  const subjectGroups = computed<SubjectGroup[]>(() => {
    if (opts.flat.value) {
      return [
        {
          subject: '',
          showHead: false,
          count: opts.questions.value.length,
          groups: [{ chapter: '', questions: opts.questions.value }],
        },
      ]
    }
    const bySubject = new Map<string, Question[]>()
    for (const q of opts.questions.value) {
      const key = subjectOf(q)
      const list = bySubject.get(key)
      if (list) list.push(q)
      else bySubject.set(key, [q])
    }
    const entries = [...bySubject.entries()].sort(([a], [b]) => a.localeCompare(b, 'zh'))
    const showHead = entries.length > 1
    return entries.map(([subject, qs]) => ({
      subject,
      showHead,
      count: qs.length,
      groups: chapterGroups(qs),
    }))
  })

  const seqOf = computed(() => {
    const m = new Map<string, number>()
    opts.questions.value.forEach((q, i) => m.set(q.id, i + 1))
    return m
  })

  function toggleCollapse(chapter: string) {
    const s = new Set(collapsed.value)
    if (s.has(chapter)) s.delete(chapter)
    else s.add(chapter)
    collapsed.value = s
  }

  return { collapsed, subjectGroups, seqOf, toggleCollapse }
}
