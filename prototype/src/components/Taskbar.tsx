import { Icon } from './Icon'
import { EYE_PATHS } from './Mascot'

interface Props {
  time: string
  date: string
  accent: string
}

export function Taskbar({ time, date, accent }: Props) {
  return (
    <div className="taskbar">
      <div className="tb-left">
        <span className="tb-start">
          <i />
          <i />
          <i />
          <i />
        </span>
        <span className="tb-search" />
      </div>

      <div className="tb-center">
        <span className="tb-app">
          <Icon name="folder" size={15} />
        </span>
        <span className="tb-app">
          <Icon name="image" size={15} />
        </span>
        <span className="tb-app tb-lieve" style={{ color: accent }}>
          <svg viewBox="0 6 160 48" width="20" height="6">
            {EYE_PATHS.map((d) => (
              <path key={d} d={d} fill="currentColor" />
            ))}
          </svg>
        </span>
        <span className="tb-app">
          <Icon name="headphones" size={15} />
        </span>
      </div>

      <div className="tb-right">
        <span className="tb-tray">
          <Icon name="wifi" size={13} />
          <Icon name="volume" size={13} />
          <Icon name="battery" size={13} />
        </span>
        <span className="tb-clock">
          <b>{time}</b>
          <small>{date}</small>
        </span>
      </div>
    </div>
  )
}
