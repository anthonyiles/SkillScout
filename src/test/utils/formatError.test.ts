import { describe, it, expect } from 'vitest'
import { formatError } from '../../utils/formatError'

describe('formatError', () => {
  it('returns a plain string unchanged', () => {
    expect(formatError('something went wrong', 'fallback')).toBe('something went wrong')
  })

  it('returns Error.message for Error instances', () => {
    expect(formatError(new Error('bad request'), 'fallback')).toBe('bad request')
  })

  it('returns fallback for a plain object', () => {
    expect(formatError({ code: 404 }, 'fallback')).toBe('fallback')
  })

  it('returns fallback for null', () => {
    expect(formatError(null, 'fallback')).toBe('fallback')
  })

  it('returns fallback for undefined', () => {
    expect(formatError(undefined, 'fallback')).toBe('fallback')
  })

  it('returns fallback for an Error with an empty message', () => {
    const err = new Error()
    err.message = ''
    expect(formatError(err, 'fallback')).toBe('fallback')
  })

  it('returns fallback for numbers', () => {
    expect(formatError(42, 'fallback')).toBe('fallback')
  })
})
