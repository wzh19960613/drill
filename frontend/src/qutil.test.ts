import { describe, expect, it } from 'vitest'
import { answerIds, displayOptions, letterOf, remapIds, stemExcerptHtml } from './qutil'
import type { QCore } from './types'

function q(partial: Partial<QCore>): QCore {
  return {
    id: 'P1-1',
    source: 's1',
    chapter: 'c',
    qtype: '选择题',
    stem: ['题干'],
    options: [
      { id: 1, text: '甲' },
      { id: 2, text: '乙' },
      { id: 3, text: '丙' },
    ],
    correct_id: 2,
    correct_ids: [2],
    answer_line: '**(B)**。',
    solution: [],
    ...partial,
  }
}

describe('letterOf', () => {
  it('maps 1..6 to A..F and beyond via charCode', () => {
    expect(letterOf(1)).toBe('A')
    expect(letterOf(6)).toBe('F')
    expect(letterOf(7)).toBe('G')
  })
})

describe('displayOptions', () => {
  it('follows the given order', () => {
    const out = displayOptions(q({}), [2, 0, 1])
    expect(out.map((o) => o.text)).toEqual(['丙', '甲', '乙'])
    expect(out.map((o) => o.letter)).toEqual(['A', 'B', 'C'])
  })

  it('falls back to the original order when order is missing or mismatched', () => {
    expect(displayOptions(q({}), null).map((o) => o.text)).toEqual(['甲', '乙', '丙'])
    expect(displayOptions(q({}), [0, 1]).map((o) => o.text)).toEqual(['甲', '乙', '丙'])
    expect(displayOptions(q({}), undefined).map((o) => o.text)).toEqual(['甲', '乙', '丙'])
  })
})

describe('answerIds', () => {
  it('prefers correct_ids, falls back to correct_id, then empty', () => {
    expect(answerIds(q({ correct_ids: [1, 3], correct_id: 1 }))).toEqual([1, 3])
    expect(answerIds(q({ correct_ids: [], correct_id: 3 }))).toEqual([3])
    expect(answerIds(q({ correct_ids: [], correct_id: null }))).toEqual([])
  })
})

describe('remapIds', () => {
  it('remaps correct ids into display positions, sorted', () => {
    // option 2 sits at position 0, option 1 at position 1
    expect(remapIds(q({ correct_ids: [2, 1] }), [1, 0, 2])).toEqual([1, 2])
  })

  it('returns null without an order, with no answers, or on length mismatch', () => {
    expect(remapIds(q({}), null)).toBeNull()
    expect(remapIds(q({ correct_ids: [], correct_id: null }), [0, 1, 2])).toBeNull()
    expect(remapIds(q({}), [0, 1])).toBeNull()
  })
})

describe('stemExcerptHtml', () => {
  it('drops image paragraphs and joins the first three stem paragraphs', () => {
    const html = stemExcerptHtml(
      q({ stem: ['第一段', '![[图.svg]]', '第二段', '第三段', '第四段'] }),
    )
    expect(html).toContain('第一段 第二段 第三段')
    expect(html).not.toContain('第四段')
  })

  it('escapes html in the stem', () => {
    expect(stemExcerptHtml(q({ stem: ['a<b>&c'] }))).toContain('&lt;b&gt;&amp;')
  })
})
