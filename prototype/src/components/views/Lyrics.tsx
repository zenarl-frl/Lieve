import { useEffect, useRef } from 'react'
import { LYRICS } from '../../data'
import type { Model } from '../../types'
import { AlbumArt } from '../AlbumArt'
import { Icon } from '../Icon'
import { SeekBar } from '../SeekBar'

export function LyricsView({ model }: { model: Model }) {
  const { player, accent } = model
  const active = LYRICS.reduce((acc, line, i) => (player.position >= line.at ? i : acc), 0)
  const listRef = useRef<HTMLDivElement>(null)
  const firstRun = useRef(true)

  useEffect(() => {
    const list = listRef.current
    const el = list?.querySelector<HTMLElement>('[data-active="true"]')
    if (!list || !el) return
    const listRect = list.getBoundingClientRect()
    const elRect = el.getBoundingClientRect()
    const delta = elRect.top - listRect.top - listRect.height / 2 + elRect.height / 2
    list.scrollTo({
      top: Math.max(0, list.scrollTop + delta),
      behavior: firstRun.current ? 'auto' : 'smooth',
    })
    firstRun.current = false
  }, [active])

  return (
    <div className="lyrics-view">
      <aside className="lyrics-side">
        <AlbumArt art={player.track.art} size={76} radius={16} />
        <div className="lyrics-meta">
          <span className="eyebrow">{player.track.album}</span>
          <strong>{player.track.title}</strong>
          <span className="muted">{player.track.artist}</span>
        </div>

        <SeekBar
          position={player.position}
          duration={player.track.duration}
          onSeek={player.seek}
          accent={accent}
        />

        <div className="controls small">
          <button className="icon-btn" onClick={player.prev} aria-label="Previous">
            <Icon name="prev" size={14} />
          </button>
          <button
            className="icon-btn play"
            style={{ background: accent, color: '#0a0508' }}
            onClick={() => player.setPlaying(!player.playing)}
            aria-label={player.playing ? 'Pause' : 'Play'}
          >
            <Icon name={player.playing ? 'pause' : 'play'} size={14} />
          </button>
          <button className="icon-btn" onClick={player.next} aria-label="Next">
            <Icon name="next" size={14} />
          </button>
        </div>
      </aside>

      <div className="lyrics-lines" ref={listRef}>
        {LYRICS.map((line, i) => (
          <button
            key={line.at}
            className="lyric-line"
            data-active={i === active}
            style={i === active ? { color: accent } : undefined}
            onClick={() => player.seek(line.at + 0.2)}
          >
            {line.text}
          </button>
        ))}
      </div>
    </div>
  )
}
