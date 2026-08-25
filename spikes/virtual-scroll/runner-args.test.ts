import { describe, expect, it } from "vitest";

import { parseOutputArgument } from "./runner-args";

describe("virtual-scroll runner arguments", () => {
  it("accepts direct Node arguments", () => {
    expect(parseOutputArgument(["--output", "result.json"])).toBe("result.json");
  });

  it("accepts the leading separator forwarded by pnpm 11", () => {
    expect(parseOutputArgument(["--", "--output", "result.json"])).toBe("result.json");
  });

  it.each([
    { arguments_: [] },
    { arguments_: ["--"] },
    { arguments_: ["--output"] },
    { arguments_: ["--output", ""] },
    { arguments_: ["--unknown", "result.json"] },
    { arguments_: ["--output", "result.json", "extra"] },
    { arguments_: ["--", "--", "--output", "result.json"] },
  ])("rejects unsupported argument lists: $arguments_", ({ arguments_ }) => {
    expect(() => parseOutputArgument(arguments_)).toThrow(
      "usage: pnpm run spike:virtual-scroll -- --output <result.json>",
    );
  });
});
