import { useCallback, useEffect, useRef, useState } from 'react'
import { DevBar } from './components/DevBar'
import { Hud } from './components/Hud'
import { Notch } from './components/Notch'
import { Taskbar } from './components/Taskbar'
import { SHELF, TODOS, WATER_GOAL, WATER_LOGS } from './data'
import { useAccent, useClock, useEyeFollow, useIdle, usePlayer, useSettings } from './lib/hooks'
import type { HudKind, HudState, Mood, ShelfItem, Stage, TabId, TodoItem, WaterState } from './types'

const HUD_PRESETS: Record<HudKind, HudState> = {
  volume: { kind: 'volume', value: 62, label: 'Speakers (Realtek)' },
  brightness: { kind: 'brightness', value: 48 },
  caps: { kind: 'caps', value: 0 },
  lang: { kind: 'lang', value: 0, label: 'English (United States)' },
  battery: { kind: 'battery', value: 70, label: 'Charger connected' },
  timer: { kind: 'timer', value: 100 },
}

const kindFromName = (name: string): ShelfItem['kind'] => {
  const ext = name.split('.').pop()?.toLowerCase()
  if (['png', 'jpg', 'jpeg', 'gif', 'webp'].includes(ext ?? '')) return 'image'
  if (['mp3', 'wav', 'flac'].includes(ext ?? '')) return 'audio'
  if (ext === 'pdf' || ext === 'doc' || ext === 'docx') return 'pdf'
  return 'folder'
}

