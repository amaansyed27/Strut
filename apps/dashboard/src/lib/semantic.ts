import { listJournal, saveJournal, type JournalDraft, type JournalRecord } from "./journalBridge";

export const EMBED_MODEL = "nomic-embed-text";
const EMBED_TIMEOUT_MS = 1600;

export interface ScoredJournalRecord {
  record: JournalRecord;
  score: number;
}

export async function checkEmbeddingModel(baseUrl: string, model = EMBED_MODEL): Promise<boolean | null> {
  try {
    const response = await fetch(`${baseUrl.replace(/\/$/, "")}/api/tags`, { signal: AbortSignal.timeout(1800) });
    if (!response.ok) return null;
    const payload = await response.json() as { models?: Array<{ name?: string; model?: string }> };
    return (payload.models ?? []).some((item) => isSameModel(item.name ?? item.model ?? "", model));
  } catch {
    return null;
  }
}

export async function makeEmbedding(baseUrl: string, input: string, model = EMBED_MODEL): Promise<number[] | null> {
  const text = input.trim();
  if (!text) return null;
  try {
    const response = await fetch(`${baseUrl.replace(/\/$/, "")}/api/embed`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      signal: AbortSignal.timeout(EMBED_TIMEOUT_MS),
      body: JSON.stringify({ model, input: text })
    });
    if (!response.ok) return null;
    const payload = await response.json() as { embeddings?: number[][] };
    return payload.embeddings?.[0] ?? null;
  } catch {
    return null;
  }
}

export async function saveEntry(baseUrl: string, draft: JournalDraft): Promise<void> {
  const embedding = await makeEmbedding(baseUrl, draft.content);
  await saveJournal({
    ...draft,
    embeddingModel: embedding ? EMBED_MODEL : undefined,
    vector: embedding ?? undefined
  });
}

export async function relatedEntries(baseUrl: string, query: string, limit = 6): Promise<ScoredJournalRecord[]> {
  const records = await listJournal(160);
  const queryEmbedding = await makeEmbedding(baseUrl, query);
  if (!queryEmbedding) return keywordSearch(records, query, limit);

  const semanticMatches = records
    .filter((record) => Array.isArray(record.vector) && record.vector.length === queryEmbedding.length)
    .map((record) => ({ record, score: cosine(queryEmbedding, record.vector ?? []) }))
    .filter((item) => item.score > 0.18)
    .sort((a, b) => b.score - a.score)
    .slice(0, limit);

  return semanticMatches.length > 0 ? semanticMatches : keywordSearch(records, query, limit);
}

function keywordSearch(records: JournalRecord[], query: string, limit: number): ScoredJournalRecord[] {
  const terms = query.toLowerCase().split(/\s+/).filter((term) => term.length > 2);
  if (terms.length === 0) return [];
  return records
    .map((record) => {
      const haystack = `${record.title} ${record.content} ${record.tags.join(" ")}`.toLowerCase();
      const matches = terms.filter((term) => haystack.includes(term)).length;
      return { record, score: matches / terms.length };
    })
    .filter((item) => item.score > 0)
    .sort((a, b) => b.score - a.score)
    .slice(0, limit);
}

function isSameModel(candidate: string, expected: string): boolean {
  const cleanCandidate = candidate.replace(/:latest$/, "");
  const cleanExpected = expected.replace(/:latest$/, "");
  return cleanCandidate === cleanExpected || candidate === expected || candidate === `${expected}:latest`;
}

function cosine(left: number[], right: number[]): number {
  let dot = 0;
  let leftMagnitude = 0;
  let rightMagnitude = 0;
  for (let index = 0; index < left.length; index += 1) {
    dot += left[index] * right[index];
    leftMagnitude += left[index] * left[index];
    rightMagnitude += right[index] * right[index];
  }
  if (leftMagnitude === 0 || rightMagnitude === 0) return 0;
  return dot / (Math.sqrt(leftMagnitude) * Math.sqrt(rightMagnitude));
}
