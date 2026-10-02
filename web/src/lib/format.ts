export function formatPlace(place: number, language: string): string {
  if (language.startsWith("fr")) {
    return place === 1 ? "1er" : `${place}e`;
  }
  const rule = new Intl.PluralRules("en", { type: "ordinal" }).select(place);
  const suffix = { one: "st", two: "nd", few: "rd" }[rule as string] ?? "th";
  return `${place}${suffix}`;
}

export function formatDate(iso: string, language: string): string {
  return new Intl.DateTimeFormat(language, {
    weekday: "short",
    day: "numeric",
    month: "short",
  }).format(new Date(iso));
}

/** French elides "de" before a vowel: "au tour d'Alex", "au tour de Sam". */
export function elides(name: string): boolean {
  return /^[aeiouyàâäéèêëîïôöùûüh]/i.test(name.normalize("NFC"));
}
