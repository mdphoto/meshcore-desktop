import { describe, it, expect } from 'vitest'
import { haversineDistance, formatDistance } from '../utils/geo'
import { parseMeshCoreURI, buildContactURI, buildChannelURI } from '../utils/meshcore-uri'

describe('geo utils', () => {
  it('calcule la distance haversine entre deux points', () => {
    // Paris → Lyon ~ 392 km
    const d = haversineDistance(48.8566, 2.3522, 45.7640, 4.8357)
    expect(d).toBeGreaterThan(380000)
    expect(d).toBeLessThan(400000)
  })

  it('formatDistance en mètres', () => {
    expect(formatDistance(500)).toBe('500 m')
  })

  it('formatDistance en km', () => {
    expect(formatDistance(1500)).toBe('1.5 km')
  })
})

describe('meshcore-uri', () => {
  it('parse une URI contact', () => {
    const result = parseMeshCoreURI('meshcore://contact/abc123?name=Alice')
    expect(result).toEqual({ type: 'contact', pubkey: 'abc123', name: 'Alice' })
  })

  it('parse une URI channel', () => {
    const result = parseMeshCoreURI('meshcore://channel/3?name=General&psk=aabbccdd')
    expect(result).toEqual({ type: 'channel', idx: 3, name: 'General', psk: 'aabbccdd' })
  })

  it('retourne null pour une URI invalide', () => {
    expect(parseMeshCoreURI('https://google.com')).toBeNull()
    expect(parseMeshCoreURI('not-a-url')).toBeNull()
  })

  it('build une URI contact', () => {
    const uri = buildContactURI('abc123', 'Alice')
    expect(uri).toBe('meshcore://contact/abc123?name=Alice')
  })

  it('build une URI channel', () => {
    const uri = buildChannelURI(3, 'General', 'aabbcc')
    expect(uri).toContain('meshcore://channel/3')
    expect(uri).toContain('name=General')
    expect(uri).toContain('psk=aabbcc')
  })
})
