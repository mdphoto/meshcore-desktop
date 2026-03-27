import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { useStore } from '../store'
import { tauri } from '../hooks/useTauri'
import { Settings, Globe, Sun, Moon } from 'lucide-react'
import { languages } from '../i18n'

export function SettingsView() {
  const { t, i18n } = useTranslation()
  const theme = useStore((s) => s.theme)
  const setTheme = useStore((s) => s.setTheme)

  useEffect(() => {
    tauri.getSetting('theme').then((v) => {
      if (v === 'light' || v === 'dark') setTheme(v)
    }).catch(() => {})
    tauri.getSetting('language').then((v) => {
      if (v) i18n.changeLanguage(v)
    }).catch(() => {})
  }, [setTheme, i18n])

  const handleThemeChange = (t: 'dark' | 'light') => {
    setTheme(t)
    document.documentElement.classList.toggle('dark', t === 'dark')
    tauri.setSetting('theme', t).catch(() => {})
  }

  const handleLanguageChange = (lng: string) => {
    i18n.changeLanguage(lng)
    tauri.setSetting('language', lng).catch(() => {})
  }

  return (
    <div className="p-6">
      <h2 className="mb-6 text-xl font-semibold text-white">{t('settings.title')}</h2>

      <div className="max-w-md space-y-4">
        {/* Langue */}
        <div className="rounded-xl border border-gray-800 bg-gray-900 p-5">
          <div className="mb-4 flex items-center gap-3">
            <Globe className="h-5 w-5 text-blue-400" />
            <span className="text-sm font-medium text-white">{t('settings.language')}</span>
          </div>
          <select
            value={i18n.language}
            onChange={(e) => handleLanguageChange(e.target.value)}
            className="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-white focus:border-emerald-500 focus:outline-none"
          >
            {languages.map((lng) => (
              <option key={lng.code} value={lng.code}>{lng.name}</option>
            ))}
          </select>
        </div>

        {/* Thème */}
        <div className="rounded-xl border border-gray-800 bg-gray-900 p-5">
          <div className="mb-4 flex items-center gap-3">
            <Settings className="h-5 w-5 text-gray-400" />
            <span className="text-sm font-medium text-white">{t('settings.theme')}</span>
          </div>
          <div className="flex gap-3">
            <button
              onClick={() => handleThemeChange('dark')}
              className={`flex flex-1 items-center justify-center gap-2 rounded-lg px-4 py-3 text-sm transition-colors ${
                theme === 'dark'
                  ? 'bg-gray-700 text-white ring-2 ring-emerald-500'
                  : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
              }`}
            >
              <Moon className="h-4 w-4" />
              {t('settings.dark')}
            </button>
            <button
              onClick={() => handleThemeChange('light')}
              className={`flex flex-1 items-center justify-center gap-2 rounded-lg px-4 py-3 text-sm transition-colors ${
                theme === 'light'
                  ? 'bg-gray-700 text-white ring-2 ring-emerald-500'
                  : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
              }`}
            >
              <Sun className="h-4 w-4" />
              {t('settings.light')}
            </button>
          </div>
        </div>

        {/* Info */}
        <div className="rounded-xl border border-gray-800 bg-gray-900 p-5 text-center">
          <p className="text-sm text-gray-400">MeshCore Desktop v0.1.0</p>
          <p className="mt-1 text-xs text-gray-600">Rust + Tauri 2 + React</p>
        </div>
      </div>
    </div>
  )
}
