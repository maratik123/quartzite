/* App.jsx — interactive demo composing the kit.
 *
 * Layout (top → bottom inside a single WindowFrame):
 *
 *   ┌─ Counter ──────┬─ New note ─────────────────┐
 *   │   Label + 4 btns│ name LineEdit              │
 *   │   pause toggle  │ email LineEdit (read-only) │
 *   │                 │ body  TextEdit             │
 *   │                 │ Submit / Clear buttons     │
 *   ├─ Notes list (ScrollArea) ───────────────────┤
 *   └─ Signal log (ScrollArea) ───────────────────┘
 */

function App() {
  // Theme — flipping this swaps the data-theme attribute on the WindowFrame
  // which retargets every var(--qz-*) through the [data-theme="dark"] block
  // in kit.css. Real Quartzite themes swap the StyleRegistry, not CSS vars.
  const [theme, setTheme] = React.useState("light");

  // Propagate to <body> so the desktop background flips with the window.
  React.useEffect(() => {
    document.body.setAttribute("data-theme", theme);
    return () => document.body.removeAttribute("data-theme");
  }, [theme]);

  // Counter state — mirrors examples/combined.rs Counter
  const [count, setCount] = React.useState(0);
  const [paused, setPaused] = React.useState(false);

  // Form state — mirrors LineEdit/TextEdit props
  const [name, setName] = React.useState("");
  const [email, setEmail] = React.useState("a@example.com");
  const [emailReadOnly, setEmailReadOnly] = React.useState(true);
  const [body, setBody] = React.useState("");

  // Submitted notes
  const [notes, setNotes] = React.useState([
    { title: "First note", body: "Pre-populated to show the ScrollArea is non-empty.\nQuartzite paints chrome only — content is dispatched separately." },
  ]);

  // Signal log
  const [log, setLog] = React.useState([
    { sig: "Counter::count_changed", val: "0" },
  ]);

  const emit = React.useCallback((sig, val) => {
    setLog(prev => [...prev, { sig, val }].slice(-40));
  }, []);

  // -- counter slots -----------------------------------------------------
  const increment = () => {
    if (paused) return;
    setCount(c => {
      const next = c + 1;
      emit("Counter::count_changed", String(next));
      return next;
    });
  };
  const decrement = () => {
    if (paused) return;
    setCount(c => {
      const next = Math.max(0, c - 1);
      emit("Counter::count_changed", String(next));
      if (next === 0) emit("Counter::zeroed", "()");
      return next;
    });
  };
  const reset = () => {
    setCount(0);
    emit("Counter::count_changed", "0");
    emit("Counter::zeroed", "()");
  };
  const togglePause = (checked) => {
    setPaused(checked);
    emit("Counter::toggled", checked ? "true" : "false");
    emit("Counter::clicked", checked ? "true" : "false");
  };

  // -- form slots --------------------------------------------------------
  const submit = () => {
    if (!name.trim() && !body.trim()) return;
    const title = name.trim() || "(untitled)";
    setNotes(ns => [...ns, { title, body }]);
    emit("Note::submitted", `(${title})`);
    setName("");
    setBody("");
  };
  const clearForm = () => {
    setName("");
    setBody("");
    emit("LineEdit::text_changed", "\"\"");
    emit("TextEdit::text_changed", "\"\"");
  };

  return (
    <WindowFrame
      title="quartzite-widgets demo"
      subtitle="quartzite::widgets + DefaultStyle"
      theme={theme}
      onToggleTheme={() => setTheme(t => t === "light" ? "dark" : "light")}
    >
      <div style={{ display: "grid", gridTemplateColumns: "260px 1fr", gap: 8 }}>

        <Container legend="Counter">
          <div style={{ display: "flex", justifyContent: "center" }}>
            <Label
              text={`count = ${count}`}
              alignment="Center"
              style={{ fontSize: "18pt", fontWeight: 700, padding: "10px 0" }}
            />
          </div>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8 }}>
            <Button text="−" disabled={paused} onClicked={decrement} />
            <Button text="+" disabled={paused} onClicked={increment} />
            <Button text="reset" onClicked={reset} style={{ gridColumn: "span 2" }} />
          </div>
          <div style={{ marginTop: 4 }}>
            <Button
              text={paused ? "paused" : "pause"}
              checkable
              checked={paused}
              onToggled={togglePause}
              style={{ width: "100%" }}
            />
          </div>
        </Container>

        <Container legend="New note">
          <div className="qz-row">
            <Label text="title" />
            <div className="qz-stretch">
              <LineEdit
                text={name}
                placeholder="note title"
                onTextChanged={(t) => { setName(t); emit("LineEdit::text_changed", JSON.stringify(t)); }}
              />
            </div>
          </div>
          <div className="qz-row">
            <Label text="email" />
            <div className="qz-stretch">
              <LineEdit
                text={email}
                readOnly={emailReadOnly}
                onTextChanged={(t) => { setEmail(t); emit("LineEdit::text_changed", JSON.stringify(t)); }}
              />
            </div>
            <Button
              text={emailReadOnly ? "unlock" : "lock"}
              checkable
              checked={!emailReadOnly}
              onToggled={(c) => { setEmailReadOnly(!c); emit("Button::toggled", String(c)); }}
            />
          </div>
          <div className="qz-row" style={{ alignItems: "flex-start" }}>
            <Label text="body" />
            <div className="qz-stretch">
              <TextEdit
                plainText={body}
                rows={4}
                onTextChanged={(t) => { setBody(t); emit("TextEdit::text_changed", JSON.stringify(t.length > 20 ? t.slice(0,20)+"…" : t)); }}
              />
            </div>
          </div>
          <div style={{ display: "flex", gap: 8, justifyContent: "flex-end" }}>
            <Button text="clear" onClicked={clearForm} />
            <Button text="submit" onClicked={submit} />
          </div>
        </Container>
      </div>

      <Container legend="Notes (ScrollArea content)">
        <ScrollArea style={{ height: 160 }}>
          {notes.length === 0
            ? <Label text="(no notes)" style={{ color: "rgba(0,0,0,0.5)" }} />
            : notes.map((n, i) => (
                <div key={i} className="qz-note">
                  <div className="qz-note__title">{n.title}</div>
                  {n.body
                    ? <div className="qz-note__body">{n.body}</div>
                    : null}
                </div>
              ))
          }
        </ScrollArea>
      </Container>

      <Container legend="Signal log (Signal::connect)">
        <ScrollArea style={{ height: 130 }}>
          {log.length === 0
            ? <div className="qz-log-row empty">// no signals yet</div>
            : log.slice().reverse().map((r, i) => (
                <div key={log.length - i} className="qz-log-row">
                  <span className="signal">{r.sig}</span>
                  <span> → </span>
                  <span className="value">{r.val}</span>
                </div>
              ))
          }
        </ScrollArea>
      </Container>
    </WindowFrame>
  );
}

window.App = App;
