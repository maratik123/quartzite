/* Container.jsx
 * Mirrors quartzite-widgets::Container + DefaultStyle::paint<Container>.
 *
 * Real Container paints Window fill + WindowText outline only;
 * child layout is delegated to the attached Layout.
 *
 * `legend` is a documentation affordance not present in the real
 * Container — useful for grouping in the demo. We use the native
 * <fieldset>/<legend> pair so the legend sits on the top border
 * without negative-margin hacks (which can overlap previous
 * siblings when padding is reduced).
 */

const Container = ({ legend, children, style, className = "" }) => {
  if (legend) {
    return (
      <fieldset className={`qz-container ${className}`} style={style}>
        <legend className="qz-container__legend">{legend}</legend>
        {children}
      </fieldset>
    );
  }
  return (
    <div className={`qz-container ${className}`} style={style}>
      {children}
    </div>
  );
};

window.Container = Container;
