import type { Model } from '../types'
import { EYE_PRESETS } from '../types'
import { Icon } from './Icon'
import { Mascot } from './Mascot'

export function SettingsView({ model }: { model: Model }) {
  const { settings, patchSettings, accent } = model
  const eye = settings.eyeColor

  return (
    <div className="settings-view">
      <div className="settings-block">
        <div className="settings-head">
          <span className="eyebrow">Mascot colour</span>
          <span className="mono muted">{eye.toUpperCase()}</span>
        </div>

        <div className="settings-preview">
          <Mascot color={eye} glow={settings.glow} mood="idle" width={54} />
          <span className="muted">Liebe-inspired eyes. Your colour, your glow.</span>
        </div>

        <div className="swatches">
          {EYE_PRESETS.map((preset) => (
            <button
              key={preset.hex}
              className={'swatch' + (preset.hex.toLowerCase() === eye.toLowerCase() ? ' is-active' : '')}
              style={{ background: preset.hex }}
              title={preset.name}
              onClick={() => patchSettings({ eyeColor: preset.hex })}
              aria-label={preset.name}
            />
          ))}
        </div>

        <div className="settings-row">
          <label className="colour-input">
            <input
              type="color"
              value={eye}
              onChange={(e) => patchSettings({ eyeColor: e.target.value })}
              aria-label="Custom eye colour"
            />
            <span>Custom</span>
          </label>

          <label className="slider-field">
            <span>Glow</span>
            <input
              type="range"
              min={0}
              max={100}
              value={settings.glow}
              onChange={(e) => patchSettings({ glow: Number(e.target.value) })}
            />
            <b className="mono">{settings.glow}</b>
          </label>
        </div>
      </div>

      <div className="settings-block">
        <div className="settings-head">
          <span className="eyebrow">Behaviour</span>
        </div>
        <div className="toggles">
          <Toggle
            label="Lirik di notch kecil"
            hint="Tampilkan lirik saat musik diputar"
            on={settings.compactLyrics}
            onChange={(v) => patchSettings({ compactLyrics: v })}
            accent={accent}
          />
          <Toggle
            label="Show mascot eyes"
            hint="Clock only when off"
            on={settings.mascot}
            onChange={(v) => patchSettings({ mascot: v })}
            accent={accent}
          />
          <Toggle
            label="Link accent to eyes"
            hint="UI follows the eye colour"
            on={settings.accentLinked}
            onChange={(v) => patchSettings({ accentLinked: v })}
            accent={accent}
          />
          <Toggle
            label="Minimal notch"
            hint="Thin line when closed"
            on={settings.minimal}
            onChange={(v) => patchSettings({ minimal: v })}
            accent={accent}
          />
          <Toggle
            label="Floating pill"
            hint="Detach and drag anywhere"
            on={settings.floating}
            onChange={(v) => patchSettings({ floating: v })}
            accent={accent}
          />
          {!settings.accentLinked && (
            <label className="colour-input small">
              <input
                type="color"
                value={settings.accent}
                onChange={(e) => patchSettings({ accent: e.target.value })}
                aria-label="Custom accent"
              />
              <span>Accent colour</span>
            </label>
          )}
        </div>
      </div>

      <p className="settings-foot">
        <Icon name="sparkle" size={12} /> These settings persist in this browser while you prototype.
      </p>
    </div>
  )
}

function Toggle({
  label,
  hint,
  on,
  onChange,
  accent,
}: {
  label: string
  hint: string
  on: boolean
  onChange: (value: boolean) => void
  accent: string
}) {
  return (
    <button className="toggle-row" onClick={() => onChange(!on)} aria-pressed={on}>
      <span className="toggle-text">
        <strong>{label}</strong>
        <small>{hint}</small>
      </span>
      <span className="switch" data-on={on} style={on ? { background: accent } : undefined}>
        <i />
      </span>
    </button>
  )
}
