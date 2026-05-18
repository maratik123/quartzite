/* LineEdit.jsx
 * Mirrors quartzite-widgets::LineEdit + DefaultStyle::paint<LineEdit>.
 *
 * Props: text, placeholder, read_only.
 * Signals: text_changed(String), return_pressed().
 */

const LineEdit = ({
  text = "",
  placeholder = "",
  readOnly = false,
  onTextChanged,
  onReturnPressed,
  style,
  className = "",
  ...rest
}) => {
  const handleChange = (e) => {
    if (readOnly) return;
    const next = e.target.value;
    onTextChanged && onTextChanged(next);
  };

  const handleKeyDown = (e) => {
    if (e.key === "Enter") {
      onReturnPressed && onReturnPressed();
    }
  };

  return (
    <input
      type="text"
      className={`qz-line-edit ${className}`}
      value={text}
      placeholder={placeholder}
      readOnly={readOnly}
      onChange={handleChange}
      onKeyDown={handleKeyDown}
      style={style}
      {...rest}
    />
  );
};

window.LineEdit = LineEdit;
