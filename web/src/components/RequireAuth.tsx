import type { ReactNode } from "react";
import { Navigate, useLocation } from "react-router";
import { useSession } from "../lib/queries";
import { ErrorText } from "./ErrorText";
import { Splash } from "./Loading";

export function RequireAuth({ admin = false, children }: { admin?: boolean; children: ReactNode }) {
  const session = useSession();
  const location = useLocation();
  if (session.isPending) {
    return <Splash />;
  }
  if (session.isError) {
    return (
      <div className="splash">
        <ErrorText error={session.error} />
      </div>
    );
  }
  if (!session.data) {
    const next = `${location.pathname}${location.search}`;
    return <Navigate to={`/login?next=${encodeURIComponent(next)}`} replace />;
  }
  if (admin && !session.data.user.isAdmin) {
    return <Navigate to="/" replace />;
  }
  return children;
}
