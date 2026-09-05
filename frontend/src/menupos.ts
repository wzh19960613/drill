export type MenuAlign = 'left' | 'right'

const EDGE = 8

/** Viewport placement for a popup menu; x is the anchored edge (per align),
    y the top edge; both axes clamp so the menu stays fully on screen. */
export function placeMenu(
  x: number,
  y: number,
  w: number,
  h: number,
  vw: number,
  vh: number,
  align: MenuAlign,
): { left: number; top: number } {
  const wantLeft = align === 'right' ? x - w : x
  return {
    left: Math.min(Math.max(wantLeft, EDGE), Math.max(EDGE, vw - w - EDGE)),
    top: Math.min(Math.max(y, EDGE), Math.max(EDGE, vh - h - EDGE)),
  }
}
