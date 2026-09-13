import type { Paper } from '../types'
import type { PrintPayload } from '../print'
import type { BookSession } from './session'

export interface LineSpec {
  style: 'none' | 'dashed' | 'dotted' | 'solid' | 'double' | 'thickThin'
  width: number
  gap: number
}

export const LINE_STYLES: { value: LineSpec['style']; label: string }[] = [
  { value: 'none', label: '无' },
  { value: 'dashed', label: '虚线' },
  { value: 'dotted', label: '点线' },
  { value: 'solid', label: '实线' },
  { value: 'double', label: '双实线' },
  { value: 'thickThin', label: '粗细线' },
]

export interface ExportOpts {
  paper: Paper
  perPage: number

  fontScale: number
  optsPerRow: 1 | 2 | 4
  bindingExtra: number
  marginT: number
  marginB: number
  marginL: number
  marginR: number
  sepQ: LineSpec
  sepHead: LineSpec
  sepFoot: LineSpec
  showMeta: boolean
  bindingLong: boolean
  firstPageLeft: boolean
  workbookMeta: string
  answersMeta: string
  headerBinding: string
  headerCenter: string
  headerFlip: string
  footerBinding: string
  footerCenter: string
  footerFlip: string
  customStyle: string
}

export function payloadFor(
  session: BookSession,
  doc: 'workbook' | 'answers',
  o: ExportOpts,
): PrintPayload {
  return {
    doc,
    items: session.items,
    paper: o.paper,
    perPage: o.perPage,
    date: session.date,
    title: session.title,
    bookId: session.bookId ?? null,
    meta: { workbook: o.workbookMeta, answers: o.answersMeta },
    sepQ: o.sepQ,
    sepHead: o.sepHead,
    sepFoot: o.sepFoot,
    showMeta: o.showMeta,
    bindingExtra: o.bindingExtra,
    marginT: o.marginT,
    marginB: o.marginB,
    marginL: o.marginL,
    marginR: o.marginR,
    fontScale: o.fontScale,
    optsPerRow: o.optsPerRow,
    bindingLong: o.bindingLong,
    firstPageLeft: o.firstPageLeft,
    header: { binding: o.headerBinding, center: o.headerCenter, flip: o.headerFlip },
    footer: { binding: o.footerBinding, center: o.footerCenter, flip: o.footerFlip },
    customStyle: o.customStyle,
  }
}
