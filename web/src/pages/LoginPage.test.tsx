import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { jsonResponse, renderPage } from "../test/render";
import { LoginPage, safeNext } from "./LoginPage";

describe("safeNext", () => {
  it("follows same-origin paths only", () => {
    expect(safeNext("/lists/1")).toBe("/lists/1");
    expect(safeNext("//evil.example")).toBe("/");
    expect(safeNext("https://evil.example")).toBe("/");
    expect(safeNext(null)).toBe("/");
  });
});

describe("LoginPage", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("shows why signing in failed", async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL, _init?: RequestInit) => {
      if (String(input).endsWith("/sessions/current")) {
        return jsonResponse({ status: 401, code: "unauthenticated" }, 401);
      }
      return jsonResponse({ status: 422, code: "invalid_credentials" }, 422);
    });
    vi.stubGlobal("fetch", fetchMock);
    await renderPage(<LoginPage />, "/login");

    await userEvent.type(await screen.findByLabelText("Pseudo"), "alex");
    await userEvent.type(screen.getByLabelText("Mot de passe"), "wrong");
    await userEvent.click(screen.getByRole("button", { name: "Se connecter" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("Pseudo ou mot de passe incorrect.");
    const init = fetchMock.mock.calls.at(-1)?.[1];
    expect(JSON.parse(String(init?.body))).toEqual({ username: "alex", password: "wrong" });
  });
});
