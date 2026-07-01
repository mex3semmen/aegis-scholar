import type { CitationLocator } from "./appTypes";

export function sanitizeBackendError(error: unknown) {
  return String(error)
    .replace(/[A-Za-z]:\\[^"'\n]+/g, "[path hidden]")
    .replace(/\.aegis[\\/][^"'\n]+/g, "[path hidden]");
}

export function locatorSummary(locator: CitationLocator) {
  const section = locator.section ? `section=${locator.section}` : null;
  const paragraph = locator.paragraph_index !== null && locator.paragraph_index !== undefined ? `paragraph=${locator.paragraph_index}` : null;
  const range = `chars=${locator.start_char}-${locator.end_char}`;
  return [locator.label, section, paragraph, range].filter(Boolean).join(" | ");
}

export function compactTextPreview(text: string, maxChars = 240) {
  const compacted = text.split(/\s+/).filter(Boolean).join(" ").trim();
  if (compacted.length <= maxChars) {
    return compacted;
  }
  return `${compacted.slice(0, Math.max(0, maxChars - 1)).trimEnd()}...`;
}
