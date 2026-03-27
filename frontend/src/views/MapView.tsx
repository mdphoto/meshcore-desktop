import { useMemo, useState, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { useStore } from '../store'
import { MapPin, Download, Layers, Eye } from 'lucide-react'
import { downloadGpx } from '../utils/gpx'
import { formatLastSeen } from '../utils/date'
import { toMGRS, haversineDistance, formatDistance } from '../utils/geo'
import { MapContainer, TileLayer, Marker, Popup, Polyline, useMap } from 'react-leaflet'
import { ElevationProfile } from '../components/ElevationProfile'
import L from 'leaflet'
import 'leaflet/dist/leaflet.css'
import type { StoredContact, LosResult } from '../types'
import { invoke } from '@tauri-apps/api/core'

function createIcon(nodeType: number, selected?: boolean): L.Icon {
  const colors: Record<number, string> = {
    0: '#6b7280', // unknown - gray
    1: '#10b981', // client - emerald
    2: '#3b82f6', // repeater - blue
    3: '#8b5cf6', // room server - purple
    4: '#f97316', // sensor - orange
  }
  const color = colors[nodeType] ?? '#6b7280'
  const stroke = selected ? 'stroke="#fbbf24" stroke-width="2"' : ''
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="28" height="36" viewBox="0 0 28 36">
    <path d="M14 0C6.268 0 0 6.268 0 14c0 10.5 14 22 14 22s14-11.5 14-22C28 6.268 21.732 0 14 0z" fill="${color}" ${stroke}/>
    <circle cx="14" cy="13" r="6" fill="white" opacity="0.9"/>
  </svg>`
  return L.icon({
    iconUrl: `data:image/svg+xml;base64,${btoa(svg)}`,
    iconSize: [28, 36],
    iconAnchor: [14, 36],
    popupAnchor: [0, -36],
  })
}

function FitBounds({ nodes }: { nodes: StoredContact[] }) {
  const map = useMap()
  const mapFocus = useStore((s) => s.mapFocus)
  const setMapFocus = useStore((s) => s.setMapFocus)

  useMemo(() => {
    if (mapFocus) {
      map.setView([mapFocus.lat, mapFocus.lon], 14)
      setMapFocus(null)
    } else if (nodes.length > 0) {
      const bounds = L.latLngBounds(nodes.map((n) => [n.lat, n.lon] as [number, number]))
      map.fitBounds(bounds, { padding: [50, 50], maxZoom: 14 })
    }
  }, [nodes, map, mapFocus, setMapFocus])
  return null
}

const nodeTypeLabels: Record<number, string> = {
  0: 'Unknown',
  1: 'Client',
  2: 'Repeater',
  3: 'Room Server',
  4: 'Sensor',
}

type TileSource = 'osm' | 'topo' | 'satellite'

const tileSources: Record<TileSource, { url: string; attribution: string; name: string }> = {
  osm: { url: 'https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', attribution: '&copy; <a href="https://osm.org/copyright">OSM</a>', name: 'OpenStreetMap' },
  topo: { url: 'https://{s}.tile.opentopomap.org/{z}/{x}/{y}.png', attribution: '&copy; <a href="https://opentopomap.org">OpenTopoMap</a>', name: 'Topographique' },
  satellite: { url: 'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}', attribution: '&copy; Esri', name: 'Satellite' },
}

export function MapView() {
  const { t } = useTranslation()
  const contacts = useStore((s) => s.contacts)
  const deviceInfo = useStore((s) => s.deviceInfo)
  const [tileSource, setTileSource] = useState<TileSource>('osm')

  // LOS analysis state
  const [losMode, setLosMode] = useState(false)
  const [losNodeA, setLosNodeA] = useState<string | null>(null)
  const [losResult, setLosResult] = useState<LosResult | null>(null)
  const [losLoading, setLosLoading] = useState(false)

  const nodes = contacts.filter((c) => c.lat !== 0 || c.lon !== 0)

  const allNodes = useMemo(() => {
    const list = [...nodes]
    if (deviceInfo && (deviceInfo.lat !== 0 || deviceInfo.lon !== 0)) {
      const selfExists = list.some((n) => n.public_key === deviceInfo.public_key)
      if (!selfExists) {
        list.push({
          public_key: deviceInfo.public_key,
          name: `${deviceInfo.name} (moi)`,
          node_type: deviceInfo.adv_type,
          flags: 0,
          path: [],
          path_len: 0,
          lat: deviceInfo.lat,
          lon: deviceInfo.lon,
          last_seen: new Date().toISOString(),
          is_favorite: false,
          group_name: null,
        })
      }
    }
    return list
  }, [nodes, deviceInfo])

  // Fréquence radio en MHz pour le calcul Fresnel
  const freqMhz = useMemo(() => {
    if (!deviceInfo?.radio_freq) return 915.0
    // radio_freq est en Hz dans meshcore-rs
    return deviceInfo.radio_freq > 1_000_000 ? deviceInfo.radio_freq / 1_000_000 : deviceInfo.radio_freq
  }, [deviceInfo])

  const handleNodeClick = useCallback(async (pubkey: string) => {
    if (!losMode) return

    if (!losNodeA) {
      setLosNodeA(pubkey)
      return
    }

    // 2ème nœud sélectionné → lancer l'analyse
    const nodeA = allNodes.find((n) => n.public_key === losNodeA)
    const nodeB = allNodes.find((n) => n.public_key === pubkey)
    if (!nodeA || !nodeB || nodeA.public_key === nodeB.public_key) {
      setLosNodeA(null)
      return
    }

    setLosLoading(true)
    try {
      const result = await invoke<LosResult>('analyze_los', {
        lat1: nodeA.lat,
        lon1: nodeA.lon,
        lat2: nodeB.lat,
        lon2: nodeB.lon,
        freqMhz: freqMhz,
        antHeight1: 2.0, // hauteur antenne par défaut
        antHeight2: 2.0,
      })
      setLosResult(result)
    } catch (e) {
      console.error('Erreur analyse LOS:', e)
    } finally {
      setLosLoading(false)
      setLosNodeA(null)
    }
  }, [losMode, losNodeA, allNodes, freqMhz])

  const toggleLosMode = () => {
    if (losMode) {
      setLosMode(false)
      setLosNodeA(null)
      setLosResult(null)
    } else {
      setLosMode(true)
      setLosResult(null)
      setLosNodeA(null)
    }
  }

  // Ligne entre les 2 nœuds analysés
  const losLine = useMemo(() => {
    if (!losResult) return null
    const points = losResult.profile
    return points.map((p) => [p.lat, p.lon] as [number, number])
  }, [losResult])

  if (allNodes.length === 0) {
    return (
      <div className="flex h-full flex-col items-center justify-center text-gray-500">
        <MapPin className="mb-3 h-12 w-12" />
        <p>{t('map.noNodes')}</p>
      </div>
    )
  }

  const center: [number, number] = [allNodes[0].lat, allNodes[0].lon]

  return (
    <div className="relative h-full w-full">
      {/* Légende */}
      <div className="absolute right-3 top-3 z-[1000] rounded-lg border border-gray-700 bg-gray-900/90 px-3 py-2 text-xs backdrop-blur">
        <div className="mb-1 font-medium text-white">Légende</div>
        {Object.entries(nodeTypeLabels).map(([type, label]) => (
          <div key={type} className="flex items-center gap-2 text-gray-300">
            <span
              className="inline-block h-2.5 w-2.5 rounded-full"
              style={{ backgroundColor: { '0': '#6b7280', '1': '#10b981', '2': '#3b82f6', '3': '#8b5cf6', '4': '#f97316' }[type] ?? '#6b7280' }}
            />
            {label} ({allNodes.filter((n) => n.node_type === Number(type)).length})
          </div>
        ))}
        <div className="mt-1 text-gray-500">{allNodes.length} nœuds</div>

        {/* Bouton analyse LOS */}
        <button
          onClick={toggleLosMode}
          className={`mt-2 flex w-full items-center justify-center gap-1.5 rounded px-2 py-1 transition-colors ${
            losMode
              ? 'bg-amber-600 text-white'
              : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
          }`}
        >
          <Eye className="h-3 w-3" />
          {losMode ? (losNodeA ? 'Cliquez nœud B' : 'Cliquez nœud A') : 'Ligne de vue'}
        </button>
        {losLoading && (
          <div className="mt-1 text-center text-amber-400 animate-pulse">Analyse...</div>
        )}

        <button
          onClick={() => downloadGpx(allNodes, deviceInfo?.name)}
          className="mt-2 flex w-full items-center justify-center gap-1.5 rounded bg-gray-700 px-2 py-1 text-gray-300 hover:bg-gray-600 transition-colors"
        >
          <Download className="h-3 w-3" />
          Export GPX
        </button>
        <div className="mt-2 space-y-0.5">
          {(Object.entries(tileSources) as [TileSource, typeof tileSources.osm][]).map(([key, src]) => (
            <button
              key={key}
              onClick={() => setTileSource(key)}
              className={`flex w-full items-center gap-1.5 rounded px-2 py-0.5 transition-colors ${
                tileSource === key ? 'bg-emerald-600 text-white' : 'text-gray-400 hover:bg-gray-700'
              }`}
            >
              <Layers className="h-3 w-3" />
              {src.name}
            </button>
          ))}
        </div>
      </div>

      <MapContainer center={center} zoom={10} className="h-full w-full">
        <TileLayer
          key={tileSource}
          attribution={tileSources[tileSource].attribution}
          url={tileSources[tileSource].url}
        />
        <FitBounds nodes={allNodes} />

        {/* Ligne LOS entre les 2 nœuds */}
        {losLine && (
          <Polyline
            positions={losLine}
            pathOptions={{
              color: losResult?.visible ? '#22c55e' : '#ef4444',
              weight: 3,
              dashArray: losResult?.visible ? undefined : '8 4',
              opacity: 0.8,
            }}
          />
        )}

        {allNodes.map((node) => (
          <Marker
            key={node.public_key}
            position={[node.lat, node.lon]}
            icon={createIcon(node.node_type, losNodeA === node.public_key)}
            eventHandlers={{
              click: () => {
                if (losMode) handleNodeClick(node.public_key)
              },
            }}
          >
            <Popup>
              <div className="text-sm min-w-[180px]">
                <div className="font-bold">{node.name}</div>
                <div className="text-gray-600">{nodeTypeLabels[node.node_type] ?? `Type ${node.node_type}`}</div>
                {node.path_len >= 0 && <div>{node.path_len} hop{node.path_len !== 1 ? 's' : ''}</div>}
                <div className="text-xs text-gray-500 mt-1">
                  {node.lat.toFixed(5)}, {node.lon.toFixed(5)}
                </div>
                <div className="text-xs font-mono text-gray-500">
                  MGRS: {toMGRS(node.lat, node.lon)}
                </div>
                {deviceInfo && (deviceInfo.lat !== 0 || deviceInfo.lon !== 0) && node.public_key !== deviceInfo.public_key && (
                  <div className="text-xs text-blue-500 font-medium">
                    {formatDistance(haversineDistance(deviceInfo.lat, deviceInfo.lon, node.lat, node.lon))}
                  </div>
                )}
                <div className="text-xs text-gray-400">
                  Vu le {formatLastSeen(node.last_seen)}
                </div>
                {/* Bouton LOS rapide vers ce nœud depuis le device */}
                {!losMode && deviceInfo && (deviceInfo.lat !== 0 || deviceInfo.lon !== 0) && node.public_key !== deviceInfo.public_key && (
                  <button
                    className="mt-1 w-full rounded bg-blue-600 px-2 py-0.5 text-xs text-white hover:bg-blue-500"
                    onClick={async (e) => {
                      e.stopPropagation()
                      setLosLoading(true)
                      try {
                        const result = await invoke<LosResult>('analyze_los', {
                          lat1: deviceInfo.lat,
                          lon1: deviceInfo.lon,
                          lat2: node.lat,
                          lon2: node.lon,
                          freqMhz: freqMhz,
                          antHeight1: 2.0,
                          antHeight2: 2.0,
                        })
                        setLosResult(result)
                      } catch (e) {
                        console.error('Erreur LOS:', e)
                      } finally {
                        setLosLoading(false)
                      }
                    }}
                  >
                    <Eye className="inline h-3 w-3 mr-1" />
                    Ligne de vue
                  </button>
                )}
              </div>
            </Popup>
          </Marker>
        ))}
      </MapContainer>

      {/* Profil d'élévation */}
      {losResult && (
        <ElevationProfile result={losResult} onClose={() => setLosResult(null)} />
      )}
    </div>
  )
}
