import { reactive } from 'vue'
import type { ExportOpts, LineSpec } from '../book'
import { readJSON, writeJSON, StorageKeys } from '../storage'

export const FONT_STEPS: { fs: number; label: string }[] = [
  { fs: 9 / 10.5, label: '小五' },
  { fs: 1, label: '五号' },
  { fs: 12 / 10.5, label: '小四' },
  { fs: 14 / 10.5, label: '四号' },
  { fs: 15 / 10.5, label: '小三' },
  { fs: 16 / 10.5, label: '三号' },
]

const MARGIN_DEF = { marginT: 14, marginB: 14, marginL: 16, marginR: 16 } as const

const LINE_DEF: Record<'sepQ' | 'sepHead' | 'sepFoot', LineSpec> = {
  sepQ: { style: 'dashed', width: 0.6, gap: 2 },
  sepHead: { style: 'dashed', width: 0.6, gap: 2 },
  sepFoot: { style: 'none', width: 0.6, gap: 2 },
}

export function loadExportOpts(): ExportOpts {
  const def = defaultExportOpts()
  const saved = readJSON<Partial<ExportOpts>>(StorageKeys.exportOpts)
  if (saved && typeof saved === 'object') Object.assign(def, saved)
  return sanitizeExportOpts(def)
}

function defaultExportOpts(): ExportOpts {
  return {
    paper: 'A4',
    perPage: 2,
    bindingExtra: 6,
    marginT: MARGIN_DEF.marginT,
    marginB: MARGIN_DEF.marginB,
    marginL: MARGIN_DEF.marginL,
    marginR: MARGIN_DEF.marginR,
    sepQ: { ...LINE_DEF.sepQ },
    sepHead: { ...LINE_DEF.sepHead },
    sepFoot: { ...LINE_DEF.sepFoot },
    showMeta: true,
    workbookMeta: '',
    answersMeta: '',
    fontScale: 1,
    optsPerRow: 4,
    bindingLong: false,
    firstPageLeft: false,
    headerBinding: '',
    headerCenter: '',
    headerFlip: '',
    footerBinding: '',
    footerCenter: '',
    footerFlip: '',
    customStyle: '',
  }
}

function sanitizeExportOpts(def: ExportOpts): ExportOpts {
  if (def.paper !== 'A4' && def.paper !== 'B5') def.paper = 'A4'
  if (!FONT_STEPS.some((s) => s.fs === def.fontScale)) def.fontScale = 1
  if (![4, 2, 1].includes(def.optsPerRow)) def.optsPerRow = 4
  if (![0, 1, 2, 3, 4].includes(def.perPage)) def.perPage = 2
  if (!Number.isFinite(def.bindingExtra) || def.bindingExtra < 0) def.bindingExtra = 0
  def.bindingExtra = Math.min(30, Math.round(def.bindingExtra))
  for (const key of ['marginT', 'marginB', 'marginL', 'marginR'] as const) {
    if (!Number.isFinite(def[key])) def[key] = MARGIN_DEF[key]
    def[key] = Math.min(40, Math.max(5, Math.round(def[key])))
  }
  const styles = ['none', 'dashed', 'dotted', 'solid', 'double', 'thickThin']
  for (const key of ['sepQ', 'sepHead', 'sepFoot'] as const) {
    const spec = def[key]
    if (!spec || typeof spec !== 'object') {
      def[key] = { ...LINE_DEF[key] }
      continue
    }
    if (!styles.includes(spec.style)) spec.style = LINE_DEF[key].style
    spec.width = Math.min(4, Math.max(0.2, Number(spec.width) || 0.6))
    spec.gap = Math.min(12, Math.max(0.5, Number(spec.gap) || 2))
  }
  return def
}

const sharedOpts = reactive<ExportOpts>(loadExportOpts())

export function useExportPrefs() {
  function persist() {
    writeJSON(StorageKeys.exportOpts, sharedOpts)
  }

  return { opts: sharedOpts, persist }
}
