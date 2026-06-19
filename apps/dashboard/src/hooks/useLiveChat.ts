import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { OllamaClient, createMessage, defaultRuntimeSettings, type ChatMessage, type ModelInfo } from "@live-runtime/core";
import { speakText } from "../lib/tauriBridge";
import { saveEntry, relatedEntries, type ScoredJournalRecord } from "../lib/semantic";

const KEY = "live-runtime.chat.messages";
const MODEL_KEY = "live-runtime.chat.model";
const BASE_URL_KEY = "live-runtime.chat.base-url";
const SPEAK_KEY = "live-runtime.chat.speak";
const RETRIEVAL_TIMEOUT_MS = 1200;

export interface RuntimeChatState {
  messages: ChatMessage[];
  models: ModelInfo[];
  model: string;
  baseUrl: string;
  isLoading: boolean;
  error: string | null;
  speakResponses: boolean;
  setModel(model: string): void;
  setBaseUrl(value: string): void;
  setSpeakResponses(value: boolean): void;
  reloadModels(): Promise<void>;
  send(content: string): Promise<void>;
  clear(): void;
  resetAll(): void;
}

export function useRuntimeChat(): RuntimeChatState {
  const [messages, setMessages] = useState<ChatMessage[]>(() => readStoredMessages());
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [model, updateModel] = useState(() => window.localStorage.getItem(MODEL_KEY) ?? defaultRuntimeSettings.defaultModel);
  const [baseUrl, updateBaseUrl] = useState(() => window.localStorage.getItem(BASE_URL_KEY) ?? defaultRuntimeSettings.providerBaseUrl);
  const [speakResponses, updateSpeakResponses] = useState(() => {
    const stored = window.localStorage.getItem(SPEAK_KEY);
    return stored === null ? defaultRuntimeSettings.speakResponses : stored === "true";
  });
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const abortRef = useRef<AbortController | null>(null);
  const client = useMemo(() => new OllamaClient({ baseUrl }), [baseUrl]);

  const setModel = useCallback((value: string) => {
    updateModel(value);
    window.localStorage.setItem(MODEL_KEY, value);
  }, []);

  const setBaseUrl = useCallback((value: string) => {
    updateBaseUrl(value);
    window.localStorage.setItem(BASE_URL_KEY, value);
  }, []);

  const setSpeakResponses = useCallback((value: boolean) => {
    updateSpeakResponses(value);
    window.localStorage.setItem(SPEAK_KEY, String(value));
  }, []);

  useEffect(() => window.localStorage.setItem(KEY, JSON.stringify(messages)), [messages]);

  const reloadModels = useCallback(async () => {
    try {
      setError(null);
      const found = await client.listModels();
      const chatModels = found.filter((item) => !isEmbeddingOnlyModel(item));
      const usableModels = chatModels.length > 0 ? chatModels : found;
      setModels(usableModels);
      if (usableModels.length > 0 && !usableModels.some((item) => item.name === model)) setModel(usableModels[0].name);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Unable to connect to Ollama");
    }
  }, [client, model, setModel]);

  useEffect(() => { void reloadModels(); }, [reloadModels]);

  const send = useCallback(async (content: string) => {
    const text = content.trim();
    if (!text || isLoading) return;

    if (isEmbeddingOnlyName(model)) {
      setError("The selected model is an embedding model. Choose a chat model like qwen3.5:4b, gemma4:4b, granite4:8b, or nemotron-3-nano:4b.");
      return;
    }

    abortRef.current?.abort();
    const abort = new AbortController();
    abortRef.current = abort;

    const userMessage = createMessage("user", text);
    const assistantMessage = createMessage("assistant", "");
    const history = messages.filter((message) => message.content.trim() && message.content !== "Thinking...");
    setMessages((current) => [...current, userMessage, assistantMessage]);
    setIsLoading(true);
    setError(null);

    let notes: ScoredJournalRecord[] = [];
    try {
      notes = await withFallback(relatedEntries(baseUrl, text, 5), [], RETRIEVAL_TIMEOUT_MS);
    } catch {
      notes = [];
    }

    const noteMessage = notes.length > 0 ? createMessage("system", summarizeNotes(notes)) : null;
    void saveEntry(baseUrl, { kind: "chat", scope: "longTerm", title: "User message", content: text, source: "chat:user", confidence: 1, tags: ["chat", "user"] });

    let reply = "";
    try {
      for await (const chunk of client.chat({
        model,
        signal: abort.signal,
        messages: [
          { role: "system", content: "You are Live Runtime, a concise local AI companion running through Ollama. Answer directly. If relevant memory notes are supplied, use them silently and do not mention them unless asked." },
          ...history,
          ...(noteMessage ? [noteMessage] : []),
          userMessage
        ],
        temperature: defaultRuntimeSettings.temperature
      })) {
        reply += chunk.content;
        setMessages((current) => current.map((message) => message.id === assistantMessage.id ? { ...message, content: reply || "Thinking..." } : message));
      }

      if (!reply.trim()) {
        reply = "I reached Ollama but received an empty response. Try another chat model from Settings.";
        setMessages((current) => current.map((message) => message.id === assistantMessage.id ? { ...message, content: reply } : message));
      }

      void saveEntry(baseUrl, { kind: "chat", scope: "longTerm", title: "Assistant reply", content: reply, source: "chat:assistant", confidence: 0.9, tags: ["chat", "assistant", model] });
      if (speakResponses) await speakText(reply);
    } catch (cause) {
      if (!abort.signal.aborted) {
        const message = cause instanceof Error ? cause.message : "Chat request failed";
        setError(message);
        setMessages((current) => current.map((item) => item.id === assistantMessage.id ? { ...item, content: `I could not get a response from Ollama. ${message}` } : item));
      }
    } finally {
      setIsLoading(false);
    }
  }, [baseUrl, client, isLoading, messages, model, speakResponses]);

  const clear = useCallback(() => {
    const fresh = [createMessage("assistant", "New chat started. What should we work on?")];
    setMessages(fresh);
    window.localStorage.setItem(KEY, JSON.stringify(fresh));
  }, []);

  const resetAll = useCallback(() => {
    Object.keys(window.localStorage)
      .filter((key) => key.startsWith("live-runtime."))
      .forEach((key) => window.localStorage.removeItem(key));
    window.location.reload();
  }, []);

  return { messages, models, model, baseUrl, isLoading, error, speakResponses, setModel, setBaseUrl, setSpeakResponses, reloadModels, send, clear, resetAll };
}

