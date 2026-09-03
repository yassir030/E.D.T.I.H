import "./App.css";

function App() {
  return (
    <div className="edith">

      <aside className="sidebar">

        <h1 className="logo">
          E.D.I.T.H.
        </h1>

        <button>🧠 Dashboard</button>
        <button>💬 Assistant</button>
        <button>📋 Tasks</button>
        <button>💻 Coding</button>
        <button>📁 Files</button>
        <button>⚙ Settings</button>

        <div className="online">
          ● AI Core Online
        </div>

      </aside>


      <main className="main">

        <h2>
          Welcome back, Yassir
        </h2>

        <p className="subtitle">
          E.D.I.T.H. is ready.
        </p>


        <div className="cards">

          <div className="card">
            <h3>AI Intelligence</h3>
            <p>
              Waiting for API connection
            </p>
          </div>


          <div className="card">
            <h3>System Control</h3>
            <p>
              Desktop assistant modules offline
            </p>
          </div>


          <div className="card">
            <h3>Memory</h3>
            <p>
              Local memory database ready
            </p>
          </div>

        </div>


        <div className="chatbox">

          <p>
            Hello Yassir. I am E.D.I.T.H.
          </p>

          <div className="input-area">

            <input 
              placeholder="Talk to E.D.I.T.H..."
            />

            <button>
              Send
            </button>

          </div>

        </div>


      </main>

    </div>
  );
}

export default App;