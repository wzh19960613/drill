export interface QOption {
  id: number
  text: string
}

export interface QCore {
  id: string
  source: string
  subject?: string
  origin?: string
  locate?: string
  chapter: string
  qtype: string
  stem: string[]
  options: QOption[]
  correct_id: number | null
  correct_ids?: number[]
  answer_line: string
  solution: string[]
  notes?: string[]
}

export interface Stats {
  attempts: number
  wrong: number
  last_correct: boolean | null
  last_at: number | null
  recent_wrong5: number
}

export type Question = QCore

export interface Rec {
  id: number
  questionId: string
  /** The question's owning source id (question ids are only unique per source) */
  source: string
  correct: boolean
  at: number
  ms?: number
}

export type Paper = 'A4' | 'B5'

export interface BookItemDef {
  id: string
  /** Owning source id: same question id can exist in different sources */
  source: string
  optionOrder: number[] | null
}

export interface BookDef {
  id: string
  name: string
  seed: string
  date: string
  createdAt: number
  items: BookItemDef[]
}

export interface PausedResult {
  qid: string
  id: number
  correct: boolean
  ms?: number
}

/** An interrupted study session (single slot, server persisted) */
export interface PausedSession {
  title: string
  seed: string
  date: string
  bookId: string | null
  idx: number
  sessionMs: number
  items: BookItemDef[]
  results: PausedResult[]
}
