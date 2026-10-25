import { AnimatePresence, motion } from 'motion/react'
import type { HudKind, HudState } from '../types'
import { Icon, type IconName } from './Icon'

const META: Record<HudKind, { icon: IconName; width: number }> = {
  volume: { icon: 'volume', width: 258 },
  brightness: { icon: 'sun', width: 258 },
  caps: { icon: 'keyboard', width: 236 },
  lang: { icon: 'keyboard', width: 228 },
  battery: { icon: 'battery', width: 264 },
  timer: { icon: 'timer', width: 286 },
}

export function Hud({ hud, accent }: { hud: HudState | null; accent: string }) {
  return (
    <AnimatePresence>
      {hud && (
        <motion.div
          key={hud.kind}
          className="hud"
          initial={{ opacity: 0, y: -16, scale: 0.94 }}
          animate={{ opacity: 1, y: 0, scale: 1, width: META[hud.kind].width }}
          exit={{ opacity: 0, y: -16, scale: 0.96 }}
          transition={{ type: 'spring', stiffness: 420, damping: 34 }}
        >
          <span className="hud-icon" style={{ color: accent }}>
            <Icon name={META[hud.kind].icon} size={16} />
          </span>

          {(hud.kind === 'volume' || hud.kind === 'brightness' || hud.kind === 'battery') && (
            <>
              <div className="hud-bars">
                {Array.from({ length: 16 }, (_, i) => {
                  const on = i < Math.round((hud.value / 100) * 16)
                  return (
                    <span
                      key={i}
                      className={on ? 'on' : ''}
                      style={on ? { background: accent, boxShadow: `0 0 6px ${accent}` } : undefined}
                    />
                  )
                })}
              </div>
              <span className="hud-value mono">{Math.round(hud.value)}%</span>
            </>
          )}

          {hud.kind === 'caps' && (
            <div className="hud-text">
              <strong>Caps Lock on</strong>
              <span>Keyboard</span>
            </div>
          )}

          {hud.kind === 'lang' && (
            <div className="hud-text wide">
              <strong className="mono big">EN</strong>
              <span>{hud.label ?? 'English (United States)'}</span>
            </div>
          )}

          {hud.kind === 'timer' && (
            <div className="hud-text accent" style={{ color: accent }}>
              <strong>Focus session complete</strong>
              <span>Time for a five minute break</span>
            </div>
          )}
        </motion.div>
      )}
    </AnimatePresence>
  )
}
