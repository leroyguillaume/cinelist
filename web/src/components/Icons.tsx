import type { ReactNode } from "react";

type IconProps = { size?: number };

function Stroke({ size = 22, children }: IconProps & { children: ReactNode }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2.2"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {children}
    </svg>
  );
}

export const BackIcon = (props: IconProps) => (
  <Stroke {...props}>
    <path d="M15 5l-7 7 7 7" />
  </Stroke>
);

export const CloseIcon = (props: IconProps) => (
  <Stroke {...props}>
    <path d="M6 6l12 12M18 6L6 18" />
  </Stroke>
);

export const PlusIcon = (props: IconProps) => (
  <Stroke {...props}>
    <path d="M12 5v14M5 12h14" />
  </Stroke>
);

export const PlayIcon = (props: IconProps) => (
  <Stroke {...props}>
    <path d="M7 4l12 8-12 8z" />
  </Stroke>
);

export const CheckIcon = (props: IconProps) => (
  <Stroke {...props}>
    <path d="M5 12.5l4.5 4.5L19 7.5" />
  </Stroke>
);

export const UndoIcon = (props: IconProps) => (
  <Stroke {...props}>
    <path d="M9 14L4 9l5-5" />
    <path d="M4 9h10.5a5.5 5.5 0 0 1 0 11H11" />
  </Stroke>
);

export const UsersIcon = (props: IconProps) => (
  <Stroke {...props}>
    <circle cx="9" cy="8" r="3.5" />
    <path d="M2.5 20a6.5 6.5 0 0 1 13 0" />
    <path d="M16 4.5a3.5 3.5 0 0 1 0 7M18 14.5a6.5 6.5 0 0 1 3.5 5.5" />
  </Stroke>
);

export const SettingsIcon = (props: IconProps) => (
  <Stroke {...props}>
    <circle cx="12" cy="12" r="3" />
    <path d="M12 2v3M12 19v3M4.2 4.2l2.1 2.1M17.7 17.7l2.1 2.1M2 12h3M19 12h3M4.2 19.8l2.1-2.1M17.7 6.3l2.1-2.1" />
  </Stroke>
);

export const ChartIcon = (props: IconProps) => (
  <Stroke {...props}>
    <path d="M4 20V10M10 20V4M16 20v-7M22 20H2" />
  </Stroke>
);

export const MoreIcon = ({ size = 18 }: IconProps) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
    <circle cx="5" cy="12" r="1.8" />
    <circle cx="12" cy="12" r="1.8" />
    <circle cx="19" cy="12" r="1.8" />
  </svg>
);
