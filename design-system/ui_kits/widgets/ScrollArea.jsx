/* ScrollArea.jsx
 * Mirrors quartzite-widgets::ScrollArea + DefaultStyle::paint<ScrollArea>.
 *
 * Real ScrollArea paints chrome only — a Base fill plus a 1 px
 * WindowText outline — and does not traverse children. The browser
 * scrollbar is a documentation hint; the framework does not yet
 * render a bar primitive.
 */

const ScrollArea = ({
  horizontalPolicy = "AsNeeded",
  verticalPolicy = "AsNeeded",
  children,
  style,
  className = "",
}) => {
  const overflowX =
    horizontalPolicy === "AlwaysOn"  ? "scroll" :
    horizontalPolicy === "AlwaysOff" ? "hidden" : "auto";
  const overflowY =
    verticalPolicy === "AlwaysOn"    ? "scroll" :
    verticalPolicy === "AlwaysOff"   ? "hidden" : "auto";

  return (
    <div className={`qz-scroll-area ${className}`} style={style}>
      <div className="qz-scroll-area__viewport" style={{ overflowX, overflowY }}>
        {children}
      </div>
    </div>
  );
};

window.ScrollArea = ScrollArea;
