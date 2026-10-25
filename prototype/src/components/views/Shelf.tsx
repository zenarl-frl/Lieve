import { useState } from 'react'
import type { Model, ShelfItem } from '../../types'
import { Icon, type IconName } from '../Icon'

const kindIcon: Record<ShelfItem['kind'], IconName> = {
  image: 'image',
  folder: 'folder',
  pdf: 'file',
  audio: 'music',
}

export function ShelfView({ model }: { model: Model }) {
  const [over, setOver] = useState(false)

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault()
    setOver(false)
    const names = Array.from(e.dataTransfer.files).map((f) => f.name)
    if (names.length) model.addShelf(names)
    else model.addShelf([`note-${Math.floor(Math.random() * 90 + 10)}.txt`])
  }

  return (
    <div className="shelf-view">
      <div className="shelf-grid">
        {model.shelf.map((item) => (
          <div className="shelf-card" key={item.id}>
            <span className="shelf-icon">
              <Icon name={kindIcon[item.kind]} size={16} />
            </span>
            <div className="shelf-meta">
              <strong title={item.name}>{item.name}</strong>
              <span>{item.meta}</span>
            </div>
            <button
              className="shelf-remove"
              onClick={() => model.removeShelf(item.id)}
              aria-label={`Remove ${item.name}`}
            >
              <Icon name="close" size={11} />
            </button>
          </div>
        ))}
      </div>

      <div
        className={'dropzone' + (over ? ' is-over' : '')}
        onDragOver={(e) => {
          e.preventDefault()
          setOver(true)
        }}
        onDragLeave={() => setOver(false)}
        onDrop={handleDrop}
        onClick={() => model.addShelf([`screenshot-${Date.now().toString().slice(-4)}.png`])}
      >
        <Icon name="plus" size={14} />
        <span>{over ? 'Release to add' : 'Drop files here'}</span>
        <small>Prototype · click to add a sample file</small>
      </div>
    </div>
  )
}
