import { useEffect, useRef } from 'react'
import { listen } from '@tauri-apps/api/event'
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification'
import { onOpenUrl } from '@tauri-apps/plugin-deep-link'
import { useStore } from '../store'
import { tauri } from './useTauri'
import { parseMeshCoreURI } from '../utils/meshcore-uri'
import type { AppEvent } from '../types'

async function notify(title: string, body: string) {
  try {
    let granted = await isPermissionGranted()
    if (!granted) {
      const permission = await requestPermission()
      granted = permission === 'granted'
    }
    if (granted) {
      sendNotification({ title, body })
    }
  } catch {
    // Notification non disponible
  }
}

const isTauri = () => '__TAURI_INTERNALS__' in window

function handleEvent(e: AppEvent) {
  const store = useStore.getState()
  console.log('[meshcore-event]', e.type, e)
  switch (e.type) {
    case 'Connected':
      store.setConnected(e.payload.device_name)
      tauri.getAllContacts().then((c) => store.setContacts(c)).catch(() => {})
      tauri.getAllChannels().then((c) => store.setChannels(c)).catch(() => {})
      tauri.syncContacts().catch(() => {})
      notify('MeshCore', `Connecté à ${e.payload.device_name}`)
      break
    case 'Disconnected':
      store.setDisconnected()
      break
    case 'Reconnecting':
      store.setReconnecting()
      break
    case 'DirectMessageReceived':
      store.appendDirectMessage(e.payload.sender_pubkey, {
        id: crypto.randomUUID(),
        direction: 'incoming',
        sender_pubkey: e.payload.sender_pubkey,
        sender_name: e.payload.sender_name,
        recipient_pubkey: null,
        channel_idx: null,
        text: e.payload.text,
        timestamp: new Date().toISOString(),
        status: 'received',
        snr: e.payload.snr,
        rssi: null, path_len: null, attempt: 0, reply_to: null, reaction: null,
      })
      notify(e.payload.sender_name, e.payload.text)
      break
    case 'ChannelMessageReceived':
      store.appendChannelMessage(e.payload.channel_idx, {
        id: crypto.randomUUID(),
        direction: 'incoming',
        sender_pubkey: null,
        sender_name: e.payload.sender_name,
        recipient_pubkey: null,
        channel_idx: e.payload.channel_idx,
        text: e.payload.text,
        timestamp: new Date().toISOString(),
        status: 'received',
        snr: null, rssi: null, path_len: null, attempt: 0, reply_to: null, reaction: null,
      })
      { const chName = store.channels.find((c) => c.idx === e.payload.channel_idx)?.name ?? `Canal ${e.payload.channel_idx}`
        notify(`# ${chName}`, `${e.payload.sender_name}: ${e.payload.text}`) }
      break
    case 'MessageSent':
      store.updateMessageStatus(e.payload.message_id, 'sent')
      break
    case 'MessageDelivered':
      store.updateMessageStatus(e.payload.message_id, 'delivered')
      break
    case 'MessageFailed':
      store.updateMessageStatus(e.payload.message_id, 'failed')
      break
    case 'ContactsSynced':
      tauri.getAllContacts().then((c) => store.setContacts(c)).catch(() => {})
      tauri.getAllChannels().then((c) => store.setChannels(c)).catch(() => {})
      break
    case 'BatteryUpdate':
      store.setBattery(e.payload.millivolts, e.payload.percent)
      break
    case 'StatsReceived':
      store.addRadioStats({
        noise_floor: e.payload.noise_floor,
        last_rssi: e.payload.last_rssi,
        snr: e.payload.snr,
        timestamp: new Date().toISOString(),
      })
      break
    case 'Error':
      store.setError(e.payload.message)
      break
  }
}

export function useEvents() {
  // Listener Tauri pour les événements temps réel
  useEffect(() => {
    if (!isTauri()) return
    console.log('[useEvents] Setting up Tauri event listener...')
    const unlisten = listen<AppEvent>('meshcore-event', (event) => {
      handleEvent(event.payload)
    })
    return () => { unlisten.then((fn) => fn()) }
  }, [])

  // Polling de secours : recharge les messages du canal/contact actif toutes les 3s
  const pollRef = useRef<ReturnType<typeof setInterval>>(undefined)
  useEffect(() => {
    if (!isTauri()) return
    pollRef.current = setInterval(() => {
      const store = useStore.getState()
      if (store.connectionStatus !== 'connected') return

      const { selectedChannel, selectedContact } = store
      if (selectedChannel !== null) {
        tauri.getChannelMessages(selectedChannel, 50, 0).then((msgs) => {
          const current = store.channelMessages[selectedChannel] ?? []
          if (msgs.length !== current.length) {
            store.setChannelMessages(selectedChannel, msgs.reverse())
          }
        }).catch(() => {})
      } else if (selectedContact) {
        tauri.getDirectMessages(selectedContact, 50, 0).then((msgs) => {
          const current = store.directMessages[selectedContact] ?? []
          if (msgs.length !== current.length) {
            store.setDirectMessages(selectedContact, msgs.reverse())
          }
        }).catch(() => {})
      }
    }, 3000)

    return () => { if (pollRef.current) clearInterval(pollRef.current) }
  }, [])

  // Deep link handler (meshcore:// URIs)
  useEffect(() => {
    if (!isTauri()) return
    console.log('[useEvents] Setting up deep link listener...')
    const unlisten = onOpenUrl((urls) => {
      for (const url of urls) {
        console.log('[deep-link] Received:', url)
        const parsed = parseMeshCoreURI(url)
        if (!parsed) continue

        const store = useStore.getState()
        if (parsed.type === 'contact') {
          // Sélectionner le contact et naviguer vers le chat
          store.selectContact(parsed.pubkey)
          window.location.hash = '#/chat'
        } else if (parsed.type === 'channel') {
          store.selectChannel(parsed.idx)
          window.location.hash = '#/chat'
        }
      }
    })
    return () => { unlisten.then((fn) => fn()) }
  }, [])
}
