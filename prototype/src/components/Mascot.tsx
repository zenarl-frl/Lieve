import { useEffect, useRef, useState } from 'react'
import { motion } from 'motion/react'
import type { Mood } from '../types'

interface Props {
  color: string
  glow: number
  mood: Mood
  follow?: { x: number; y: number }
  width?: number
}

/** Liebe-inspired eyes: two solid petals, no sclera, no pupil. */
export const EYE_PATHS = [
  // Outer corners high, inner corners taper down, following the reference.
  'M12 7 C37 8 59 26 72 48 C48 52 28 45 19 30 C15 22 13 14 12 7 Z',
  'M148 7 C123 8 101 26 88 48 C112 52 132 45 141 30 C145 22 147 14 148 7 Z',
]

export function Mascot({ color, glow, mood, follow = { x: 0, y: 0 }, width = 46 }: Props) {
  const [blink, setBlink] = useState(false)
  const alive = useRef(true)

  useEffect(() => {
    alive.current = true
    const timers: number[] = []

    const schedule = () => {
      timers.push(
        window.setTimeout(() => {
          if (!alive.current) return
          setBlink(true)
          timers.push(
            window.setTimeout(() => {
              if (!alive.current) return
              setBlink(false)
              if (Math.random() < 0.2) {
                // double blink, then back to the normal rhythm
                timers.push(
                  window.setTimeout(() => {
                    if (!alive.current) return
                    setBlink(true)
                    timers.push(
                      window.setTimeout(() => {
                        if (!alive.current) return
                        setBlink(false)
                        schedule()
                      }, 110),
                    )
                  }, 140),
                )
              } else {
                schedule()
              }
            }, 120),
          )
        }, 2800 + Math.random() * 5200),
      )
    }
    schedule()

    return () => {
      alive.current = false
      timers.forEach((t) => window.clearTimeout(t))
    }
  }, [])

  const sleepy = mood === 'sleepy'
  const alert = mood === 'alert'
  const curious = mood === 'curious'

  const squash = blink ? 0.06 : sleepy ? 0.26 : curious ? 0.72 : 1
  const scale = alert ? 1.12 : 1
  const tilt = sleepy ? 3.5 : alert ? -2 : 0

  const gx = sleepy ? 0 : follow.x * 1.6
  const gy = sleepy ? 0 : follow.y * 1.1
  const rotate = sleepy ? tilt : follow.x * 1.4

  return (
    <svg
      width={width}
      height={(width / 160) * 48}
      viewBox="0 6 160 48"
      className={'lieve-eyes' + (sleepy ? ' is-sleepy' : '')}
      style={{ overflow: 'visible' }}
    >
      <defs>
        <filter id="lieve-bloom" x="-60%" y="-60%" width="220%" height="220%">
          <feGaussianBlur stdDeviation="3" />
        </filter>
      </defs>

      <g
        style={{
          transform: `translate(${gx}px, ${gy}px) rotate(${rotate}deg) scale(${scale})`,
          transformOrigin: '80px 34px',
          transition: 'transform 600ms cubic-bezier(.22,1,.36,1)',
        }}
      >
        <motion.g
          animate={{ scaleY: squash }}
          transition={{ duration: blink ? 0.1 : 0.5, ease: [0.22, 1, 0.36, 1] }}
          style={{ transformOrigin: '80px 36px' }}
        >
          <g fill={color} opacity={(glow / 100) * 0.7} filter="url(#lieve-bloom)">
            {EYE_PATHS.map((d) => (
              <path key={d} d={d} />
            ))}
          </g>
          <g fill={color}>
            {EYE_PATHS.map((d) => (
              <path key={d} d={d} />
            ))}
          </g>
        </motion.g>
      </g>
    </svg>
  )
}
