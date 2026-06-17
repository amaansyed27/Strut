/**
 * Workspace top bar.
 *
 * Extracted from App.tsx — title, activity pill, view switcher, and context menus.
 */

import {
  PanelRight,
  MoreHorizontal,
  Pencil,
  Pin,
  Trash2,
  FolderOpen,
  Download,
} from "lucide-react";
import type { ChatThread, ProjectRecord, ViewMode, SidebarMenuState } from "../types";

type WorkspaceTopbarProps = {
  activeProject: ProjectRecord | null;
  activeChat: ChatThread | null;
  workspaceTitle?: string;
  viewMode: ViewMode;
  showViewSwitcher?: boolean;
  activity: string;
  topbarMenu: SidebarMenuState;
  onSetViewMode: (mode: ViewMode) => void;
  onSetTopbarMenu: (menu: SidebarMenuState) => void;
  onToggleChatPinned: (projectId: string, chatId: string) => void;
  onRenameChat: (projectId: string, chatId: string) => void;
  onDeleteChat: (projectId: string, chatId: string) => void;
  onToggleProjectPinned: (projectId: string) => void;
  onRenameProject: (projectId: string) => void;
  onRemoveProject: (projectId: string) => void;
  onOpenProjectFolder: (project: ProjectRecord) => void;
  onSetMainPanel: (panel: "chat") => void;
  onOpenExportDialog?: () => void;
};

export function WorkspaceTopbar({
  activeProject,
  activeChat,
  workspaceTitle,
  viewMode,
  showViewSwitcher = true,
  activity,
  topbarMenu,
  onSetViewMode,
  onSetTopbarMenu,
  onToggleChatPinned,
  onRenameChat,
  onDeleteChat,
  onToggleProjectPinned,
  onRenameProject,
  onRemoveProject,
  onOpenProjectFolder,
  onSetMainPanel,
  onOpenExportDialog,
}: WorkspaceTopbarProps) {
  return (
    <header className="workspace-top">
      <div className="workspace-context">
        <strong data-testid="workspace-title">
          {workspaceTitle ?? activeChat?.title ?? activeProject?.name ?? "Home"}
        </strong>
        {!workspaceTitle && activeChat && activeProject ? (
          <button
            aria-label={`Title options for ${activeChat.title}`}
            className="title-menu-button"
            type="button"
            onClick={() =>
              onSetTopbarMenu(
                topbarMenu?.kind === "chat" && topbarMenu.chatId === activeChat.id
                  ? null
                  : { kind: "chat", projectId: activeProject.id, chatId: activeChat.id },
              )
            }
          >
            <MoreHorizontal size={15} />
          </button>
        ) : !workspaceTitle && activeProject ? (
          <button
            aria-label={`Title options for ${activeProject.name}`}
            className="title-menu-button"
            type="button"
            onClick={() =>
              onSetTopbarMenu(
                topbarMenu?.kind === "project" && topbarMenu.projectId === activeProject.id
                  ? null
                  : { kind: "project", projectId: activeProject.id },
              )
            }
          >
            <MoreHorizontal size={15} />
          </button>
        ) : null}
        {activeChat && activeProject && topbarMenu?.kind === "chat" && topbarMenu.chatId === activeChat.id ? (
          <div className="topbar-menu" role="menu">
            <button role="menuitem" type="button" onClick={() => onToggleChatPinned(activeProject.id, activeChat.id)}>
              <Pin size={14} />
              {activeChat.pinned ? "Unpin chat" : "Pin chat"}
            </button>
            <button role="menuitem" type="button" onClick={() => onRenameChat(activeProject.id, activeChat.id)}>
              <Pencil size={14} />
              Rename chat
            </button>
            <button role="menuitem" type="button" onClick={() => onDeleteChat(activeProject.id, activeChat.id)}>
              <Trash2 size={14} />
              Delete chat
            </button>
          </div>
        ) : activeProject && topbarMenu?.kind === "project" && topbarMenu.projectId === activeProject.id ? (
          <div className="topbar-menu" role="menu">
            <button role="menuitem" type="button" onClick={() => onToggleProjectPinned(activeProject.id)}>
              <Pin size={14} />
              {activeProject.pinned ? "Unpin project" : "Pin project"}
            </button>
            <button role="menuitem" type="button" onClick={() => onOpenProjectFolder(activeProject)}>
              <FolderOpen size={14} />
              Open in Explorer
            </button>
            {onOpenExportDialog && (
              <button role="menuitem" type="button" onClick={() => {
                onOpenExportDialog();
                onSetTopbarMenu(null);
              }}>
                <Download size={14} />
                Export animation
              </button>
            )}
            <button role="menuitem" type="button" onClick={() => onRenameProject(activeProject.id)}>
              <Pencil size={14} />
              Rename project
            </button>
            <button role="menuitem" type="button" onClick={() => onRemoveProject(activeProject.id)}>
              <Trash2 size={14} />
              Delete project
            </button>
          </div>
        ) : null}
        {activeProject && onOpenExportDialog ? (
          <button
            aria-label="Export current animation"
            className="topbar-export-button"
            title="Export current animation"
            type="button"
            onClick={onOpenExportDialog}
          >
            <Download size={15} />
            <span>Export</span>
          </button>
        ) : null}
        <span className="sr-status" data-testid="activity-pill">{activity}</span>
      </div>
      {showViewSwitcher ? (
        <nav className="view-switcher" aria-label="View mode">
          <button
            aria-pressed={viewMode === "preview"}
            className={viewMode === "preview" ? "active" : ""}
            type="button"
            onClick={() => {
              onSetViewMode(viewMode === "preview" ? "chat" : "preview");
              onSetMainPanel("chat");
            }}
          >
            <PanelRight size={15} />
            Preview
          </button>
        </nav>
      ) : null}
    </header>
  );
}
