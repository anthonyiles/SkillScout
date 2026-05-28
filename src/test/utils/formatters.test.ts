import { describe, it, expect } from 'vitest'
import { getProjectName } from '../../utils/formatters'

describe('getProjectName', () => {
  it('extracts the last segment of a unix path', () => {
    expect(getProjectName('/home/user/my-project')).toBe('my-project')
  })

  it('extracts the last segment of a windows path', () => {
    expect(getProjectName('C:\\Users\\user\\my-project')).toBe('my-project')
  })

  it('returns New Project for an empty string', () => {
    expect(getProjectName('')).toBe('New Project')
  })

  it('handles a trailing unix slash', () => {
    expect(getProjectName('/home/user/my-project/')).toBe('my-project')
  })

  it('handles a single path segment with no separators', () => {
    expect(getProjectName('project')).toBe('project')
  })

  it('handles a deeply nested path', () => {
    expect(getProjectName('/a/b/c/d/e/repo')).toBe('repo')
  })
})