function summarizeNotes(items: ScoredJournalRecord[]): string {
  const lines = items.map((item, index) => `${index + 1}. ${item.record.content.replace(/\s+/g, " ").slice(0, 180)}`);
  return `Relevant local notes for this reply:\n${lines.join("\n")}`;
}

function readStoredMessages(): ChatMessage[] {
  try {
    const raw = window.localStorage.getItem(KEY);
    if (!raw) return [createMessage("assistant", "Live Runtime is ready. Start Ollama, choose a chat model, then speak or type a message.")];
    const parsed = JSON.parse(raw) as ChatMessage[];
    return Array.isArray(parsed) && parsed.length > 0 ? parsed : [createMessage("assistant", "Live Runtime is ready.")];
  } catch {
    return [createMessage("assistant", "Live Runtime is ready.")];
  }
}

function isEmbeddingOnlyModel(model: ModelInfo): boolean {
  return isEmbeddingOnlyName(model.name) || model.family === "bert";
}

function isEmbeddingOnlyName(name: string): boolean {
  const normalized = name.toLowerCase();
  return normalized.includes("embed") || normalized.includes("embedding") || normalized.includes("nomic-embed");
}

async function withFallback<T>(promise: Promise<T>, fallback: T, timeoutMs: number): Promise<T> {
  let timeoutId: number | undefined;
  const timeout = new Promise<T>((resolve) => {
    timeoutId = window.setTimeout(() => resolve(fallback), timeoutMs);
  });

  try {
    return await Promise.race([promise, timeout]);
  } finally {
    if (typeof timeoutId === "number") window.clearTimeout(timeoutId);
  }
}
