import type { LyricLine, ShelfItem, TodoItem, Track, WaterLog } from './types'

export const TRACKS: Track[] = [
  {
    id: 't1',
    title: 'Paper Sky',
    artist: 'Lumen Coast',
    album: 'Summer Flows',
    duration: 193,
    art: { from: '#ff8a5c', to: '#7a3cff', accent: '#ffb27a' },
  },
  {
    id: 't2',
    title: 'Salt & Static',
    artist: 'Lumen Coast',
    album: 'Summer Flows',
    duration: 221,
    art: { from: '#3ad0c9', to: '#123a63', accent: '#6ff0e6' },
  },
  {
    id: 't3',
    title: 'Paper Birds',
    artist: 'Lumen Coast',
    album: 'Summer Flows',
    duration: 184,
    art: { from: '#ff5c8a', to: '#3c1a5e', accent: '#ff9ab5' },
  },
]

export const LYRICS: LyricLine[] = [
  { at: 0, text: 'we fold the summer light' },
  { at: 18, text: 'into paper birds tonight' },
  { at: 36, text: 'and send them through the sky' },
  { at: 54, text: 'to find you by the sea' },
  { at: 72, text: 'every wing a little song' },
  { at: 90, text: 'that carries us along' },
  { at: 108, text: 'and if the wind forgets our name' },
  { at: 126, text: "we'll write it in the rain" },
  { at: 144, text: 'paper sky, paper sky' },
  { at: 162, text: 'hold the morning up so high' },
  { at: 180, text: 'paper sky, paper sky' },
  { at: 198, text: "don't let the summer say goodbye" },
]

export const SHELF: ShelfItem[] = [
  { id: 's1', name: 'beach.jpg', meta: '2.4 MB', kind: 'image' },
  { id: 's2', name: 'Project files', meta: 'Folder', kind: 'folder' },
  { id: 's3', name: 'brief-final.pdf', meta: '810 KB', kind: 'pdf' },
]

export const TODOS: TodoItem[] = [
  { id: 'd1', text: 'Revise the report', done: true },
  { id: 'd2', text: 'Reply to Dina', done: true },
  { id: 'd3', text: 'Post weekly update', done: true },
  { id: 'd4', text: 'Export the prototype', done: false, at: '16:30' },
  { id: 'd5', text: 'Water the plants', done: false, at: '18:00' },
]

export const WATER_LOGS: WaterLog[] = [
  { hour: '09', ml: 400 },
  { hour: '12', ml: 600 },
  { hour: '15', ml: 500 },
]

export const WATER_GOAL = 1500
