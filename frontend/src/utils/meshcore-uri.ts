export interface MeshCoreContactURI {
  type: 'contact'
  pubkey: string
  name?: string
}

export interface MeshCoreChannelURI {
  type: 'channel'
  idx: number
  name?: string
  psk?: string
}

export type MeshCoreURI = MeshCoreContactURI | MeshCoreChannelURI

export function parseMeshCoreURI(uri: string): MeshCoreURI | null {
  try {
    if (!uri.startsWith('meshcore://')) return null
    // Remplacer le scheme custom par https pour que URL parse correctement
    const url = new URL(uri.replace('meshcore://', 'https://meshcore/'))
    const path = url.pathname.replace(/^\//, '')
    const parts = path.split('/')
    const params = url.searchParams

    if (parts[0] === 'contact' && parts[1]) {
      return {
        type: 'contact',
        pubkey: parts[1],
        name: params.get('name') ?? undefined,
      }
    }

    if (parts[0] === 'channel' && parts[1]) {
      return {
        type: 'channel',
        idx: parseInt(parts[1], 10),
        name: params.get('name') ?? undefined,
        psk: params.get('psk') ?? undefined,
      }
    }

    return null
  } catch {
    return null
  }
}

export function buildContactURI(pubkey: string, name?: string): string {
  const base = `meshcore://contact/${pubkey}`
  return name ? `${base}?name=${encodeURIComponent(name)}` : base
}

export function buildChannelURI(idx: number, name?: string, psk?: string): string {
  const params = new URLSearchParams()
  if (name) params.set('name', name)
  if (psk) params.set('psk', psk)
  const qs = params.toString()
  return `meshcore://channel/${idx}${qs ? `?${qs}` : ''}`
}
