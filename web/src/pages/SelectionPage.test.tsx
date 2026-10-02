import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { selection } from "../test/fixtures";
import { jsonResponse, renderPage } from "../test/render";
import { EliminationView, VerdictView } from "./SelectionPage";

describe("EliminationView", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("elides the French turn label before a vowel", async () => {
    await renderPage(<EliminationView listId="list-1" selection={selection()} />);
    expect(screen.getByRole("heading", { level: 1 })).toHaveTextContent("Au tour d'Alex");
  });

  it("eliminates the film that is tapped", async () => {
    const fetchMock = vi.fn(async (_input: RequestInfo | URL, _init?: RequestInit) =>
      jsonResponse(selection(), 201),
    );
    vi.stubGlobal("fetch", fetchMock);
    await renderPage(<EliminationView listId="list-1" selection={selection()} />);

    await userEvent.click(screen.getByRole("button", { name: "Drive" }));

    const [url, init] = fetchMock.mock.calls[0] ?? [];
    expect(url).toBe("/api/v1/selections/selection-1/eliminations");
    expect(JSON.parse(String(init?.body))).toEqual({ filmId: "drive" });
  });
});

describe("VerdictView", () => {
  it("names the film that takes over when the chosen one is abandoned", async () => {
    await renderPage(
      <VerdictView
        listId="list-1"
        selection={selection({
          phase: "awaitingVerdict",
          currentParticipant: null,
          candidate: { filmId: "alien", title: "Alien", place: 1 },
          remainingFilms: [{ id: "alien", title: "Alien" }],
          eliminations: [
            {
              id: "e2",
              filmId: "drive",
              filmTitle: "Drive",
              participantId: "sam",
              participantName: "Sam",
              place: 2,
            },
          ],
        })}
      />,
    );
    expect(screen.getByRole("heading", { level: 1 })).toHaveTextContent("Alien");
    expect(
      screen.getByText("Sam a éliminé Drive en finale.", { exact: false }),
    ).toBeInTheDocument();
    expect(screen.getByText("Drive prend sa place")).toBeInTheDocument();
  });
});
