import { useEffect, useRef, useState } from 'react'
import type { Model } from '../../types'
import { Icon } from '../Icon'

const PRESETS = [
  { label: '25', caption: 'Focus', seconds: 25 * 60 },
  { label: '50', caption: 'Deep', seconds: 50 * 60 },
  { label: '5', caption: 'Break', seconds: 5 * 60 },
]

export function FocusView({ model }: { model: Model }) {
  const [total, setTotal] = useState(25 * 60)
  const [left, setLeft] = useState(25 * 60)
  const [running, setRunning] = useState(false)
  const firedRef = useRef(false)

  useEffect(() => {
    if (!running) return
    const id = window.setInterval(() => setLeft((l) => Math.max(0, l - 1)), 1000)
    return () => window.clearInterval(id)
  }, [running])

  useEffect(() => {
    if (left !== 0 || firedRef.current) return
    firedRef.current = true
    setRunning(false)
    model.onTimerDone()
  }, [left, model])

  const pick = (seconds: number) => {
    setTotal(seconds)
    setLeft(seconds)
    setRunning(false)
    firedRef.current = false
  }

  const progress = 1 - left / total
  const radius = 42
  const circumference = 2 * Math.PI * radius

  return (
    <div className="focus-view">
      <div className="ring-wrap">
        <svg width="116" height="116" viewBox="0 0 116 116">
          <circle cx="58" cy="58" r={radius} className="ring-track" />
          <circle
            cx="58"
            cy="58"
            r={radius}
            className="ring-fill"
            stroke={model.accent}
            strokeDasharray={circumference}
            strokeDashoffset={circumference * (1 - progress)}
          />
        </svg>
        <div className="ring-label">
          <span className="mono">
            {String(Math.floor(left / 60)).padStart(2, '0')}:
            {String(left % 60).padStart(2, '0')}
          </span>
          <small>{running ? 'in focus' : left === total ? 'ready' : 'paused'}</small>
        </div>
      </div>

      <div className="focus-side">
        <div className="chips">
          {PRESETS.map((p) => (
            <button
              key={p.label}
              className={'chip' + (p.seconds === total ? ' is-active' : '')}
              onClick={() => pick(p.seconds)}
            >
              <b>{p.label}</b>
              <span>{p.caption}</span>
            </button>
          ))}
        </div>

        <div className="row-actions">
          <button
            className="primary-btn"
            style={{ background: model.accent }}
            onClick={() => {
              if (left === 0) pick(total)
              setRunning((r) => !r)
            }}
          >
            <Icon name={running ? 'pause' : 'play'} size={13} />
            {running ? 'Pause' : left === 0 ? 'Restart' : 'Start'}
          </button>
          <button className="ghost-btn" onClick={() => pick(total)}>
            Reset
          </button>
        </div>

        <p className="hint-line">
          <Icon name="sparkle" size={12} /> Preview timer · keep this panel open
        </p>
      </div>
    </div>
  )
}
