/* Label.jsx
 * Mirrors quartzite-widgets::Label + DefaultStyle::paint<Label>.
 * Paints Window fill + WindowText foreground at the declared alignment.
 */

const Label = ({ text = "", alignment = "Left", style, className = "" }) => {
  const align =
    alignment === "Center" ? "qz-label--center" :
    alignment === "Right"  ? "qz-label--right"  : "";
  return (
    <span className={`qz-label ${align} ${className}`} style={style}>
      {text}
    </span>
  );
};

window.Label = Label;
