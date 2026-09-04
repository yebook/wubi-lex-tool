const MAX_VISIBLE_TEXT_SCALARS = 512;

export function boundVisibleText(
  value: string,
  maximum = MAX_VISIBLE_TEXT_SCALARS,
): string {
  return [...value].slice(0, maximum).join("");
}
