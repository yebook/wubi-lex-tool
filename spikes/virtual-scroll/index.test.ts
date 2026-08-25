import { describe, expect, it } from "vitest";

import html from "./index.html?raw";

describe("virtual-scroll harness document", () => {
  it("uses an inline favicon instead of requesting a missing resource", () => {
    expect(html).toContain('<link rel="icon" href="data:," />');
  });
});
