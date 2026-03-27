import { useState, useCallback, useEffect, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
import { useStore } from '../store'
import { tauri } from '../hooks/useTauri'
import type { RepeaterNeighbour, StoredContact } from '../types'
import {
  Repeat, Activity, Radio, Signal, MapPin, LogIn, LogOut,
  Clock, Users, Send, Loader2, AlertCircle,
  Terminal, Settings, BarChart3, Shield, ChevronRight,
} from 'lucide-react'

type Tab = 'status' | 'config' | 'neighbours' | 'terminal'

// Commandes rapides pour le terminal
const QUICK_COMMANDS = [
  { label: 'Version', cmd: 'ver' },
  { label: 'Nom', cmd: 'get name' },
  { label: 'Radio', cmd: 'get radio' },
  { label: 'TX', cmd: 'get tx' },
  { label: 'Horloge', cmd: 'clock' },
  { label: 'Advert', cmd: 'advert' },
  { label: 'Stats', cmd: 'clear stats' },
  { label: 'Voisins', cmd: 'neighbors' },
  { label: 'GPS on', cmd: 'gps on' },
  { label: 'GPS off', cmd: 'gps off' },
  { label: 'GPS sync', cmd: 'gps sync' },
  { label: 'Log start', cmd: 'log start' },
  { label: 'Log stop', cmd: 'log stop' },
  { label: 'Log erase', cmd: 'log erase' },
  { label: 'Reboot', cmd: 'reboot' },
]

export function RepeaterView() {
  const { t } = useTranslation()
  const status = useStore((s) => s.connectionStatus)
  const radioStats = useStore((s) => s.radioStats)
  const contacts = useStore((s) => s.contacts)
  const navigate = useNavigate()
  const setMapFocus = useStore((s) => s.setMapFocus)

  const repeaters = contacts.filter((c) => c.node_type === 2)
  const roomServers = contacts.filter((c) => c.node_type === 3)
  const adminNodes = [...repeaters, ...roomServers]
  const latest = radioStats.length > 0 ? radioStats[radioStats.length - 1] : null

  // Admin state
  const adminNodePubkey = useStore((s) => s.adminNode)
  const adminLoggedIn = useStore((s) => s.adminLoggedIn)
  const setAdminNode = useStore((s) => s.setAdminNode)
  const setAdminLoggedIn = useStore((s) => s.setAdminLoggedIn)

  const selectedNode = adminNodePubkey ? contacts.find((c) => c.public_key === adminNodePubkey) ?? null : null
  const [loginPassword, setLoginPassword] = useState('')
  const loggedIn = adminLoggedIn

  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  // Statut via CLI (clé/valeur brutes des réponses)
  const [cliStatus, setCliStatus] = useState<Record<string, string>>({})
  const [neighbours, setNeighbours] = useState<RepeaterNeighbour[]>([])
  const [acl, setAcl] = useState<{ pubkey_hex: string; permissions: number; name: string | null }[]>([])
  const [tab, setTab] = useState<Tab>('status')

  // Terminal state
  const [cliInput, setCliInput] = useState('')
  const [cliHistory, setCliHistory] = useState<{ cmd: string; response: string; time: string }[]>([])
  const [cliLoading, setCliLoading] = useState(false)
  const cliEndRef = useRef<HTMLDivElement>(null)
  const [cmdHistoryIdx, setCmdHistoryIdx] = useState(-1)

  const handleLogin = useCallback(async () => {
    if (!selectedNode || !loginPassword) return
    setLoading(true)
    setError(null)
    try {
      await tauri.repeaterLogin(selectedNode.public_key, loginPassword)
      setAdminLoggedIn(true)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }, [selectedNode, loginPassword, setAdminLoggedIn])

  const handleLogout = useCallback(async () => {
    if (!selectedNode) return
    try { await tauri.repeaterLogout(selectedNode.public_key) } catch {}
    setAdminLoggedIn(false)
    setCliStatus({})
    setNeighbours([])
    setAcl([])
    setCliHistory([])
  }, [selectedNode, setAdminLoggedIn])

  // Rafraîchir via commandes CLI (plus fiable que le protocole binaire)
  const refreshAll = useCallback(async () => {
    if (!selectedNode) return
    setLoading(true)
    setError(null)
    const status: Record<string, string> = {}

    const cmds = ['ver', 'get name', 'get radio', 'get tx', 'clock', 'neighbors']
    for (const cmd of cmds) {
      try {
        const resp = await tauri.repeaterSendCli(selectedNode.public_key, cmd)
        status[cmd] = resp
      } catch (e) {
        console.warn(`[repeater] CLI "${cmd}" failed:`, e)
        status[cmd] = `Erreur: ${e}`
      }
    }
    setCliStatus(status)

    // Essayer aussi les requêtes binaires en arrière-plan (elles marchent sur certains firmwares)
    tauri.repeaterNeighbours(selectedNode.public_key, 50, 0)
      .then(setNeighbours)
      .catch(() => {})
    tauri.repeaterAcl(selectedNode.public_key)
      .then(setAcl)
      .catch(() => {})

    setLoading(false)
  }, [selectedNode])

  // Charger automatiquement après login
  useEffect(() => {
    if (selectedNode && loggedIn && Object.keys(cliStatus).length === 0 && !loading) {
      refreshAll()
    }
  }, [selectedNode?.public_key, loggedIn])

  // Auto-scroll terminal
  useEffect(() => {
    cliEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [cliHistory.length])

  const handleSelectNode = (node: StoredContact) => {
    setAdminNode(node.public_key, false)
    setCliStatus({})
    setNeighbours([])
    setAcl([])
    setCliHistory([])
    setError(null)
    setLoginPassword('')
    setTab('status')
  }

  const sendCli = useCallback(async (cmd: string) => {
    if (!selectedNode || !cmd.trim()) return
    setCliLoading(true)
    const time = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
    try {
      const response = await tauri.repeaterSendCli(selectedNode.public_key, cmd.trim())
      setCliHistory((h) => [...h, { cmd: cmd.trim(), response, time }])
    } catch (e) {
      setCliHistory((h) => [...h, { cmd: cmd.trim(), response: `ERREUR: ${e}`, time }])
    } finally {
      setCliLoading(false)
      setCliInput('')
      setCmdHistoryIdx(-1)
    }
  }, [selectedNode])

  const handleCliKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      sendCli(cliInput)
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      const cmds = cliHistory.map((h) => h.cmd)
      const newIdx = Math.min(cmdHistoryIdx + 1, cmds.length - 1)
      setCmdHistoryIdx(newIdx)
      if (cmds.length > 0) setCliInput(cmds[cmds.length - 1 - newIdx])
    } else if (e.key === 'ArrowDown') {
      e.preventDefault()
      const cmds = cliHistory.map((h) => h.cmd)
      const newIdx = cmdHistoryIdx - 1
      if (newIdx < 0) { setCmdHistoryIdx(-1); setCliInput('') }
      else { setCmdHistoryIdx(newIdx); setCliInput(cmds[cmds.length - 1 - newIdx]) }
    }
  }

  if (status !== 'connected') {
    return (
      <div className="flex h-full flex-col items-center justify-center text-gray-500">
        <Repeat className="mb-3 h-12 w-12" />
        <p className="text-sm">{t('repeater.noData')}</p>
      </div>
    )
  }

  return (
    <div className="flex h-full">
      {/* Panneau gauche */}
      <div className="w-64 shrink-0 overflow-y-auto border-r border-gray-800 bg-gray-950 p-4">
        <h3 className="mb-3 text-sm font-medium text-gray-400">
          <Repeat className="mr-1 inline h-4 w-4" />
          Nœuds ({adminNodes.length})
        </h3>
        {adminNodes.length === 0 ? (
          <p className="text-xs text-gray-600">Aucun repeater découvert</p>
        ) : (
          <div className="space-y-1">
            {adminNodes.map((node) => (
              <button
                key={node.public_key}
                onClick={() => handleSelectNode(node)}
                className={`flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left transition-colors ${
                  selectedNode?.public_key === node.public_key
                    ? 'bg-blue-600/20 text-blue-300'
                    : 'text-gray-300 hover:bg-gray-800'
                }`}
              >
                <div className={`flex h-7 w-7 shrink-0 items-center justify-center rounded-full text-xs font-bold ${
                  node.node_type === 2 ? 'bg-blue-900 text-blue-300' : 'bg-purple-900 text-purple-300'
                }`}>
                  {node.node_type === 2 ? 'R' : 'S'}
                </div>
                <div className="min-w-0 flex-1">
                  <div className="truncate text-sm">{node.name || 'Sans nom'}</div>
                  <div className="text-xs text-gray-500">{node.path_len >= 0 ? `${node.path_len}h` : ''}</div>
                </div>
              </button>
            ))}
          </div>
        )}

        {latest && (
          <div className="mt-6 space-y-1 text-xs text-gray-400">
            <div className="mb-1 text-gray-500 font-medium">Radio locale</div>
            <div className="flex justify-between"><span>Noise</span><span className="text-white">{latest.noise_floor} dBm</span></div>
            <div className="flex justify-between"><span>RSSI</span><span className="text-white">{latest.last_rssi} dBm</span></div>
            <div className="flex justify-between"><span>SNR</span><span className="text-white">{latest.snr.toFixed(1)} dB</span></div>
          </div>
        )}
      </div>

      {/* Panneau droit */}
      <div className="flex flex-1 flex-col overflow-hidden">
        {!selectedNode ? (
          <div className="flex h-full flex-col items-center justify-center text-gray-500">
            <Radio className="mb-3 h-12 w-12" />
            <p>Sélectionnez un nœud à administrer</p>
          </div>
        ) : !loggedIn ? (
          /* Login */
          <div className="flex h-full flex-col items-center justify-center">
            <div className="w-80 rounded-2xl border border-gray-800 bg-gray-900 p-6">
              <div className="mb-4 flex items-center gap-3">
                <div className={`flex h-10 w-10 items-center justify-center rounded-full text-sm font-bold ${
                  selectedNode.node_type === 2 ? 'bg-blue-900 text-blue-300' : 'bg-purple-900 text-purple-300'
                }`}>{selectedNode.node_type === 2 ? 'R' : 'S'}</div>
                <div>
                  <div className="font-medium text-white">{selectedNode.name}</div>
                  <div className="text-xs text-gray-500">{selectedNode.node_type === 2 ? 'Repeater' : 'Room Server'}</div>
                </div>
              </div>
              {error && <p className="mb-3 text-xs text-red-400"><AlertCircle className="mr-1 inline h-3 w-3" />{error}</p>}
              <div className="flex gap-2">
                <input
                  type="password"
                  placeholder="Mot de passe admin"
                  value={loginPassword}
                  onChange={(e) => setLoginPassword(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleLogin()}
                  className="flex-1 rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 focus:border-amber-500 focus:outline-none"
                  autoFocus
                />
                <button onClick={handleLogin} disabled={loading || !loginPassword}
                  className="rounded-lg bg-amber-600 px-4 py-2 text-sm text-white hover:bg-amber-500 disabled:opacity-50">
                  {loading ? <Loader2 className="h-4 w-4 animate-spin" /> : <LogIn className="h-4 w-4" />}
                </button>
              </div>
            </div>
          </div>
        ) : (
          <>
            {/* Header + onglets */}
            <div className="border-b border-gray-800 px-4 py-2">
              <div className="mb-2 flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <span className="font-medium text-white">{selectedNode.name}</span>
                  <span className="rounded bg-green-600/20 px-2 py-0.5 text-xs text-green-400">Admin</span>
                </div>
                <div className="flex items-center gap-2">
                  <button onClick={refreshAll} disabled={loading}
                    className="flex items-center gap-1.5 rounded-lg bg-blue-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-blue-500 disabled:opacity-50 transition-colors">
                    {loading ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <Activity className="h-3.5 w-3.5" />}
                    Rafraîchir
                  </button>
                  {(selectedNode.lat !== 0 || selectedNode.lon !== 0) && (
                    <button onClick={() => { setMapFocus({ lat: selectedNode.lat, lon: selectedNode.lon }); navigate('/map') }}
                      className="rounded-lg bg-gray-800 px-2 py-1.5 text-gray-400 hover:text-blue-400 transition-colors"><MapPin className="h-4 w-4" /></button>
                  )}
                  <button onClick={handleLogout}
                    className="flex items-center gap-1.5 rounded-lg bg-red-900/30 px-3 py-1.5 text-xs text-red-400 hover:bg-red-900/50 transition-colors">
                    <LogOut className="h-3.5 w-3.5" />Déconnexion
                  </button>
                </div>
              </div>
              <div className="flex gap-1">
                {([
                  { id: 'status' as Tab, icon: BarChart3, label: 'Statut' },
                  { id: 'config' as Tab, icon: Settings, label: 'Configuration' },
                  { id: 'neighbours' as Tab, icon: Users, label: `Voisins (${neighbours.length})` },
                  { id: 'terminal' as Tab, icon: Terminal, label: 'Terminal' },
                ]).map((t) => (
                  <button
                    key={t.id}
                    onClick={() => setTab(t.id)}
                    className={`flex items-center gap-1.5 rounded-t-lg px-3 py-1.5 text-xs transition-colors ${
                      tab === t.id ? 'bg-gray-800 text-white' : 'text-gray-500 hover:text-gray-300'
                    }`}
                  >
                    <t.icon className="h-3.5 w-3.5" />{t.label}
                  </button>
                ))}
              </div>
            </div>

            {/* Contenu des onglets */}
            <div className="flex-1 overflow-y-auto p-4">
              {error && <p className="mb-3 text-sm text-red-400"><AlertCircle className="mr-1 inline h-4 w-4" />{error}</p>}

              {/* ONGLET STATUT */}
              {tab === 'status' && Object.keys(cliStatus).length > 0 && (
                <div className="space-y-4">
                  <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                    {cliStatus['ver'] && (
                      <StatCard icon={<Activity className="h-4 w-4 text-blue-400" />} label="Firmware" value={cliStatus['ver']} />
                    )}
                    {cliStatus['get name'] && (
                      <StatCard icon={<Radio className="h-4 w-4 text-emerald-400" />} label="Nom" value={cliStatus['get name']} />
                    )}
                    {cliStatus['get tx'] && (
                      <StatCard icon={<Send className="h-4 w-4 text-amber-400" />} label="Puissance TX" value={cliStatus['get tx']} />
                    )}
                    {cliStatus['get radio'] && (
                      <StatCard icon={<Signal className="h-4 w-4 text-purple-400" />} label="Radio" value={cliStatus['get radio']} />
                    )}
                    {cliStatus['clock'] && (
                      <StatCard icon={<Clock className="h-4 w-4 text-gray-400" />} label="Horloge" value={cliStatus['clock']} />
                    )}
                  </div>

                  {/* Voisins (depuis la commande CLI neighbors) */}
                  {cliStatus['neighbors'] && (
                    <div className="rounded-xl border border-gray-800 bg-gray-900 p-4">
                      <h4 className="mb-2 flex items-center gap-2 text-sm font-medium text-gray-300">
                        <Users className="h-4 w-4" />Voisins (réponse CLI)
                      </h4>
                      <pre className="whitespace-pre-wrap text-xs text-gray-400 font-mono">{cliStatus['neighbors']}</pre>
                    </div>
                  )}

                  {/* Réponses brutes pour debug */}
                  <details className="rounded-xl border border-gray-800 bg-gray-900">
                    <summary className="cursor-pointer px-4 py-2 text-xs text-gray-500 hover:text-gray-300">
                      Réponses brutes (debug)
                    </summary>
                    <div className="border-t border-gray-800 px-4 py-3 space-y-2">
                      {Object.entries(cliStatus).map(([cmd, resp]) => (
                        <div key={cmd}>
                          <div className="text-xs font-mono text-emerald-400">&gt; {cmd}</div>
                          <div className="text-xs font-mono text-gray-400 whitespace-pre-wrap">{resp}</div>
                        </div>
                      ))}
                    </div>
                  </details>
                </div>
              )}
              {tab === 'status' && Object.keys(cliStatus).length === 0 && (
                <div className="flex flex-col items-center justify-center py-12">
                  {loading ? (
                    <div className="flex items-center gap-2 text-sm text-gray-400">
                      <Loader2 className="h-5 w-5 animate-spin text-blue-400" />
                      Interrogation du repeater via CLI...
                    </div>
                  ) : (
                    <>
                      <BarChart3 className="mb-3 h-10 w-10 text-gray-600" />
                      <p className="mb-3 text-sm text-gray-500">Aucune donnée reçue du repeater</p>
                      <button onClick={refreshAll}
                        className="flex items-center gap-2 rounded-lg bg-blue-600 px-4 py-2 text-sm text-white hover:bg-blue-500 transition-colors">
                        <Activity className="h-4 w-4" />
                        Rafraîchir le statut
                      </button>
                    </>
                  )}
                </div>
              )}

              {/* ONGLET CONFIGURATION */}
              {tab === 'config' && (
                <div className="space-y-4">
                  <p className="text-xs text-gray-500">Utilisez le terminal pour configurer le repeater. Commandes courantes :</p>
                  <div className="grid gap-2 sm:grid-cols-2">
                    {[
                      { label: 'Changer le nom', cmd: 'set name MonRepeater' },
                      { label: 'Mot de passe admin', cmd: 'password NouveauMDP' },
                      { label: 'Mot de passe invité', cmd: 'set guest.password hello' },
                      { label: 'Radio (freq,bw,sf,cr)', cmd: 'set radio 869.525,125,9,5' },
                      { label: 'Puissance TX (dBm)', cmd: 'set tx 20' },
                      { label: 'Activer répétition', cmd: 'set repeat on' },
                      { label: 'Désactiver répétition', cmd: 'set repeat off' },
                      { label: 'Latitude', cmd: 'set lat 44.7500' },
                      { label: 'Longitude', cmd: 'set lon 4.8500' },
                      { label: 'Flood max hops', cmd: 'set flood.max 6' },
                      { label: 'Intervalle advert (min)', cmd: 'set advert.interval 120' },
                      { label: 'Intervalle flood advert (h)', cmd: 'set flood.advert.interval 12' },
                      { label: 'Multi-acks', cmd: 'set multi.acks 1' },
                      { label: 'Mode lecture seule', cmd: 'set allow.read.only on' },
                      { label: 'Multiplicateur ADC', cmd: 'set adc.multiplier 1.0' },
                      { label: 'Mode mise à jour firmware', cmd: 'set dfu on' },
                    ].map((item) => (
                      <button
                        key={item.cmd}
                        onClick={() => { setTab('terminal'); setCliInput(item.cmd) }}
                        className="flex items-center justify-between rounded-lg border border-gray-800 bg-gray-900 px-3 py-2 text-left text-sm text-gray-300 hover:bg-gray-800 transition-colors"
                      >
                        <span>{item.label}</span>
                        <ChevronRight className="h-3 w-3 text-gray-600" />
                      </button>
                    ))}
                  </div>
                  <div className="mt-4 rounded-lg border border-amber-900/30 bg-amber-900/10 p-3">
                    <p className="text-xs text-amber-400">
                      <AlertCircle className="mr-1 inline h-3 w-3" />
                      Les commandes de configuration sont envoyées comme des messages texte au repeater.
                      Éditez la commande dans le terminal avant de l'envoyer.
                    </p>
                  </div>
                </div>
              )}

              {/* ONGLET VOISINS + ACL */}
              {tab === 'neighbours' && (
                <div className="space-y-4">
                  <div className="rounded-xl border border-gray-800 bg-gray-900 p-4">
                    <h4 className="mb-3 flex items-center gap-2 text-sm font-medium text-gray-300">
                      <Users className="h-4 w-4" />Voisins ({neighbours.length})
                    </h4>
                    {neighbours.length === 0 ? (
                      <p className="text-sm text-gray-600">Aucun voisin détecté</p>
                    ) : (
                      <div className="space-y-1">
                        {neighbours.map((n, i) => (
                          <div key={i} className="flex items-center justify-between rounded-lg bg-gray-800 px-4 py-2">
                            <div>
                              <span className="text-sm text-white">{n.name ?? n.pubkey_hex.slice(0, 12) + '…'}</span>
                              <span className="ml-2 font-mono text-xs text-gray-500">{n.pubkey_hex.slice(0, 12)}</span>
                            </div>
                            <div className="flex items-center gap-4 text-xs text-gray-400">
                              <span>SNR: <span className={n.snr > 0 ? 'text-emerald-400' : 'text-red-400'}>{n.snr.toFixed(1)} dB</span></span>
                              <span>{n.secs_ago < 60 ? `${n.secs_ago}s` : n.secs_ago < 3600 ? `${Math.floor(n.secs_ago / 60)}m` : `${Math.floor(n.secs_ago / 3600)}h`}</span>
                            </div>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>

                  {acl.length > 0 && (
                    <div className="rounded-xl border border-gray-800 bg-gray-900 p-4">
                      <h4 className="mb-3 flex items-center gap-2 text-sm font-medium text-gray-300">
                        <Shield className="h-4 w-4" />ACL ({acl.length})
                      </h4>
                      <div className="space-y-1">
                        {acl.map((e, i) => (
                          <div key={i} className="flex items-center justify-between rounded-lg bg-gray-800 px-4 py-2 text-sm">
                            <span className="text-white">{e.name ?? e.pubkey_hex.slice(0, 12) + '…'}</span>
                            <span className="font-mono text-xs text-gray-400">perm: 0x{e.permissions.toString(16).padStart(2, '0')}</span>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              )}

              {/* ONGLET TERMINAL */}
              {tab === 'terminal' && (
                <div className="flex h-full flex-col">
                  {/* Commandes rapides */}
                  <div className="mb-3 flex flex-wrap gap-1">
                    {QUICK_COMMANDS.map((qc) => (
                      <button
                        key={qc.cmd}
                        onClick={() => sendCli(qc.cmd)}
                        disabled={cliLoading}
                        className="rounded bg-gray-800 px-2 py-1 text-xs text-gray-400 hover:bg-gray-700 hover:text-white disabled:opacity-50 transition-colors"
                      >
                        {qc.label}
                      </button>
                    ))}
                  </div>

                  {/* Historique */}
                  <div className="flex-1 overflow-y-auto rounded-lg border border-gray-800 bg-black p-3 font-mono text-xs">
                    {cliHistory.length === 0 && (
                      <p className="text-gray-600">Tapez une commande ou utilisez les boutons rapides ci-dessus.</p>
                    )}
                    {cliHistory.map((entry, i) => (
                      <div key={i} className="mb-2">
                        <div className="text-emerald-400">
                          <span className="text-gray-600">[{entry.time}]</span> &gt; {entry.cmd}
                        </div>
                        <div className={`whitespace-pre-wrap ${entry.response.startsWith('ERREUR') ? 'text-red-400' : 'text-gray-300'}`}>
                          {entry.response}
                        </div>
                      </div>
                    ))}
                    {cliLoading && (
                      <div className="text-amber-400 animate-pulse">En attente de réponse...</div>
                    )}
                    <div ref={cliEndRef} />
                  </div>

                  {/* Input */}
                  <div className="mt-2 flex gap-2">
                    <input
                      type="text"
                      value={cliInput}
                      onChange={(e) => setCliInput(e.target.value)}
                      onKeyDown={handleCliKeyDown}
                      placeholder="Entrez une commande CLI..."
                      disabled={cliLoading}
                      className="flex-1 rounded-lg border border-gray-700 bg-gray-900 px-3 py-2 font-mono text-sm text-emerald-400 placeholder-gray-600 focus:border-emerald-500 focus:outline-none disabled:opacity-50"
                      autoFocus={tab === 'terminal'}
                    />
                    <button
                      onClick={() => sendCli(cliInput)}
                      disabled={cliLoading || !cliInput.trim()}
                      className="rounded-lg bg-emerald-600 px-4 py-2 text-white hover:bg-emerald-700 disabled:opacity-50"
                    >
                      {cliLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : <Send className="h-4 w-4" />}
                    </button>
                  </div>
                </div>
              )}
            </div>
          </>
        )}
      </div>
    </div>
  )
}

function StatCard({ icon, label, value }: { icon: React.ReactNode; label: string; value: string }) {
  return (
    <div className="rounded-lg border border-gray-800 bg-gray-950 p-3">
      <div className="mb-1 flex items-center gap-1.5">
        {icon}
        <span className="text-xs text-gray-500">{label}</span>
      </div>
      <span className="text-sm font-semibold text-white">{value}</span>
    </div>
  )
}
