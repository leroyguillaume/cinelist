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

  it("keeps the following rows in place when a participant is removed", async () => {
    await renderPage(<NewListPage />, "/lists/new");
    await userEvent.type(screen.getByLabelText("Prénom 1"), "Alice");
    await userEvent.type(screen.getByLabelText("Prénom 2"), "Bob");
    await userEvent.click(screen.getByRole("button", { name: "Ajouter un participant" }));
    const chloe = screen.getByLabelText("Prénom 3");
    await userEvent.type(chloe, "Chloé");

    await userEvent.click(screen.getByRole("button", { name: "Retirer Bob" }));

    expect(chloe).toBeInTheDocument();
    expect(chloe).toHaveValue("Chloé");
    expect(screen.getByLabelText("Prénom 1")).toHaveValue("Alice");
  });
});
