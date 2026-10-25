export type Stage = 'idle' | 'compact' | 'panel'

export type TabId = 'music' | 'lyrics' | 'focus' | 'water' | 'shelf' | 'todo' | 'settings'

export type Mood = 'idle' | 'sleepy' | 'alert' | 'curious'

export type HudKind = 'volume' | 'brightness' | 'caps' | 'lang' | 'battery' | 'timer'

export interface HudState {
  kind: HudKind
  value: number
  label?: string
}

export interface Art {
  from: string
  to: string
  accent: string
}

export interface Track {
  id: string
  title: string
  artist: string
  album: string
  duration: number
  art: Art
}

export interface LyricLine {
  at: number
  text: string
}

export interface ShelfItem {
  id: string
  name: string
  meta: string
  kind: 'image' | 'folder' | 'pdf' | 'audio'
}

export interface TodoItem {
  id: string
  text: string
  done: boolean
  at?: string
}

export interface WaterLog {
  hour: string
  ml: number
}

export interface Player {
  track: Track
  tracks: Track[]
  playing: boolean
  position: number
  setPlaying: (playing: boolean) => void
  seek: (seconds: number) => void
  next: () => void
  prev: () => void
}

export interface WaterState {
  intake: number
  logs: WaterLog[]
  goal: number
}

export interface Model {
  stage: Stage
  tab: TabId
  setTab: (tab: TabId) => void
  pinned: boolean
  togglePin: () => void
  player: Player
  settings: Settings
  patchSettings: (next: Partial<Settings>) => void
  accent: string
  mood: Mood
  follow: { x: number; y: number }
  time: string
  date: string
  openSettings: () => void
  todos: TodoItem[]
  toggleTodo: (id: string) => void
  addTodo: (text: string) => void
  water: WaterState
  drink: (ml: number) => void
  resetWater: () => void
  shelf: ShelfItem[]
  addShelf: (names: string[]) => void
  removeShelf: (id: string) => void
  onTimerDone: () => void
}

export interface Settings {
  compactLyrics: boolean
  eyeColor: string
  accentLinked: boolean
  accent: string
  glow: number
  mascot: boolean
  minimal: boolean
  floating: boolean
}

export const EYE_PRESETS = [
  { name: 'Crimson', hex: '#E23636' },
  { name: 'Blood', hex: '#C1121F' },
  { name: 'Ember', hex: '#FF5C2B' },
  { name: 'Violet', hex: '#9B4DFF' },
  { name: 'Emerald', hex: '#22C55E' },
  { name: 'Ice', hex: '#7FE7FF' },
  { name: 'Bone', hex: '#F2F2F2' },
] as const

export const DEFAULT_SETTINGS: Settings = {
  compactLyrics: true,
  eyeColor: '#E23636',
  accentLinked: true,
  accent: '#E23636',
  glow: 70,
  mascot: true,
  minimal: false,
  floating: false,
}
