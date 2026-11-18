import { beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

import { getAppVersion } from "./ipc";

describe("ipc", () => {
  beforeEach(() => {
    invoke.mockReset();
  });

  it("getAppVersion calls the app_version command", async () => {
    invoke.mockResolvedValue("0.1.0");

    await expect(getAppVersion()).resolves.toBe("0.1.0");
    expect(invoke).toHaveBeenCalledWith("app_version");
  });
});
