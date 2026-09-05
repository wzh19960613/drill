import { describe, expect, it } from 'vitest'
import { randomSeed, randomToken, shuffleWithSeed } from './rng'

describe('shuffleWithSeed', () => {
  it('is deterministic for the same seed', () => {
    const a = shuffleWithSeed([1, 2, 3, 4, 5, 6, 7, 8], 'SEED')
    const b = shuffleWithSeed([1, 2, 3, 4, 5, 6, 7, 8], 'SEED')
    expect(a).toEqual(b)
  })

  it('produces a permutation (no losses or duplicates)', () => {
    const out = shuffleWithSeed(Array.from({ length: 50 }, (_, i) => i), 'x')
    expect([...out].sort((a, b) => a - b)).toEqual(Array.from({ length: 50 }, (_, i) => i))
  })

  it('differs across seeds (statistically)', () => {
    const base = Array.from({ length: 20 }, (_, i) => i)
    const outA = shuffleWithSeed(base, 'a')
    const outB = shuffleWithSeed(base, 'b')
    expect(outA).not.toEqual(outB)
  })

  it('handles empty and single-element arrays', () => {
    expect(shuffleWithSeed([], 's')).toEqual([])
    expect(shuffleWithSeed([42], 's')).toEqual([42])
  })

  it('does not mutate the input', () => {
    const input = [1, 2, 3]
    shuffleWithSeed(input, 's')
    expect(input).toEqual([1, 2, 3])
  })
})

describe('randomSeed', () => {
  it('yields 4 uppercase alphanumeric characters', () => {
    for (let i = 0; i < 50; i++) {
      expect(randomSeed()).toMatch(/^[0-9A-Z]{4}$/)
    }
  })
})

describe('randomToken', () => {
  it('yields lowercase alphanumeric tokens of the requested length', () => {
    for (let i = 0; i < 50; i++) {
      expect(randomToken()).toMatch(/^[0-9a-z]{8}$/)
      expect(randomToken(12)).toMatch(/^[0-9a-z]{12}$/)
    }
  })
})
