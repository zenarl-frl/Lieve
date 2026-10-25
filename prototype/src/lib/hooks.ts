import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { TRACKS } from '../data'
import type { Settings } from '../types'
import { DEFAULT_SETTINGS } from '../types'

const clamp = (v: number, min: number, max: number) => Math.min(max, Math.max(min, v))

export function useClock() {
  const [now, setNow] = useState(() => new Date())
  useEffect(() => {
    const id = setInterval(() => setNow(new Date()), 1000)
    return () => clearInterval(id)
  }, [])
  const time = now.toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' })
  const date = now.toLocaleDateString('en-GB', { weekday: 'short', day: 'numeric', month: 'short' })
  return { now, time, date }
}

export function usePlayer() {
  const [index, setIndex] = useState(0)
  const [playing, setPlaying] = useState(true)
  const [position, setPosition] = useState(66)
  const track = TRACKS[index]

  useEffect(() => {
    if (!playing) return
    const id = setInterval(() => {
      setPosition((p) => (p + 1 >= track.duration ? 0 : p + 1))
    }, 1000)
    return () => clearInterval(id)
  }, [playing, track.duration])

  const seek = useCallback(
    (seconds: number) => setPosition(clamp(seconds, 0, track.duration)),
    [track.duration],
  )

  const next = useCallback(() => {
    setIndex((i) => (i + 1) % TRACKS.length)
    setPosition(0)
  }, [])

  const prev = useCallback(() => {
    setPosition((p) => {
      if (p > 4) return 0
      setIndex((i) => (i - 1 + TRACKS.length) % TRACKS.length)
      return 0
    })
  }, [])

  return { track, tracks: TRACKS, playing, position, setPlaying, seek, next, prev }
}

/** True once the pointer/keyboard has been quiet for `ms`. */
export function useIdle(ms: number) {
  const [idle, setIdle] = useState(false)
  const timer = useRef<number | null>(null)

  useEffect(() => {
    const arm = () => {
      setIdle(false)
      if (timer.current) window.clearTimeout(timer.current)
      timer.current = window.setTimeout(() => setIdle(true), ms)
    }
    arm()
    window.addEventListener('pointermove', arm)
    window.addEventListener('pointerdown', arm)
    window.addEventListener('keydown', arm)
    return () => {
      window.removeEventListener('pointermove', arm)
      window.removeEventListener('pointerdown', arm)
      window.removeEventListener('keydown', arm)
      if (timer.current) window.clearTimeout(timer.current)
    }
  }, [ms])

  return idle
}

/** Normalised pointer offset from viewport centre, -1 .. 1 on each axis. */
export function useEyeFollow(enabled: boolean) {
  const [offset, setOffset] = useState({ x: 0, y: 0 })
  useEffect(() => {
    if (!enabled) {
      setOffset({ x: 0, y: 0 })
      return
    }
    const onMove = (e: PointerEvent) => {
      setOffset({
        x: (e.clientX / window.innerWidth - 0.5) * 2,
        y: (e.clientY / window.innerHeight - 0.5) * 2,
      })
    }
    window.addEventListener('pointermove', onMove)
    return () => window.removeEventListener('pointermove', onMove)
  }, [enabled])
  return offset
}

const SETTINGS_KEY = 'lieve.settings.v1'

export function useSettings() {
  const [settings, setSettings] = useState<Settings>(() => {
    try {
      const raw = localStorage.getItem(SETTINGS_KEY)
      if (raw) return { ...DEFAULT_SETTINGS, ...(JSON.parse(raw) as Partial<Settings>) }
    } catch {
      /* ignore corrupted storage */
    }
    return DEFAULT_SETTINGS
  })

  useEffect(() => {
    localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings))
  }, [settings])

  const patch = useCallback(
    (next: Partial<Settings>) => setSettings((s) => ({ ...s, ...next })),
    [],
  )

  return { settings, patchSettings: patch }
}

export function useAccent(settings: Settings) {
  return useMemo(
    () => (settings.accentLinked ? settings.eyeColor : settings.accent),
    [settings.accentLinked, settings.eyeColor, settings.accent],
  )
}

export function formatTime(total: number) {
  const m = Math.floor(total / 60)
  const s = Math.floor(total % 60)
  return `${m}:${s.toString().padStart(2, '0')}`
}
