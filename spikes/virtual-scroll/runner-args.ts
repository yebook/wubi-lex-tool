const USAGE = "usage: pnpm run spike:virtual-scroll -- --output <result.json>";

export function parseOutputArgument(arguments_: readonly string[]): string {
  const forwarded = arguments_[0] === "--" ? arguments_.slice(1) : arguments_;
  if (forwarded.length !== 2 || forwarded[0] !== "--output" || !forwarded[1]) {
    throw new Error(USAGE);
  }
  return forwarded[1];
}
