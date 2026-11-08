import { QueryClientProvider } from "@tanstack/react-query";
import { act, render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { afterEach, describe, expect, it, vi } from "vitest";
import { initI18n } from "../i18n";
import { createQueryClient } from "../lib/queries";
import { jsonResponse } from "../test/render";
import { RequireAuth } from "./RequireAuth";

const session = { user: { id: "alex", username: "alex", isAdmin: false } };

function setOnline(online: boolean) {
  vi.spyOn(navigator, "onLine", "get").mockReturnValue(online);
}

async function renderGuarded() {
  await initI18n("fr");
  return render(
    <QueryClientProvider client={createQueryClient()}>
      <MemoryRouter>
        <RequireAuth>
          <p>Mes listes</p>
        </RequireAuth>
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

describe("RequireAuth", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it("says the app is offline and recovers once the connection is back", async () => {
    setOnline(false);
    const fetchMock = vi.fn(async (): Promise<Response> => {
      throw new TypeError("Failed to fetch");
    });
    vi.stubGlobal("fetch", fetchMock);
    await renderGuarded();

    expect(await screen.findByRole("alert")).toHaveTextContent("Tu es hors ligne.");
    expect(fetchMock).toHaveBeenCalledOnce();

    fetchMock.mockImplementation(async () => jsonResponse(session));
    setOnline(true);
    act(() => {
      window.dispatchEvent(new Event("online"));
    });

    expect(await screen.findByText("Mes listes")).toBeInTheDocument();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });
});
