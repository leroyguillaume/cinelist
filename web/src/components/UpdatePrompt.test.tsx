import type { RegisterSWOptions } from "virtual:pwa-register/react";
import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { initI18n } from "../i18n";
import { UpdatePrompt } from "./UpdatePrompt";

const sw = vi.hoisted(() => ({
  needRefresh: false,
  setNeedRefresh: vi.fn(),
  updateServiceWorker: vi.fn(async () => {}),
  options: undefined as RegisterSWOptions | undefined,
}));

vi.mock("virtual:pwa-register/react", () => ({
  useRegisterSW: (options: RegisterSWOptions) => {
    sw.options = options;
    return {
      needRefresh: [sw.needRefresh, sw.setNeedRefresh],
      offlineReady: [false, vi.fn()],
      updateServiceWorker: sw.updateServiceWorker,
    };
  },
}));

async function renderPrompt() {
  await initI18n("fr");
  return render(<UpdatePrompt />);
}

describe("UpdatePrompt", () => {
  beforeEach(() => {
    sw.needRefresh = false;
    sw.options = undefined;
    vi.clearAllMocks();
  });

  it("stays hidden while no new version is waiting", async () => {
    await renderPrompt();

    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });

  it("reloads onto the new version on request", async () => {
    sw.needRefresh = true;
    await renderPrompt();

    expect(screen.getByRole("status")).toHaveTextContent(
      "Une nouvelle version de CineList est disponible.",
    );
    await userEvent.click(screen.getByRole("button", { name: "Recharger" }));

    expect(sw.updateServiceWorker).toHaveBeenCalledOnce();
  });

  it("can be dismissed without reloading", async () => {
    sw.needRefresh = true;
    await renderPrompt();

    await userEvent.click(screen.getByRole("button", { name: "Plus tard" }));

    expect(sw.setNeedRefresh).toHaveBeenCalledWith(false);
    expect(sw.updateServiceWorker).not.toHaveBeenCalled();
  });

  it("looks for a new version when the app comes back to the foreground", async () => {
    const update = vi.fn(async () => {});
    const { unmount } = await renderPrompt();
    act(() => {
      sw.options?.onRegisteredSW?.("/sw.js", { update } as unknown as ServiceWorkerRegistration);
    });

    document.dispatchEvent(new Event("visibilitychange"));
    expect(update).toHaveBeenCalledOnce();

    unmount();
    document.dispatchEvent(new Event("visibilitychange"));
    expect(update).toHaveBeenCalledOnce();
  });
});
