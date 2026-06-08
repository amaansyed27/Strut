import scene from "./scene.json";

type StrutNode = {
  id: string;
  name: string;
  kind: string;
  role?: string;
  shape: { type: string; [key: string]: unknown };
  style: { fill?: string | null; stroke?: string | null; stroke_width?: number; opacity?: number };
  children?: StrutNode[];
};

function paint(value: string | null | undefined) {
  return value ?? "none";
}

function renderNode(node: StrutNode): React.ReactNode {
  const style = node.style ?? {};
  const common = {
    key: node.id,
    fill: paint(style.fill),
    stroke: paint(style.stroke),
    strokeWidth: style.stroke_width ?? 0,
    opacity: style.opacity ?? 1,
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
    "data-strut-node": node.name,
    "data-strut-role": node.role ?? "",
  };
  const shape = node.shape ?? { type: "none" };
  if (shape.type === "rect") {
    return <rect {...common} x={shape.x as number} y={shape.y as number} width={shape.width as number} height={shape.height as number} rx={shape.rx as number} />;
  }
  if (shape.type === "ellipse") {
    return <ellipse {...common} cx={shape.cx as number} cy={shape.cy as number} rx={shape.rx as number} ry={shape.ry as number} />;
  }
  if (shape.type === "path") {
    return <path {...common} d={shape.d as string} />;
  }
  if (shape.type === "text") {
    return <text {...common} x={shape.x as number} y={shape.y as number} fontSize={shape.size as number}>{shape.value as string}</text>;
  }
  return <g key={node.id}>{node.children?.map(renderNode)}</g>;
}

export function StrutAnimation({ state = "idle", title = "Helpful Mascot Motion" }: { state?: string; title?: string }) {
  const artboard = scene.artboards[0];
  return (
    <svg viewBox={`0 0 ${artboard.width} ${artboard.height}`} role="img" aria-label={title} data-strut-state={state}>
      {artboard.nodes.map(renderNode)}
    </svg>
  );
}

export default StrutAnimation;
