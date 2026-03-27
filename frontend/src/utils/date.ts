/** Parse un last_seen qui peut être un timestamp Unix (secondes) ou une date ISO */
export function parseLastSeen(value: string): Date {
  const num = Number(value)
  if (!isNaN(num) && num > 1000000000 && num < 9999999999) {
    return new Date(num * 1000)
  }
  return new Date(value)
}

export function formatLastSeen(value: string): string {
  const d = parseLastSeen(value)
  if (isNaN(d.getTime())) return '—'
  return d.toLocaleDateString()
}
