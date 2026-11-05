/**
 * candyAI wordmark.
 *
 * Replaces the upstream "Handy" outline path with live text, so the brand can
 * change without redrawing an SVG. Two tones: "candy" in the soft orange,
 * "AI" in the primary orange. `textLength` pins the layout to the viewBox so
 * the mark keeps its proportions at every size.
 */
const HandyTextLogo = ({
  width,
  height,
  className,
}: {
  width?: number;
  height?: number;
  className?: string;
}) => {
  return (
    <svg
      width={width}
      height={height}
      className={className}
      viewBox="0 0 930 328"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      role="img"
      aria-label="candyAI"
    >
      <text
        x="0"
        y="238"
        textLength="930"
        lengthAdjust="spacingAndGlyphs"
        fontFamily="var(--font-display, ui-rounded, system-ui, sans-serif)"
        fontSize="250"
        fontWeight="800"
        letterSpacing="-6"
      >
        <tspan fill="var(--color-orange-soft, #ffc79a)">candy</tspan>
        <tspan fill="var(--color-logo-primary, #ff8a3d)">AI</tspan>
      </text>
    </svg>
  );
};

export default HandyTextLogo;
