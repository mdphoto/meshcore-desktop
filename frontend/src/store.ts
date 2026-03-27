import { create } from 'zustand'
import type { ConnectionStatus, StoredContact, StoredMessage, StoredChannel, DeviceInfoSummary, StoredCompanion } from './types'

export interface RadioStats {
  noise_floor: number
  last_rssi: number
  snr: number
  timestamp: string
}

interface AppStore {
  // Connection
  connectionStatus: ConnectionStatus
  deviceName: string | null
  setConnected: (name: string) => void
  setDisconnected: () => void
  setReconnecting: () => void

  // Contacts
  contacts: StoredContact[]
  setContacts: (contacts: StoredContact[]) => void
  updateContact: (contact: StoredContact) => void

  // Messages
  directMessages: Record<string, StoredMessage[]>
  channelMessages: Record<number, StoredMessage[]>
  setDirectMessages: (pubkey: string, msgs: StoredMessage[]) => void
  setChannelMessages: (idx: number, msgs: StoredMessage[]) => void
  appendDirectMessage: (pubkey: string, msg: StoredMessage) => void
  appendChannelMessage: (idx: number, msg: StoredMessage) => void
  updateMessageStatus: (id: string, status: string) => void

  // Channels
  channels: StoredChannel[]
  setChannels: (channels: StoredChannel[]) => void

  // Device
  deviceInfo: DeviceInfoSummary | null
  battery: { millivolts: number; percent: number } | null
  setDeviceInfo: (info: DeviceInfoSummary) => void
  setBattery: (mv: number, pct: number) => void

  // UI
  selectedContact: string | null
  selectedChannel: number | null
  selectContact: (pubkey: string | null) => void
  selectChannel: (idx: number | null) => void

  // Map focus
  mapFocus: { lat: number; lon: number } | null
  setMapFocus: (f: { lat: number; lon: number } | null) => void

  // Radio stats
  radioStats: RadioStats[]
  addRadioStats: (stats: RadioStats) => void

  // Companions
  companions: StoredCompanion[]
  setCompanions: (c: StoredCompanion[]) => void

  // Theme
  theme: 'dark' | 'light'
  setTheme: (t: 'dark' | 'light') => void

  // Repeater admin
  adminNode: string | null // pubkey du nœud en cours d'administration
  adminLoggedIn: boolean
  setAdminNode: (pubkey: string | null, loggedIn?: boolean) => void
  setAdminLoggedIn: (v: boolean) => void

  // Errors
  lastError: string | null
  setError: (msg: string | null) => void
}

export const useStore = create<AppStore>((set) => ({
  // Connection
  connectionStatus: 'disconnected',
  deviceName: null,
  setConnected: (name) => set({ connectionStatus: 'connected', deviceName: name }),
  setDisconnected: () => set({
    connectionStatus: 'disconnected',
    deviceName: null,
    deviceInfo: null,
    battery: null,
    contacts: [],
    channels: [],
    directMessages: {},
    channelMessages: {},
    radioStats: [],
    selectedContact: null,
    selectedChannel: null,
  }),
  setReconnecting: () => set({ connectionStatus: 'reconnecting' }),

  // Contacts
  contacts: [],
  setContacts: (contacts) => set({ contacts }),
  updateContact: (contact) => set((s) => ({
    contacts: s.contacts.some((c) => c.public_key === contact.public_key)
      ? s.contacts.map((c) => c.public_key === contact.public_key ? contact : c)
      : [...s.contacts, contact],
  })),

  // Messages
  directMessages: {},
  channelMessages: {},
  setDirectMessages: (pubkey, msgs) => set((s) => ({
    directMessages: { ...s.directMessages, [pubkey]: msgs },
  })),
  setChannelMessages: (idx, msgs) => set((s) => ({
    channelMessages: { ...s.channelMessages, [idx]: msgs },
  })),
  appendDirectMessage: (pubkey, msg) => set((s) => ({
    directMessages: {
      ...s.directMessages,
      [pubkey]: [...(s.directMessages[pubkey] ?? []), msg],
    },
  })),
  appendChannelMessage: (idx, msg) => set((s) => ({
    channelMessages: {
      ...s.channelMessages,
      [idx]: [...(s.channelMessages[idx] ?? []), msg],
    },
  })),
  updateMessageStatus: (id, status) => set((s) => {
    const updateMsgs = (msgs: StoredMessage[]) =>
      msgs.map((m) => m.id === id ? { ...m, status } : m)
    const directMessages = Object.fromEntries(
      Object.entries(s.directMessages).map(([k, v]) => [k, updateMsgs(v)])
    )
    const channelMessages = Object.fromEntries(
      Object.entries(s.channelMessages).map(([k, v]) => [k, updateMsgs(v)])
    )
    return { directMessages, channelMessages }
  }),

  // Channels
  channels: [],
  setChannels: (channels) => set({ channels }),

  // Device
  deviceInfo: null,
  battery: null,
  setDeviceInfo: (info) => set({ deviceInfo: info }),
  setBattery: (millivolts, percent) => set({ battery: { millivolts, percent } }),

  // UI
  selectedContact: null,
  selectedChannel: null,
  selectContact: (pubkey) => set({ selectedContact: pubkey, selectedChannel: null }),
  selectChannel: (idx) => set({ selectedChannel: idx, selectedContact: null }),

  // Map focus
  mapFocus: null,
  setMapFocus: (mapFocus) => set({ mapFocus }),

  // Radio stats (garder les 100 derniers)
  radioStats: [],
  addRadioStats: (stats) => set((s) => ({
    radioStats: [...s.radioStats.slice(-99), stats],
  })),

  // Companions
  companions: [],
  setCompanions: (companions) => set({ companions }),

  // Theme
  theme: 'dark',
  setTheme: (theme) => set({ theme }),

  // Repeater admin
  adminNode: null,
  adminLoggedIn: false,
  setAdminNode: (pubkey, loggedIn) => set({ adminNode: pubkey, adminLoggedIn: loggedIn ?? false }),
  setAdminLoggedIn: (v) => set({ adminLoggedIn: v }),

  // Errors
  lastError: null,
  setError: (msg) => set({ lastError: msg }),
}))
