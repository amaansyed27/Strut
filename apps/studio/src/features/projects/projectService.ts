/**
 * Project service — typed wrappers for project-related Tauri commands.
 */

import { tauriInvoke } from "../../lib/tauriClient";
import type {
  ProjectAnimationRecord,
  ProjectInfo,
  ProjectSnapshot,
  OperationBatch,
  StrutDocument,
  LayerUiState,
} from "../../types";

export type ExportResult = {
  success: boolean;
  outputDir: string;
  files: Array<{ name: string; path: string }>;
};

export const projectService = {
  async createProject(name: string, location: string): Promise<ProjectInfo> {
    return tauriInvoke<ProjectInfo>("create_project", { name, location });
  },

  async defaultProjectLocation(): Promise<string> {
    return tauriInvoke<string>("default_project_location");
  },

  async openProjectFolder(path: string): Promise<void> {
    await tauriInvoke<void>("open_project_folder", { path });
  },

  async saveProjectSnapshot(
    projectPath: string,
    projectName: string,
    document: StrutDocument,
    operationBatches: OperationBatch[],
    selection?: { activeState: string; selectedNodeId: string | null; layerUi: Record<string, LayerUiState> },
  ): Promise<ProjectSnapshot> {
    return tauriInvoke<ProjectSnapshot>("save_project_snapshot", {
      projectPath,
      projectName,
      document,
      operationBatches,
      selection,
    });
  },

  async loadProjectSnapshot(projectPath: string): Promise<ProjectSnapshot> {
    return tauriInvoke<ProjectSnapshot>("load_project_snapshot", { projectPath });
  },

  async saveProjectAnimation(
    projectPath: string,
    projectName: string,
    chatId: string,
    animationName: string,
    document: StrutDocument,
    operationBatches: OperationBatch[],
    selection?: { activeState: string; selectedNodeId: string | null; layerUi: Record<string, LayerUiState> },
  ): Promise<ProjectAnimationRecord> {
    return tauriInvoke<ProjectAnimationRecord>("save_project_animation", {
      projectPath,
      projectName,
      chatId,
      animationName,
      document,
      operationBatches,
      selection,
    });
  },

  async deleteProjectAnimation(projectPath: string, animationId: string): Promise<void> {
    await tauriInvoke<void>("delete_project_animation", { projectPath, animationId });
  },

  async exportAnimationToReact(
    projectPath: string,
    document: StrutDocument,
    animationName: string,
    outputDir?: string,
  ): Promise<ExportResult> {
    return tauriInvoke<ExportResult>("export_animation_to_react", {
      projectPath,
      document,
      animationName,
      outputDir: outputDir?.trim() ? outputDir.trim() : null,
    });
  },
};
