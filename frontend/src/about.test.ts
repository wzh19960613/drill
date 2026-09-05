import { describe, expect, it } from 'vitest'
import pkg from '../package.json'
import { APP_VERSION, REPO_URL } from './about'

describe('about', () => {
  it('version stays in sync with package.json', () => {
    expect(APP_VERSION).toBe(pkg.version)
  })

  it('repo url points at the project GitHub', () => {
    expect(REPO_URL).toBe('https://github.com/wzh19960613/drill')
  })
})
