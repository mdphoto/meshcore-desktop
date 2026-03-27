import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { useStore } from '../store'
import { tauri } from '../hooks/useTauri'
import { RefreshCw, Star, StarOff, MessageSquare, Search, QrCode, LogIn, Radio, Info } from 'lucide-react'
import { QrCodeModal } from '../components/QrCodeModal'
import { formatLastSeen } from '../utils/date'

const nodeTypeLabels: Record<number, string> = {
  0: 'Unknown',
  1: 'Client',
  2: 'Repeater',
  3: 'Room Server',
  4: 'Sensor',
}

export function ContactsView() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const contacts = useStore((s) => s.contacts)
  const setContacts = useStore((s) => s.setContacts)
  const selectContact = useStore((s) => s.selectContact)
  const setError = useStore((s) => s.setError)
  const status = useStore((s) => s.connectionStatus)
  const [search, setSearch] = useState('')
  const [syncing, setSyncing] = useState(false)
  const [filterType, setFilterType] = useState<number | null>(null)
  const [loginNode, setLoginNode] = useState<{ pubkey: string; name: string; nodeType: number; mode: 'admin' | 'chat' } | null>(null)
  const [adminPassword, setAdminPassword] = useState('hello')
  const [loginStatus, setLoginStatus] = useState('')
  const setAdminNode = useStore((s) => s.setAdminNode)
  const [infoNode, setInfoNode] = useState<string | null>(null)
  const [qrContact, setQrContact] = useState<{ name: string; pubkey: string } | null>(null)

  const [loading, setLoading] = useState(true)

  useEffect(() => {
    setLoading(true)
    tauri.getAllContacts()
      .then(setContacts)
      .catch(() => {})
      .finally(() => setLoading(false))
  }, [setContacts])

  const handleSync = async () => {
    setSyncing(true)
    try {
      await tauri.syncContacts()
      const updated = await tauri.getAllContacts()
      setContacts(updated)
    } catch (e) {
      setError(String(e))
    } finally {
      setSyncing(false)
    }
  }

  const handleToggleFav = async (pk: string, current: boolean) => {
    try {
      await tauri.toggleFavorite(pk, !current)
      const updated = await tauri.getAllContacts()
      setContacts(updated)
    } catch (e) {
      setError(String(e))
    }
  }

  const filtered = contacts
    .filter((c) => c.name.toLowerCase().includes(search.toLowerCase()))
    .filter((c) => filterType === null || c.node_type === filterType)

  const favorites = filtered.filter((c) => c.is_favorite)
  const others = filtered.filter((c) => !c.is_favorite)

  const doLogin = () => {
    if (!loginNode || !adminPassword) return
    setLoginStatus('Connexion...')
    if (loginNode.mode === 'chat') {
      // Mode chat : sendLogin puis naviguer vers le chat
      tauri.sendLogin(loginNode.pubkey, adminPassword)
        .then((r) => {
          setLoginStatus(r)
          setTimeout(() => { setLoginNode(null); setLoginStatus(''); selectContact(loginNode.pubkey); navigate('/chat') }, 800)
        })
        .catch((e) => setLoginStatus(String(e)))
    } else {
      // Mode admin : repeaterLogin puis naviguer vers la page repeater
      tauri.repeaterLogin(loginNode.pubkey, adminPassword)
        .then(() => {
          setLoginStatus('Connecté !')
          setAdminNode(loginNode.pubkey, true)
          setTimeout(() => { setLoginNode(null); setLoginStatus(''); navigate('/repeater') }, 500)
        })
        .catch((e) => setLoginStatus(String(e)))
    }
  }

  return (
    <div className="p-6">
      <div className="mb-4 flex items-center justify-between">
        <h2 className="text-xl font-semibold text-white">{t('contacts.title')}</h2>
        <button
          onClick={handleSync}
          disabled={syncing || status !== 'connected'}
          className="flex items-center gap-2 rounded-lg bg-gray-800 px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-700 disabled:opacity-50 transition-colors"
        >
          <RefreshCw className={`h-4 w-4 ${syncing ? 'animate-spin' : ''}`} />
          {t('contacts.sync')}
        </button>
      </div>

      <div className="relative mb-4">
        <Search className="absolute left-3 top-2.5 h-4 w-4 text-gray-500" />
        <input
          type="text"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder={t('contacts.search')}
          className="w-full rounded-lg border border-gray-700 bg-gray-800 py-2 pl-10 pr-3 text-sm text-white placeholder-gray-500 focus:border-emerald-500 focus:outline-none"
        />
      </div>

      {/* Filtres par type */}
      <div className="mb-3 flex flex-wrap gap-1.5">
        {([null, 1, 2, 3, 4, 0] as const).map((type) => {
          const label = type === null ? 'Tous' : nodeTypeLabels[type] ?? `Type ${type}`
          const count = type === null ? contacts.length : contacts.filter((c) => c.node_type === type).length
          if (type !== null && count === 0) return null
          return (
            <button
              key={String(type)}
              onClick={() => setFilterType(type)}
              className={`rounded-lg px-2.5 py-1 text-xs transition-colors ${
                filterType === type
                  ? 'bg-emerald-600 text-white'
                  : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
              }`}
            >
              {label} ({count})
            </button>
          )
        })}
      </div>

      {loading ? (
        <div className="flex items-center justify-center py-12 text-sm text-gray-500">
          <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
          {t('common.loading')}
        </div>
      ) : contacts.length === 0 ? (
        <p className="text-center text-sm text-gray-500 py-12">{t('contacts.noContacts')}</p>
      ) : (
        <div className="space-y-1">
          {[...favorites, ...others].map((c) => (
            <div
              key={c.public_key}
              className="relative flex items-center gap-3 rounded-lg border border-gray-800 bg-gray-900 px-4 py-3 hover:bg-gray-800/70 transition-colors"
            >
              <div
                className={`flex h-9 w-9 items-center justify-center rounded-full text-xs font-bold ${
                  c.node_type === 2
                    ? 'bg-blue-900 text-blue-300'
                    : c.node_type === 3
                      ? 'bg-purple-900 text-purple-300'
                      : c.node_type === 4
                        ? 'bg-orange-900 text-orange-300'
                        : 'bg-emerald-900 text-emerald-300'
                }`}
              >
                {c.name.slice(0, 2).toUpperCase()}
              </div>
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="truncate text-sm font-medium text-white">{c.name}</span>
                  <span className="rounded bg-gray-800 px-1.5 py-0.5 text-[10px] text-gray-400">
                    {nodeTypeLabels[c.node_type] ?? `Type ${c.node_type}`}
                  </span>
                </div>
                <span className="text-xs text-gray-500">
                  {t('contacts.lastSeen')} {formatLastSeen(c.last_seen)}
                  {c.path_len >= 0 && ` · ${c.path_len} hops`}
                </span>
              </div>
              <button
                onClick={() => handleToggleFav(c.public_key, c.is_favorite)}
                className="text-gray-500 hover:text-yellow-400 transition-colors"
              >
                {c.is_favorite ? <Star className="h-4 w-4 fill-yellow-400 text-yellow-400" /> : <StarOff className="h-4 w-4" />}
              </button>
              <button
                onClick={() => setQrContact({ name: c.name, pubkey: c.public_key })}
                className="text-gray-500 hover:text-blue-400 transition-colors"
                title="QR Code"
              >
                <QrCode className="h-4 w-4" />
              </button>
              {/* Bouton message pour les clients */}
              {(c.node_type === 1 || c.node_type === 0) && (
                <button
                  onClick={() => { selectContact(c.public_key); navigate('/chat') }}
                  className="text-gray-500 hover:text-emerald-400 transition-colors"
                  title={t('chat.direct')}
                >
                  <MessageSquare className="h-4 w-4" />
                </button>
              )}
              {/* Bouton chat room server → login requis */}
              {c.node_type === 3 && (
                <button
                  onClick={() => setLoginNode({ pubkey: c.public_key, name: c.name, nodeType: 3, mode: 'chat' })}
                  className="text-gray-500 hover:text-emerald-400 transition-colors"
                  title="Chat Room Server (login)"
                >
                  <MessageSquare className="h-4 w-4" />
                </button>
              )}
              {/* Bouton info publique pour repeaters et room servers */}
              {(c.node_type === 2 || c.node_type === 3) && (
                <button
                  onClick={() => setInfoNode(infoNode === c.public_key ? null : c.public_key)}
                  className="text-gray-500 hover:text-blue-400 transition-colors"
                  title="Infos publiques"
                >
                  <Info className="h-4 w-4" />
                </button>
              )}
              {/* Bouton administrer pour repeaters et room servers */}
              {(c.node_type === 2 || c.node_type === 3) && (
                <button
                  onClick={() => setLoginNode({ pubkey: c.public_key, name: c.name, nodeType: c.node_type, mode: 'admin' })}
                  className="text-gray-500 hover:text-amber-400 transition-colors"
                  title="Administrer"
                >
                  <Radio className="h-4 w-4" />
                </button>
              )}
              {/* Info publique dépliable */}
              {infoNode === c.public_key && (c.node_type === 2 || c.node_type === 3) && (
                <div className="absolute left-0 right-0 top-full z-10 mt-1 rounded-lg border border-gray-700 bg-gray-950 p-3 text-xs shadow-xl">
                  <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-gray-400">
                    <span>Type</span>
                    <span className="text-white">{nodeTypeLabels[c.node_type]}</span>
                    <span>Clé publique</span>
                    <span className="font-mono text-white">{c.public_key.slice(0, 24)}…</span>
                    <span>Hops</span>
                    <span className="text-white">{c.path_len >= 0 ? c.path_len : '—'}</span>
                    {(c.lat !== 0 || c.lon !== 0) && (<>
                      <span>Position</span>
                      <span className="text-white">{c.lat.toFixed(5)}, {c.lon.toFixed(5)}</span>
                    </>)}
                    <span>Dernier signal</span>
                    <span className="text-white">{formatLastSeen(c.last_seen)}</span>
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Modale login admin repeater/room server */}
      {loginNode && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" onClick={() => { setLoginNode(null); setLoginStatus('') }}>
          <div className="w-80 rounded-2xl border border-gray-700 bg-gray-900 p-6 shadow-xl" onClick={(e) => e.stopPropagation()}>
            <h3 className="mb-1 text-sm font-semibold text-white">
              {loginNode.mode === 'chat' ? 'Login Room Server' : `Administrer ${loginNode.nodeType === 2 ? 'le repeater' : 'le room server'}`}
            </h3>
            <p className="mb-4 text-xs text-gray-400">{loginNode.name}</p>
            <label className="block mb-3">
              <span className="mb-1 block text-xs font-medium text-gray-400">Mot de passe</span>
              <input
                type="password"
                value={adminPassword}
                onChange={(e) => setAdminPassword(e.target.value)}
                placeholder="hello"
                className={`w-full rounded-lg border border-gray-600 bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 focus:outline-none ${
                  loginNode.mode === 'chat' ? 'focus:border-purple-500' : 'focus:border-amber-500'
                }`}
                autoFocus
                onKeyDown={(e) => { if (e.key === 'Enter') doLogin() }}
              />
            </label>
            {loginStatus && (
              <p className={`mb-3 text-xs ${loginStatus.includes('échoué') || loginStatus.includes('Erreur') ? 'text-red-400' : 'text-emerald-400'}`}>
                {loginStatus}
              </p>
            )}
            <div className="flex gap-2">
              <button
                onClick={() => { setLoginNode(null); setLoginStatus('') }}
                className="flex-1 rounded-lg bg-gray-800 px-3 py-2 text-sm text-gray-300 hover:bg-gray-700 transition-colors"
              >
                Annuler
              </button>
              <button
                onClick={doLogin}
                className={`flex-1 rounded-lg px-3 py-2 text-sm text-white transition-colors ${
                  loginNode.mode === 'chat'
                    ? 'bg-purple-600 hover:bg-purple-500'
                    : 'bg-amber-600 hover:bg-amber-500'
                }`}
              >
                <LogIn className="mr-1 inline h-3.5 w-3.5" />
                {loginNode.mode === 'chat' ? 'Connexion' : 'Administrer'}
              </button>
            </div>
          </div>
        </div>
      )}

      {qrContact && (
        <QrCodeModal
          title={qrContact.name}
          data={`meshcore://contact/${qrContact.pubkey}?name=${encodeURIComponent(qrContact.name)}`}
          onClose={() => setQrContact(null)}
        />
      )}
    </div>
  )
}
