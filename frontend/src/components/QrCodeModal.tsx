import { useEffect, useRef, useState } from 'react'
import QRCode from 'qrcode'
import { X, Copy, Check } from 'lucide-react'

interface Props {
  data: string
  title: string
  onClose: () => void
}

export function QrCodeModal({ data, title, onClose }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const [copied, setCopied] = useState(false)

  useEffect(() => {
    if (canvasRef.current) {
      QRCode.toCanvas(canvasRef.current, data, {
        width: 256,
        margin: 2,
        color: { dark: '#000000', light: '#ffffff' },
      })
    }
  }, [data])

  const handleCopy = () => {
    navigator.clipboard.writeText(data)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" onClick={onClose}>
      <div className="rounded-2xl border border-gray-700 bg-gray-900 p-6 shadow-xl" onClick={(e) => e.stopPropagation()}>
        <div className="mb-4 flex items-center justify-between">
          <h3 className="text-sm font-semibold text-white">{title}</h3>
          <button onClick={onClose} className="text-gray-500 hover:text-gray-300">
            <X className="h-4 w-4" />
          </button>
        </div>
        <div className="flex justify-center rounded-xl bg-white p-3">
          <canvas ref={canvasRef} />
        </div>
        <div className="mt-3 flex items-center gap-2">
          <code className="flex-1 truncate rounded bg-gray-800 px-2 py-1 text-xs text-gray-400">
            {data}
          </code>
          <button
            onClick={handleCopy}
            className="rounded bg-gray-800 p-1.5 text-gray-400 hover:text-white transition-colors"
          >
            {copied ? <Check className="h-3.5 w-3.5 text-emerald-400" /> : <Copy className="h-3.5 w-3.5" />}
          </button>
        </div>
      </div>
    </div>
  )
}
