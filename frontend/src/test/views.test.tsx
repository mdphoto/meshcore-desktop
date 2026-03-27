import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { useStore } from '../store'

// Mock i18next
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: 'fr', changeLanguage: vi.fn() },
  }),
  initReactI18next: { type: '3rdParty', init: vi.fn() },
}))

// Import views après les mocks
import { ConnectionView } from '../views/ConnectionView'
import { ContactsView } from '../views/ContactsView'
import { DeviceView } from '../views/DeviceView'
import { SettingsView } from '../views/SettingsView'
import { RepeaterView } from '../views/RepeaterView'

function renderWithRouter(ui: React.ReactElement) {
  return render(<BrowserRouter>{ui}</BrowserRouter>)
}

const defaultState = {
  connectionStatus: 'disconnected' as const,
  deviceName: null,
  contacts: [],
  directMessages: {},
  channelMessages: {},
  channels: [],
  deviceInfo: null,
  battery: null,
  radioStats: [],
  companions: [],
  theme: 'dark' as const,
  selectedContact: null,
  selectedChannel: null,
  lastError: null,
}

describe('ConnectionView', () => {
  beforeEach(() => {
    useStore.setState(defaultState)
  })

  it('affiche les 3 méthodes de connexion', () => {
    renderWithRouter(<ConnectionView />)
    expect(screen.getByText('connection.serial')).toBeInTheDocument()
    expect(screen.getByText('connection.tcp')).toBeInTheDocument()
    expect(screen.getByText('connection.ble')).toBeInTheDocument()
  })

  it('affiche le bouton connecter quand déconnecté', () => {
    renderWithRouter(<ConnectionView />)
    expect(screen.getByText('connection.connect')).toBeInTheDocument()
  })

  it('affiche le bouton déconnecter quand connecté', () => {
    useStore.setState({ ...defaultState, connectionStatus: 'connected' })
    renderWithRouter(<ConnectionView />)
    expect(screen.getByText('connection.disconnect')).toBeInTheDocument()
  })

  it('affiche l\'historique vide', () => {
    renderWithRouter(<ConnectionView />)
    expect(screen.getByText('Aucun historique')).toBeInTheDocument()
  })
})

describe('ContactsView', () => {
  beforeEach(() => {
    useStore.setState({
      ...defaultState,
      contacts: [
        {
          public_key: 'pk1',
          name: 'Alice',
          node_type: 1,
          flags: 0,
          path: [],
          path_len: 1,
          lat: 44.75,
          lon: 4.82,
          last_seen: '2026-03-27T10:00:00Z',
          is_favorite: true,
          group_name: null,
        },
        {
          public_key: 'pk2',
          name: 'Bob Repeater',
          node_type: 2,
          flags: 0,
          path: [],
          path_len: 3,
          lat: 0,
          lon: 0,
          last_seen: '2026-03-26T10:00:00Z',
          is_favorite: false,
          group_name: null,
        },
      ],
      connectionStatus: 'connected',
    })
  })

  it('affiche les contacts après chargement', async () => {
    // Le mock invoke retourne [] puis le store est rempli par le beforeEach
    // On force le mock à retourner les contacts du store pour ce test
    const { invoke } = await import('@tauri-apps/api/core')
    const contacts = useStore.getState().contacts;
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === 'get_all_contacts') return Promise.resolve(contacts)
      return Promise.resolve(undefined)
    })
    renderWithRouter(<ContactsView />)
    await waitFor(() => {
      expect(screen.getByText('Alice')).toBeInTheDocument()
      expect(screen.getByText('Bob Repeater')).toBeInTheDocument()
    })
  })

  it('affiche les types de nœuds', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const contacts = useStore.getState().contacts;
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === 'get_all_contacts') return Promise.resolve(contacts)
      return Promise.resolve(undefined)
    })
    renderWithRouter(<ContactsView />)
    await waitFor(() => {
      expect(screen.getByText('Client')).toBeInTheDocument()
      expect(screen.getByText('Repeater')).toBeInTheDocument()
    })
  })

  it('affiche la barre de recherche', async () => {
    renderWithRouter(<ContactsView />)
    await waitFor(() => {
      expect(screen.getByPlaceholderText('contacts.search')).toBeInTheDocument()
    })
  })
})

describe('DeviceView', () => {
  it('affiche le message non connecté', () => {
    useStore.setState(defaultState)
    renderWithRouter(<DeviceView />)
    expect(screen.getByText('device.notConnected')).toBeInTheDocument()
  })

  it('affiche les infos quand connecté', () => {
    useStore.setState({
      ...defaultState,
      connectionStatus: 'connected',
      deviceInfo: {
        name: 'TestNode',
        public_key: 'aabbccdd',
        adv_type: 0,
        tx_power: 10,
        max_tx_power: 22,
        radio_freq: 868000000,
        radio_bw: 125000,
        sf: 12,
        cr: 5,
        lat: 44.75,
        lon: 4.82,
      },
      battery: { millivolts: 3800, percent: 75 },
    })
    renderWithRouter(<DeviceView />)
    expect(screen.getByText('TestNode')).toBeInTheDocument()
    expect(screen.getByText('75%')).toBeInTheDocument()
    expect(screen.getByText('3800 mV')).toBeInTheDocument()
  })
})

describe('RepeaterView', () => {
  it('affiche le message déconnecté', () => {
    useStore.setState(defaultState)
    renderWithRouter(<RepeaterView />)
    expect(screen.getByText('repeater.noData')).toBeInTheDocument()
  })

  it('affiche les stats quand connecté', () => {
    useStore.setState({
      ...defaultState,
      connectionStatus: 'connected',
      contacts: [],
      radioStats: [{
        noise_floor: -110,
        last_rssi: -85,
        snr: 8.5,
        timestamp: '2026-03-27T10:00:00Z',
      }],
    })
    renderWithRouter(<RepeaterView />)
    expect(screen.getByText('-110 dBm')).toBeInTheDocument()
    expect(screen.getByText('-85 dBm')).toBeInTheDocument()
    expect(screen.getByText('8.5 dB')).toBeInTheDocument()
  })
})

describe('SettingsView', () => {
  beforeEach(() => {
    useStore.setState(defaultState)
  })

  it('affiche les options de thème', () => {
    renderWithRouter(<SettingsView />)
    expect(screen.getByText('settings.dark')).toBeInTheDocument()
    expect(screen.getByText('settings.light')).toBeInTheDocument()
  })

  it('affiche le sélecteur de langue', () => {
    renderWithRouter(<SettingsView />)
    expect(screen.getByText('Français')).toBeInTheDocument()
    expect(screen.getByText('English')).toBeInTheDocument()
  })
})
