/* WindowFrame.jsx
 * A top-level surface using the `Window` color role for its background
 * and `WindowText` for its border. Stands in for the OS-window chrome
 * a real `WindowedApplication` would draw via the renderer (vello+winit).
 *
 * `theme` is a documentation affordance, not a framework concept —
 * Quartzite themes are swapped by registering a different Style in the
 * StyleRegistry. The toggle here flips a `data-theme` attribute so all
 * downstream widgets pick up the dark palette via CSS variables.
 */

const WindowFrame = ({ title = "quartzite", subtitle = "", theme = "light", onToggleTheme, children, style }) => {
  const isDark = theme === "dark";
  const markSrc = isDark ? "../../assets/quartzite-mark-dark.svg" : "../../assets/quartzite-mark.svg";
  return (
    <div className="qz-window" data-theme={theme} style={style}>
      <div className="qz-window__titlebar">
        <img src={markSrc} width="22" height="22" alt="" />
        <span className="qz-window__title">{title}</span>
        {subtitle ? <span className="qz-window__title-mono">{subtitle}</span> : null}
        <span style={{ flex: 1 }}></span>
        <button
          className="qz-window__chrome-btn qz-window__chrome-btn--wide"
          title={isDark ? "switch to light palette" : "switch to dark palette"}
          onClick={onToggleTheme}
        >
          {isDark ? "☀ light" : "☾ dark"}
        </button>
        <button className="qz-window__chrome-btn" title="minimize">_</button>
        <button className="qz-window__chrome-btn" title="maximize">▢</button>
        <button className="qz-window__chrome-btn" title="close">×</button>
      </div>
      <div className="qz-window__body">{children}</div>
    </div>
  );
};

window.WindowFrame = WindowFrame;
