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
    <div className="html-component-preview" data-renderer={renderer}>
      {issues.length ? (
        <div className="html-component-preview-warnings" role="status">
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
        title={`${component.name} preview`}
      />
    </div>
  );
}
