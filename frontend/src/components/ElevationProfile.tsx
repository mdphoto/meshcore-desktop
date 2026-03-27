import { useMemo } from 'react'
import type { LosResult } from '../types'
import { formatDistance } from '../utils/geo'

interface Props {
  result: LosResult
  onClose: () => void
}

export function ElevationProfile({ result, onClose }: Props) {
  const { profile, visible, distance_m, min_fresnel_clearance_m, worst_obstruction } = result

  // Calcul des bornes pour le SVG
  const { minElev, maxElev, width, height, padding } = useMemo(() => {
    const elevs = profile.flatMap((p) => [p.terrain_m, p.los_m])
    const min = Math.min(...elevs) - 20
    const max = Math.max(...elevs) + 20
    return { minElev: min, maxElev: max, width: 700, height: 200, padding: { top: 10, right: 20, bottom: 30, left: 50 } }
  }, [profile])

  const chartW = width - padding.left - padding.right
  const chartH = height - padding.top - padding.bottom

  const xScale = (d: number) => padding.left + (d / distance_m) * chartW
  const yScale = (e: number) => padding.top + chartH - ((e - minElev) / (maxElev - minElev)) * chartH

  // Chemin du terrain (rempli)
  const terrainPath = profile
    .map((p, i) => `${i === 0 ? 'M' : 'L'}${xScale(p.distance_m).toFixed(1)},${yScale(p.terrain_m).toFixed(1)}`)
    .join(' ')
  const terrainFill = `${terrainPath} L${xScale(distance_m).toFixed(1)},${yScale(minElev).toFixed(1)} L${xScale(0).toFixed(1)},${yScale(minElev).toFixed(1)} Z`

  // Ligne de vue directe
  const losPath = profile
    .map((p, i) => `${i === 0 ? 'M' : 'L'}${xScale(p.distance_m).toFixed(1)},${yScale(p.los_m).toFixed(1)}`)
    .join(' ')

  // Zone de Fresnel (bande autour de la LOS)
  const fresnelUpper = profile
    .map((p) => `${xScale(p.distance_m).toFixed(1)},${yScale(p.los_m + p.fresnel_radius_m).toFixed(1)}`)
    .join(' ')
  const fresnelLower = [...profile]
    .reverse()
    .map((p) => `${xScale(p.distance_m).toFixed(1)},${yScale(p.los_m - p.fresnel_radius_m).toFixed(1)}`)
    .join(' ')

  // Axes Y : quelques repères d'altitude
  const yTicks = useMemo(() => {
    const range = maxElev - minElev
    const step = range > 500 ? 200 : range > 200 ? 100 : range > 50 ? 25 : 10
    const ticks: number[] = []
    let v = Math.ceil(minElev / step) * step
    while (v <= maxElev) {
      ticks.push(v)
      v += step
    }
    return ticks
  }, [minElev, maxElev])

  return (
    <div className="absolute bottom-3 left-3 right-3 z-[1000] rounded-lg border border-gray-700 bg-gray-900/95 p-3 backdrop-blur">
      <div className="mb-2 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <span className="text-sm font-medium text-white">
            Profil d'élévation — {formatDistance(distance_m)}
          </span>
          <span className={`rounded px-2 py-0.5 text-xs font-medium ${visible ? 'bg-green-600/20 text-green-400' : 'bg-red-600/20 text-red-400'}`}>
            {visible ? '✓ Visible' : '✗ Obstrué'}
          </span>
          <span className="text-xs text-gray-400">
            Fresnel : {min_fresnel_clearance_m >= 0 ? '+' : ''}{min_fresnel_clearance_m.toFixed(1)} m
          </span>
        </div>
        <button onClick={onClose} className="text-gray-400 hover:text-white text-lg leading-none">×</button>
      </div>

      <svg width={width} height={height} className="w-full" viewBox={`0 0 ${width} ${height}`}>
        {/* Grille Y */}
        {yTicks.map((tick) => (
          <g key={tick}>
            <line x1={padding.left} y1={yScale(tick)} x2={width - padding.right} y2={yScale(tick)} stroke="#374151" strokeWidth={0.5} />
            <text x={padding.left - 4} y={yScale(tick) + 3} textAnchor="end" fill="#9ca3af" fontSize={9}>
              {tick}m
            </text>
          </g>
        ))}

        {/* Axe X : distance */}
        <text x={padding.left} y={height - 4} fill="#9ca3af" fontSize={9}>0</text>
        <text x={width - padding.right} y={height - 4} textAnchor="end" fill="#9ca3af" fontSize={9}>
          {formatDistance(distance_m)}
        </text>

        {/* Zone de Fresnel */}
        <polygon points={`${fresnelUpper} ${fresnelLower}`} fill="rgba(59,130,246,0.1)" />

        {/* Terrain */}
        <path d={terrainFill} fill="rgba(34,197,94,0.3)" />
        <path d={terrainPath} fill="none" stroke="#22c55e" strokeWidth={1.5} />

        {/* Ligne de vue */}
        <path d={losPath} fill="none" stroke={visible ? '#3b82f6' : '#ef4444'} strokeWidth={1.5} strokeDasharray="6 3" />

        {/* Point d'obstruction */}
        {worst_obstruction && (
          <circle
            cx={xScale(worst_obstruction.distance_m)}
            cy={yScale(worst_obstruction.terrain_m)}
            r={4}
            fill="#ef4444"
            stroke="white"
            strokeWidth={1}
          />
        )}
      </svg>
    </div>
  )
}
