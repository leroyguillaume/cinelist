type SegmentsProps = {
  total: number;
  done: number;
  /** Highlights the segment right after the done ones. */
  showCurrent?: boolean;
  label: string;
};

export function Segments({ total, done, showCurrent = false, label }: SegmentsProps) {
  return (
    <div className="segments" role="img" aria-label={label}>
      {Array.from({ length: total }, (_, index) => {
        const state =
          index < done ? " segment-done" : showCurrent && index === done ? " segment-current" : "";
        return <div key={index} className={`segment${state}`} />;
      })}
    </div>
  );
}
