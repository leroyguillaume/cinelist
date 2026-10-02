import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { renderPage } from "../test/render";
import { NewListPage } from "./NewListPage";

describe("NewListPage", () => {
  it("says how many films are missing for everyone to play as many turns", async () => {
    await renderPage(<NewListPage />, "/lists/new");
    const films = screen.getByLabelText("Films");

    await userEvent.type(films, "Heat{enter}Drive");
    expect(
      screen.getByText("Il faudra encore 1 film pour que chacun élimine autant de films."),
    ).toBeInTheDocument();

    await userEvent.type(films, "{enter}Alien");
    expect(screen.getByText("3 films : chacun élimine 1 film par sélection.")).toBeInTheDocument();
  });
});
