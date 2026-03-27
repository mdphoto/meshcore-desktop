import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useStore } from '../store'
import { tauri } from '../hooks/useTauri'
import { Cpu, Battery, RefreshCw, Power, Zap, Edit3, MapPin } from 'lucide-react'

export function DeviceView() {
  const { t } = useTranslation()
  const status = useStore((s) => s.connectionStatus)
  const deviceInfo = useStore((s) => s.deviceInfo)
  const battery = useStore((s) => s.battery)
  const setDeviceInfo = useStore((s) => s.setDeviceInfo)
  const setBattery = useStore((s) => s.setBattery)
  const setError = useStore((s) => s.setError)

  const [editingName, setEditingName] = useState(false)
  const [nameInput, setNameInput] = useState('')
  const [txPower, setTxPower] = useState(0)
  const [chemistry, setChemistry] = useState('lipo')

  useEffect(() => {
    if (status === 'connected') {
      tauri.getDeviceInfo().then((info) => {
        setDeviceInfo(info)
        setTxPower(info.tx_power)
        setNameInput(info.name)
      }).catch(() => {})
      tauri.getBattery(chemistry).then(([mv, pct]) => setBattery(mv, pct)).catch(() => {})
    }
  }, [status, chemistry, setDeviceInfo, setBattery])

  if (status !== 'connected') {
    return (
      <div className="flex h-full flex-col items-center justify-center text-gray-500">
        <Cpu className="mb-3 h-12 w-12" />
        <p>{t('device.notConnected')}</p>
      </div>
    )
  }

  const handleSaveName = async () => {
    try {
      await tauri.setDeviceName(nameInput)
      setEditingName(false)
      const info = await tauri.getDeviceInfo()
      setDeviceInfo(info)
    } catch (e) {
      setError(String(e))
    }
  }

  const handleSetTxPower = async (value: number) => {
    setTxPower(value)
    try {
      await tauri.setTxPower(value)
    } catch (e) {
      setError(String(e))
    }
  }

  const handleSyncTime = async () => {
    try { await tauri.syncTime() } catch (e) { setError(String(e)) }
  }

  const handleReboot = async () => {
    try { await tauri.reboot() } catch (e) { setError(String(e)) }
  }

  const info = deviceInfo

  return (
    <div className="p-6">
      <h2 className="mb-6 text-xl font-semibold text-white">{t('device.title')}</h2>

      <div className="grid max-w-2xl gap-4">
        {/* Nom et identité */}
        <div className="rounded-xl border border-gray-800 bg-gray-900 p-5">
          <div className="mb-4 flex items-center gap-3">
            <Cpu className="h-5 w-5 text-emerald-400" />
            {editingName ? (
              <div className="flex flex-1 items-center gap-2">
                <input
                  type="text"
                  value={nameInput}
                  onChange={(e) => setNameInput(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleSaveName()}
                  className="flex-1 rounded border border-gray-600 bg-gray-800 px-2 py-1 text-sm text-white focus:border-emerald-500 focus:outline-none"
                  autoFocus
                />
                <button onClick={handleSaveName} className="text-xs text-emerald-400 hover:text-emerald-300">
                  {t('common.save')}
                </button>
                <button onClick={() => setEditingName(false)} className="text-xs text-gray-500 hover:text-gray-400">
                  {t('common.cancel')}
                </button>
              </div>
            ) : (
              <div className="flex flex-1 items-center gap-2">
                <span className="text-sm font-medium text-white">{info?.name ?? '—'}</span>
                <button onClick={() => setEditingName(true)} className="text-gray-500 hover:text-gray-400">
                  <Edit3 className="h-3.5 w-3.5" />
                </button>
              </div>
            )}
          </div>
          {info && (
            <div className="mb-3 rounded bg-gray-800 px-3 py-1.5 text-xs text-gray-400 font-mono break-all">
              {info.public_key}
            </div>
          )}
          <dl className="grid grid-cols-2 gap-x-6 gap-y-3 text-sm">
            <InfoRow label={t('device.frequency')} value={info ? `${(info.radio_freq / 1e6).toFixed(2)} MHz` : '—'} />
            <InfoRow label={t('device.bandwidth')} value={info ? `${(info.radio_bw / 1e3).toFixed(1)} kHz` : '—'} />
            <InfoRow label={t('device.spreadingFactor')} value={info ? `SF${info.sf}` : '—'} />
            <InfoRow label={t('device.codingRate')} value={info ? `4/${info.cr}` : '—'} />
          </dl>
        </div>

        {/* TX Power */}
        <div className="rounded-xl border border-gray-800 bg-gray-900 p-5">
          <div className="mb-3 flex items-center gap-3">
            <Zap className="h-5 w-5 text-orange-400" />
            <span className="text-sm font-medium text-white">{t('device.txPower')}</span>
            <span className="ml-auto text-sm text-white font-medium">{txPower} dBm</span>
          </div>
          <input
            type="range"
            min={0}
            max={info?.max_tx_power ?? 22}
            value={txPower}
            onChange={(e) => handleSetTxPower(Number(e.target.value))}
            className="w-full accent-emerald-500"
          />
          <div className="mt-1 flex justify-between text-xs text-gray-500">
            <span>0 dBm</span>
            <span>{info?.max_tx_power ?? 22} dBm (max)</span>
          </div>
        </div>

        {/* Batterie */}
        <div className="rounded-xl border border-gray-800 bg-gray-900 p-5">
          <div className="mb-3 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Battery className="h-5 w-5 text-yellow-400" />
              <span className="text-sm font-medium text-white">{t('device.battery')}</span>
            </div>
            <select
              value={chemistry}
              onChange={(e) => setChemistry(e.target.value)}
              className="rounded border border-gray-700 bg-gray-800 px-2 py-1 text-xs text-gray-300 focus:outline-none"
            >
              <option value="lipo">LiPo</option>
              <option value="lifepo4">LiFePO4</option>
              <option value="nimh">NiMH</option>
            </select>
          </div>
          {battery ? (
            <div className="space-y-2">
              <div className="flex justify-between text-sm">
                <span className="text-gray-400">{battery.millivolts} mV</span>
                <span className="text-white font-medium">{battery.percent}%</span>
              </div>
              <div className="h-2.5 rounded-full bg-gray-800">
                <div
                  className={`h-2.5 rounded-full transition-all ${
                    battery.percent > 50 ? 'bg-green-500' : battery.percent > 20 ? 'bg-yellow-500' : 'bg-red-500'
                  }`}
                  style={{ width: `${battery.percent}%` }}
                />
              </div>
            </div>
          ) : (
            <span className="text-sm text-gray-500">—</span>
          )}
        </div>

        {/* GPS */}
        {info && (info.lat !== 0 || info.lon !== 0) && (
          <div className="rounded-xl border border-gray-800 bg-gray-900 p-5">
            <div className="mb-2 flex items-center gap-3">
              <MapPin className="h-5 w-5 text-red-400" />
              <span className="text-sm font-medium text-white">Position GPS</span>
            </div>
            <p className="text-sm text-gray-300">{info.lat.toFixed(6)}, {info.lon.toFixed(6)}</p>
          </div>
        )}

        {/* Actions */}
        <div className="flex gap-3">
          <button
            onClick={handleSyncTime}
            className="flex items-center gap-2 rounded-lg bg-gray-800 px-4 py-2 text-sm text-gray-300 hover:bg-gray-700 transition-colors"
          >
            <RefreshCw className="h-4 w-4" />
            {t('device.syncTime')}
          </button>
          <button
            onClick={handleReboot}
            className="flex items-center gap-2 rounded-lg bg-red-900/50 px-4 py-2 text-sm text-red-300 hover:bg-red-900/70 transition-colors"
          >
            <Power className="h-4 w-4" />
            {t('device.reboot')}
          </button>
        </div>
      </div>
    </div>
  )
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <>
      <dt className="text-gray-500">{label}</dt>
      <dd className="text-white">{value}</dd>
    </>
  )
}
