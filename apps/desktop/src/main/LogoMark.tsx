// The note-tice! mark (a geometric "N" with an exclamation mark), measured from
// assets/brand/notetice-logo.png. Source: assets/brand/notetice-mark.svg. Uses currentColor so
// it follows the theme.
const MARK_PATH =
  "M289 243H398V684H289Z M422 243L615 516V684L422 405Z M629 243H736L682 587Z M682 607L723 648L682 689L641 648Z";

export function LogoMark({ size = 20 }: { size?: number }) {
  return (
    <svg
      viewBox="289 243 447 446"
      height={size}
      width={Math.round((size * 447) / 446)}
      aria-hidden="true"
      focusable="false"
    >
      <path fill="currentColor" d={MARK_PATH} />
    </svg>
  );
}
