import { FormEvent, useMemo, useState } from "react";
import { createBrowserSpeechRecognition } from "@live-runtime/core";

interface ChatComposerProps {
  disabled?: boolean;
  onSend(content: string): Promise<void>;
  onNewChat?: () => void;
  onResetAll?: () => void;
}

export function ChatComposer({ disabled, onSend, onNewChat, onResetAll }: ChatComposerProps) {
  const [input, setInput] = useState("");
  const [partial, setPartial] = useState("");
  const [isListening, setIsListening] = useState(false);
  const recognition = useMemo(() => createBrowserSpeechRecognition(), []);

  async function sendCurrentInput() {
    const content = input.trim();
    if (!content || disabled) return;
    setInput("");
    await onSend(content);
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await sendCurrentInput();
  }

  function toggleListening() {
    if (!recognition.supported) {
      setPartial("Voice input is unavailable in this WebView. Type instead.");
      return;
    }

    if (isListening) {
      recognition.stop();
      setIsListening(false);
      return;
    }

    setPartial("");
    setIsListening(true);
    recognition.start({
      onPartial: setPartial,
      onFinal(text) {
        setInput((current) => [current, text].filter(Boolean).join(" "));
        setPartial("");
        setIsListening(false);
      },
      onError(error) {
        setPartial(error);
        setIsListening(false);
      }
    });
  }

  function fallbackNewChat() {
    if (onNewChat) {
      onNewChat();
      return;
    }

    window.localStorage.removeItem("live-runtime.chat.messages");
    window.location.reload();
  }

  function fallbackResetAll() {
    const confirmed = window.confirm("Reset Live Runtime UI state on this device? This clears local chat, settings, routines, skills, and cached UI preferences.");
    if (!confirmed) return;

    if (onResetAll) {
      onResetAll();
      return;
    }

    Object.keys(window.localStorage)
      .filter((key) => key.startsWith("live-runtime."))
      .forEach((key) => window.localStorage.removeItem(key));
    window.location.reload();
  }

  return (
    <form className="composer" onSubmit={submit}>
      <div className="composer-toolbar" aria-label="Chat controls">
        <button type="button" onClick={fallbackNewChat}>
          New Chat
        </button>
        <button type="button" className="danger-soft" onClick={fallbackResetAll}>
          Reset App
        </button>
      </div>
      <div className="composer-input-wrap">
        <textarea
          value={input}
          disabled={disabled}
          rows={3}
          placeholder="Ask Live Runtime anything..."
          onChange={(event) => setInput(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter" && !event.shiftKey) {
              event.preventDefault();
              void sendCurrentInput();
            }
          }}
        />
        {partial && <span className="voice-partial">{partial}</span>}
      </div>
      <div className="composer-actions">
        <button type="button" className={isListening ? "recording" : ""} onClick={toggleListening}>
          {isListening ? "Listening" : "Voice"}
        </button>
        <button type="submit" disabled={disabled || !input.trim()}>
          Send
        </button>
      </div>
    </form>
  );
}
