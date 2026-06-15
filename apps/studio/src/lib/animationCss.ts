/**
 * CSS animation generation engine for Strut document preview.
 *
 * Converts StrutDocument timelines and state machines into CSS @keyframes
 * and animation rules that drive the SVG preview rendering.
 *
 * Extracted from App.tsx (~230 lines of pure functions).
 */

import type { StrutDocument, StrutNode, Timeline } from "../types";

type TimelineTrack = NonNullable<Timeline["tracks"]>[number];
type NumericTimelineKeyframe = TimelineTrack["keyframes"][number] & { value: { type: "number"; value: number } };
type ResolvedTransform = Required<NonNullable<StrutNode["transform"]>>;
export type StateNodeOverride = {
  opacity?: number;
  transform?: string;
};

/**
 * Generate all CSS needed to animate the document in the given state.
 */
export function documentAnimationCss(document: StrutDocument, activeState: string): string {
  const timelines = timelinesForState(document, activeState);
  const transforms = nodeTransformMap(document);
  const shapes = nodeShapeMap(document);
  return timelines
    .flatMap((timeline) => [
      timelineAnimationCss(timeline, transforms, shapes),
      ...stateTimelineCss(timeline, transforms),
    ])
    .filter(Boolean)
    .join("\n");
}

/**
 * Find timelines that should play for the given state.
 * Matches by state name and by transitions that lead to this state.
 */
export function timelinesForState(document: StrutDocument, activeState: string): Timeline[] {
  const machine = document.state_machines[0];
  const timelineNames = new Set([activeState]);
  machine?.transitions?.filter((t) => t.to === activeState).forEach((t) => timelineNames.add(t.timeline));
  return document.timelines.filter((timeline) => timelineNames.has(timeline.name));
}

function timelineAnimationCss(timeline: Timeline, transforms: Map<string, StrutNode["transform"]>, shapes: Map<string, StrutNode["shape"]>): string {
  return Array.from(timelineTrackGroups(timeline).entries())
    .flatMap(([target, tracks]) => [
      transformTracksCss(timeline, target, tracks.filter((t) => isTransformProperty(t.property)), transforms.get(target)),
      ...tracks.filter((t) => isScalarProperty(t.property)).map((t) => scalarTrackCss(timeline, t, shapes.get(target))),
    ])
    .filter(Boolean)
    .join("\n");
}

function transformTracksCss(
  timeline: Timeline,
  target: string,
  tracks: TimelineTrack[],
  baseTransform: StrutNode["transform"],
): string {
  if (!tracks.length) return "";
  const times = sortedTimelineTimes(timeline, tracks);
  const frames = times
    .map((time) => {
      const percent = Math.max(0, Math.min(100, (time / timeline.duration_ms) * 100));
      const base = normalizeTransform(baseTransform);
      const tx = base.translate_x + trackValue(tracks, "translation.x", time, 0);
      const ty = base.translate_y + trackValue(tracks, "translation.y", time, 0);
      const rotate = base.rotate + trackValue(tracks, "rotation", time, 0);
      const rotate_x = base.rotate_x + trackValue(tracks, "rotation.x", time, 0);
      const rotate_y = base.rotate_y + trackValue(tracks, "rotation.y", time, 0);
      const scale = trackValue(tracks, "scale", time, 1);
      const sx = base.scale_x * scale * trackValue(tracks, "scale.x", time, 1);
      const sy = base.scale_y * scale * trackValue(tracks, "scale.y", time, 1);
      return `${percent}% { transform: translate(${round(tx)}px, ${round(ty)}px) rotateZ(${round(rotate)}deg) rotateX(${round(rotate_x)}deg) rotateY(${round(rotate_y)}deg) scale(${round(sx)}, ${round(sy)}); }`;
    })
    .join("\n");
  return `@keyframes ${transformAnimationName(timeline, target)} { ${frames} }`;
}

function scalarTrackCss(timeline: Timeline, track: TimelineTrack, shape: StrutNode["shape"]): string {
  const numericKeyframes = numericTrackKeyframes(track);
  if (numericKeyframes.length < 2) return "";
  const frames = numericKeyframes
    .map((kf) => {
      const percent = Math.max(0, Math.min(100, (kf.time_ms / timeline.duration_ms) * 100));
      if (track.property === "frame" && shape?.type === "sprite") {
        const frame = Math.round(Number(kf.value.value));
        const x = (frame % shape.columns) * shape.frame_width;
        const y = Math.floor(frame / shape.columns) * shape.frame_height;
        return `${percent}% { --sprite-x: -${x}px; --sprite-y: -${y}px; }`;
      }
      return `${percent}% { ${track.property}: ${round(Number(kf.value.value))}; }`;
    })
    .join("\n");
  return `@keyframes ${scalarAnimationName(timeline, track)} { ${frames} }`;
}

