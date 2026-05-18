/* TextEdit.jsx
 * Mirrors quartzite-widgets::TextEdit + DefaultStyle::paint<TextEdit>.
 *
 * Props: plain_text, read_only.
 * Signals: text_changed(String).
 */

const TextEdit = ({
  plainText = "",
  readOnly = false,
  rows = 4,
  onTextChanged,
  style,
  className = "",
  ...rest
}) => {
  const handleChange = (e) => {
    if (readOnly) return;
    const next = e.target.value;
    onTextChanged && onTextChanged(next);
  };

  return (
    <textarea
      className={`qz-text-edit ${className}`}
      value={plainText}
      rows={rows}
      readOnly={readOnly}
      onChange={handleChange}
      style={style}
      {...rest}
    />
  );
};

window.TextEdit = TextEdit;
