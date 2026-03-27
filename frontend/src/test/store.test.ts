import { describe, it, expect, beforeEach } from 'vitest'
import { useStore } from '../store'

describe('Store Zustand', () => {
  beforeEach(() => {
    // Reset store entre les tests
    useStore.setState({
      connectionStatus: 'disconnected',
      deviceName: null,
      contacts: [],
      directMessages: {},
      channelMessages: {},
      channels: [],
      deviceInfo: null,
      battery: null,
      radioStats: [],
      companions: [],
      theme: 'dark',
      selectedContact: null,
      selectedChannel: null,
      lastError: null,
    })
  })

  describe('Connection', () => {
    it('setConnected met à jour le statut et le nom', () => {
      useStore.getState().setConnected('MeshCore-1234')
      expect(useStore.getState().connectionStatus).toBe('connected')
      expect(useStore.getState().deviceName).toBe('MeshCore-1234')
    })

    it('setDisconnected réinitialise tout', () => {
      useStore.getState().setConnected('test')
      useStore.getState().setDisconnected()
      expect(useStore.getState().connectionStatus).toBe('disconnected')
      expect(useStore.getState().deviceName).toBeNull()
      expect(useStore.getState().deviceInfo).toBeNull()
    })

    it('setReconnecting met le statut', () => {
      useStore.getState().setReconnecting()
      expect(useStore.getState().connectionStatus).toBe('reconnecting')
    })
  })

  describe('Contacts', () => {
    const contact = {
      public_key: 'abc123',
      name: 'Alice',
      node_type: 0,
      flags: 0,
      path: [],
      path_len: 2,
      lat: 44.75,
      lon: 4.82,
      last_seen: '2026-03-27T10:00:00Z',
      is_favorite: false,
      group_name: null,
    }

    it('setContacts remplace la liste', () => {
      useStore.getState().setContacts([contact])
      expect(useStore.getState().contacts).toHaveLength(1)
      expect(useStore.getState().contacts[0].name).toBe('Alice')
    })

    it('updateContact ajoute un nouveau contact', () => {
      useStore.getState().updateContact(contact)
      expect(useStore.getState().contacts).toHaveLength(1)
    })

    it('updateContact met à jour un contact existant', () => {
      useStore.getState().setContacts([contact])
      useStore.getState().updateContact({ ...contact, name: 'Bob' })
      expect(useStore.getState().contacts).toHaveLength(1)
      expect(useStore.getState().contacts[0].name).toBe('Bob')
    })
  })

  describe('Messages', () => {
    const msg = {
      id: 'msg-1',
      direction: 'outgoing',
      sender_pubkey: null,
      sender_name: '',
      recipient_pubkey: 'abc123',
      channel_idx: null,
      text: 'Hello',
      timestamp: '2026-03-27T10:00:00Z',
      status: 'pending',
      snr: null,
      rssi: null,
      path_len: null,
      attempt: 1,
      reply_to: null,
      reaction: null,
    }

    it('appendDirectMessage ajoute un message', () => {
      useStore.getState().appendDirectMessage('abc123', msg)
      expect(useStore.getState().directMessages['abc123']).toHaveLength(1)
    })

    it('updateMessageStatus modifie le statut', () => {
      useStore.getState().appendDirectMessage('abc123', msg)
      useStore.getState().updateMessageStatus('msg-1', 'delivered')
      expect(useStore.getState().directMessages['abc123'][0].status).toBe('delivered')
    })

    it('appendChannelMessage ajoute un message canal', () => {
      useStore.getState().appendChannelMessage(0, { ...msg, channel_idx: 0 })
      expect(useStore.getState().channelMessages[0]).toHaveLength(1)
    })
  })

  describe('Radio Stats', () => {
    it('addRadioStats accumule les stats', () => {
      const stat = { noise_floor: -110, last_rssi: -85, snr: 8.5, timestamp: '2026-03-27T10:00:00Z' }
      useStore.getState().addRadioStats(stat)
      useStore.getState().addRadioStats({ ...stat, snr: 9.0 })
      expect(useStore.getState().radioStats).toHaveLength(2)
    })

    it('addRadioStats garde max 100 entrées', () => {
      for (let i = 0; i < 110; i++) {
        useStore.getState().addRadioStats({
          noise_floor: -110, last_rssi: -85, snr: i, timestamp: `t${i}`,
        })
      }
      expect(useStore.getState().radioStats).toHaveLength(100)
    })
  })

  describe('UI', () => {
    it('selectContact désélectionne le canal', () => {
      useStore.getState().selectChannel(0)
      useStore.getState().selectContact('abc')
      expect(useStore.getState().selectedContact).toBe('abc')
      expect(useStore.getState().selectedChannel).toBeNull()
    })

    it('selectChannel désélectionne le contact', () => {
      useStore.getState().selectContact('abc')
      useStore.getState().selectChannel(0)
      expect(useStore.getState().selectedChannel).toBe(0)
      expect(useStore.getState().selectedContact).toBeNull()
    })

    it('setTheme change le thème', () => {
      useStore.getState().setTheme('light')
      expect(useStore.getState().theme).toBe('light')
    })
  })
})
