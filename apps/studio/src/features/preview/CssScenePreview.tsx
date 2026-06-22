import type { StrutDocument } from "../../types";

export function CssScenePreview({ document }: { document: StrutDocument }) {
  const artboard = document.artboards[0];
  return <div className="css-scene-preview">{artboard?.name ?? document.name}</div>;
}
