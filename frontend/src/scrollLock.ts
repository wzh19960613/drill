/**
 * Locks page scrolling while a dialog is open.
 *
 * On touch screens, the overlay's touch-action: none and the scroll container's
 * overscroll-behavior: contain still leave a few paths that scroll the layer beneath
 * (e.g. dragging on the dialog's own non-scrolling area, the old iOS trick), so this
 * simply locks the document scroller to overflow: hidden — reference counting supports
 * dialogs stacked on dialogs; on desktop the scrollbar width is compensated for so the
 * page does not jitter when a dialog opens.
 *
 * Lock html rather than body: style.css sets overflow-x: clip on html (site-wide ban on
 * horizontal scrolling). Once html's overflow is no longer visible, body's overflow no
 * longer propagates to the viewport — locking body would just turn body into a
 * self-clipping box: the page keeps scrolling, and the bottom navigation's sticky
 * anchor moves to body and jumps away from the viewport bottom. Locking html
 * (i.e. the viewport scroller itself) has neither problem.
 */
let count = 0
let savedOverflow = ''
let savedPaddingRight = ''
/** Cached at lock time: if .app is rebuilt between lock and unlock, restoring
 *  on the freshly queried element would leave the locked one hidden */
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
