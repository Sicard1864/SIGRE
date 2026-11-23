import { useEffect, useState } from 'react'
import './App.css'

// À terme ce sera configurable par environnement du front (VITE_API_URL).
const API_URL = import.meta.env.VITE_API_URL ?? 'http://localhost:3000'

type Question = {
  id: string
  question_type: string
  publication_status: string
  created_at: string
}

function App() {
  const [questions, setQuestions] = useState<Question[] | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    fetch(`${API_URL}/api/v1/questions`)
      .then(async (response) => {
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`)
        }
        return (await response.json()) as Question[]
      })
      .then(setQuestions)
      .catch((err) => setError(String(err)))
  }, [])

  return (
    <main>
      <h1>SIGRE</h1>
      <h2>Questions</h2>
      {error && <p role="alert">Erreur de chargement : {error}</p>}
      {questions === null && !error && <p>Chargement…</p>}
      {questions !== null && (
        <ul>
          {questions.map((question) => (
            <li key={question.id}>
              {question.question_type} — {question.publication_status}
            </li>
          ))}
          {questions.length === 0 && <li>Aucune question pour l'instant.</li>}
        </ul>
      )}
    </main>
  )
}

export default App
