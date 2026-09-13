import { readJSON, removeKey, writeJSON, StorageKeys } from './storage'

export type HotkeyAction =
  | 'showAnswer'
  | 'hideAnswer'
  | 'markRight'
  | 'markWrong'
  | 'toggleFavorite'
  | 'prev'
  | 'next'
  | 'timerToggle'
  | 'timerReset'
  | 'markMastered'

export interface HotkeyActionDef {
  action: HotkeyAction
  label: string
  def: string

  def2?: string
}

export const HOTKEY_ACTIONS: HotkeyActionDef[] = [
  { action: 'showAnswer', label: '显示答案', def: 'Space' },
  { action: 'markRight', label: '做对了', def: 'Enter' },
  { action: 'hideAnswer', label: '隐藏答案', def: 'Space' },
  { action: 'markWrong', label: '做错了', def: 'KeyX' },
  { action: 'toggleFavorite', label: '收藏 / 取消', def: 'KeyC' },
  { action: 'prev', label: '上一题', def: 'Comma', def2: 'ArrowLeft' },
  { action: 'next', label: '跳过（下一题）', def: 'Period', def2: 'ArrowRight' },
  { action: 'timerToggle', label: '暂停 / 恢复计时', def: 'KeyZ' },
  { action: 'timerReset', label: '重置计时', def: 'KeyN' },
  { action: 'markMastered', label: '已熟练 / 取消', def: 'Slash', def2: 'Backspace' },
]

export type HotkeyMap = Record<HotkeyAction, string>

const STORAGE = StorageKeys.hotkeys
const SCHEMA_V = 4

export function defaultHotkeys(): HotkeyMap {
  const m = {} as HotkeyMap
  for (const a of HOTKEY_ACTIONS) m[a.action] = a.def2 ? `${a.def}|${a.def2}` : a.def
  return m
}

export function keysOf(v: string): string[] {
  return v ? v.split('|').filter(Boolean) : []
}

export function loadHotkeys(): HotkeyMap {
  const base = defaultHotkeys()
  const saved = readJSON<{ v: number; map?: Record<string, string> }>(STORAGE)
  if (saved?.v === SCHEMA_V) {
    for (const [action, v] of Object.entries(saved.map ?? {})) {
      if (action in base) base[action as HotkeyAction] = v
    }
  }
  return base
}

export function saveHotkeys(m: HotkeyMap) {
  writeJSON(STORAGE, { v: SCHEMA_V, map: m })
}

export function resetHotkeys() {
  removeKey(STORAGE)
}

const CODE_LABELS: Record<string, string> = {
  Space: '空格',
  Enter: '回车',
  Escape: 'Esc',
  Backspace: '⌫',
  ArrowLeft: '←',
  ArrowRight: '→',
  ArrowUp: '↑',
  ArrowDown: '↓',
  Minus: '-',
  Equal: '=',
  BracketLeft: '[',
  BracketRight: ']',
  Semicolon: ';',
  Quote: "'",
  Backquote: '`',
  Comma: ',',
  Period: '.',
  Slash: '/',
}

function fmtSingle(code: string): string {
  if (!code) return '未绑定'
  if (code.startsWith('Key')) return code.slice(3)
  if (code.startsWith('Digit')) return code.slice(5)
  if (code.startsWith('Numpad')) return '小键盘 ' + code.slice(6)
  return CODE_LABELS[code] ?? code
}

export function primaryCode(v: string): string {
  return keysOf(v)[0] ?? ''
}

export function fmtKey(v: string): string {
  const parts = keysOf(v).map(fmtSingle)
  return parts.length ? parts.join(' / ') : '未绑定'
}

/** Space is bound to both show and hide: each side only acts on the
 *  opposite press-time state, so one key toggles cleanly instead of the two
 *  cancelling out. */
export function applyAnswerActions(openAtPress: boolean, actions: HotkeyAction[]): boolean {
  let open = openAtPress
  for (const a of actions) {
    if (a === 'showAnswer' && !openAtPress) open = true
    else if (a === 'hideAnswer' && openAtPress) open = false
  }
  return open
}

export function codeIndex(m: HotkeyMap): Map<string, HotkeyAction[]> {
  const idx = new Map<string, HotkeyAction[]>()
  for (const [action, v] of Object.entries(m)) {
    for (const code of keysOf(v)) {
      const list = idx.get(code) ?? []
      list.push(action as HotkeyAction)
      idx.set(code, list)
    }
  }
  return idx
}
