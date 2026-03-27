const IMAGE_REGEX = /https?:\/\/\S+\.(?:png|jpg|jpeg|gif|webp|svg)(?:\?\S*)?/gi

export function MessageContent({ text }: { text: string }) {
  const parts: { type: 'text' | 'image'; value: string }[] = []
  let lastIndex = 0

  for (const match of text.matchAll(IMAGE_REGEX)) {
    if (match.index! > lastIndex) {
      parts.push({ type: 'text', value: text.slice(lastIndex, match.index!) })
    }
    parts.push({ type: 'image', value: match[0] })
    lastIndex = match.index! + match[0].length
  }
  if (lastIndex < text.length) {
    parts.push({ type: 'text', value: text.slice(lastIndex) })
  }

  if (parts.length === 0) return <>{text}</>

  return (
    <>
      {parts.map((part, i) =>
        part.type === 'image' ? (
          <img
            key={i}
            src={part.value}
            alt=""
            className="mt-1 max-w-[280px] max-h-[200px] rounded-lg object-cover cursor-pointer"
            loading="lazy"
            onClick={() => window.open(part.value, '_blank')}
          />
        ) : (
          <span key={i}>{part.value}</span>
        )
      )}
    </>
  )
}
