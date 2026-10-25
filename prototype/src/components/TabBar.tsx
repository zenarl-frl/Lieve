import { motion } from 'motion/react'
import type { IconName } from './Icon'
import { Icon } from './Icon'
import type { Model, TabId } from '../types'

const TABS: { id: TabId; label: string; icon: IconName }[] = [
  { id: 'music', label: 'Music', icon: 'music' },
  { id: 'lyrics', label: 'Lyrics', icon: 'text' },
  { id: 'focus', label: 'Focus', icon: 'timer' },
  { id: 'water', label: 'Water', icon: 'droplet' },
  { id: 'shelf', label: 'Shelf', icon: 'layers' },
  { id: 'todo', label: 'To-do', icon: 'check' },
]

export function TabBar({ model }: { model: Model }) {
  return (
    <nav className="tabs" aria-label="Lieve panels">
      {TABS.map((tab) => {
        const active = model.tab === tab.id
        return (
          <button
            key={tab.id}
            className={'tab' + (active ? ' is-active' : '')}
            onClick={() => model.setTab(tab.id)}
            aria-current={active}
          >
            {active && (
              <motion.span
                layoutId="tab-pill"
                className="tab-pill"
                style={{ background: `color-mix(in srgb, ${model.accent} 16%, transparent)` }}
                transition={{ type: 'spring', stiffness: 480, damping: 38 }}
              />
            )}
            <span className="tab-icon" style={active ? { color: model.accent } : undefined}>
              <Icon name={tab.icon} size={13} />
            </span>
            <span className="tab-label">{tab.label}</span>
          </button>
        )
      })}
    </nav>
  )
}
