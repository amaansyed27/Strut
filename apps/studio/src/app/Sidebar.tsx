/**
 * Sidebar component.
 *
 * Extracted from App.tsx — the full sidebar with brand, navigation,
 * project list, pinned items, context menus, and footer.
 */

import { useMemo } from "react";
import {
  ChevronRight,
  Folder,
  FolderOpen,
  FolderPlus,
  Home,
  MessageSquarePlus,
  MoreHorizontal,
  Pencil,
  Pin,
  Plus,
  Search,
  Settings2,
  Trash2,
} from "lucide-react";
import type { ProjectRecord, ChatThread, SidebarMenuState } from "../types";
import { relativeTimeLabel } from "../lib/documentUtils";

type SidebarProps = {
  projects: ProjectRecord[];
  /** @deprecated kept for compatibility; not currently used */
  activeChatId: string | null;
  collapsedProjectIds: Set<string>;
  sidebarMenu: SidebarMenuState;
  clockTick: number;
  onNavigateHome: () => void;
  onNewChat: (projectId?: string) => void;
  onOpenNewProject: () => void;
  onOpenSearch: () => void;
  onOpenSettings: () => void;
  onOpenProject: (projectId: string) => void;
  onOpenChat: (projectId: string, chatId: string) => void;
  onToggleProjectCollapsed: (projectId: string) => void;
  onToggleProjectPinned: (projectId: string) => void;
  onToggleChatPinned: (projectId: string, chatId: string) => void;
  onRenameProject: (projectId: string) => void;
  onRenameChat: (projectId: string, chatId: string) => void;
  onRemoveProject: (projectId: string) => void;
  onDeleteChat: (projectId: string, chatId: string) => void;
  onOpenProjectFolder: (project: ProjectRecord) => void;
  onSetSidebarMenu: (menu: SidebarMenuState) => void;
};

