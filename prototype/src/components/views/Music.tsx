import type { Model } from '../../types'
import { AlbumArt } from '../AlbumArt'
import { Icon } from '../Icon'
import { SeekBar } from '../SeekBar'

export function MusicView({ model }: { model: Model }) {
  const { player, accent } = model
  const { track, playing, position } = player

  return (
    <div className="music-view">
      <AlbumArt art={track.art} size={104} radius={20} />

      <div className="music-main">
        <span className="eyebrow">{track.album}</span>
        <h3 className="track-title">{track.title}</h3>
        <p className="track-artist">{track.artist}</p>

        <SeekBar
          position={position}
          duration={track.duration}
          onSeek={player.seek}
          accent={accent}
          showTimes
        />

        <div className="controls">
          <button className="icon-btn" onClick={player.prev} aria-label="Previous">
            <Icon name="prev" size={15} />
          </button>
          <button
            className="icon-btn play"
            style={{ background: accent, color: '#0a0508' }}
            onClick={() => player.setPlaying(!playing)}
            aria-label={playing ? 'Pause' : 'Play'}
          >
            <Icon name={playing ? 'pause' : 'play'} size={15} />
          </button>
          <button className="icon-btn" onClick={player.next} aria-label="Next">
            <Icon name="next" size={15} />
          </button>

          <span className="spacer" />

          <span className="eq" style={{ color: accent }}>
            <i /> <i /> <i /> <i />
          </span>
          <button className="ghost-btn" onClick={() => model.setTab('lyrics')}>
            <Icon name="mic" size={13} /> Lyrics
          </button>
        </div>
      </div>
    </div>
  )
}
