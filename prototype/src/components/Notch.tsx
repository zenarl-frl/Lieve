import type { CSSProperties, RefObject } from 'react'
import { AnimatePresence, motion } from 'motion/react'
import type { Model, Settings, Stage } from '../types'
import { AlbumArt } from './AlbumArt'
import { Icon } from './Icon'
import { Mascot } from './Mascot'
import { SettingsView } from './SettingsView'
import { TabBar } from './TabBar'
import { FocusView } from './views/Focus'
import { LyricsView } from './views/Lyrics'
import { MusicView } from './views/Music'
import { ShelfView } from './views/Shelf'
import { TodoView } from './views/Todo'
import { WaterView } from './views/Water'
import { LYRICS } from '../data'
import { CompactLyric } from './CompactLyric'

function dims(stage: Stage, settings: Settings, tab: Model['tab']) {
  if (stage === 'idle') {
    if (settings.minimal) return { width: 128, height: 6, radius: 6 }
    return { width: settings.mascot ? 196 : 116, height: 34, radius: 20 }
  }
  if (stage === 'compact') {
    if (settings.minimal) return { width: 190, height: 26, radius: 13 }
    return { width: 306, height: 38, radius: 20 }
  }
  return tab === 'lyrics'
    ? { width: 648, height: 322, radius: 30 }
    : tab === 'settings'
      ? { width: 492, height: 346, radius: 30 }
      : { width: 470, height: 322, radius: 30 }
}

interface Props {
  model: Model
  stage: Stage
  hover: boolean
  onHover: (hovering: boolean) => void
  onTogglePin: () => void
  onCollapse: () => void
  boundsRef: RefObject<HTMLDivElement | null>
}

export function Notch({ model, stage, hover, onHover, onTogglePin, onCollapse, boundsRef }: Props) {
  const { settings, accent, player, mood, follow, time } = model
  const showLyric = settings.compactLyrics && player.playing && stage !== 'panel'
  // The sample lyrics belong to Paper Sky only; other tracks use their title.
  const lyric = player.track.id === 't1'
    ? LYRICS.filter((line) => line.at <= player.position).at(-1)?.text
    : undefined
  const compactText = lyric ?? player.track.title
  const { width, height, radius } = showLyric
    ? { width: Math.min(390, Math.max(240, compactText.length * 7 + 90)), height: 38, radius: 20 }
    : dims(stage, settings, model.tab)
  const progress = Math.min(100, (player.position / player.track.duration) * 100)

  return (
    <div className={'notch-wrap' + (settings.floating ? ' is-floating' : '')}>
      <motion.div
        className={'notch' + (hover ? ' is-hover' : '') + (settings.minimal ? ' is-minimal' : '')}
        style={{ '--accent': accent } as CSSProperties}
        initial={false}
        animate={{
          width,
          height,
          borderRadius: settings.floating ? 26 : `0px 0px ${radius}px ${radius}px`,
        }}
        transition={{ type: 'spring', stiffness: 360, damping: 32, mass: 0.85 }}
        onPointerEnter={() => onHover(true)}
        onPointerLeave={() => onHover(false)}
        drag={settings.floating}
        dragMomentum={false}
        dragElastic={0.04}
        dragConstraints={boundsRef}
        whileDrag={{ cursor: 'grabbing' }}
        key={settings.floating ? 'float' : 'dock'}
      >
        <span className="notch-shine" />
        {!settings.minimal && (
          <span className="notch-bloom" style={{ background: accent, opacity: player.playing ? 0.14 : 0 }} />
        )}

        <div className="stage">
          <AnimatePresence mode="wait" initial={false}>
            {stage === 'idle' && !settings.minimal && !showLyric && (
              <motion.div
                key="idle"
                className="idle-content"
                initial={{ opacity: 0, y: -4 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -4 }}
                transition={{ duration: 0.18 }}
              >
                {settings.mascot ? (
                  <Mascot color={settings.eyeColor} glow={settings.glow} mood={mood} follow={follow} width={62} />
                ) : (
                  <span className="status-dot" style={{ background: accent }} />
                )}
                <span className="clock">{time}</span>
              </motion.div>
            )}

            {(stage === 'compact' || showLyric) && (
              <motion.div
                key="compact"
                className="compact-content"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                transition={{ duration: 0.14 }}
              >
                <AlbumArt art={player.track.art} size={settings.minimal ? 14 : 24} radius={7} />
                {showLyric ? <CompactLyric text={compactText} /> : <div className="compact-text">
                  <strong>{player.track.title}</strong>
                  <span>{player.track.artist}</span>
                </div>}
                {!settings.minimal && (
                  <span className={'eq' + (player.playing ? '' : ' is-paused')} style={{ color: accent }}>
                    <i />
                    <i />
                    <i />
                    <i />
                  </span>
                )}
                <button
                  className="mini-play"
                  style={{ color: accent }}
                  onClick={(e) => {
                    e.stopPropagation()
                    player.setPlaying(!player.playing)
                  }}
                  aria-label={player.playing ? 'Pause' : 'Play'}
                >
                  <Icon name={player.playing ? 'pause' : 'play'} size={12} />
                </button>
                <span className="edge-progress" style={{ width: `${progress}%`, background: accent }} />
              </motion.div>
            )}

            {stage === 'panel' && (
              <motion.div
                key="panel"
                className="panel"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                transition={{ duration: 0.16 }}
              >
                <header className="panel-head">
                  <AlbumArt art={player.track.art} size={26} radius={8} />
                  <div className="head-text">
                    <strong>{model.tab === 'settings' ? 'Appearance' : player.track.title}</strong>
                    <span>
                      {model.tab === 'settings'
                        ? 'Make Lieve yours'
                        : `${player.track.artist} · ${player.track.album}`}
                    </span>
                  </div>

                  <div className="head-actions">
                    <button
                      className={'icon-btn subtle' + (model.pinned ? ' is-on' : '')}
                      onClick={onTogglePin}
                      aria-label={model.pinned ? 'Unpin' : 'Pin open'}
                      title={model.pinned ? 'Unpin' : 'Pin open'}
                    >
                      <Icon name="pin" size={13} />
                    </button>
                    <button
                      className={'icon-btn subtle' + (model.tab === 'settings' ? ' is-on' : '')}
                      onClick={model.openSettings}
                      aria-label="Appearance"
                      title="Appearance"
                    >
                      <Icon name="gear" size={13} />
                    </button>
                    <button className="icon-btn subtle" onClick={onCollapse} aria-label="Collapse" title="Collapse">
                      <Icon name="close" size={13} />
                    </button>
                  </div>
                </header>

                <TabBar model={model} />

                <div className="panel-body">
                  {model.tab === 'music' && <MusicView model={model} />}
                  {model.tab === 'lyrics' && <LyricsView model={model} />}
                  {model.tab === 'focus' && <FocusView model={model} />}
                  {model.tab === 'water' && <WaterView model={model} />}
                  {model.tab === 'shelf' && <ShelfView model={model} />}
                  {model.tab === 'todo' && <TodoView model={model} />}
                  {model.tab === 'settings' && <SettingsView model={model} />}
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </motion.div>
    </div>
  )
}