function stateTimelineCss(timeline: Timeline, transforms: Map<string, StrutNode["transform"]>): string[] {
  const iteration = timelineLoops(timeline) ? "infinite" : "1 both";
  return Array.from(timelineTrackGroups(timeline).entries())
    .map(([target, tracks]) => {
      const animations = [
        tracks.some((t) => isTransformProperty(t.property))
          ? `${transformAnimationName(timeline, target)} ${timeline.duration_ms}ms ${groupEasing(tracks)} ${iteration}`
          : "",
        ...tracks
          .filter((t) => isScalarProperty(t.property))
          .map((t) => `${scalarAnimationName(timeline, t)} ${timeline.duration_ms}ms ${cssEasing(t.keyframes[0]?.easing ?? "linear")} ${iteration}`),
      ].filter(Boolean);
      if (!animations.length) return "";
      const base = transforms.get(target);
      const baseRule = tracks.some((t) => isTransformProperty(t.property))
        ? ` transform: ${transformCss(normalizeTransform(base))};`
        : "";
      return `
.document-scene.state-${cssIdent(timeline.name)} [data-node-id="${target}"] {
  transform-box: fill-box;
  transform-origin: center;
  ${baseRule}
  animation: ${animations.join(", ")};
}`;
    })
    .filter(Boolean);
}

export function stateNodeOverrides(document: StrutDocument, activeState: string, elapsedMs?: number): Map<string, StateNodeOverride> {
  const overrides = new Map<string, StateNodeOverride>();
  const transforms = nodeTransformMap(document);
  for (const timeline of timelinesForState(document, activeState)) {
    const sampleTime = timelineSampleTime(timeline, elapsedMs);
    for (const [target, tracks] of timelineTrackGroups(timeline).entries()) {
      const override = overrides.get(target) ?? {};
      const transformTracks = tracks.filter((track) => isTransformProperty(track.property));
      if (transformTracks.length) {
        override.transform = transformAtTime(transformTracks, transforms.get(target), sampleTime);
      }
      for (const track of tracks.filter((track) => isScalarProperty(track.property))) {
        const value = interpolatedTrackValue(track, sampleTime, Number.NaN);
        if (Number.isFinite(value) && track.property === "opacity") override.opacity = value;
      }
      overrides.set(target, override);
    }
  }
  return overrides;
}

export function stateAnimationDuration(document: StrutDocument, activeState: string): number {
  return Math.max(0, ...timelinesForState(document, activeState).map((timeline) => timeline.duration_ms || 0));
}

export function stateAnimationLoops(document: StrutDocument, activeState: string): boolean {
  return timelinesForState(document, activeState).some(timelineLoops);
}



function transformAtTime(tracks: TimelineTrack[], baseTransform: StrutNode["transform"], time: number): string {
  const base = normalizeTransform(baseTransform);
  const tx = base.translate_x + trackValue(tracks, "translation.x", time, 0);
  const ty = base.translate_y + trackValue(tracks, "translation.y", time, 0);
  const rotate = base.rotate + trackValue(tracks, "rotation", time, 0);
  const rotate_x = base.rotate_x + trackValue(tracks, "rotation.x", time, 0);
  const rotate_y = base.rotate_y + trackValue(tracks, "rotation.y", time, 0);
  const scale = trackValue(tracks, "scale", time, 1);
  const sx = base.scale_x * scale * trackValue(tracks, "scale.x", time, 1);
  const sy = base.scale_y * scale * trackValue(tracks, "scale.y", time, 1);
  return transformCss({ translate_x: tx, translate_y: ty, rotate, rotate_x, rotate_y, scale_x: sx, scale_y: sy });
}

function timelineLoops(timeline: Timeline): boolean {
  return timeline.loops ?? false;
}

function timelineSampleTime(timeline: Timeline, elapsedMs: number | undefined): number {
  const duration = Math.max(1, timeline.duration_ms || 1);
  if (elapsedMs === undefined) return duration;
  const elapsed = Math.max(0, elapsedMs);
  return timelineLoops(timeline) ? elapsed % duration : Math.min(elapsed, duration);
}


function timelineTrackGroups(timeline: Timeline): Map<string, TimelineTrack[]> {
  const groups = new Map<string, TimelineTrack[]>();
  for (const track of timeline.tracks ?? []) {
    if (!hasNumericMotion(track) || (!isTransformProperty(track.property) && !isScalarProperty(track.property))) continue;
    groups.set(track.target, [...(groups.get(track.target) ?? []), track]);
  }
  return groups;
}

