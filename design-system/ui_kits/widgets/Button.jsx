/* Button.jsx
 * Mirrors quartzite-widgets::Button + DefaultStyle::paint<Button>.
 *
 * Real Button has three props (text, checkable, checked) and three
 * signals (text_changed, clicked, toggled). State precedence on the
 * fill axis is pressed > checked > hovered > idle.
 */

const Button = ({
  text = "",
  checkable = false,
  checked = false,
  disabled = false,
  focused = false,
  onClicked,
  onToggled,
  style,
  className = "",
  ...rest
}) => {
  // Track hover so we can mirror the painter's "hovered" arm without relying
  // on :hover (so it composes with `focused` props in storybook contexts).
  const [hover, setHover] = React.useState(false);

  const classes = [
    "qz-button",
    checked && "is-checked",
    focused && "is-focused",
    disabled && "is-disabled",
  ].filter(Boolean).join(" ");

  const handleClick = (e) => {
    if (disabled) return;
    if (checkable) {
      const next = !checked;
      // emit!(self.toggled, &(new_checked,)); emit!(self.clicked, &(new_checked,))
      onToggled && onToggled(next);
      onClicked && onClicked(next);
    } else {
      // emit!(self.clicked, &(false,))
      onClicked && onClicked(false);
    }
  };

  return (
    <button
      type="button"
      className={`${classes} ${className}`}
      disabled={disabled}
      onClick={handleClick}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      style={style}
      {...rest}
    >
      {text}
    </button>
  );
};

window.Button = Button;
