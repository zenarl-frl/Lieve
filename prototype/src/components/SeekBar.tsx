import { useRef } from 'react'
import { formatTime } from '../lib/hooks'

interface Props {
  position: number
  duration: number
  onSeek: (seconds: number) => void
  accent: string
  compact?: boolean
  showTimes?: boolean
}

export function SeekBar({ position, duration, onSeek, accent, compact, showTimes }: Props) {
  const ref = useRef<HTMLDivElement>(null)
  const pct = Math.min(100, (position / duration) * 100)

  const seekTo = (clientX: number) => {
    const el = ref.current
    if (!el) return
    const rect = el.getBoundingClientRect()
    const ratio = Math.min(1, Math.max(0, (clientX - rect.left) / rect.width))
    onSeek(ratio * duration)
  }

  return (
    <div className={compact ? 'seek-wrap compact' : 'seek-wrap'}>
      <div
        ref={ref}
        className="seek"
        onPointerDown={(e) => {
          e.currentTarget.setPointerCapture(e.pointerId)
          seekTo(e.clientX)
        }}
        onPointerMove={(e) => {
          if (e.buttons === 1) seekTo(e.clientX)
        }}
        role="slider"
        aria-valuemin={0}
        aria-valuemax={duration}
        aria-valuenow={Math.round(position)}
        tabIndex={0}
      >
        <span className="seek-track" />
        <span className="seek-fill" style={{ width: `${pct}%`, background: accent }} />
        <span className="seek-knob" style={{ left: `${pct}%`, background: accent }} />
      </div>
      {showTimes && (
        <div className="time-row">
          <span>{formatTime(position)}</span>
          <span>-{formatTime(Math.max(0, duration - position))}</span>
        </div>
      )}
    </div>
  )
}
