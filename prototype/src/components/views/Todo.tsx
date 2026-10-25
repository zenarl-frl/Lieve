import { useState } from 'react'
import type { Model } from '../../types'
import { Icon } from '../Icon'

export function TodoView({ model }: { model: Model }) {
  const [draft, setDraft] = useState('')
  const done = model.todos.filter((t) => t.done).length

  const submit = () => {
    const text = draft.trim()
    if (!text) return
    model.addTodo(text)
    setDraft('')
  }

  return (
    <div className="todo-view">
      <div className="todo-head">
        <div>
          <span className="eyebrow">Task done today</span>
          <div className="todo-count">
            <b>{done}</b>
            <span>of {model.todos.length}</span>
          </div>
        </div>
        <div className="todo-progress">
          <span
            style={{
              width: `${model.todos.length ? (done / model.todos.length) * 100 : 0}%`,
              background: model.accent,
            }}
          />
        </div>
      </div>

      <ul className="todo-list">
        {model.todos.map((todo) => (
          <li key={todo.id} className={todo.done ? 'is-done' : ''}>
            <button
              className="todo-check"
              onClick={() => model.toggleTodo(todo.id)}
              style={todo.done ? { background: model.accent, borderColor: model.accent } : undefined}
              aria-label={todo.done ? 'Mark as open' : 'Mark as done'}
            >
              {todo.done && <Icon name="check" size={10} />}
            </button>
            <span className="todo-text">{todo.text}</span>
            {todo.at && <span className="todo-time">{todo.at}</span>}
          </li>
        ))}
      </ul>

      <div className="todo-add">
        <Icon name="plus" size={13} />
        <input
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') submit()
          }}
          placeholder="Add a task, press Enter"
          aria-label="New task"
        />
      </div>
    </div>
  )
}
