import { useState } from 'react'
import { SmilePlus } from 'lucide-react'

const EMOJIS = ['👍', '❤️', '😂', '😮', '😢', '🔥', '👏', '🎉', '💪', '🙏', '✅', '❌']

export function EmojiPicker({ onSelect }: { onSelect: (emoji: string) => void }) {
  const [open, setOpen] = useState(false)

  return (
    <div className="relative">
      <button
        onClick={() => setOpen(!open)}
        className="text-gray-600 hover:text-gray-400 transition-colors"
      >
        <SmilePlus className="h-3.5 w-3.5" />
      </button>
      {open && (
        <div className="absolute bottom-6 left-0 z-10 flex flex-wrap gap-1 rounded-lg border border-gray-700 bg-gray-900 p-2 shadow-lg">
          {EMOJIS.map((e) => (
            <button
              key={e}
              onClick={() => { onSelect(e); setOpen(false) }}
              className="rounded p-1 text-base hover:bg-gray-800 transition-colors"
            >
              {e}
            </button>
          ))}
        </div>
      )}
    </div>
  )
}
