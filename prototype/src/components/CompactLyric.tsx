import { useEffect, useRef, useState } from 'react'
import { motion, useReducedMotion } from 'motion/react'

export function CompactLyric({ text }: { text: string }) {
  const container = useRef<HTMLDivElement>(null)
  const line = useRef<HTMLSpanElement>(null)
  const [overflow, setOverflow] = useState(0)
  const reduced = useReducedMotion()

  useEffect(() => {
    const measure = () => setOverflow(Math.max(0,
      (line.current?.scrollWidth ?? 0) - (container.current?.clientWidth ?? 0)))
    const observer = new ResizeObserver(measure)
    if (container.current) observer.observe(container.current)
    if (line.current) observer.observe(line.current)
    measure()
    return () => observer.disconnect()
  }, [text])

  return (
    <div ref={container} className="compact-lyric" title={text}>
      <motion.span
        key={text}
        ref={line}
        initial={{ opacity: 0, y: reduced ? 0 : 5 }}
        animate={{ opacity: 1, y: 0, x: !reduced && overflow > 0 ? [0, -overflow] : 0 }}
        transition={{ opacity: { duration: 0.22 }, y: { duration: 0.22 },
          x: { delay: 1.5, duration: Math.max(3, overflow / 20), repeat: Infinity,
            repeatType: 'reverse', repeatDelay: 1.5, ease: 'linear' } }}
      >{text}</motion.span>
    </div>
  )
}
