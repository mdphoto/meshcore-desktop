import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useStore } from '../store'
import { tauri } from '../hooks/useTauri'
import { Radio, Usb, Wifi, Bluetooth, Trash2, Clock, Search, Signal, Loader2 } from 'lucide-react'
import type { ConnectionMethod, StoredCompanion, BleDevice } from '../types'

export function ConnectionView() {
  const { t } = useTranslation()
  const status = useStore((s) => s.connectionStatus)
  const setConnectedStore = useStore((s) => s.setConnected)
  const setDisconnectedStore = useStore((s) => s.setDisconnected)
  const companions = useStore((s) => s.companions)
  const setCompanions = useStore((s) => s.setCompanions)
  const setError = useStore((s) => s.setError)

  const [method, setMethod] = useState<ConnectionMethod>('serial')
  const [serialPort, setSerialPort] = useState('/dev/ttyUSB0')
  const [baudRate, setBaudRate] = useState(115200)
  const [tcpHost, setTcpHost] = useState('127.0.0.1')
  const [tcpPort, setTcpPort] = useState(5000)
  const [bleName, setBleName] = useState('')
  const [connecting, setConnecting] = useState(false)
  const [serialPorts, setSerialPorts] = useState<string[]>([])
  const [scanning, setScanning] = useState(false)
  const [bleDevices, setBleDevices] = useState<BleDevice[]>([])
  const [bleScanning, setBleScanning] = useState(false)
  const [blePin, setBlePin] = useState('123456')
  const [bleSelectedAddr, setBleSelectedAddr] = useState('')
  const [pairing, setPairing] = useState(false)
  const [pairStatus, setPairStatus] = useState('')

  useEffect(() => {
    tauri.getAllCompanions().then(setCompanions).catch(() => {})
  }, [setCompanions])

  const scanPorts = async () => {
    setScanning(true)
    try {
      const ports = await tauri.scanSerialPorts()
      setSerialPorts(ports)
      if (ports.length > 0 && serialPort === '/dev/ttyUSB0') {
        setSerialPort(ports[0])
      }
    } catch (e) {
      setError(String(e))
    } finally {
      setScanning(false)
    }
  }

  const scanBle = async () => {
    setBleScanning(true)
    setBleDevices([])
    try {
      const devices = await tauri.scanBleDevices(5)
      setBleDevices(devices)
      if (devices.length > 0 && !bleName) {
        setBleName(devices[0].address)
      }
    } catch (e) {
      setError(String(e))
    } finally {
      setBleScanning(false)
    }
  }

  const handleConnect = async () => {
    setConnecting(true)
    try {
      let address = ''
      let transportType = ''
      switch (method) {
        case 'serial':
          await tauri.connectSerial(serialPort, baudRate)
          address = serialPort
          transportType = 'serial'
          break
        case 'tcp':
          await tauri.connectTcp(tcpHost, tcpPort)
          address = `${tcpHost}:${tcpPort}`
          transportType = 'tcp'
          break
        case 'ble':
          await tauri.connectBle(bleName)
          address = bleName
          transportType = 'ble'
          break
      }
      // Mettre à jour le store immédiatement
      setConnectedStore(address)
      // Sauvegarder dans l'historique
      await tauri.saveCompanion(transportType, address, address, null)
      tauri.getAllCompanions().then(setCompanions).catch(() => {})
      // Lancer le sync contacts en arrière-plan
      tauri.syncContacts().then(() => {
        tauri.getAllContacts().then((c) => useStore.getState().setContacts(c)).catch(() => {})
      }).catch(() => {})
    } catch (e) {
      setError(String(e))
    } finally {
      setConnecting(false)
    }
  }

  const [disconnecting, setDisconnecting] = useState(false)

  const handleDisconnect = async () => {
    setDisconnecting(true)
    try {
      await tauri.disconnect()
      // Forcer le reset du store immédiatement (l'événement Tauri arrivera aussi)
      setDisconnectedStore()
    } catch (e) {
      // Même en cas d'erreur, on reset le state car la connexion est probablement perdue
      setDisconnectedStore()
      setError(String(e))
    } finally {
      setDisconnecting(false)
    }
  }

  const handleQuickConnect = async (comp: StoredCompanion) => {
    setConnecting(true)
    try {
      switch (comp.transport_type) {
        case 'serial':
          await tauri.connectSerial(comp.address, 115200)
          break
        case 'tcp': {
          const [host, port] = comp.address.split(':')
          await tauri.connectTcp(host, Number(port))
          break
        }
        case 'ble':
          await tauri.connectBle(comp.address)
          break
      }
      setConnectedStore(comp.address)
      await tauri.saveCompanion(comp.transport_type, comp.address, comp.address, null)
      tauri.getAllCompanions().then(setCompanions).catch(() => {})
    } catch (e) {
      setError(String(e))
    } finally {
      setConnecting(false)
    }
  }

  const handleDeleteCompanion = async (id: number) => {
    try {
      await tauri.deleteCompanion(id)
      tauri.getAllCompanions().then(setCompanions).catch(() => {})
    } catch (e) {
      setError(String(e))
    }
  }

  const methods: { key: ConnectionMethod; icon: typeof Radio; label: string }[] = [
    { key: 'serial', icon: Usb, label: t('connection.serial') },
    { key: 'tcp', icon: Wifi, label: t('connection.tcp') },
    { key: 'ble', icon: Bluetooth, label: t('connection.ble') },
  ]

  const transportIcon = (type: string) => {
    switch (type) {
      case 'serial': return <Usb className="h-4 w-4" />
      case 'tcp': return <Wifi className="h-4 w-4" />
      case 'ble': return <Bluetooth className="h-4 w-4" />
      default: return <Radio className="h-4 w-4" />
    }
  }

  return (
    <div className="p-6">
      <h2 className="mb-6 text-xl font-semibold text-white">{t('connection.title')}</h2>

      <div className="grid max-w-2xl gap-6 lg:grid-cols-2">
        {/* Formulaire de connexion */}
        <div>
          <div className="mb-4 flex gap-2">
            {methods.map(({ key, icon: Icon, label }) => (
              <button
                key={key}
                onClick={() => setMethod(key)}
                className={`flex items-center gap-2 rounded-lg px-4 py-2 text-sm transition-colors ${
                  method === key
                    ? 'bg-emerald-600 text-white'
                    : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
                }`}
              >
                <Icon className="h-4 w-4" />
                {label}
              </button>
            ))}
          </div>

          <div className="space-y-4 rounded-xl bg-gray-900 p-5 border border-gray-800">
            {method === 'serial' && (
              <>
                <div>
                  <div className="mb-1 flex items-center justify-between">
                    <span className="text-xs font-medium text-gray-400">{t('connection.port')}</span>
                    <button
                      onClick={scanPorts}
                      disabled={scanning}
                      className="flex items-center gap-1 text-xs text-emerald-400 hover:text-emerald-300"
                    >
                      <Search className={`h-3 w-3 ${scanning ? 'animate-spin' : ''}`} />
                      Scanner
                    </button>
                  </div>
                  {serialPorts.length > 0 ? (
                    <select
                      value={serialPort}
                      onChange={(e) => setSerialPort(e.target.value)}
                      className="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-white focus:border-emerald-500 focus:outline-none"
                    >
                      {serialPorts.map((p) => (
                        <option key={p} value={p}>{p}</option>
                      ))}
                    </select>
                  ) : (
                    <input
                      type="text"
                      value={serialPort}
                      onChange={(e) => setSerialPort(e.target.value)}
                      className="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 focus:border-emerald-500 focus:outline-none"
                    />
                  )}
                </div>
                <Field
                  label={t('connection.baudRate')}
                  value={String(baudRate)}
                  onChange={(v) => setBaudRate(Number(v))}
                  type="number"
                />
              </>
            )}

            {method === 'tcp' && (
              <>
                <Field label={t('connection.host')} value={tcpHost} onChange={setTcpHost} />
                <Field
                  label={t('connection.portNumber')}
                  value={String(tcpPort)}
                  onChange={(v) => setTcpPort(Number(v))}
                  type="number"
                />
              </>
            )}

            {method === 'ble' && (
              <div className="space-y-3">
                <div>
                  <div className="mb-1 flex items-center justify-between">
                    <span className="text-xs font-medium text-gray-400">{t('connection.deviceName')}</span>
                    <button
                      onClick={scanBle}
                      disabled={bleScanning}
                      className="flex items-center gap-1 text-xs text-blue-400 hover:text-blue-300"
                    >
                      {bleScanning ? (
                        <Loader2 className="h-3 w-3 animate-spin" />
                      ) : (
                        <Search className="h-3 w-3" />
                      )}
                      {bleScanning ? 'Scan en cours...' : 'Scanner BLE'}
                    </button>
                  </div>
                  <input
                    type="text"
                    value={bleName}
                    onChange={(e) => setBleName(e.target.value)}
                    placeholder="MeshCore-XXXX ou adresse"
                    className="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 focus:border-emerald-500 focus:outline-none"
                  />
                </div>

                {/* Résultats du scan BLE */}
                {bleDevices.length > 0 && (
                  <div className="space-y-1">
                    <span className="text-xs text-gray-500">{bleDevices.length} appareil(s) trouvé(s)</span>
                    {bleDevices.map((d) => (
                      <button
                        key={d.address}
                        onClick={() => { setBleName(d.name); setBleSelectedAddr(d.address) }}
                        className={`flex w-full items-center gap-3 rounded-lg border px-3 py-2 text-left text-sm transition-colors ${
                          bleName === d.name
                            ? 'border-emerald-600 bg-emerald-900/30 text-white'
                            : 'border-gray-700 bg-gray-800 text-gray-300 hover:border-gray-600'
                        }`}
                      >
                        <Bluetooth className="h-4 w-4 shrink-0 text-blue-400" />
                        <div className="flex-1 min-w-0">
                          <div className="truncate font-medium">{d.name}</div>
                          <div className="text-xs text-gray-500 truncate">{d.address}</div>
                        </div>
                        {d.rssi !== null && (
                          <div className="flex items-center gap-1 text-xs text-gray-400">
                            <Signal className="h-3 w-3" />
                            {d.rssi} dBm
                          </div>
                        )}
                      </button>
                    ))}
                  </div>
                )}

                {bleScanning && bleDevices.length === 0 && (
                  <div className="flex items-center justify-center gap-2 py-4 text-sm text-gray-500">
                    <Loader2 className="h-4 w-4 animate-spin" />
                    Recherche de périphériques MeshCore...
                  </div>
                )}

                {/* PIN et Appairage */}
                {(bleName || bleSelectedAddr) && (
                  <div className="space-y-2 rounded-lg border border-gray-700 bg-gray-800/50 p-3">
                    <label className="block">
                      <span className="mb-1 block text-xs font-medium text-gray-400">Code PIN</span>
                      <input
                        type="text"
                        value={blePin}
                        onChange={(e) => setBlePin(e.target.value)}
                        placeholder="123456"
                        className="w-full rounded-lg border border-gray-600 bg-gray-800 px-3 py-1.5 text-sm text-white placeholder-gray-500 focus:border-blue-500 focus:outline-none"
                      />
                    </label>
                    <button
                      onClick={async () => {
                        const addr = bleSelectedAddr || bleDevices.find(d => d.name === bleName)?.address || bleName
                        setPairing(true)
                        setPairStatus('')
                        try {
                          const result = await tauri.pairBleDevice(addr, blePin)
                          setPairStatus(result)
                        } catch (e) {
                          setPairStatus(String(e))
                        } finally {
                          setPairing(false)
                        }
                      }}
                      disabled={pairing || !blePin}
                      className="flex w-full items-center justify-center gap-2 rounded-lg bg-blue-600 px-3 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50 transition-colors"
                    >
                      {pairing ? <Loader2 className="h-4 w-4 animate-spin" /> : <Bluetooth className="h-4 w-4" />}
                      {pairing ? 'Appairage en cours...' : 'Appairer avec PIN'}
                    </button>
                    {pairStatus && (
                      <p className={`text-xs ${pairStatus.includes('réussi') || pairStatus.includes('succeeded') ? 'text-emerald-400' : 'text-yellow-400'}`}>
                        {pairStatus}
                      </p>
                    )}
                  </div>
                )}
              </div>
            )}

            {status === 'connected' ? (
              <button
                onClick={handleDisconnect}
                disabled={disconnecting}
                className="w-full rounded-lg bg-red-600 px-4 py-2.5 text-sm font-medium text-white hover:bg-red-700 disabled:opacity-50 transition-colors"
              >
                {disconnecting ? t('common.loading') : t('connection.disconnect')}
              </button>
            ) : (
              <button
                onClick={handleConnect}
                disabled={connecting}
                className="w-full rounded-lg bg-emerald-600 px-4 py-2.5 text-sm font-medium text-white hover:bg-emerald-700 disabled:opacity-50 transition-colors"
              >
                {connecting ? t('common.loading') : t('connection.connect')}
              </button>
            )}
          </div>
        </div>

        {/* Historique des connexions */}
        <div>
          <h3 className="mb-3 flex items-center gap-2 text-sm font-medium text-gray-400">
            <Clock className="h-4 w-4" />
            Connexions récentes
          </h3>
          {companions.length === 0 ? (
            <p className="rounded-xl border border-gray-800 bg-gray-900 p-5 text-center text-sm text-gray-600">
              Aucun historique
            </p>
          ) : (
            <div className="space-y-1.5">
              {companions.map((comp) => (
                <div
                  key={comp.id}
                  className="flex items-center gap-3 rounded-lg border border-gray-800 bg-gray-900 px-4 py-2.5 hover:bg-gray-800/70 transition-colors"
                >
                  <span className="text-gray-500">{transportIcon(comp.transport_type)}</span>
                  <button
                    onClick={() => handleQuickConnect(comp)}
                    disabled={connecting || status === 'connected'}
                    className="flex-1 text-left disabled:opacity-50"
                  >
                    <div className="text-sm text-white">{comp.address}</div>
                    <div className="text-xs text-gray-500">
                      {comp.transport_type} · {new Date(comp.last_used).toLocaleDateString()}
                    </div>
                  </button>
                  <button
                    onClick={() => comp.id !== null && handleDeleteCompanion(comp.id)}
                    className="text-gray-600 hover:text-red-400 transition-colors"
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

function Field({
  label,
  value,
  onChange,
  type = 'text',
  placeholder,
}: {
  label: string
  value: string
  onChange: (v: string) => void
  type?: string
  placeholder?: string
}) {
  return (
    <label className="block">
      <span className="mb-1 block text-xs font-medium text-gray-400">{label}</span>
      <input
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 focus:border-emerald-500 focus:outline-none focus:ring-1 focus:ring-emerald-500"
      />
    </label>
  )
}
