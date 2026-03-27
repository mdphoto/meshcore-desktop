import '@testing-library/jest-dom/vitest'
import { vi } from 'vitest'

// Mock @tauri-apps/api/core — retourne des valeurs par défaut sensées
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockImplementation((cmd: string) => {
    switch (cmd) {
      case 'get_all_contacts': return Promise.resolve([])
      case 'get_all_channels': return Promise.resolve([])
      case 'get_all_companions': return Promise.resolve([])
      case 'get_all_settings': return Promise.resolve([])
      case 'get_setting': return Promise.resolve(null)
      case 'is_connected': return Promise.resolve(false)
      case 'scan_serial_ports': return Promise.resolve([])
      case 'search_messages': return Promise.resolve([])
      case 'delete_conversation': return Promise.resolve(0)
      default: return Promise.resolve(undefined)
    }
  }),
}))

// Mock @tauri-apps/plugin-notification
vi.mock('@tauri-apps/plugin-notification', () => ({
  isPermissionGranted: vi.fn().mockResolvedValue(true),
  requestPermission: vi.fn().mockResolvedValue('granted'),
  sendNotification: vi.fn(),
}))

// Mock @tauri-apps/api/event
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  emit: vi.fn().mockResolvedValue(undefined),
}))

// Mock leaflet (avoid DOM issues in tests)
vi.mock('leaflet', () => ({
  default: {
    icon: vi.fn(() => ({})),
    latLngBounds: vi.fn(() => ({ extend: vi.fn() })),
  },
  icon: vi.fn(() => ({})),
  latLngBounds: vi.fn(() => ({ extend: vi.fn() })),
}))

vi.mock('react-leaflet', () => ({
  MapContainer: ({ children }: { children: React.ReactNode }) => children,
  TileLayer: () => null,
  Marker: ({ children }: { children: React.ReactNode }) => children,
  Popup: ({ children }: { children: React.ReactNode }) => children,
  useMap: () => ({ fitBounds: vi.fn() }),
}))
