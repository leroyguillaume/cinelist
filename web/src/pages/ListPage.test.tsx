import { screen } from "@testing-library/react";
import { Route, Routes } from "react-router";
import { afterEach, describe, expect, it, vi } from "vitest";
import { jsonResponse, renderPage } from "../test/render";
import { ListPage } from "./ListPage";

const list = {
  id: "list-1",
  name: "Soirées ciné",
  filmCount: 1,
  filmsToAdd: 1,
  createdAt: "2026-09-27T20:00:00Z",
  participants: [{ id: "alex", name: "Alex" }],
  nextStarterId: "alex",
  activeSelection: null,
  films: [{ id: "heat", title: "Heat", addedAt: "2026-09-27T20:00:00Z" }],
};

describe("ListPage", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("links every film to its statistics", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async (input: RequestInfo | URL) =>
        String(input).includes("/selections")
          ? jsonResponse({ items: [], page: 1, perPage: 20, total: 0 })
          : jsonResponse(list),
      ),
    );
    await renderPage(
      <Routes>
        <Route path="/lists/:listId" element={<ListPage />} />
      </Routes>,
      "/lists/list-1",
    );

    const link = await screen.findByRole("link", { name: "Statistiques de Heat" });
    expect(link).toHaveAttribute("href", "/lists/list-1/films/heat");
  });
});
