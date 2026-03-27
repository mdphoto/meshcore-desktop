import type { StoredContact } from '../types'

const nodeTypeLabels: Record<number, string> = {
  0: 'Unknown',
  1: 'Client',
  2: 'Repeater',
  3: 'Room Server',
  4: 'Sensor',
}

export function generateGpx(nodes: StoredContact[], deviceName?: string): string {
  const escapeXml = (s: string) =>
    s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')

  const waypoints = nodes
    .filter((n) => n.lat !== 0 || n.lon !== 0)
    .map((n) => {
      const type = nodeTypeLabels[n.node_type] ?? `Type ${n.node_type}`
      return `  <wpt lat="${n.lat}" lon="${n.lon}">
    <name>${escapeXml(n.name)}</name>
    <desc>${escapeXml(type)}${n.path_len >= 0 ? ` - ${n.path_len} hops` : ''}</desc>
    <type>${escapeXml(type)}</type>
    <time>${n.last_seen}</time>
  </wpt>`
    })
    .join('\n')

  return `<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="MeshCore Desktop"
  xmlns="http://www.topografix.com/GPX/1/1"
  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
  xsi:schemaLocation="http://www.topografix.com/GPX/1/1 http://www.topografix.com/GPX/1/1/gpx.xsd">
  <metadata>
    <name>MeshCore Nodes${deviceName ? ` - ${escapeXml(deviceName)}` : ''}</name>
    <time>${new Date().toISOString()}</time>
  </metadata>
${waypoints}
</gpx>`
}

export function downloadGpx(nodes: StoredContact[], deviceName?: string) {
  const gpx = generateGpx(nodes, deviceName)
  const blob = new Blob([gpx], { type: 'application/gpx+xml' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `meshcore-nodes-${new Date().toISOString().slice(0, 10)}.gpx`
  a.click()
  URL.revokeObjectURL(url)
}
