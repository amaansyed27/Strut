import { useState, useMemo } from "react";
import { Search, Folder, MessageSquarePlus } from "lucide-react";
import { Dialog, DialogBody } from "../../components/ui/Dialog";
import type { ProjectRecord, ChatThread } from "../../types";

/* ------------------------------------------------------------------ */
/*  Utility                                                            */
/* ------------------------------------------------------------------ */

function relativeTimeLabel(value: string, nowMs = Date.now()) {
  const parsed = Date.parse(value);
  if (!Number.isFinite(parsed)) return value === "now" ? "now" : value;
  const seconds = Math.max(0, Math.floor((nowMs - parsed) / 1000));
  if (seconds < 60) return "now";
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h`;
  const days = Math.floor(hours / 24);
  if (days < 7) return `${days}d`;
  const weeks = Math.floor(days / 7);
  if (weeks < 5) return `${weeks}w`;
  const months = Math.floor(days / 30);
  if (months < 12) return `${Math.max(1, months)}mo`;
  return `${Math.max(1, Math.floor(days / 365))}y`;
}

/* ------------------------------------------------------------------ */
/*  Types                                                              */
/* ------------------------------------------------------------------ */

type FilteredProject = {
  project: ProjectRecord;
  matchingChats: ChatThread[];
  nameMatches: boolean;
};

type SearchCommandModalProps = {
  open: boolean;
  onClose: () => void;
  projects: ProjectRecord[];
  clockTick: number;
  onOpenProject: (projectId: string) => void;
  onOpenChat: (projectId: string, chatId: string) => void;
};

/* ------------------------------------------------------------------ */
/*  Component                                                          */
/* ------------------------------------------------------------------ */

export function SearchCommandModal({
  open,
  onClose,
  projects,
  clockTick,
  onOpenProject,
  onOpenChat,
}: SearchCommandModalProps) {
  const [searchQuery, setSearchQuery] = useState("");

  const filteredProjects = useMemo<FilteredProject[]>(() => {
    const q = searchQuery.trim().toLowerCase();

    return projects.reduce<FilteredProject[]>((acc, project) => {
      const nameMatches = !q || project.name.toLowerCase().includes(q);
      const matchingChats = project.chats.filter(
        (chat) => !q || chat.title.toLowerCase().includes(q),
      );

      if (nameMatches || matchingChats.length > 0) {
        acc.push({ project, matchingChats, nameMatches });
      }

      return acc;
    }, []);
    // clockTick ensures stale relative-time labels are recalculated
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [projects, searchQuery, clockTick]);

  const hasResults = filteredProjects.length > 0;

  return (
    <Dialog
      open={open}
      onClose={onClose}
      size="lg"
      title="Search"
      subtitle="Find a project or chat"
    >
      <DialogBody>
        <div className="dialog-search-field">
          <Search size={16} aria-hidden />
          <input
            type="text"
            placeholder="Search projects and chats…"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            autoFocus
            aria-label="Search projects and chats"
          />
        </div>

        <div className="dialog-search-results" role="list">
          {hasResults ? (
            filteredProjects.map(({ project, matchingChats, nameMatches }) => (
              <div
                key={project.id}
                className="search-project"
                role="listitem"
              >
                {nameMatches && (
                  <button
                    type="button"
                    onClick={() => {
                      onOpenProject(project.id);
                      onClose();
                    }}
                  >
                    <Folder size={16} aria-hidden />
                    {project.name}
                  </button>
                )}

                {matchingChats.map((chat) => (
                  <button
                    key={chat.id}
                    type="button"
                    onClick={() => {
                      onOpenChat(project.id, chat.id);
                      onClose();
                    }}
                  >
                    <MessageSquarePlus size={16} aria-hidden />
                    <span>{chat.title}</span>
                    <span className="muted">
                      {relativeTimeLabel(chat.updated)}
                    </span>
                  </button>
                ))}
              </div>
            ))
          ) : (
            <p className="panel-empty">
              No projects or chats match this search.
            </p>
          )}
        </div>
      </DialogBody>
    </Dialog>
  );
}
