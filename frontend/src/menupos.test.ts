import { describe, expect, it } from 'vitest'
import { placeMenu } from './menupos'

describe('placeMenu', () => {
  it('places at x/y for left align', () => {
    expect(placeMenu(100, 50, 200, 100, 1280, 800, 'left')).toEqual({ left: 100, top: 50 })
  })

  it('right align puts the menu left of the anchor edge', () => {
    expect(placeMenu(600, 50, 200, 100, 1280, 800, 'right')).toEqual({ left: 400, top: 50 })
  })

  it('clamps to the right/bottom viewport edges', () => {
    expect(placeMenu(1270, 780, 200, 100, 1280, 800, 'left')).toEqual({ left: 1072, top: 692 })
  })

  it('clamps to the left/top edges', () => {
    expect(placeMenu(0, 0, 200, 100, 1280, 800, 'left')).toEqual({ left: 8, top: 8 })
    expect(placeMenu(50, 0, 200, 100, 1280, 800, 'right')).toEqual({ left: 8, top: 8 })
  })

  it('keeps the edge margin when the menu exceeds the viewport', () => {
    expect(placeMenu(600, 400, 1300, 900, 1280, 800, 'left')).toEqual({ left: 8, top: 8 })
  })
})
