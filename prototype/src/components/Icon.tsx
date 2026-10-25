import type { SVGProps } from 'react'

export type IconName =
  | 'music'
  | 'text'
  | 'mic'
  | 'timer'
  | 'droplet'
  | 'layers'
  | 'check'
  | 'play'
  | 'pause'
  | 'prev'
  | 'next'
  | 'plus'
  | 'volume'
  | 'sun'
  | 'close'
  | 'pin'
  | 'gear'
  | 'trash'
  | 'folder'
  | 'file'
  | 'image'
  | 'headphones'
  | 'wifi'
  | 'battery'
  | 'keyboard'
  | 'sparkle'

const STROKE: Partial<Record<IconName, string>> = {
  music: 'M9 18V6l11-2v12M9 18a3 3 0 1 1-3-3M20 16a3 3 0 1 1-3-3',
  text: 'M4.5 6.5h15M4.5 11.5h12M4.5 16.5h15M4.5 21h8',
  mic: 'M12 3a3 3 0 0 1 3 3v5a3 3 0 0 1-6 0V6a3 3 0 0 1 3-3ZM5.5 11a6.5 6.5 0 0 0 13 0M12 17.5V21',
  timer: 'M12 21a8 8 0 1 0 0-16 8 8 0 0 0 0 16ZM12 9v4.5l3 2M9 2.5h6',
  droplet: 'M12 3.2c3.4 3.9 5.8 6.6 5.8 9.6a5.8 5.8 0 0 1-11.6 0c0-3 2.4-5.7 5.8-9.6Z',
  layers: 'M12 3 3.5 7.5 12 12l8.5-4.5L12 3ZM3.5 12.5 12 17l8.5-4.5',
  check: 'M4.5 12.5 9.5 17.5 19.5 6.5',
  plus: 'M12 5v14M5 12h14',
  volume: 'M4 9.5h3L11 6v12l-4-3.5H4v-5ZM15 9a4 4 0 0 1 0 6M17.8 6.5a8 8 0 0 1 0 11',
  sun: 'M12 16.5a4.5 4.5 0 1 0 0-9 4.5 4.5 0 0 0 0 9ZM12 2v2M12 20v2M2 12h2M20 12h2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M19.1 4.9l-1.4 1.4M6.3 17.7l-1.4 1.4',
  close: 'M6 6l12 12M18 6 6 18',
  pin: 'M9.5 3.5h5l-.8 6 3.3 3.3H7l3.3-3.3-.8-6ZM12 12.8V21',
  gear: 'M12 15.2a3.2 3.2 0 1 0 0-6.4 3.2 3.2 0 0 0 0 6.4ZM19.4 13.5a7.6 7.6 0 0 0 0-3l1.8-1.4-2-3.4-2.1.9a7.6 7.6 0 0 0-2.6-1.5L14.2 2.7h-4l-.3 2.4a7.6 7.6 0 0 0-2.6 1.5l-2.1-.9-2 3.4 1.8 1.4a7.6 7.6 0 0 0 0 3l-1.8 1.4 2 3.4 2.1-.9a7.6 7.6 0 0 0 2.6 1.5l.3 2.4h4l.3-2.4a7.6 7.6 0 0 0 2.6-1.5l2.1.9 2-3.4-1.8-1.4Z',
  trash: 'M4 7h16M9.5 7V4.8h5V7M6.5 7l.9 13.2h9.2L17.5 7M10 11v6M14 11v6',
  folder: 'M3 7.5A2 2 0 0 1 5 5.5h4l1.8 2H19a2 2 0 0 1 2 2v7.5a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7.5Z',
  file: 'M7 3.5h7l4 4v13H7v-17ZM14 3.5v4.2h4',
  image: 'M3.5 6.5h17v11h-17v-11ZM8.6 10.6a1.4 1.4 0 1 0 0-2.8 1.4 1.4 0 0 0 0 2.8ZM20.5 15.5 15 10.8l-8.5 6.7',
  headphones: 'M4 15.5v-3a8 8 0 0 1 16 0v3M4 14h3.2v7H5.5A1.5 1.5 0 0 1 4 19.5V14ZM20 14h-3.2v7h1.7a1.5 1.5 0 0 0 1.5-1.5V14Z',
  wifi: 'M2.5 8.5a15 15 0 0 1 19 0M5.5 12a10.5 10.5 0 0 1 13 0M8.5 15.5a6 6 0 0 1 7 0M12 19h.01',
  battery: 'M2.5 8.5h16v7h-16v-7ZM21 11v2M5 11v2M8 11v2',
  keyboard: 'M3 7.5h18v9H3v-9ZM6.5 10.5h.01M9.5 10.5h.01M12.5 10.5h.01M15.5 10.5h.01M8 13.5h8',
  sparkle: 'M12 3.5l1.9 5.6 5.6 1.9-5.6 1.9L12 18.5l-1.9-5.6L4.5 11l5.6-1.9L12 3.5Z',
}

const FILLED: Partial<Record<IconName, string>> = {
  play: 'M8.5 5.4v13.2L18.8 12 8.5 5.4Z',
  pause: 'M8.5 5.5h3v13h-3v-13ZM14.5 5.5h3v13h-3v-13Z',
  prev: 'M17.5 5.5v13L7.8 12l9.7-6.5ZM5 5.5h2v13H5v-13Z',
  next: 'M6.5 5.5v13L16.2 12 6.5 5.5ZM17 5.5h2v13h-2v-13Z',
}

interface Props extends SVGProps<SVGSVGElement> {
  name: IconName
  size?: number
}

export function Icon({ name, size = 16, ...rest }: Props) {
  const filled = FILLED[name]
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill={filled ? 'currentColor' : 'none'}
      stroke={filled ? 'none' : 'currentColor'}
      strokeWidth={filled ? 0 : 1.6}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden
      {...rest}
    >
      {filled ? <path d={filled} /> : <path d={STROKE[name]} />}
    </svg>
  )
}
