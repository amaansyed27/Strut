import { useMemo } from "react";
import { buildComponentPreviewHtml, verifyRuntimeComponent, type MotionRenderer, type RuntimeComponent } from "../../lib/motionArtifacts";

export function HtmlComponentPreview({
  component,
  renderer,
}: {
  component: RuntimeComponent;
  renderer: MotionRenderer;
}) {
  const srcDoc = useMemo(() => buildComponentPreviewHtml(component), [component]);
  const issues = useMemo(() => verifyRuntimeComponent(component, renderer), [component, renderer]);

  return (
    <div
      className="html-component-preview"
      data-renderer={renderer}
      style={{ display: "grid", gap: 10, height: "100%", minHeight: 320 }}
    >
      {issues.length ? (
        <div
          className="html-component-preview-warnings"
          role="status"
          style={{ border: "1px solid rgba(245, 158, 11, .45)", borderRadius: 12, padding: 12 }}
        >
          <strong>Verifier warnings</strong>
          <ul>
            {issues.map((issue) => (
              <li key={issue}>{issue}</li>
            ))}
          </ul>
        </div>
      ) : null}
      <iframe
        className="html-component-preview-frame"
        sandbox="allow-scripts"
        srcDoc={srcDoc}
        style={{ border: 0, borderRadius: 16, height: "100%", minHeight: 320, width: "100%" }}
        title={`${component.name} preview`}
      />
    </div>
  );
}
