import { Routes, Route, Navigate } from 'react-router-dom'
import { useEffect } from 'react'
import { Sidebar } from './components/Sidebar'
import { ErrorBoundary } from './components/ErrorBoundary'
import { ConnectionView } from './views/ConnectionView'
import { ContactsView } from './views/ContactsView'
import { ChatView } from './views/ChatView'
import { MapView } from './views/MapView'
import { DeviceView } from './views/DeviceView'
import { RepeaterView } from './views/RepeaterView'
import { SettingsView } from './views/SettingsView'
import { useEvents } from './hooks/useEvents'
import { tauri } from './hooks/useTauri'
import { useStore } from './store'

const isTauri = '__TAURI_INTERNALS__' in window

export default function App() {
  useEvents()

  const lastError = useStore((s) => s.lastError)
  const setError = useStore((s) => s.setError)
  const theme = useStore((s) => s.theme)
  const setTheme = useStore((s) => s.setTheme)

  // Charger le thème persisté au démarrage
  useEffect(() => {
    if (!isTauri) return
    tauri.getSetting('theme').then((v) => {
      if (v === 'light' || v === 'dark') setTheme(v)
    }).catch(() => {})
  }, [setTheme])

  // Auto-connexion au dernier companion (délai 1s pour que useEvents soit prêt)
  useEffect(() => {
    if (!isTauri) return
    const timer = setTimeout(async () => {
      try {
        const autoSetting = await tauri.getSetting('auto_connect').catch(() => null)
        if (autoSetting === 'false') return
        const companions = await tauri.getAllCompanions()
        if (companions.length === 0) return
        const last = companions[0]
        // connect retourne vite (handshake minimal), l'événement Connected déclenche le sync
        switch (last.transport_type) {
          case 'serial': await tauri.connectSerial(last.address, 115200); break
          case 'tcp': {
            const [host, port] = last.address.split(':')
            await tauri.connectTcp(host, Number(port)); break
          }
          case 'ble': await tauri.connectBle(last.address); break
        }
        // Forcer la mise à jour du store après connexion (l'event Connected peut avoir été raté)
        const connected = await tauri.isConnected()
        if (connected) {
          const store = useStore.getState()
          store.setConnected(last.address)
          tauri.getAllContacts().then((c) => store.setContacts(c)).catch(() => {})
          tauri.getAllChannels().then((c) => store.setChannels(c)).catch(() => {})
          // Lancer le sync contacts depuis le device
          tauri.syncContacts().then(() => {
            tauri.getAllContacts().then((c) => store.setContacts(c)).catch(() => {})
          }).catch(() => {})
        }
      } catch {
        // Silencieux si le device n'est pas allumé
      }
    }, 1000)
    return () => clearTimeout(timer)
  }, [])

  useEffect(() => {
    if (lastError) {
      const t = setTimeout(() => setError(null), 5000)
      return () => clearTimeout(t)
    }
  }, [lastError, setError])

  return (
    <div className={`${theme === 'dark' ? 'dark' : ''} flex h-screen bg-white text-gray-800 dark:bg-gray-950 dark:text-gray-200`}>
      <Sidebar />
      <main className="flex-1 overflow-auto">
        {lastError && (
          <div className="mx-4 mt-2 rounded-lg bg-red-50 px-4 py-2 text-sm text-red-700 border border-red-200 dark:bg-red-900/50 dark:text-red-300 dark:border-red-800">
            {lastError}
          </div>
        )}
        <ErrorBoundary>
          <Routes>
            <Route path="/" element={<Navigate to="/connection" replace />} />
            <Route path="/connection" element={<ConnectionView />} />
            <Route path="/contacts" element={<ContactsView />} />
            <Route path="/chat" element={<ChatView />} />
            <Route path="/map" element={<MapView />} />
            <Route path="/device" element={<DeviceView />} />
            <Route path="/repeater" element={<RepeaterView />} />
            <Route path="/settings" element={<SettingsView />} />
          </Routes>
        </ErrorBoundary>
      </main>
    </div>
  )
}
