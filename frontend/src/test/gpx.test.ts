import { describe, it, expect } from 'vitest'
import { generateGpx } from '../utils/gpx'

describe('generateGpx', () => {
  const nodes = [
    {
      public_key: 'pk1',
      name: 'Alice',
      node_type: 1,
      flags: 0,
      path: [],
      path_len: 2,
      lat: 44.75,
      lon: 4.82,
      last_seen: '2026-03-27T10:00:00Z',
      is_favorite: false,
      group_name: null,
    },
    {
      public_key: 'pk2',
      name: 'Repeater-1',
      node_type: 2,
      flags: 0,
      path: [],
      path_len: 0,
      lat: 44.76,
      lon: 4.83,
      last_seen: '2026-03-27T11:00:00Z',
      is_favorite: false,
      group_name: null,
    },
    {
      public_key: 'pk3',
      name: 'NoGPS',
      node_type: 0,
      flags: 0,
      path: [],
      path_len: -1,
      lat: 0,
      lon: 0,
      last_seen: '2026-03-27T12:00:00Z',
      is_favorite: false,
      group_name: null,
    },
  ]

  it('génère un fichier GPX valide', () => {
    const gpx = generateGpx(nodes, 'TestDevice')
    expect(gpx).toContain('<?xml version="1.0"')
    expect(gpx).toContain('<gpx version="1.1"')
    expect(gpx).toContain('MeshCore Desktop')
    expect(gpx).toContain('TestDevice')
  })

  it('inclut les waypoints avec coordonnées', () => {
    const gpx = generateGpx(nodes)
    expect(gpx).toContain('<wpt lat="44.75" lon="4.82">')
    expect(gpx).toContain('<name>Alice</name>')
    expect(gpx).toContain('<wpt lat="44.76" lon="4.83">')
    expect(gpx).toContain('<name>Repeater-1</name>')
  })

  it('exclut les nœuds sans GPS', () => {
    const gpx = generateGpx(nodes)
    expect(gpx).not.toContain('NoGPS')
  })

  it('inclut le type de nœud', () => {
    const gpx = generateGpx(nodes)
    expect(gpx).toContain('<type>Client</type>')
    expect(gpx).toContain('<type>Repeater</type>')
  })

  it('inclut les hops dans la description', () => {
    const gpx = generateGpx(nodes)
    expect(gpx).toContain('2 hops')
    expect(gpx).toContain('0 hops')
  })

  it('échappe les caractères spéciaux XML', () => {
    const special = [{
      ...nodes[0],
      name: 'Node <&> "Test"',
    }]
    const gpx = generateGpx(special)
    expect(gpx).toContain('&lt;&amp;&gt;')
    expect(gpx).toContain('&quot;')
  })
})