function sortedTimelineTimes(timeline: Timeline, tracks: TimelineTrack[]): number[] {
  return Array.from(
    new Set([0, timeline.duration_ms, ...tracks.flatMap((t) => numericTrackKeyframes(t).map((kf) => kf.time_ms))]),
  ).sort((a, b) => a - b);
}

function trackValue(tracks: TimelineTrack[], property: string, time: number, fallback: number): number {
  const track = tracks.find((t) => t.property === property);
  return track ? interpolatedTrackValue(track, time, fallback) : fallback;
}

function interpolatedTrackValue(track: TimelineTrack, time: number, fallback: number): number {
  const keyframes = numericTrackKeyframes(track).sort((a, b) => a.time_ms - b.time_ms);
  if (!keyframes.length) return fallback;
  if (time <= keyframes[0].time_ms) return Number(keyframes[0].value.value);
  const last = keyframes[keyframes.length - 1];
  if (time >= last.time_ms) return Number(last.value.value);
  for (let i = 0; i < keyframes.length - 1; i += 1) {
    const left = keyframes[i];
    const right = keyframes[i + 1];
    if (time >= left.time_ms && time <= right.time_ms) {
      const span = Math.max(1, right.time_ms - left.time_ms);
      const progress = (time - left.time_ms) / span;
      return Number(left.value.value) + (Number(right.value.value) - Number(left.value.value)) * progress;
    }
  }
  return fallback;
}

function numericTrackKeyframes(track: TimelineTrack): NumericTimelineKeyframe[] {
  return track.keyframes.filter((kf): kf is NumericTimelineKeyframe => kf.value.type === "number");
}

function hasNumericMotion(track: TimelineTrack): boolean {
  return numericTrackKeyframes(track).length > 1;
}

function isTransformProperty(property: string): boolean {
  return ["translation.x", "translation.y", "rotation", "rotation.x", "rotation.y", "scale", "scale.x", "scale.y"].includes(property);
}

function isScalarProperty(property: string): boolean {
  return property === "opacity" || property === "frame";
}

function groupEasing(tracks: TimelineTrack[]): string {
  return cssEasing(tracks[0]?.keyframes[0]?.easing ?? "linear");
}

function cssEasing(easing: string): string {
  if (easing === "ease_in") return "ease-in";
  if (easing === "ease_out") return "ease-out";
  if (easing === "ease_in_out") return "ease-in-out";
  if (easing === "steps") return "step-end";
  return "linear";
}

export function nodeTransformMap(document: StrutDocument): Map<string, StrutNode["transform"]> {
  const transforms = new Map<string, StrutNode["transform"]>();
  const visit = (node: StrutNode) => {
    transforms.set(node.id, node.transform ?? {});
    for (const child of node.children ?? []) visit(child);
  };
  for (const artboard of document.artboards) {
    for (const node of artboard.nodes) visit(node);
  }
  return transforms;
}

function normalizeTransform(transform: StrutNode["transform"]): ResolvedTransform {
  return {
    translate_x: transform?.translate_x ?? 0,
    translate_y: transform?.translate_y ?? 0,
    rotate: transform?.rotate ?? 0,
    rotate_x: transform?.rotate_x ?? 0,
    rotate_y: transform?.rotate_y ?? 0,
    scale_x: transform?.scale_x ?? 1,
    scale_y: transform?.scale_y ?? 1,
  };
}

function transformCss(transform: ResolvedTransform): string {
  return `translate(${round(transform.translate_x)}px, ${round(transform.translate_y)}px) rotateZ(${round(transform.rotate)}deg) rotateX(${round(transform.rotate_x)}deg) rotateY(${round(transform.rotate_y)}deg) scale(${round(transform.scale_x)}, ${round(transform.scale_y)})`;
}

export function nodeShapeMap(document: StrutDocument): Map<string, StrutNode["shape"]> {
  const shapes = new Map<string, StrutNode["shape"]>();
  const visit = (node: StrutNode) => {
    shapes.set(node.id, node.shape ?? { type: "none" });
    for (const child of node.children ?? []) visit(child);
  };
  for (const artboard of document.artboards) {
    for (const node of artboard.nodes) visit(node);
  }
  return shapes;
}

function transformAnimationName(timeline: Timeline, target: string): string {
  return `studio-${cssIdent(timeline.name)}-${cssIdent(target)}-transform`;
}

function scalarAnimationName(timeline: Timeline, track: TimelineTrack): string {
  return `studio-${cssIdent(timeline.name)}-${cssIdent(track.target)}-${cssIdent(track.property)}`;
}

function round(value: number): number {
  return Number(value.toFixed(4));
}

export function cssIdent(value: string): string {
  return value.replace(/[^a-zA-Z0-9_-]/g, "-");
}
