export type MenuAlign = 'left' | 'right'

const EDGE = 8

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
