/**
 * New Project Dialog component.
 *
 * Modal dialog for creating new Strut projects. Replaces the inline
 * project-sheet that was previously rendered inside the workspace layout.
 */

import { useState, useEffect } from "react";
import { FolderOpen } from "lucide-react";
import { Dialog, DialogBody, DialogFooter } from "../../components/ui/Dialog";
import { projectService } from "./projectService";

type NewProjectDialogProps = {
  open: boolean;
  onClose: () => void;
  desktopRuntime: boolean;
  defaultLocation: string;
  onProjectCreated: (info: { id: string; name: string; path: string }) => void;
};

export function NewProjectDialog({
  open,
  onClose,
  desktopRuntime,
  defaultLocation,
  onProjectCreated,
}: NewProjectDialogProps) {
  const [projectName, setProjectName] = useState("Untitled Strut Project");
  const [projectLocation, setProjectLocation] = useState(defaultLocation);
  const [error, setError] = useState("");
  const [isCreating, setIsCreating] = useState(false);
  const [isChoosingLocation, setIsChoosingLocation] = useState(false);

  // Reset state when dialog opens
  useEffect(() => {
    if (open) {
      setProjectName("Untitled Strut Project");
      setProjectLocation(defaultLocation);
      setError("");
      setIsCreating(false);
      setIsChoosingLocation(false);
    }
  }, [open, defaultLocation]);

  async function handleChooseLocation() {
    setError("");
    setIsChoosingLocation(true);
    try {
      const selected = await projectService.chooseProjectLocation(projectLocation);
      if (selected) setProjectLocation(selected);
    } catch (err) {
      setError(
        desktopRuntime
          ? err instanceof Error ? err.message : String(err)
          : "Folder picking is available in the desktop app. You can still type a location for browser preview.",
      );
    } finally {
      setIsChoosingLocation(false);
    }
  }

  async function handleCreate() {
    const trimmedName = projectName.trim();
    if (!trimmedName) {
      setError("Project name is required");
      return;
    }

    setError("");
    setIsCreating(true);

    if (!desktopRuntime) {
      const id = `project-${Date.now()}`;
      onProjectCreated({
        id,
        name: trimmedName,
        path: projectLocation,
      });
      onClose();
      return;
    }

    try {
      const created = await projectService.createProject(trimmedName, projectLocation);
      onProjectCreated({
        id: `project-${Date.now()}`,
        name: created.name,
        path: created.path,
      });
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsCreating(false);
    }
  }

  return (
    <Dialog
      open={open}
      onClose={onClose}
      size="md"
      title="New project"
      subtitle="Choose where Strut should create the editable scene files."
    >
      <DialogBody>
        <label>
          <span>Name</span>
          <input
            aria-label="Project name"
            autoFocus
            value={projectName}
            onChange={(event) => {
              setProjectName(event.currentTarget.value);
              if (error) setError("");
            }}
          />
        </label>
        <label>
          <span>Location</span>
          <div className="location-picker-row">
            <input
              aria-label="Project location"
              value={projectLocation}
              onChange={(event) => setProjectLocation(event.currentTarget.value)}
            />
            <button
              aria-label="Choose project location"
              className="location-picker-button"
              type="button"
              disabled={isChoosingLocation}
              onClick={() => void handleChooseLocation()}
            >
              <FolderOpen size={15} />
              {isChoosingLocation ? "Opening..." : "Choose"}
            </button>
          </div>
        </label>
        {error ? <div className="dialog-error">{error}</div> : null}
      </DialogBody>
      <DialogFooter>
        <button type="button" onClick={onClose}>Cancel</button>
        <button
          type="button"
          disabled={isCreating}
          onClick={() => void handleCreate()}
        >
          {isCreating ? "Creating..." : "Create project"}
        </button>
      </DialogFooter>
    </Dialog>
  );
}