export default function App() {
  const params = new URLSearchParams(window.location.search)
  const initialStage = (params.get('stage') as Stage | null) ?? 'idle'
  const initialTab = (params.get('tab') as TabId | null) ?? 'music'
  const initialHud = params.get('hud') as HudKind | null
  const clean = params.get('clean') === '1'
  const zoom = params.get('zoom') === '1'

  const { settings, patchSettings } = useSettings()
  const accent = useAccent(settings)
  const { time, date } = useClock()
  const player = usePlayer()
  const idle = useIdle(26_000)

  const [stage, setStage] = useState<Stage>(initialStage)
  const [pinned, setPinned] = useState(initialStage === 'panel')
  const [tab, setTab] = useState<TabId>(initialTab)
  const [hud, setHud] = useState<HudState | null>(
    initialHud && HUD_PRESETS[initialHud] ? HUD_PRESETS[initialHud] : null,
  )
  const [forcedMood, setForcedMood] = useState<Mood | null>(null)

  const [todos, setTodos] = useState<TodoItem[]>(TODOS)
  const [water, setWater] = useState<WaterState>({
    intake: WATER_LOGS.reduce((sum, log) => sum + log.ml, 0),
    logs: WATER_LOGS,
    goal: WATER_GOAL,
  })
  const [shelf, setShelf] = useState<ShelfItem[]>(SHELF)

  const hoverTimer = useRef<number | null>(null)
  const hudTimer = useRef<number | null>(null)
  const moodTimer = useRef<number | null>(null)
  const boundsRef = useRef<HTMLDivElement>(null)

  const mood: Mood = forcedMood ?? (idle && stage === 'idle' ? 'sleepy' : 'idle')
  const follow = useEyeFollow(stage === 'idle' && mood === 'idle')

  useEffect(() => {
    document.documentElement.style.setProperty('--accent', accent)
    document.documentElement.style.setProperty('--eye', settings.eyeColor)
  }, [accent, settings.eyeColor])

  useEffect(() => {
    const q = new URLSearchParams(window.location.search)
    if (q.get('minimal') === '1') patchSettings({ minimal: true })
    if (q.get('float') === '1') patchSettings({ floating: true })
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const clearHover = () => {
    if (hoverTimer.current) window.clearTimeout(hoverTimer.current)
    hoverTimer.current = null
  }

  const onHover = (hovering: boolean) => {
    clearHover()
    if (hovering) {
      if (stage === 'panel' || pinned) return
      setStage('compact')
      hoverTimer.current = window.setTimeout(() => setStage('panel'), 230)
    } else if (!pinned) {
      setStage('idle')
    }
  }

  const togglePin = () => {
    if (pinned) {
      setPinned(false)
      setStage('idle')
    } else {
      setPinned(true)
      setStage('panel')
    }
  }

  const collapse = () => {
    setPinned(false)
    setStage('idle')
  }

  const openSettings = () => {
    setTab('settings')
    setPinned(true)
    setStage('panel')
  }

  const showHud = useCallback((kind: HudKind) => {
    if (hudTimer.current) window.clearTimeout(hudTimer.current)
    setHud(HUD_PRESETS[kind])
    hudTimer.current = window.setTimeout(() => setHud(null), kind === 'timer' ? 3600 : 2400)
  }, [])

  const onTimerDone = useCallback(() => {
    showHud('timer')
    setForcedMood('alert')
    if (moodTimer.current) window.clearTimeout(moodTimer.current)
    moodTimer.current = window.setTimeout(() => setForcedMood(null), 2200)
  }, [showHud])

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setHud(null)
        setPinned(false)
        setStage('idle')
        setTab((t) => (t === 'settings' ? 'music' : t))
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [])

  useEffect(
    () => () => {
      [hoverTimer, hudTimer, moodTimer].forEach((ref) => {
        if (ref.current) window.clearTimeout(ref.current)
      })
    },
    [],
  )

  const toggleTodo = (id: string) =>
    setTodos((list) => list.map((todo) => (todo.id === id ? { ...todo, done: !todo.done } : todo)))

  const addTodo = (text: string) =>
    setTodos((list) => [...list, { id: `t${Date.now()}`, text, done: false }])

  const drink = (ml: number) =>
    setWater((w) => {
      if (w.intake >= w.goal) return w
      const hour = new Date().toLocaleTimeString('en-GB', { hour: '2-digit' }).slice(0, 2)
      return {
        ...w,
        intake: Math.min(w.goal, w.intake + ml),
        logs: w.logs.some((log) => log.hour === hour)
          ? w.logs.map((log) => log.hour === hour ? { ...log, ml: log.ml + Math.min(ml, w.goal - w.intake) } : log)
          : [...w.logs, { hour, ml: Math.min(ml, w.goal - w.intake) }],
      }
    })

  const resetWater = () => setWater((w) => ({ ...w, intake: 0, logs: [] }))

  const addShelf = (names: string[]) =>
    setShelf((items) => [
      ...names.map((name, i) => ({
        id: `s${Date.now()}-${i}`,
        name,
        meta: kindFromName(name) === 'folder' ? 'Folder' : `${(Math.random() * 4 + 0.4).toFixed(1)} MB`,
        kind: kindFromName(name),
      })),
      ...items,
    ])

  const removeShelf = (id: string) => setShelf((items) => items.filter((item) => item.id !== id))

  const model = {
    stage,
    tab,
    setTab,
    pinned,
    togglePin,
    player,
    settings,
    patchSettings,
    accent,
    mood,
    follow,
    time,
    date,
    openSettings,
    todos,
    toggleTodo,
    addTodo,
    water,
    drink,
    resetWater,
    shelf,
    addShelf,
    removeShelf,
    onTimerDone,
  }

  return (
    <div className={'desktop' + (clean ? ' is-clean' : '') + (zoom ? ' is-zoom' : '')}>
      <div className="wallpaper" aria-hidden />
      <div className="wallpaper-glow" aria-hidden />
      <div className="bounds" ref={boundsRef} />

      <div className={'notch-layer' + (hud ? ' is-hidden' : '')}>
        <Notch
          model={model}
          stage={stage}
          hover={stage !== 'idle'}
          onHover={onHover}
          onTogglePin={togglePin}
          onCollapse={collapse}
          boundsRef={boundsRef}
        />
      </div>

      <Hud hud={hud} accent={accent} />

      <p className="hint">
        Hover the notch to open · Click the pin to keep it · <kbd>Esc</kbd> to close
      </p>

      <Taskbar time={time} date={date} accent={accent} />

      <DevBar
        model={model}
        stage={stage}
        onStage={setStage}
        onHud={showHud}
        mood={mood}
        onMood={(m) => setForcedMood(m === 'idle' ? null : m)}
        onReset={() => {
          patchSettings({
            eyeColor: '#E23636',
            accent: '#E23636',
            accentLinked: true,
            glow: 70,
            mascot: true,
            compactLyrics: true,
            minimal: false,
            floating: false,
          })
          setStage('idle')
          setTab('music')
          setPinned(false)
        }}
      />
    </div>
  )
}
