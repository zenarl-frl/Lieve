import { useState } from 'react'
import type { HudKind, Model, Mood, Stage } from '../types'

const STAGES: { id: Stage; label: string }[] = [
  { id: 'idle', label: 'Idle' },
  { id: 'compact', label: 'Compact' },
  { id: 'panel', label: 'Panel' },
]

const HUDS: { id: HudKind; label: string }[] = [
  { id: 'volume', label: 'Volume' },
  { id: 'brightness', label: 'Bright' },
  { id: 'caps', label: 'Caps' },
  { id: 'lang', label: 'Lang' },
  { id: 'battery', label: 'Battery' },
  { id: 'timer', label: 'Timer' },
]

const MOODS: { id: Mood; label: string }[] = [
  { id: 'idle', label: 'Awake' },
  { id: 'sleepy', label: 'Sleepy' },
  { id: 'alert', label: 'Alert' },
]

interface Props {
  model: Model
  stage: Stage
  onStage: (stage: Stage) => void
  onHud: (kind: HudKind) => void
  mood: Mood
  onMood: (mood: Mood) => void
  onReset: () => void
}

export function DevBar({ model, stage, onStage, onHud, mood, onMood, onReset }: Props) {
  const [open, setOpen] = useState(true)
  const { settings, patchSettings } = model

  return (
    <div className={'devbar' + (open ? ' is-open' : '')}>
      <button className="dev-toggle" onClick={() => setOpen((o) => !o)}>
        <span className="dev-dot" />
        DEV
      </button>

      {open && (
        <div className="dev-body">
          <div className="dev-group">
            <span className="dev-label">Stage</span>
            <div className="dev-btns">
              {STAGES.map((s) => (
                <button
                  key={s.id}
                  className={stage === s.id ? 'is-active' : ''}
                  onClick={() => onStage(s.id)}
                >
                  {s.label}
                </button>
              ))}
            </div>
          </div>

          <div className="dev-group">
            <span className="dev-label">HUD</span>
            <div className="dev-btns wrap">
              {HUDS.map((h) => (
                <button key={h.id} onClick={() => onHud(h.id)}>
                  {h.label}
                </button>
              ))}
            </div>
          </div>

          <div className="dev-group">
            <span className="dev-label">Eyes</span>
            <div className="dev-btns">
              {MOODS.map((m) => (
                <button key={m.id} className={mood === m.id ? 'is-active' : ''} onClick={() => onMood(m.id)}>
                  {m.label}
                </button>
              ))}
            </div>
          </div>

          <div className="dev-group">
            <span className="dev-label">Mode</span>
            <div className="dev-btns wrap">
              <button
                className={settings.minimal ? 'is-active' : ''}
                onClick={() => patchSettings({ minimal: !settings.minimal })}
              >
                Minimal
              </button>
              <button
                className={settings.floating ? 'is-active' : ''}
                onClick={() => patchSettings({ floating: !settings.floating })}
              >
                Floating
              </button>
              <button className={model.pinned ? 'is-active' : ''} onClick={model.togglePin}>
                Pin
              </button>
              <button onClick={onReset}>Reset</button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
