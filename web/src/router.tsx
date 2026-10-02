import { createBrowserRouter } from "react-router";
import { AppShell } from "./components/AppShell";
import { RequireAuth } from "./components/RequireAuth";
import { FilmStatsPage } from "./pages/FilmStatsPage";
import { HistoryPage } from "./pages/HistoryPage";
import { InvitationPage } from "./pages/InvitationPage";
import { ListPage } from "./pages/ListPage";
import { ListSettingsPage } from "./pages/ListSettingsPage";
import { ListsPage } from "./pages/ListsPage";
import { LoginPage } from "./pages/LoginPage";
import { NewListPage } from "./pages/NewListPage";
import { NotFoundPage } from "./pages/NotFoundPage";
import { SelectionPage } from "./pages/SelectionPage";
import { UsersPage } from "./pages/UsersPage";

export const router = createBrowserRouter([
  { path: "/login", element: <LoginPage /> },
  { path: "/invitation/:token", element: <InvitationPage /> },
  {
    path: "/lists/:listId/selections/:selectionId",
    element: (
      <RequireAuth>
        <SelectionPage />
      </RequireAuth>
    ),
  },
  {
    element: (
      <RequireAuth>
        <AppShell />
      </RequireAuth>
    ),
    children: [
      { path: "/", element: <ListsPage /> },
      { path: "/lists/new", element: <NewListPage /> },
      { path: "/lists/:listId", element: <ListPage /> },
      { path: "/lists/:listId/history", element: <HistoryPage /> },
      { path: "/lists/:listId/settings", element: <ListSettingsPage /> },
      { path: "/lists/:listId/films/:filmId", element: <FilmStatsPage /> },
      {
        path: "/users",
        element: (
          <RequireAuth admin>
            <UsersPage />
          </RequireAuth>
        ),
      },
      { path: "*", element: <NotFoundPage /> },
    ],
  },
]);
