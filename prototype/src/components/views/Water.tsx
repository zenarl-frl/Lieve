import type { Model } from '../../types'
import { Icon } from '../Icon'

export function WaterView({ model }: { model: Model }) {
  const { intake, goal, logs } = model.water
  const pct = Math.round((intake / goal) * 100)
  const segments = 15
  const filled = Math.round((intake / goal) * segments)
  const maxLog = Math.max(...logs.map((l) => l.ml), 1)

  return (
    <div className="water-view">
      <div className="water-main">
        <div className="water-head">
          <div>
            <span className="eyebrow">Today</span>
            <div className="water-value">
              <b>{(intake / 1000).toFixed(1)}</b>
              <span>/ {(goal / 1000).toFixed(1)} L</span>
            </div>
          </div>
          <span className="pill" style={{ color: model.accent }}>
            {pct}%
          </span>
        </div>

        <div className="segments">
          {Array.from({ length: segments }, (_, i) => (
            <span
              key={i}
              className={i < filled ? 'on' : ''}
              style={i < filled ? { background: model.accent } : undefined}
            />
          ))}
        </div>

        <div className="row-actions">
          <button className="primary-btn" style={{ background: model.accent }} onClick={() => model.drink(200)}>
            <Icon name="droplet" size={13} /> +200 ml
          </button>
          <button className="ghost-btn" onClick={() => model.drink(400)}>
            +400 ml
          </button>
          <button className="ghost-btn quiet" onClick={model.resetWater}>
            Reset
          </button>
        </div>
      </div>

      <div className="water-chart">
        <div className="bars">
          {logs.map((log) => (
            <div className="bar-col" key={log.hour}>
              <div className="bar-track">
                <div
                  className="bar-fill"
                  style={{ height: `${(log.ml / maxLog) * 100}%` }}
                />
              </div>
              <span>{log.hour}</span>
            </div>
          ))}
        </div>
        <small className="muted">
          {intake >= goal ? 'Goal reached · nice' : `${goal - intake} ml to go`}
        </small>
      </div>
    </div>
  )
}
