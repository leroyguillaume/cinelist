import { afterEach, describe, expect, it, vi } from "vitest";
import { ApiError } from "../lib/api";
import { errorKey } from ".";

describe("errorKey", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("explains an API error by its code", () => {
    expect(errorKey(new ApiError(409, "list_full"))).toBe("errors.list_full");
    expect(errorKey(new ApiError(500, "internal"))).toBe("errors.generic");
  });

  it("says the device is offline when a request could not leave it", () => {
    vi.spyOn(navigator, "onLine", "get").mockReturnValue(false);

    expect(errorKey(new TypeError("Failed to fetch"))).toBe("errors.offline");
  });

  it("keeps a network failure while online generic", () => {
    vi.spyOn(navigator, "onLine", "get").mockReturnValue(true);

    expect(errorKey(new TypeError("Failed to fetch"))).toBe("errors.generic");
  });
});
