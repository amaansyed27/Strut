import { useEffect, useRef, type CSSProperties } from "react";
import {
  loadStrutUrl,
  mountStrut,
  type MountedStrut,
  type StrutDocument,
  type StrutPackage,
} from "@strut/runtime-web";

export type StrutProps = {
  src?: string;
  document?: StrutDocument;
  strutPackage?: StrutPackage;
  artboard?: string;
  stateMachine?: string;
  state?: string;
  inputs?: Record<string, boolean | number | string>;
  bindings?: Record<string, string>;
  reducedMotion?: boolean;
  className?: string;
  style?: CSSProperties;
  onLoad?: (player: MountedStrut) => void;
  onError?: (error: Error) => void;
};

export function Strut({
  src,
  document,
  strutPackage,
  artboard,
  stateMachine,
  state,
  inputs,
  bindings,
  reducedMotion,
  className,
  style,
  onLoad,
  onError,
}: StrutProps) {
  const hostRef = useRef<HTMLDivElement | null>(null);
  const playerRef = useRef<MountedStrut | null>(null);

  useEffect(() => {
    let cancelled = false;
    async function mount() {
      try {
        const host = hostRef.current;
        if (!host) {
          return;
        }
        const resolved = document ?? strutPackage?.document ?? (src ? (await loadStrutUrl(src)).document : null);
        if (!resolved) {
          throw new Error("Strut requires src, document, or strutPackage");
        }
        if (cancelled) {
          return;
        }
        playerRef.current?.destroy();
        const player = mountStrut(host, resolved, {
          artboard,
          stateMachine,
          initialState: state,
          reducedMotion,
        });
        playerRef.current = player;
        onLoad?.(player);
      } catch (error) {
        onError?.(error instanceof Error ? error : new Error(String(error)));
      }
    }

    void mount();
    return () => {
      cancelled = true;
      playerRef.current?.destroy();
      playerRef.current = null;
    };
  }, [src, document, strutPackage, artboard, stateMachine, reducedMotion]);

  useEffect(() => {
    if (state) {
      playerRef.current?.setState(state);
    }
  }, [state]);

  useEffect(() => {
    if (!inputs) {
      return;
    }
    for (const [name, value] of Object.entries(inputs)) {
      playerRef.current?.setInput(name, value);
    }
  }, [inputs]);

  useEffect(() => {
    if (!bindings) {
      return;
    }
    for (const [name, value] of Object.entries(bindings)) {
      playerRef.current?.setBinding(name, value);
    }
  }, [bindings]);

  return <div ref={hostRef} className={className} style={style} data-strut-react="" />;
}

export type { MountedStrut, StrutDocument, StrutPackage };