export function Sidebar({
  projects,
  activeChatId,
  collapsedProjectIds,
  sidebarMenu,
  clockTick,
  onNavigateHome,
  onNewChat,
  onOpenNewProject,
  onOpenSearch,
  onOpenSettings,
  onOpenProject,
  onOpenChat,
  onToggleProjectCollapsed,
  onToggleProjectPinned,
  onToggleChatPinned,
  onRenameProject,
  onRenameChat,
  onRemoveProject,
  onDeleteChat,
  onOpenProjectFolder,
  onSetSidebarMenu,
}: SidebarProps) {
  const pinnedProjects = useMemo(() => projects.filter((p) => p.pinned), [projects]);
  const pinnedChats = useMemo(
    () =>
      projects.flatMap((project) =>
        project.chats
          .filter((chat: ChatThread) => chat.pinned)
          .map((chat: ChatThread) => ({ project, chat })),
      ),
    [projects],
  );

  return (
    <aside className="sidebar">
      <div className="sidebar-brand">
        <img src="/strut-mark.svg" alt="" />
        <span>Strut</span>
      </div>

      <div className="sidebar-actions">
        <button type="button" onClick={onNavigateHome}>
          <Home size={16} />
          Home
        </button>
        <button type="button" onClick={() => onNewChat()}>
          <MessageSquarePlus size={16} />
          New chat
        </button>
        <button type="button" onClick={onOpenNewProject}>
          <FolderPlus size={16} />
          New project
        </button>
        <button type="button" onClick={onOpenSearch}>
          <Search size={16} />
          Search
        </button>
      </div>

      <div className="project-list">
        {pinnedProjects.length || pinnedChats.length ? (
          <div className="pinned-list">
            <span className="section-label">Pinned</span>
            {pinnedProjects.map((project) => (
              <button
                aria-label={`Pinned project ${project.name}`}
                className="pinned-row"
                key={`project-${project.id}`}
                type="button"
                onClick={() => onOpenProject(project.id)}
              >
                <Folder size={14} />
                <span>{project.name}</span>
              </button>
            ))}
            {pinnedChats.map(({ project, chat }) => (
              <button
                aria-label={`Pinned chat ${chat.title}`}
                className="pinned-row"
                key={`chat-${chat.id}`}
                type="button"
                onClick={() => onOpenChat(project.id, chat.id)}
              >
                <MessageSquarePlus size={14} />
                <span>{chat.title}</span>
              </button>
            ))}
          </div>
        ) : null}
        <span className="section-label">Projects</span>
        {projects.map((project) => {
          const isCollapsed = collapsedProjectIds.has(project.id);
          const projectMenuOpen = sidebarMenu?.kind === "project" && sidebarMenu.projectId === project.id;
          return (
            <div className="project-group" key={project.id}>
              <div
                className="project-button"
                onContextMenu={(event) => {
                  event.preventDefault();
                  onSetSidebarMenu({ kind: "project", projectId: project.id });
                }}
              >
                <button
                  aria-expanded={!isCollapsed}
                  className="project-open"
                  type="button"
                  onClick={() => {
                    onOpenProject(project.id);
                    onToggleProjectCollapsed(project.id);
                  }}
                >
                  <ChevronRight className={isCollapsed ? "" : "expanded"} size={14} />
                  <Folder size={15} />
                  <span>{project.name}</span>
                </button>
                <div className="project-actions">
                  <button
                    aria-label={`New chat in ${project.name}`}
                    className="inline-add"
                    type="button"
                    onClick={(event) => {
                      event.stopPropagation();
                      onNewChat(project.id);
                    }}
                  >
                    <Plus size={13} />
                  </button>
                  <button
                    aria-label={`Project options for ${project.name}`}
                    className="inline-menu"
                    type="button"
                    onClick={(event) => {
                      event.stopPropagation();
                      onSetSidebarMenu(projectMenuOpen ? null : { kind: "project", projectId: project.id });
                    }}
                  >
                    <MoreHorizontal size={14} />
                  </button>
                </div>
                {projectMenuOpen ? (
                  <div className="sidebar-menu" role="menu">
                    <button role="menuitem" type="button" onClick={() => onToggleProjectPinned(project.id)}>
                      <Pin size={14} />
                      {project.pinned ? "Unpin project" : "Pin project"}
                    </button>
                    <button role="menuitem" type="button" onClick={() => onOpenProjectFolder(project)}>
                      <FolderOpen size={14} />
                      Open in Explorer
                    </button>
                    <button role="menuitem" type="button" onClick={() => onRenameProject(project.id)}>
                      <Pencil size={14} />
                      Rename project
                    </button>
                    <button role="menuitem" type="button" onClick={() => onRemoveProject(project.id)}>
                      <Trash2 size={14} />
                      Delete project
                    </button>
                  </div>
                ) : null}
              </div>
              {!isCollapsed
                ? project.chats.map((chat: ChatThread) => {
                    const chatMenuOpen =
                      sidebarMenu?.kind === "chat" &&
                      sidebarMenu.projectId === project.id &&
                      sidebarMenu.chatId === chat.id;
                    return (
                      <div
                        className={chat.id === activeChatId ? "chat-row active" : "chat-row"}
                        key={chat.id}
                        onContextMenu={(event) => {
                          event.preventDefault();
                          onSetSidebarMenu({ kind: "chat", projectId: project.id, chatId: chat.id });
                        }}
                      >
                        <button className="chat-link" type="button" onClick={() => onOpenChat(project.id, chat.id)}>
                          <span>{chat.title}</span>
                          <em>{relativeTimeLabel(chat.updated, clockTick)}</em>
                        </button>
                        <button
                          aria-label={`Chat options for ${chat.title}`}
                          className="chat-menu-button"
                          type="button"
                          onClick={(event) => {
                            event.stopPropagation();
                            onSetSidebarMenu(
                              chatMenuOpen ? null : { kind: "chat", projectId: project.id, chatId: chat.id },
                            );
                          }}
                        >
                          <MoreHorizontal size={13} />
                        </button>
                        {chatMenuOpen ? (
                          <div className="sidebar-menu chat-menu" role="menu">
                            <button
                              role="menuitem"
                              type="button"
                              onClick={() => onToggleChatPinned(project.id, chat.id)}
                            >
                              <Pin size={14} />
                              {chat.pinned ? "Unpin chat" : "Pin chat"}
                            </button>
                            <button role="menuitem" type="button" onClick={() => onRenameChat(project.id, chat.id)}>
                              <Pencil size={14} />
                              Rename chat
                            </button>
                            <button role="menuitem" type="button" onClick={() => onDeleteChat(project.id, chat.id)}>
                              <Trash2 size={14} />
                              Delete chat
                            </button>
                          </div>
                        ) : null}
                      </div>
                    );
                  })
                : null}
            </div>
          );
        })}
      </div>

      <div className="sidebar-footer">
        <button type="button" onClick={onOpenSettings}>
          <Settings2 size={16} />
          Settings
        </button>
      </div>
    </aside>
  );
}
