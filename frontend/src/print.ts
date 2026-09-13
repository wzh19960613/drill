import type { Question } from './types'

export interface PrintItem extends Question {
  seq: number
  optionOrder: number[] | null
}

export interface PrintPayload {
  doc: 'workbook' | 'answers'
  paper: 'A4' | 'B5'
  perPage?: number
  sepQ?: { style: string; width: number; gap: number }
  sepHead?: { style: string; width: number; gap: number }
  sepFoot?: { style: string; width: number; gap: number }
  showMeta?: boolean
  bindingExtra?: number
  marginT?: number
  marginB?: number
  marginL?: number
  marginR?: number
  date: string
  title?: string
  bookId?: string | null

  meta?: { workbook: string; answers: string }
  fontScale?: number
  optsPerRow?: 1 | 2 | 4
  bindingLong?: boolean
  firstPageLeft?: boolean
  header?: { binding: string; center: string; flip: string }
  footer?: { binding: string; center: string; flip: string }
  customStyle?: string
  items: PrintItem[]
}
