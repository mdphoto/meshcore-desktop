import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import fr from './i18n/fr'
import en from './i18n/en'
import es from './i18n/es'
import de from './i18n/de'
import it from './i18n/it'
import pt from './i18n/pt'
import nl from './i18n/nl'
import pl from './i18n/pl'
import ja from './i18n/ja'
import zh from './i18n/zh'
import ko from './i18n/ko'
import ru from './i18n/ru'

const resources = {
  fr: { translation: fr },
  en: { translation: en },
  es: { translation: es },
  de: { translation: de },
  it: { translation: it },
  pt: { translation: pt },
  nl: { translation: nl },
  pl: { translation: pl },
  ja: { translation: ja },
  zh: { translation: zh },
  ko: { translation: ko },
  ru: { translation: ru },
}

export const languages = [
  { code: 'fr', name: 'Français' },
  { code: 'en', name: 'English' },
  { code: 'es', name: 'Español' },
  { code: 'de', name: 'Deutsch' },
  { code: 'it', name: 'Italiano' },
  { code: 'pt', name: 'Português' },
  { code: 'nl', name: 'Nederlands' },
  { code: 'pl', name: 'Polski' },
  { code: 'ja', name: '日本語' },
  { code: 'zh', name: '中文' },
  { code: 'ko', name: '한국어' },
  { code: 'ru', name: 'Русский' },
]

i18n.use(initReactI18next).init({
  resources,
  lng: 'fr',
  fallbackLng: 'en',
  interpolation: { escapeValue: false },
})

export default i18n
