import type { Art } from '../types'

interface Props {
  art: Art
  size?: number
  radius?: number
  className?: string
}

/** Stand-in album art: gradient + concentric rings + grain, no assets needed. */
export function AlbumArt({ art, size = 96, radius = 18, className = '' }: Props) {
  return (
    <div
      className={`art ${className}`}
      style={{
        width: size,
        height: size,
        borderRadius: radius,
        background: `radial-gradient(120% 120% at 18% 8%, ${art.accent} 0%, transparent 58%), linear-gradient(145deg, ${art.from} 0%, ${art.to} 100%)`,
      }}
    >
      <span className="art-rings" />
      <span className="art-gloss" />
    </div>
  )
}
