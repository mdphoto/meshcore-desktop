import { NavLink } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { useStore } from '../store'
import {
  Radio,
  Users,
  MessageSquare,
  Map,
  Cpu,
  Repeat,
  Settings,
} from 'lucide-react'

const navItems = [
  { to: '/connection', icon: Radio, label: 'nav.connection' },
  { to: '/contacts', icon: Users, label: 'nav.contacts' },
  { to: '/chat', icon: MessageSquare, label: 'nav.chat' },
  { to: '/map', icon: Map, label: 'nav.map' },
  { to: '/device', icon: Cpu, label: 'nav.device' },
  { to: '/repeater', icon: Repeat, label: 'nav.repeater' },
  { to: '/settings', icon: Settings, label: 'nav.settings' },
] as const

export function Sidebar() {
  const { t } = useTranslation()
  const status = useStore((s) => s.connectionStatus)
  const deviceName = useStore((s) => s.deviceName)

  const statusColor =
    status === 'connected'
      ? 'bg-green-500'
      : status === 'reconnecting'
        ? 'bg-yellow-500 animate-pulse'
        : 'bg-gray-600'

  return (
    <aside className="flex w-56 flex-col border-r border-gray-200 bg-gray-50 dark:border-gray-800 dark:bg-gray-900">
      <div className="flex items-center gap-3 border-b border-gray-200 px-4 py-4 dark:border-gray-800">
        <Radio className="h-6 w-6 text-emerald-500" />
        <div>
          <h1 className="text-sm font-bold text-gray-900 dark:text-white">MeshCore</h1>
          <div className="flex items-center gap-1.5 text-xs text-gray-500 dark:text-gray-400">
            <span className={`inline-block h-2 w-2 rounded-full ${statusColor}`} />
            {deviceName ?? t('connection.disconnected')}
          </div>
        </div>
      </div>

      <nav className="flex-1 space-y-1 px-2 py-3">
        {navItems.map(({ to, icon: Icon, label }) => (
          <NavLink
            key={to}
            to={to}
            className={({ isActive }) =>
              `flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors ${
                isActive
                  ? 'bg-gray-200 text-gray-900 dark:bg-gray-800 dark:text-white'
                  : 'text-gray-500 hover:bg-gray-200/50 hover:text-gray-700 dark:text-gray-400 dark:hover:bg-gray-800/50 dark:hover:text-gray-200'
              }`
            }
          >
            <Icon className="h-4 w-4" />
            {t(label)}
          </NavLink>
        ))}
      </nav>

      <div className="border-t border-gray-200 px-4 py-3 text-xs text-gray-400 dark:border-gray-800 dark:text-gray-600">
        MeshCore Desktop v0.1.0
      </div>
    </aside>
  )
}
