import { useCallback, useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useStore } from '../store'
import { tauri } from '../hooks/useTauri'
import { EmojiPicker } from '../components/EmojiPicker'
import { MessageContent } from '../components/MessageContent'
import { Send, Hash, User, Plus, Trash2, Check, CheckCheck, X, AlertCircle, ChevronUp, Reply, Search, Server, Loader2 } from 'lucide-react'
import type { StoredMessage } from '../types'

const PAGE_SIZE = 50

export function ChatView() {
  const { t } = useTranslation()
  const channels = useStore((s) => s.channels)
  const setChannels = useStore((s) => s.setChannels)
  const contacts = useStore((s) => s.contacts)
  const selectedChannel = useStore((s) => s.selectedChannel)
  const selectedContact = useStore((s) => s.selectedContact)
  const selectChannel = useStore((s) => s.selectChannel)
  const selectContact = useStore((s) => s.selectContact)
  const channelMessages = useStore((s) => s.channelMessages)
  const directMessages = useStore((s) => s.directMessages)
  const setChannelMessages = useStore((s) => s.setChannelMessages)
  const setDirectMessages = useStore((s) => s.setDirectMessages)
  const appendChannelMessage = useStore((s) => s.appendChannelMessage)
  const appendDirectMessage = useStore((s) => s.appendDirectMessage)
  const setError = useStore((s) => s.setError)
  const status = useStore((s) => s.connectionStatus)

  const [input, setInput] = useState('')
  const [showNewChannel, setShowNewChannel] = useState(false)
  const [newChName, setNewChName] = useState('')
  const [newChType, setNewChType] = useState<'custom' | 'public'>('custom')
  const [loadingMsgs, setLoadingMsgs] = useState(false)
  const [loadingMore, setLoadingMore] = useState(false)
  const [hasMore, setHasMore] = useState(true)
  const [replyTo, setReplyTo] = useState<{ id: string; sender: string; text: string } | null>(null)
  const [searchOpen, setSearchOpen] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')
  const [searchResults, setSearchResults] = useState<StoredMessage[]>([])
  const [dmPubkeys, setDmPubkeys] = useState<string[]>([])
  const scrollRef = useRef<HTMLDivElement>(null)

  // Séparer les DM clients des rooms


  useEffect(() => {
    tauri.getAllChannels().then(setChannels).catch(() => {})
    tauri.getDmContactPubkeys().then(setDmPubkeys).catch(() => {})
  }, [setChannels])

  // Charger les messages initiaux
  useEffect(() => {
    if (selectedChannel !== null) {
      setLoadingMsgs(true)
      setHasMore(true)
      tauri.getChannelMessages(selectedChannel, PAGE_SIZE, 0)
        .then((msgs) => {
          setChannelMessages(selectedChannel, msgs.reverse())
          setHasMore(msgs.length === PAGE_SIZE)
        })
        .catch(() => {})
        .finally(() => setLoadingMsgs(false))
      tauri.markAsRead(selectedChannel).catch(() => {})
    }
  }, [selectedChannel, setChannelMessages])

  useEffect(() => {
    if (selectedContact) {
      setLoadingMsgs(true)
      setHasMore(true)
      tauri.getDirectMessages(selectedContact, PAGE_SIZE, 0)
        .then((msgs) => {
          setDirectMessages(selectedContact, msgs.reverse())
          setHasMore(msgs.length === PAGE_SIZE)
        })
        .catch(() => {})
        .finally(() => setLoadingMsgs(false))
    }
  }, [selectedContact, setDirectMessages])

  const currentMessages = selectedChannel !== null
    ? channelMessages[selectedChannel] ?? []
    : selectedContact
      ? directMessages[selectedContact] ?? []
      : []

  const loadOlder = useCallback(async () => {
    if (loadingMore || !hasMore) return
    setLoadingMore(true)
    try {
      const offset = currentMessages.length
      let older: typeof currentMessages = []
      if (selectedChannel !== null) {
        older = await tauri.getChannelMessages(selectedChannel, PAGE_SIZE, offset)
        if (older.length > 0) {
          const existing = useStore.getState().channelMessages[selectedChannel] ?? []
          setChannelMessages(selectedChannel, [...older.reverse(), ...existing])
        }
      } else if (selectedContact) {
        older = await tauri.getDirectMessages(selectedContact, PAGE_SIZE, offset)
        if (older.length > 0) {
          const existing = useStore.getState().directMessages[selectedContact] ?? []
          setDirectMessages(selectedContact, [...older.reverse(), ...existing])
        }
      }
      setHasMore(older.length === PAGE_SIZE)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoadingMore(false)
    }
  }, [loadingMore, hasMore, currentMessages.length, selectedChannel, selectedContact, setChannelMessages, setDirectMessages, setError])

  const messagesEndRef = useRef<HTMLDivElement>(null)
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [currentMessages.length])

  const handleSend = async () => {
    const text = input.trim()
    if (!text) return
    setInput('')
    setReplyTo(null)
    try {
      if (selectedChannel !== null) {
        const msgId = await tauri.sendChannelMessage(selectedChannel, text)
        appendChannelMessage(selectedChannel, {
          id: msgId, direction: 'outgoing', sender_pubkey: null, sender_name: '',
          recipient_pubkey: null, channel_idx: selectedChannel, text,
          timestamp: new Date().toISOString(), status: 'sent',
          snr: null, rssi: null, path_len: null, attempt: 1, reply_to: null, reaction: null,
        })
      } else if (selectedContact) {
        const msgId = await tauri.sendDirectMessage(selectedContact, text)
        appendDirectMessage(selectedContact, {
          id: msgId, direction: 'outgoing', sender_pubkey: null, sender_name: '',
          recipient_pubkey: selectedContact, channel_idx: null, text,
          timestamp: new Date().toISOString(), status: 'sent',
          snr: null, rssi: null, path_len: null, attempt: 1, reply_to: null, reaction: null,
        })
      }
    } catch (e) {
      setError(String(e))
    }
  }

  const PUBLIC_PSK = [0x8b,0x33,0x87,0xe9,0xc5,0xcd,0xea,0x6a,0xc9,0xe5,0xed,0xba,0xa1,0x15,0xcd,0x72]

  const handleCreateChannel = async () => {
    if (!newChName.trim() && newChType !== 'public') return
    try {
      const usedIdx = new Set(channels.map((c) => c.idx))
      let idx = 0
      while (usedIdx.has(idx) && idx < 8) idx++
      if (idx >= 8) { setError('Maximum 8 canaux — supprimez un canal existant'); return }

      const name = newChType === 'public' ? 'Public' : newChName.trim()
      const psk = newChType === 'public'
        ? PUBLIC_PSK
        : Array.from({ length: 16 }, () => Math.floor(Math.random() * 256))

      await tauri.upsertChannel(idx, name, 'public', psk, true)
      if (status === 'connected') {
        await tauri.syncChannelToDevice(idx, name, psk)
      }
      const updated = await tauri.getAllChannels()
      setChannels(updated)
      setShowNewChannel(false)
      setNewChName('')
      setNewChType('custom')
    } catch (e) {
      setError(String(e))
    }
  }

  const handleSearch = async (q: string) => {
    setSearchQuery(q)
    if (q.trim().length < 2) { setSearchResults([]); return }
    try {
      const results = await tauri.searchMessages(q.trim(), 20)
      setSearchResults(results)
    } catch { setSearchResults([]) }
  }

  const handleDeleteChannel = async (idx: number) => {
    try {
      await tauri.deleteChannel(idx)
      const updated = await tauri.getAllChannels()
      setChannels(updated)
      if (selectedChannel === idx) selectChannel(null)
    } catch (e) {
      setError(String(e))
    }
  }

  // Résoudre le nom du contact sélectionné
  const selectedContactInfo = selectedContact
    ? contacts.find((c) => c.public_key === selectedContact || c.public_key.startsWith(selectedContact) || selectedContact.startsWith(c.public_key))
    : null
  const contactName = selectedContactInfo?.name ?? (selectedContact ? selectedContact.slice(0, 12) + '…' : null)
  const isRoom = selectedContactInfo?.node_type === 3

  // DMs filtrés : exclure les rooms (elles ont leur propre section)
  const dmOnlyPubkeys = dmPubkeys.filter((pk) => {
    const c = contacts.find((c) => c.public_key === pk || c.public_key.startsWith(pk) || pk.startsWith(c.public_key))
    return !c || c.node_type !== 3
  })
  // Rooms : celles avec historique + la room actuellement sélectionnée (même sans messages encore)
  const roomsFromHistory = dmPubkeys.filter((pk) => {
    const c = contacts.find((c) => c.public_key === pk || c.public_key.startsWith(pk) || pk.startsWith(c.public_key))
    return c && c.node_type === 3
  })
  const allRoomPubkeys = [...new Set([
    ...roomsFromHistory,
    ...(selectedContact && selectedContactInfo?.node_type === 3 ? [selectedContact] : []),
  ])]

  const statusIcon = (s: string) => {
    switch (s) {
      case 'sent': return <Check className="h-3 w-3 text-gray-400" />
      case 'delivered': return <CheckCheck className="h-3 w-3 text-emerald-400" />
      case 'failed': return <AlertCircle className="h-3 w-3 text-red-400" />
      case 'pending': return <span className="h-3 w-3 inline-block rounded-full border border-gray-500 animate-pulse" />
      default: return null
    }
  }

  return (
    <div className="flex h-full">
      {/* Sidebar : canaux, rooms, DM */}
      <div className="w-56 border-r border-gray-800 bg-gray-900 flex flex-col">
        <div className="flex items-center justify-between p-3">
          <span className="text-xs font-semibold uppercase text-gray-500">{t('chat.title')}</span>
          <button
            onClick={() => setShowNewChannel(!showNewChannel)}
            className="text-gray-500 hover:text-emerald-400 transition-colors"
          >
            {showNewChannel ? <X className="h-4 w-4" /> : <Plus className="h-4 w-4" />}
          </button>
        </div>

        {showNewChannel && (
          <div className="mx-2 mb-2 space-y-2 rounded-lg border border-gray-700 bg-gray-800 p-2">
            <div className="flex gap-1">
              <button
                onClick={() => setNewChType('public')}
                className={`flex-1 rounded px-2 py-1 text-xs ${newChType === 'public' ? 'bg-emerald-600 text-white' : 'bg-gray-700 text-gray-400'}`}
              >
                Public
              </button>
              <button
                onClick={() => setNewChType('custom')}
                className={`flex-1 rounded px-2 py-1 text-xs ${newChType === 'custom' ? 'bg-emerald-600 text-white' : 'bg-gray-700 text-gray-400'}`}
              >
                Personnalisé
              </button>
            </div>
            {newChType === 'custom' && (
              <input
                type="text"
                value={newChName}
                onChange={(e) => setNewChName(e.target.value)}
                placeholder="Nom du canal"
                onKeyDown={(e) => e.key === 'Enter' && handleCreateChannel()}
                className="w-full rounded border border-gray-600 bg-gray-900 px-2 py-1 text-xs text-white focus:outline-none"
              />
            )}
            {newChType === 'public' && (
              <p className="text-[10px] text-gray-500">Canal Public universel MeshCore (PSK standard)</p>
            )}
            <button
              onClick={handleCreateChannel}
              className="w-full rounded bg-emerald-600 py-1 text-xs text-white hover:bg-emerald-700"
            >
              Créer (index auto)
            </button>
          </div>
        )}

        <div className="flex-1 overflow-auto space-y-0.5 px-2">
          {/* Canaux */}
          {channels.map((ch) => (
            <div key={ch.idx} className="group flex items-center">
              <button
                onClick={() => selectChannel(ch.idx)}
                className={`flex flex-1 items-center gap-2 rounded-lg px-3 py-2 text-sm transition-colors ${
                  selectedChannel === ch.idx
                    ? 'bg-gray-800 text-white'
                    : 'text-gray-400 hover:bg-gray-800/50'
                }`}
              >
                <Hash className="h-3.5 w-3.5 shrink-0" />
                <span className="truncate">{ch.name}</span>
                {ch.unread_count > 0 && (
                  <span className="ml-auto rounded-full bg-emerald-600 px-1.5 py-0.5 text-[10px] text-white">
                    {ch.unread_count}
                  </span>
                )}
              </button>
              <button
                onClick={() => handleDeleteChannel(ch.idx)}
                className="hidden group-hover:block text-gray-600 hover:text-red-400 pr-1"
                title="Supprimer ce canal"
              >
                <Trash2 className="h-3 w-3" />
              </button>
            </div>
          ))}

          {/* Section Rooms */}
          {allRoomPubkeys.length > 0 && (
            <>
              <div className="px-3 pt-3 pb-1 text-xs font-semibold uppercase text-gray-500">
                <Server className="mr-1 inline h-3 w-3" />
                Rooms ({allRoomPubkeys.length})
              </div>
              {allRoomPubkeys.map((pk) => {
                const room = contacts.find((c) => c.public_key === pk || c.public_key.startsWith(pk) || pk.startsWith(c.public_key))
                const name = room?.name ?? pk.slice(0, 12) + '…'
                return (
                  <button
                    key={pk}
                    onClick={() => selectContact(pk)}
                    className={`flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm transition-colors ${
                      selectedContact === pk
                        ? 'bg-gray-800 text-white'
                        : 'text-gray-400 hover:bg-gray-800/50'
                    }`}
                  >
                    <Server className="h-3.5 w-3.5 shrink-0 text-purple-400" />
                    <span className="truncate">{name}</span>
                  </button>
                )
              })}
            </>
          )}

          {/* Section Messages directs (clients uniquement) */}
          {dmOnlyPubkeys.length > 0 && (
            <>
              <div className="px-3 pt-3 pb-1 text-xs font-semibold uppercase text-gray-500">
                {t('chat.direct')} ({dmOnlyPubkeys.length})
              </div>
              {dmOnlyPubkeys.map((pk) => {
                const contact = contacts.find((c) => c.public_key === pk || c.public_key.startsWith(pk) || pk.startsWith(c.public_key))
                const name = contact?.name ?? pk.slice(0, 12) + '…'
                return (
                  <button
                    key={pk}
                    onClick={() => selectContact(pk)}
                    className={`flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm transition-colors ${
                      selectedContact === pk
                        ? 'bg-gray-800 text-white'
                        : 'text-gray-400 hover:bg-gray-800/50'
                    }`}
                  >
                    <User className="h-3.5 w-3.5" />
                    <span className="truncate">{name}</span>
                  </button>
                )
              })}
            </>
          )}
        </div>
      </div>

      {/* Zone de chat */}
      <div className="flex flex-1 flex-col">
        <div className="border-b border-gray-800">
          <div className="flex items-center justify-between px-4 py-3">
            <div className="flex items-center gap-2">
              {isRoom && <Server className="h-4 w-4 text-purple-400" />}
              <h3 className="text-sm font-medium text-white">
                {selectedChannel !== null
                  ? `# ${channels.find((c) => c.idx === selectedChannel)?.name ?? ''}`
                  : contactName
                    ? contactName
                    : t('chat.title')}
              </h3>
              {isRoom && (
                <span className="rounded bg-purple-900/40 px-1.5 py-0.5 text-[10px] text-purple-300">Room</span>
              )}
            </div>
            <div className="flex items-center gap-2">
              {selectedChannel !== null && (
                <span className="text-xs text-gray-500">Canal {selectedChannel}</span>
              )}
              {selectedContact && (
                <button
                  onClick={async () => {
                    const label = isRoom
                      ? 'Effacer l\'historique de cette room ? (La room reste accessible)'
                      : 'Effacer l\'historique de cette conversation ?'
                    if (!confirm(label)) return
                    try {
                      await tauri.deleteConversation(selectedContact)
                      setDirectMessages(selectedContact, [])
                      // Rafraîchir la liste des DM
                      tauri.getDmContactPubkeys().then(setDmPubkeys).catch(() => {})
                    } catch (e) { setError(String(e)) }
                  }}
                  className="flex items-center gap-1 rounded px-2 py-1 text-xs text-gray-500 hover:text-red-400 hover:bg-red-900/20 transition-colors"
                  title="Effacer l'historique des messages"
                >
                  <Trash2 className="h-3.5 w-3.5" />
                  <span className="hidden sm:inline">Effacer l'historique</span>
                </button>
              )}
              <button
                onClick={() => { setSearchOpen(!searchOpen); setSearchResults([]) }}
                className={`rounded p-1 transition-colors ${searchOpen ? 'bg-gray-700 text-white' : 'text-gray-500 hover:text-gray-300'}`}
              >
                <Search className="h-4 w-4" />
              </button>
            </div>
          </div>
          {searchOpen && (
            <div className="border-t border-gray-800 px-4 py-2">
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => handleSearch(e.target.value)}
                placeholder="Rechercher dans les messages..."
                autoFocus
                className="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-1.5 text-sm text-white placeholder-gray-500 focus:border-emerald-500 focus:outline-none"
              />
              {searchResults.length > 0 && (
                <div className="mt-2 max-h-48 overflow-auto space-y-1">
                  {searchResults.map((r) => (
                    <div key={r.id} className="rounded-lg bg-gray-800 px-3 py-2 text-xs">
                      <div className="flex items-center justify-between text-gray-500">
                        <span className="font-medium text-gray-300">{r.sender_name || 'Moi'}</span>
                        <span>{new Date(r.timestamp).toLocaleDateString()}</span>
                      </div>
                      <p className="mt-0.5 text-gray-300 truncate">{r.text}</p>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>

        {/* Zone messages */}
        <div ref={scrollRef} className="flex-1 overflow-auto">
          {loadingMsgs ? (
            <div className="flex items-center justify-center py-12 text-sm text-gray-500">
              <Loader2 className="mr-2 h-4 w-4 animate-spin text-emerald-400" />
              {isRoom ? 'Connexion à la room...' : t('common.loading')}
            </div>
          ) : currentMessages.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-sm text-gray-500">
              {isRoom ? (
                <>
                  <Server className="mb-2 h-8 w-8 text-purple-400" />
                  <p>Room connectée — en attente de messages</p>
                  <p className="mt-1 text-xs text-gray-600">Les messages apparaîtront ici</p>
                </>
              ) : (
                <p>{t('chat.noMessages')}</p>
              )}
            </div>
          ) : (
            <div className="p-4 space-y-2">
              {hasMore && (
                <div className="mb-3 flex justify-center">
                  <button
                    onClick={loadOlder}
                    disabled={loadingMore}
                    className="flex items-center gap-1.5 rounded-lg bg-gray-800 px-3 py-1.5 text-xs text-gray-400 hover:bg-gray-700 disabled:opacity-50 transition-colors"
                  >
                    {loadingMore ? (
                      <span className="h-3 w-3 animate-spin rounded-full border border-gray-500 border-t-emerald-400" />
                    ) : (
                      <ChevronUp className="h-3 w-3" />
                    )}
                    Messages plus anciens
                  </button>
                </div>
              )}

              {currentMessages.map((msg) => (
                <div key={msg.id} className={`flex flex-col ${msg.direction === 'outgoing' ? 'items-end' : 'items-start'}`}>
                  <div className="mb-0.5 flex items-center gap-2 text-xs text-gray-500">
                    <span className="font-medium">{msg.sender_name || 'Moi'}</span>
                    <span>{new Date(msg.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
                    {msg.snr !== null && <span className="text-gray-600">SNR {msg.snr.toFixed(1)}</span>}
                    {msg.rssi !== null && <span className="text-gray-600">RSSI {msg.rssi}</span>}
                  </div>
                  {msg.reply_to && (() => {
                    const quoted = currentMessages.find((m) => m.id === msg.reply_to)
                    return quoted ? (
                      <div className="mb-1 max-w-md rounded-lg border-l-2 border-emerald-500 bg-gray-800/50 px-2.5 py-1 text-xs text-gray-400">
                        <span className="font-medium text-gray-300">{quoted.sender_name || 'Moi'}</span>
                        <p className="truncate">{quoted.text}</p>
                      </div>
                    ) : null
                  })()}
                  <div className="group/msg flex items-end gap-1">
                    {msg.direction === 'incoming' && (
                      <>
                        <button
                          onClick={() => setReplyTo({ id: msg.id, sender: msg.sender_name || 'Moi', text: msg.text })}
                          className="opacity-0 group-hover/msg:opacity-100 text-gray-600 hover:text-gray-400 transition-all"
                        >
                          <Reply className="h-3.5 w-3.5" />
                        </button>
                        <EmojiPicker onSelect={() => {}} />
                      </>
                    )}
                    <div
                      className={`max-w-md rounded-2xl px-3.5 py-2 text-sm ${
                        msg.direction === 'outgoing'
                          ? 'bg-emerald-700 text-white rounded-br-sm'
                          : 'bg-gray-800 text-gray-200 rounded-bl-sm'
                      }`}
                    >
                      <MessageContent text={msg.text} />
                    </div>
                    {msg.direction === 'outgoing' && (
                      <>
                        <EmojiPicker onSelect={() => {}} />
                        <button
                          onClick={() => setReplyTo({ id: msg.id, sender: msg.sender_name || 'Moi', text: msg.text })}
                          className="opacity-0 group-hover/msg:opacity-100 text-gray-600 hover:text-gray-400 transition-all"
                        >
                          <Reply className="h-3.5 w-3.5" />
                        </button>
                      </>
                    )}
                  </div>
                  {msg.reaction && (
                    <span className="mt-0.5 inline-block rounded-full bg-gray-800 px-1.5 py-0.5 text-xs">
                      {msg.reaction}
                    </span>
                  )}
                  {msg.direction === 'outgoing' && (
                    <div className="mt-0.5 flex items-center gap-1">
                      {statusIcon(msg.status)}
                      {msg.status === 'failed' && <span className="text-[10px] text-red-400">Echec</span>}
                    </div>
                  )}
                </div>
              ))}
              <div ref={messagesEndRef} />
            </div>
          )}
        </div>

        {(selectedChannel !== null || selectedContact) && (
          <div className="border-t border-gray-800 p-3">
            {status !== 'connected' && (
              <p className="mb-2 text-center text-xs text-yellow-500">Connectez un appareil pour envoyer des messages</p>
            )}
            {replyTo && (
              <div className="mb-2 flex items-center gap-2 rounded-lg border-l-2 border-emerald-500 bg-gray-800/50 px-3 py-1.5 text-xs">
                <Reply className="h-3 w-3 shrink-0 text-emerald-400" />
                <div className="flex-1 min-w-0">
                  <span className="font-medium text-gray-300">{replyTo.sender}</span>
                  <p className="truncate text-gray-400">{replyTo.text}</p>
                </div>
                <button onClick={() => setReplyTo(null)} className="text-gray-500 hover:text-gray-300">
                  <X className="h-3.5 w-3.5" />
                </button>
              </div>
            )}
            <div className="flex gap-2">
              <input
                type="text"
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && !e.shiftKey && handleSend()}
                placeholder={t('chat.placeholder')}
                disabled={status !== 'connected'}
                className="flex-1 rounded-xl border border-gray-700 bg-gray-800 px-4 py-2 text-sm text-white placeholder-gray-500 focus:border-emerald-500 focus:outline-none disabled:opacity-50"
              />
              <button
                onClick={handleSend}
                disabled={status !== 'connected' || !input.trim()}
                className="rounded-xl bg-emerald-600 px-4 py-2 text-white hover:bg-emerald-700 transition-colors disabled:opacity-50"
              >
                <Send className="h-4 w-4" />
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
