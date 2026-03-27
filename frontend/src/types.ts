export interface StoredContact {
  public_key: string
  name: string
  node_type: number
  flags: number
  path: number[]
  path_len: number
  lat: number
  lon: number
  last_seen: string
  is_favorite: boolean
  group_name: string | null
}

export interface StoredMessage {
  id: string
  direction: string
  sender_pubkey: string | null
  sender_name: string
  recipient_pubkey: string | null
  channel_idx: number | null
  text: string
  timestamp: string
  status: string
  snr: number | null
  rssi: number | null
  path_len: number | null
  attempt: number
  reply_to: string | null
  reaction: string | null
}

export interface StoredChannel {
  idx: number
  name: string
  channel_type: string
  psk: number[]
  notifications_enabled: boolean
  unread_count: number
}

export interface DeviceInfoSummary {
  name: string
  public_key: string
  adv_type: number
  tx_power: number
  max_tx_power: number
  radio_freq: number
  radio_bw: number
  sf: number
  cr: number
  lat: number
  lon: number
}

export interface StoredCompanion {
  id: number | null
  transport_type: string
  name: string
  address: string
  pin: string | null
  last_used: string
}

export interface BleDevice {
  name: string
  address: string
  rssi: number | null
}

// Analyse ligne de vue
export interface ProfilePoint {
  distance_m: number
  terrain_m: number
  los_m: number
  fresnel_radius_m: number
  lat: number
  lon: number
}

export interface ObstructionPoint {
  distance_m: number
  terrain_m: number
  los_m: number
  clearance_m: number
  lat: number
  lon: number
}

export interface LosResult {
  visible: boolean
  distance_m: number
  profile: ProfilePoint[]
  worst_obstruction: ObstructionPoint | null
  min_fresnel_clearance_m: number
}

// Administration repeater
export interface RepeaterStatus {
  battery_mv: number
  tx_queue_len: number
  noise_floor: number
  last_rssi: number
  nb_recv: number
  nb_sent: number
  airtime: number
  uptime: number
  flood_sent: number
  direct_sent: number
  snr: number
  dup_count: number
  rx_airtime: number
}

export interface RepeaterNeighbour {
  pubkey_hex: string
  secs_ago: number
  snr: number
  name: string | null
}

export type ConnectionStatus = 'disconnected' | 'connected' | 'reconnecting'

export type ConnectionMethod = 'serial' | 'tcp' | 'ble'

// Events from Tauri backend
export type AppEvent =
  | { type: 'Connected'; payload: { device_name: string } }
  | { type: 'Disconnected' }
  | { type: 'Reconnecting'; payload: { attempt: number } }
  | { type: 'DirectMessageReceived'; payload: { sender_pubkey: string; sender_name: string; text: string; snr: number | null } }
  | { type: 'ChannelMessageReceived'; payload: { channel_idx: number; sender_name: string; text: string } }
  | { type: 'MessageSent'; payload: { message_id: string } }
  | { type: 'MessageDelivered'; payload: { message_id: string } }
  | { type: 'MessageFailed'; payload: { message_id: string; reason: string } }
  | { type: 'ContactsSynced'; payload: { count: number } }
  | { type: 'ContactDiscovered'; payload: { pubkey: string; name: string; node_type: number } }
  | { type: 'PathUpdated'; payload: { pubkey_prefix: string; path_len: number } }
  | { type: 'BatteryUpdate'; payload: { millivolts: number; percent: number } }
  | { type: 'DeviceInfoReceived'; payload: { name: string; fw_version: string } }
  | { type: 'StatsReceived'; payload: { noise_floor: number; last_rssi: number; snr: number } }
  | { type: 'Error'; payload: { message: string } }
