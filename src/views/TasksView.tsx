import { useState, type FormEvent } from "react";
import { useEdith } from "../state/EdithContext";
import type { TaskItem } from "../types";
import { createId } from "../utils";

export function TasksView() {
  const { tasks, setTasks, logActivity } = useEdith();
  const [title, setTitle] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editingTitle, setEditingTitle] = useState("");

  function addTask(event: FormEvent) {
    event.preventDefault();
    const nextTitle = title.trim();
    if (!nextTitle) {
      return;
    }
    const now = Date.now();
    const task: TaskItem = {
      id: createId(),
      title: nextTitle,
      completed: false,
      createdAt: now,
      updatedAt: now,
    };
    setTasks((current) => [task, ...current]);
    setTitle("");
    logActivity(`Taak toegevoegd: ${nextTitle}`);
  }

  function toggleTask(id: string) {
    setTasks((current) =>
      current.map((task) =>
        task.id === id
          ? { ...task, completed: !task.completed, updatedAt: Date.now() }
          : task,
      ),
    );
  }

  function removeTask(id: string) {
    setTasks((current) => current.filter((task) => task.id !== id));
    logActivity("Taak verwijderd");
  }

  function startEdit(task: TaskItem) {
    setEditingId(task.id);
    setEditingTitle(task.title);
  }

  function saveEdit(id: string) {
    const nextTitle = editingTitle.trim();
    if (!nextTitle) {
      return;
    }
    setTasks((current) =>
      current.map((task) =>
        task.id === id
          ? { ...task, title: nextTitle, updatedAt: Date.now() }
          : task,
      ),
    );
    setEditingId(null);
    setEditingTitle("");
  }

  return (
    <section className="view">
      <header className="page-header">
        <h2>Tasks</h2>
        <p className="subtitle">Lokale taken voor deze sessie. Persistente opslag volgt later.</p>
      </header>

      <form className="inline-form" onSubmit={addTask}>
        <label htmlFor="new-task" className="sr-only">
          Nieuwe taak
        </label>
        <input
          id="new-task"
          value={title}
          onChange={(event) => setTitle(event.target.value)}
          placeholder="Nieuwe taak…"
        />
        <button type="submit">Toevoegen</button>
      </form>

      {tasks.length === 0 ? (
        <div className="empty-state">
          <p>Nog geen taken. Voeg er hierboven een toe.</p>
        </div>
      ) : (
        <ul className="task-list">
          {tasks.map((task) => (
            <li key={task.id} className={task.completed ? "task done" : "task"}>
              <label>
                <input
                  type="checkbox"
                  checked={task.completed}
                  onChange={() => toggleTask(task.id)}
                />
                <span className="sr-only">
                  {task.completed ? "Markeer als open" : "Markeer als klaar"}
                </span>
              </label>
              {editingId === task.id ? (
                <input
                  className="task-edit"
                  value={editingTitle}
                  onChange={(event) => setEditingTitle(event.target.value)}
                  onBlur={() => saveEdit(task.id)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter") {
                      event.preventDefault();
                      saveEdit(task.id);
                    }
                  }}
                  aria-label="Taaktitel bewerken"
                  autoFocus
                />
              ) : (
                <button
                  type="button"
                  className="task-title"
                  onClick={() => startEdit(task)}
                >
                  {task.title}
                </button>
              )}
              <button
                type="button"
                className="ghost"
                onClick={() => removeTask(task.id)}
              >
                Verwijderen
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
