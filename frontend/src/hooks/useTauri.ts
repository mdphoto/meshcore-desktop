import { invoke } from '@tauri-apps/api/core'
import type { StoredContact, StoredMessage, StoredChannel, DeviceInfoSummary, StoredCompanion, BleDevice, RepeaterStatus, RepeaterNeighbour } from '../types'

export const tauri = {
  // Connection
  connectSerial: (port: string, baudRate: number) =>
    invoke<void>('connect_serial', { port, baudRate }),
  connectTcp: (host: string, port: number) =>
    invoke<void>('connect_tcp', { host, port }),
  connectBle: (nameOrAddr: string) =>
    invoke<void>('connect_ble', { nameOrAddr }),
  disconnect: () => invoke<void>('disconnect'),
  isConnected: () => invoke<boolean>('is_connected'),
  scanBleDevices: (durationSecs?: number) =>
    invoke<BleDevice[]>('scan_ble_devices', { durationSecs }),
  pairBleDevice: (address: string, pin: string) =>
    invoke<string>('pair_ble_device', { address, pin }),
  isBlePaired: (address: string) =>
    invoke<boolean>('is_ble_paired', { address }),

  // Messages
  sendDirectMessage: (recipientPubkey: string, text: string) =>
    invoke<string>('send_direct_message', { recipientPubkey, text }),
  sendChannelMessage: (channelIdx: number, text: string) =>
    invoke<string>('send_channel_message', { channelIdx, text }),
  getDirectMessages: (contactPubkey: string, limit: number, offset: number) =>
    invoke<StoredMessage[]>('get_direct_messages', { contactPubkey, limit, offset }),
  getChannelMessages: (channelIdx: number, limit: number, offset: number) =>
    invoke<StoredMessage[]>('get_channel_messages', { channelIdx, limit, offset }),
  deleteConversation: (contactPubkey: string) =>
    invoke<number>('delete_conversation', { contactPubkey }),
  searchMessages: (query: string, limit: number) =>
    invoke<StoredMessage[]>('search_messages', { query, limit }),
  sendLogin: (roomPubkey: string, password: string) =>
    invoke<string>('send_login', { roomPubkey, password }),
  getDmContactPubkeys: () =>
    invoke<string[]>('get_dm_contact_pubkeys'),

  // Contacts
  syncContacts: () => invoke<number>('sync_contacts'),
  getAllContacts: () => invoke<StoredContact[]>('get_all_contacts'),
  toggleFavorite: (publicKey: string, favorite: boolean) =>
    invoke<void>('toggle_favorite', { publicKey, favorite }),
  deleteContact: (publicKey: string) =>
    invoke<void>('delete_contact', { publicKey }),

  // Channels
  getAllChannels: () => invoke<StoredChannel[]>('get_all_channels'),
  markAsRead: (channelIdx: number) =>
    invoke<void>('mark_as_read', { channelIdx }),
  syncChannelToDevice: (channelIdx: number, name: string, psk: number[]) =>
    invoke<void>('sync_channel_to_device', { channelIdx, name, psk }),
  upsertChannel: (idx: number, name: string, channelType: string, psk: number[], notificationsEnabled: boolean) =>
    invoke<void>('upsert_channel', { idx, name, channelType, psk, notificationsEnabled }),
  deleteChannel: (channelIdx: number) =>
    invoke<void>('delete_channel', { channelIdx }),

  // Device
  getDeviceInfo: () => invoke<DeviceInfoSummary>('get_device_info'),
  getBattery: (chemistry: string) =>
    invoke<[number, number]>('get_battery', { chemistry }),
  syncTime: () => invoke<void>('sync_time'),
  setDeviceName: (name: string) =>
    invoke<void>('set_device_name', { name }),
  setTxPower: (power: number) =>
    invoke<void>('set_tx_power', { power }),
  reboot: () => invoke<void>('reboot'),
  scanSerialPorts: () => invoke<string[]>('scan_serial_ports'),

  // Companions
  getAllCompanions: () => invoke<StoredCompanion[]>('get_all_companions'),
  saveCompanion: (transportType: string, name: string, address: string, pin: string | null) =>
    invoke<void>('save_companion', { transportType, name, address, pin }),
  deleteCompanion: (id: number) =>
    invoke<boolean>('delete_companion', { id }),

  // Settings
  getSetting: (key: string) => invoke<string | null>('get_setting', { key }),
  setSetting: (key: string, value: string) =>
    invoke<void>('set_setting', { key, value }),
  getAllSettings: () => invoke<[string, string][]>('get_all_settings'),

  // Repeater admin
  repeaterLogin: (pubkey: string, password: string) =>
    invoke<string>('repeater_login', { pubkey, password }),
  repeaterLogout: (pubkey: string) =>
    invoke<void>('repeater_logout', { pubkey }),
  repeaterStatus: (pubkey: string) =>
    invoke<RepeaterStatus>('repeater_status', { pubkey }),
  repeaterNeighbours: (pubkey: string, count: number, offset: number) =>
    invoke<RepeaterNeighbour[]>('repeater_neighbours', { pubkey, count, offset }),
  repeaterTelemetry: (pubkey: string) =>
    invoke<number[]>('repeater_telemetry', { pubkey }),
  repeaterAcl: (pubkey: string) =>
    invoke<{ pubkey_hex: string; permissions: number; name: string | null }[]>('repeater_acl', { pubkey }),
  sendAdvert: (flood: boolean) =>
    invoke<void>('send_advert', { flood }),
  repeaterSendCli: (pubkey: string, command: string) =>
    invoke<string>('repeater_send_cli', { pubkey, command }),
}
