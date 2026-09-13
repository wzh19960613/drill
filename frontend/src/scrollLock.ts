let count = 0
let savedOverflow = ''
let savedPaddingRight = ''

let lockedEl: HTMLElement | null = null

function scroller(): HTMLElement {
  return document.querySelector('.app') ?? document.documentElement
}

export function lockBodyScroll() {
  if (count++ === 0) {
    const el = scroller()
    lockedEl = el
    const sw = el.offsetWidth - el.clientWidth
    savedOverflow = el.style.overflow
    savedPaddingRight = el.style.paddingRight
    el.style.overflow = 'hidden'
    if (sw > 0) el.style.paddingRight = `${sw}px`
  }
}

export function unlockBodyScroll() {
  if (count === 0) return
  if (--count === 0) {
    const el = lockedEl ?? scroller()
    lockedEl = null
    el.style.overflow = savedOverflow
    el.style.paddingRight = savedPaddingRight
  }
}
