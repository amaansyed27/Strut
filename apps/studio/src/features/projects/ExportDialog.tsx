/**
 * Export Animation Dialog component.
 *
 * Modal dialog for exporting Strut animations to various formats.
 * Currently supports React export with room for future formats.
 */

import { useState, useEffect } from "react";
import { Dialog, DialogBody, DialogFooter } from "../../components/ui/Dialog";
import type { StrutDocument } from "../../types";
import { FolderOpen } from "lucide-react";
import { projectService, type ExportResult } from "./projectService";

type ExportDialogProps = {
  open: boolean;
  onClose: () => void;
  desktopRuntime: boolean;
  projectPath: string;
  document: StrutDocument | null;
  animationName: string;
};

type ExportFormat = "react";

function exportSlug(value: string) {
  const slug = value.trim().toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "");
  return slug || "animation";
}

export function ExportDialog({
  open,
  onClose,
  desktopRuntime,
  projectPath,
  document,
  animationName,
}: ExportDialogProps) {
  const [selectedFormat, setSelectedFormat] = useState<ExportFormat>("react");
  const [outputDir, setOutputDir] = useState("");
  const [error, setError] = useState("");
  const [isExporting, setIsExporting] = useState(false);
  const [exportResult, setExportResult] = useState<ExportResult | null>(null);

  // Reset state when dialog opens
  useEffect(() => {
    if (open) {
      setSelectedFormat("react");
      const defaultDir = `exports/${exportSlug(animationName)}-react`;
      setOutputDir(defaultDir);
      setError("");
      setIsExporting(false);
      setExportResult(null);
    }
  }, [open, animationName]);

  async function handleExport() {
    if (!document) {
      setError("No document to export");
      return;
    }

    if (!desktopRuntime) {
      setError("Desktop app required for export functionality");
      return;
    }

    setError("");
    setIsExporting(true);

    try {
      const result = await projectService.exportAnimationToReact(projectPath, document, animationName, outputDir);

      if (result.success) {
        setExportResult(result);
      } else {
        setError("Export failed");
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsExporting(false);
    }
  }

  async function openExportFolder() {
    if (!exportResult) return;
    try {
      await projectService.openProjectFolder(exportResult.outputDir);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  function handleClose() {
    onClose();
    // Reset export result after animation completes
    setTimeout(() => setExportResult(null), 300);
  }

  return (
    <Dialog
      open={open}
      onClose={handleClose}
      size="md"
      title={exportResult ? "Export Complete" : "Export Animation"}
      subtitle={
        exportResult
          ? `Successfully exported to ${exportResult.outputDir}`
          : "Choose export format and output directory"
      }
    >
      {exportResult ? (
        <>
          <DialogBody>
            <div className="export-success">
              <div>
                <strong>Exported files</strong>
                <ul>
                  {exportResult.files.map((file) => (
                    <li key={file.path}>{file.name}</li>
                  ))}
                </ul>
              </div>
              <button
                type="button"
                onClick={() => void openExportFolder()}
              >
                <FolderOpen size={14} />
                Open Export Folder
              </button>
            </div>
          </DialogBody>
          <DialogFooter>
            <button type="button" onClick={handleClose}>
              Close
            </button>
          </DialogFooter>
        </>
      ) : (
        <>
          <DialogBody>
            <label>
              <span>Export Format</span>
              <div className="export-format-options" role="radiogroup" aria-label="Export format">
                <button
                  aria-checked={selectedFormat === "react"}
                  className={selectedFormat === "react" ? "active" : ""}
                  role="radio"
                  type="button"
                  onClick={() => setSelectedFormat("react")}
                >
                  React Component
                </button>
                <button aria-disabled="true" disabled role="radio" type="button">
                  More formats soon
                </button>
              </div>
            </label>
            <label>
              <span>Output Directory</span>
              <input
                aria-label="Output directory"
                value={outputDir}
                onChange={(event) => {
                  setOutputDir(event.currentTarget.value);
                  if (error) setError("");
                }}
                placeholder={`exports/${exportSlug(animationName)}-react`}
              />
              <em className="export-field-hint">Relative to project directory</em>
            </label>
            {error ? <div className="dialog-error">{error}</div> : null}
          </DialogBody>
          <DialogFooter>
            <button type="button" onClick={handleClose}>
              Cancel
            </button>
            <button type="button" disabled={isExporting || !document || !projectPath} onClick={() => void handleExport()}>
              {isExporting ? "Exporting..." : "Export"}
            </button>
          </DialogFooter>
        </>
      )}
    </Dialog>
  );
}
