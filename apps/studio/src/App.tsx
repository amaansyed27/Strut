/**
 * Strut Studio — Main App Component
 *
 * Refactored from a ~3,156 line monolith into a slim composition root
 * that imports types, services, utilities, and feature components from
 * dedicated modules.
 *
 * This file retains:
 * - State management (projects, active IDs, provider config, theme)
 * - Effects (initialization, storage persistence, theme)
 * - Chat/generation flow orchestration
 * - Inline rendering components (ChatMessageView, MarkdownResponse, etc.)
 * - SVG rendering components (CharacterPreview, StrutNodePreview, etc.)
 * - HomePanel, LayerRail, PreviewPane
 */

import { Fragment, useEffect, useMemo, useRef, useState, type CSSProperties, type MouseEvent, type ReactNode } from "react";
import {
  Cpu,
  Edit3,
  Film,
  FolderOpen,
  FolderPlus,
  ImagePlus,
  Layers3,
  MessageSquarePlus,
  MoreHorizontal,
  PanelRight,
  RefreshCw,
  RotateCcw,
  RotateCw,
  Route,
  Save,
  Send,
  Trash2,
  X,
} from "lucide-react";
import "./App.css";

// ── Types ──────────────────────────────────────────────────────────────────
import type {
  ChatMessage,
  ChatThread,
  GenerationContext,
  GenerationProvider,
  LayerUiState,
  LocalAdapter,
  MainPanel,
  ProjectRecord,
  ProviderMode,
  ReferenceAttachment,
  RunState,
  SidebarMenuState,
  ProjectAnimationRecord,
  StrutDocument,
  StrutNode,
  ThemeMode,
  ViewMode,
  ViewModeOption,
} from "./types";
import { browserLocalAdapters, byokProviders, emptyArtboard, emptyMachine } from "./types";

// ── Libraries ──────────────────────────────────────────────────────────────
import { cssIdent, stateAnimationDuration, stateAnimationLoops, stateNodeOverrides, type StateNodeOverride } from "./lib/animationCss";
import {
  createChat,
  createGenerationBatch,
  documentRevisionId,
  documentSummary,
  fileToAttachment,
  firstAvailableState,
  flattenNodes,
  layerReferencePrompt,
  layerToAttachment,
  layerUiFor,
  localChatFallback,
  nowStamp,
  promptTitle,
  titleCase,
  uniqueAttachments,
  validateOperationBatch,
  applyOperationBatch,
  latestPreviewForProject,
} from "./lib/documentUtils";
import { loadWorkspaceState, saveWorkspaceState } from "./lib/storage";

// ── Services ───────────────────────────────────────────────────────────────
import { generationService } from "./features/chat/generationService";
import { projectService } from "./features/projects/projectService";
import {
  animationToAttachment,
  createLocalProjectAnimationRecord,
  findProjectAnimationForDocument,
  projectAnimationLibrary,
  removeProjectAnimation,
  upsertProjectAnimation,
} from "./features/projects/projectAnimations";
import { providerService } from "./features/providers/providerService";

// ── Hooks ──────────────────────────────────────────────────────────────────
import { useHotkeys } from "./hooks/useHotkeys";
import { useDisclosure } from "./hooks/useDisclosure";

// ── Components ─────────────────────────────────────────────────────────────
import { Sidebar } from "./app/Sidebar";
import { WorkspaceTopbar } from "./app/WorkspaceTopbar";
import { SearchCommandModal } from "./features/search/SearchCommandModal";
import { NewProjectDialog } from "./features/projects/NewProjectDialog";
import { ProvidersPage } from "./features/providers/ProvidersPage";
import { SettingsPage } from "./features/settings/SettingsPage";

/* ────────────────────────────────────────────────────────────────────────── */
/*  App Component                                                            */
/* ────────────────────────────────────────────────────────────────────────── */

function App() {
  // ── State ──────────────────────────────────────────────────────────────
  const [initialWorkspace] = useState(() => loadWorkspaceState());
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const [desktopRuntime, setDesktopRuntime] = useState(true);
  const [projects, setProjects] = useState<ProjectRecord[]>(initialWorkspace.projects);
  const [activeProjectId, setActiveProjectId] = useState<string | null>(initialWorkspace.activeProjectId);
  const [activeChatId, setActiveChatId] = useState<string | null>(initialWorkspace.activeChatId);
  const [mainPanel, setMainPanel] = useState<MainPanel>("chat");
  const [viewMode, setViewMode] = useState<ViewMode>("chat");
  const [defaultLocation, setDefaultLocation] = useState("");
  const [prompt, setPrompt] = useState("");
  const [pendingReferences, setPendingReferences] = useState<ReferenceAttachment[]>([]);
  const [providerMode, setProviderMode] = useState<ProviderMode>("local");
  const [localAdapters, setLocalAdapters] = useState<LocalAdapter[]>(browserLocalAdapters);
  const [selectedLocalAdapterId, setSelectedLocalAdapterId] = useState("codex");
  const [selectedByokProviderId, setSelectedByokProviderId] = useState("openai");
  const [apiKey, setApiKey] = useState("");
  const [providerEndpoint, setProviderEndpoint] = useState(byokProviders[0].endpoint);
  const [providerModel, setProviderModel] = useState(byokProviders[0].model);
  const [activity, setActivity] = useState("Select a real local CLI, Ollama, or BYOK provider");
  const [runState, setRunState] = useState<RunState>("idle");
  const [themeMode, setThemeMode] = useState<ThemeMode>(initialWorkspace.themeMode);
  const [collapsedProjectIds, setCollapsedProjectIds] = useState<Set<string>>(() => new Set());
  const [clockTick, setClockTick] = useState(Date.now());
  const [sidebarMenu, setSidebarMenu] = useState<SidebarMenuState>(null);
  const [topbarMenu, setTopbarMenu] = useState<SidebarMenuState>(null);
  const [composerToolsOpen, setComposerToolsOpen] = useState(true);
  const [layersRailCollapsed, setLayersRailCollapsed] = useState(false);
  const [previewRefreshSignal, setPreviewRefreshSignal] = useState(0);
  const [slashMenuOpen, setSlashMenuOpen] = useState(false);
  const [selectedStrategy, setSelectedStrategy] = useState<"auto" | "svg" | "sprite" | "dynamic">("auto");

  // ── Disclosure state for modals ──────────────────────────────────────
  const searchModal = useDisclosure();
  const newProjectModal = useDisclosure();

  // ── Effects ────────────────────────────────────────────────────────────
  useEffect(() => {
    generationService.studioStatus()
      .then(() => setDesktopRuntime(true))
      .catch(() => setDesktopRuntime(false));
    projectService.defaultProjectLocation()
      .then(setDefaultLocation)
      .catch(() => {
        setDesktopRuntime(false);
        setDefaultLocation("D:\\Strut Projects");
      });
    providerService.listLocalAdapters()
      .then(setLocalAdapters)
      .catch(() => setDesktopRuntime(false));
  }, []);

  useEffect(() => {
    const timer = window.setInterval(() => setClockTick(Date.now()), 30_000);
    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => {
    if (localAdapters.some((adapter) => adapter.id === selectedLocalAdapterId)) return;
    const preferred = localAdapters.find((adapter) => adapter.id === "codex") ?? localAdapters[0];
    if (preferred) setSelectedLocalAdapterId(preferred.id);
  }, [localAdapters, selectedLocalAdapterId]);

  useEffect(() => {
    saveWorkspaceState({ projects, activeProjectId, activeChatId, themeMode });
  }, [projects, activeProjectId, activeChatId, themeMode]);

  useEffect(() => {
    if (typeof window !== "undefined") {
      window.document.documentElement.dataset.theme = themeMode;
    }
  }, [themeMode]);

  // ── Keyboard shortcuts ───────────────────────────────────────────────
  useHotkeys(useMemo(() => ({
    "ctrl+k": () => searchModal.open(),
    "meta+k": () => searchModal.open(),
  }), [searchModal.open]));

  // ── Derived state ────────────────────────────────────────────────────
  const activeProject = projects.find((project) => project.id === activeProjectId) ?? null;
  const activeChat = activeProject?.chats.find((chat) => chat.id === activeChatId) ?? null;
  const projectPreview = latestPreviewForProject(activeProject, activeChatId);
  const currentDocument = projectPreview?.document ?? null;
  const projectAnimations = useMemo(() => projectAnimationLibrary(activeProject), [activeProject]);
  const currentAnimation = findProjectAnimationForDocument(projectAnimations, currentDocument, projectPreview?.chatId);
  const currentActiveState = projectPreview?.activeState || firstAvailableState(currentDocument);
  const activeArtboard = currentDocument?.artboards[0] ?? emptyArtboard;
  const activeMachine = currentDocument?.state_machines[0] ?? emptyMachine;
  const layers = useMemo(() => flattenNodes(activeArtboard.nodes), [activeArtboard.nodes]);
  const selectedNodeId = activeChat?.selectedNodeId ?? null;
  const layerUi = activeChat?.layerUi ?? {};
  const operationBatches = activeChat?.operationBatches ?? activeChat?.operationHistory ?? [];
  const persistedLayerReferences = activeChat?.references.filter((ref) => ref.kind === "layer") ?? [];
  const composerReferences = uniqueAttachments([...persistedLayerReferences, ...pendingReferences]);
  const undoStack = activeChat?.undoStack ?? [];
  const redoStack = activeChat?.redoStack ?? [];
  const selectedLayer = layers.find((layer) => layer.id === selectedNodeId) ?? null;
  const activeLocalAdapter = localAdapters.find((adapter) => adapter.id === selectedLocalAdapterId) ?? localAdapters[0] ?? browserLocalAdapters[0];
  const activeByokProvider = byokProviders.find((provider) => provider.id === selectedByokProviderId) ?? byokProviders[0];
  const activeProviderLabel = providerMode === "local" ? activeLocalAdapter.name : activeByokProvider.name;
  const viewModes: ViewModeOption[] = [
    { id: "chat", Icon: MessageSquarePlus, label: "Chat only" },
    { id: "preview", Icon: PanelRight, label: "Chat + preview" },
  ];

  // Clear stale selection
  useEffect(() => {
    if (selectedNodeId && !layers.some((layer) => layer.id === selectedNodeId)) {
      setSelectedNode(null);
    }
  }, [layers, selectedNodeId]);

  // ── State mutation helpers ───────────────────────────────────────────
  function updateChat(projectId: string, chatId: string, updater: (chat: ChatThread) => ChatThread) {
    setProjects((current) =>
      current.map((project) =>
        project.id === projectId
          ? { ...project, chats: project.chats.map((chat) => (chat.id === chatId ? updater(chat) : chat)) }
          : project,
      ),
    );
  }

  function updateCurrentChat(updater: (chat: ChatThread) => ChatThread) {
    if (!activeProjectId || !activeChatId) return;
    updateChat(activeProjectId, activeChatId, updater);
  }

  function appendMessage(role: ChatMessage["role"], text: string, operationBatchId?: string) {
    updateCurrentChat((chat) => ({
      ...chat,
      updated: nowStamp(),
      messages: [...chat.messages, { id: Date.now() + Math.random(), role, text, operationBatchId }],
    }));
  }

  function appendUserMessage(text: string, attachments: ReferenceAttachment[]) {
    updateCurrentChat((chat) => ({
      ...chat,
      updated: nowStamp(),
      references: uniqueAttachments([...chat.references, ...attachments]),
      messages: [...chat.messages, { id: Date.now() + Math.random(), role: "user", text, attachments }],
    }));
  }

  function setSelectedNode(nodeId: string | null) {
    updateCurrentChat((chat) => ({ ...chat, selectedNodeId: nodeId, updated: nowStamp() }));
  }

  function setCurrentActiveState(state: string) {
    if (!activeProjectId) return;
    const ownerChatId = projectPreview?.chatId ?? activeChatId;
    if (!ownerChatId) return;
    updateChat(activeProjectId, ownerChatId, (chat) => ({ ...chat, activeState: state, updated: nowStamp() }));
  }

  // ── Navigation ───────────────────────────────────────────────────────
  function openChat(projectId: string, chatId: string) {
    setActiveProjectId(projectId);
    setActiveChatId(chatId);
    setMainPanel("chat");
  }

  function openProject(projectId: string) {
    const project = projects.find((item) => item.id === projectId);
    setActiveProjectId(projectId);
    setActiveChatId(project?.chats[0]?.id ?? null);
    setMainPanel("chat");
  }

  function newChat(projectId = activeProjectId ?? projects[0]?.id ?? null) {
    const project = projects.find((item) => item.id === projectId);
    if (!project) {
      newProjectModal.open();
      return;
    }
    const chat = createChat(project.id, "New motion chat");
    setProjects((current) =>
      current.map((item) => (item.id === project.id ? { ...item, chats: [chat, ...item.chats] } : item)),
    );
    setActiveProjectId(project.id);
    setActiveChatId(chat.id);
    setMainPanel("chat");
  }

  function deleteChat(projectId: string, chatId: string) {
    setProjects((current) =>
      current.map((project) =>
        project.id === projectId
          ? { ...project, chats: project.chats.filter((chat) => chat.id !== chatId) }
          : project,
      ),
    );
    if (activeProjectId === projectId && activeChatId === chatId) setActiveChatId(null);
    setSidebarMenu(null);
    setTopbarMenu(null);
  }

  function removeProject(projectId: string) {
    setProjects((current) => current.filter((project) => project.id !== projectId));
    if (activeProjectId === projectId) {
      setActiveProjectId(null);
      setActiveChatId(null);
    }
    setSidebarMenu(null);
    setTopbarMenu(null);
  }

  function toggleProjectCollapsed(projectId: string) {
    setCollapsedProjectIds((current) => {
      const next = new Set(current);
      if (next.has(projectId)) next.delete(projectId);
      else next.add(projectId);
      return next;
    });
  }

  function toggleProjectPinned(projectId: string) {
    setProjects((current) =>
      current.map((project) => (project.id === projectId ? { ...project, pinned: !project.pinned } : project)),
    );
    setSidebarMenu(null);
    setTopbarMenu(null);
  }

  function toggleChatPinned(projectId: string, chatId: string) {
    updateChat(projectId, chatId, (chat) => ({ ...chat, pinned: !chat.pinned, updated: nowStamp() }));
    setSidebarMenu(null);
    setTopbarMenu(null);
  }

  function renameProject(projectId: string) {
    const project = projects.find((item) => item.id === projectId);
    if (!project) return;
    const nextName = window.prompt("Rename project", project.name)?.trim();
    if (!nextName || nextName === project.name) {
      setSidebarMenu(null);
      setTopbarMenu(null);
      return;
    }
    setProjects((current) => current.map((item) => (item.id === projectId ? { ...item, name: nextName } : item)));
    setSidebarMenu(null);
    setTopbarMenu(null);
  }

  function renameChat(projectId: string, chatId: string) {
    const chat = projects.find((project) => project.id === projectId)?.chats.find((item) => item.id === chatId);
    if (!chat) return;
    const nextTitle = window.prompt("Rename chat", chat.title)?.trim();
    if (!nextTitle || nextTitle === chat.title) {
      setSidebarMenu(null);
      setTopbarMenu(null);
      return;
    }
    updateChat(projectId, chatId, (item) => ({ ...item, title: nextTitle, updated: nowStamp() }));
    setSidebarMenu(null);
    setTopbarMenu(null);
  }

  // ── Provider payload ─────────────────────────────────────────────────
  function providerPayload(): GenerationProvider {
    if (providerMode === "local") {
      return { mode: "local", localAdapterId: activeLocalAdapter.id };
    }
    return {
      mode: "byok",
      byok: {
        providerId: selectedByokProviderId,
        apiKey: apiKey.trim() || undefined,
        endpoint: providerEndpoint.trim(),
        model: providerModel.trim(),
      },
    };
  }

  function generationContext(): GenerationContext {
    const activeMessages = activeChat?.messages ?? [];
    const projectMessages = activeProject?.chats
      .filter((chat) => chat.id !== activeChatId)
      .flatMap((chat) => chat.messages.slice(-3).map((message) => ({ chatTitle: chat.title, message }))) ?? [];
    const chatHistory = [
      ...activeMessages.slice(-10).map((message) => ({
        role: message.role,
        text: message.text,
        attachments: message.attachments?.map((attachment) => attachment.name),
      })),
      ...projectMessages.slice(0, 6).map(({ chatTitle, message }) => ({
        role: message.role,
        text: `[${chatTitle}] ${message.text}`,
        attachments: message.attachments?.map((attachment) => attachment.name),
      })),
    ].filter((message) => message.text.trim() || (message.attachments?.length ?? 0) > 0);
    return {
      projectName: activeProject?.name,
      projectPath: activeProject?.path,
      activeChatTitle: activeChat?.title,
      currentDocumentSummary: selectedLayer
        ? `${documentSummary(currentDocument)}; selected part: ${selectedLayer.name} (${selectedLayer.id}, ${selectedLayer.kind})`
        : documentSummary(currentDocument),
      chatHistory,
      currentDocument: currentDocument ?? undefined,
    };
  }

  // ── Provider actions ─────────────────────────────────────────────────
  async function saveProvider() {
    if (providerMode !== "byok") {
      setActivity("Select BYOK first");
      return;
    }
    if (!desktopRuntime) {
      setActivity("Desktop app required for provider config");
      return;
    }
    try {
      const result = await providerService.saveByokProvider({
        providerId: selectedByokProviderId,
        apiKey: apiKey.trim() || undefined,
        endpoint: providerEndpoint.trim(),
        model: providerModel.trim(),
      });
      setActivity(result.status);
    } catch (error) {
      setActivity(String(error));
    }
  }

  async function testProvider() {
    if (!desktopRuntime) {
      setActivity("Desktop app required for real provider checks");
      return;
    }
    try {
      const result =
        providerMode === "local"
          ? await providerService.testLocalAdapter(activeLocalAdapter.id)
          : await providerService.testByokProvider({
              providerId: selectedByokProviderId,
              apiKey: apiKey.trim() || undefined,
              endpoint: providerEndpoint.trim(),
              model: providerModel.trim(),
            });
      setActivity(result.status);
    } catch (error) {
      setActivity(String(error));
    }
  }

  async function openProjectFolder(project = activeProject) {
    if (!project) {
      setActivity("Select a project first");
      return;
    }
    if (!desktopRuntime) {
      setActivity("Desktop app required to open project folder");
      setSidebarMenu(null);
      setTopbarMenu(null);
      return;
    }
    try {
      await projectService.openProjectFolder(project.path);
      setActivity(`Opened ${project.name}`);
    } catch (error) {
      setActivity(String(error));
    } finally {
      setSidebarMenu(null);
      setTopbarMenu(null);
    }
  }

  // ── Project persistence ──────────────────────────────────────────────
  async function saveActiveProject() {
    if (!activeProject || !currentDocument) {
      setActivity("Open a validated scene before saving");
      return;
    }
    if (!desktopRuntime) {
      window.localStorage.setItem(
        "strut-studio-saved-project-v1",
        JSON.stringify({ projects, activeProjectId, activeChatId, themeMode, savedAt: nowStamp() }),
      );
      setActivity(`Saved browser snapshot for ${activeProject.name}`);
      return;
    }
    try {
      const snapshot = await projectService.saveProjectSnapshot(
        activeProject.path,
        activeProject.name,
        currentDocument,
        operationBatches,
        { activeState: currentActiveState, selectedNodeId, layerUi },
      );
      setProjects((current) =>
        current.map((project) =>
          project.id === activeProject.id
            ? { ...project, name: snapshot.project.name, path: snapshot.project.path, animations: snapshot.animations ?? project.animations ?? [] }
            : project,
        ),
      );
      setActivity(`Saved ${snapshot.mainScene} with ${snapshot.operationBatches.length} operation batches`);
    } catch (error) {
      setActivity(`Save rejected: ${String(error)}`);
    }
  }

  async function loadActiveProject() {
    if (!activeProject) {
      setActivity("Select a project before loading");
      return;
    }
    if (!desktopRuntime) {
      const raw = window.localStorage.getItem("strut-studio-saved-project-v1");
      if (!raw) {
        setActivity("No browser snapshot has been saved yet");
        return;
      }
      try {
        const parsed = JSON.parse(raw);
        setProjects(parsed.projects ?? projects);
        setActiveProjectId(parsed.activeProjectId ?? activeProjectId);
        setActiveChatId(parsed.activeChatId ?? activeChatId);
        setThemeMode(parsed.themeMode ?? themeMode);
        setActivity("Reopened browser snapshot");
      } catch (error) {
        setActivity(`Browser snapshot rejected: ${String(error)}`);
      }
      return;
    }
    try {
      const snapshot = await projectService.loadProjectSnapshot(activeProject.path);
      const activeId = activeChatId ?? `chat-${Date.now()}`;
      const loadedChat: ChatThread = {
        ...(activeChat ?? createChat(activeProject.id, "Loaded scene")),
        id: activeId,
        title: activeChat?.title ?? "Loaded scene",
        projectId: activeProject.id,
        updated: nowStamp(),
        document: snapshot.document,
        activeState: snapshot.selection?.activeState ?? firstAvailableState(snapshot.document),
        selectedNodeId: snapshot.selection?.selectedNodeId ?? null,
        layerUi: snapshot.selection?.layerUi ?? {},
        pendingOperation: null,
        operationBatches: snapshot.operationBatches,
        operationHistory: snapshot.operationBatches,
        undoStack: snapshot.operationBatches.filter((batch) => batch.status === "applied").map((batch) => batch.id),
        redoStack: [],
      };
      setProjects((current) =>
        current.map((project) =>
          project.id === activeProject.id
            ? {
                ...project,
                name: snapshot.project.name,
                path: snapshot.project.path,
                animations: snapshot.animations ?? project.animations ?? [],
                chats: project.chats.some((chat) => chat.id === activeId)
                  ? project.chats.map((chat) => (chat.id === activeId ? loadedChat : chat))
                  : [loadedChat, ...project.chats],
              }
            : project,
        ),
      );
      setActiveChatId(activeId);
      setActivity(`Loaded ${snapshot.mainScene} with ${snapshot.operationBatches.length} operation batches`);
    } catch (error) {
      setActivity(`Load rejected: ${String(error)}`);
    }
  }

  // ── References ───────────────────────────────────────────────────────
  function removePendingReference(id: string) {
    setPendingReferences((current) => current.filter((reference) => reference.id !== id));
    updateCurrentChat((chat) => ({
      ...chat,
      updated: nowStamp(),
      references: chat.references.filter((reference) => reference.id !== id),
    }));
  }

  function attachLayerReference(layer: StrutNode) {
    const attachment = layerToAttachment(layer);
    const isAlreadyAttached = composerReferences.some((ref) => ref.kind === "layer" && ref.nodeId === layer.id);
    if (isAlreadyAttached) {
      removePendingReference(attachment.id);
      setSelectedNode(selectedNodeId === layer.id ? null : selectedNodeId);
      setActivity(`Removed layer ${layer.name} from the next prompt`);
      return;
    }
    setSelectedNode(layer.id);
    setPendingReferences((current) =>
      current.some((ref) => ref.kind === "layer" && ref.nodeId === layer.id)
        ? current
        : [...current, attachment],
    );
    updateCurrentChat((chat) => ({
      ...chat,
      updated: nowStamp(),
      references: uniqueAttachments([...chat.references, attachment]),
    }));
    setActivity(`Attached layer ${layer.name} to the next prompt`);
  }

  async function attachReferenceImages(files: FileList | null) {
    if (!files || files.length === 0) return;
    const imageFiles = Array.from(files).filter(
      (file) => file.type.startsWith("image/") || file.name.toLowerCase().endsWith(".svg"),
    );
    if (imageFiles.length === 0) {
      setActivity("Choose PNG, JPG, WebP, GIF, or SVG references");
      return;
    }
    try {
      const attachments = await Promise.all(imageFiles.slice(0, 6).map(fileToAttachment));
      setPendingReferences((current) => [...current, ...attachments]);
      setActivity(`${attachments.length} reference image${attachments.length === 1 ? "" : "s"} attached`);
    } catch (error) {
      setActivity(String(error));
    } finally {
      if (fileInputRef.current) fileInputRef.current.value = "";
    }
  }

  // ── Undo / Redo ──────────────────────────────────────────────────────
  function refreshCurrentAnimation() {
    if (!currentDocument) {
      setActivity("Generate or select an animation before refreshing preview state");
      return;
    }
    setCurrentActiveState(currentActiveState || firstAvailableState(currentDocument));
    setPreviewRefreshSignal((value) => value + 1);
    setActivity(`Restarted ${titleCase(currentActiveState || "idle")}`);
  }

  function openProjectAnimation(animation: ProjectAnimationRecord) {
    if (!activeProjectId) return;
    const targetChatId = animation.chatId ?? activeChatId ?? `chat-${Date.now()}`;
    const activeState = animation.selection?.activeState || firstAvailableState(animation.document);
    setProjects((current) =>
      current.map((project) => {
        if (project.id !== activeProjectId) return project;
        const existingChat = project.chats.find((chat) => chat.id === targetChatId);
        const nextChat: ChatThread = {
          ...(existingChat ?? createChat(project.id, animation.name)),
          id: targetChatId,
          title: existingChat?.title ?? animation.name,
          projectId: project.id,
          document: animation.document,
          activeState,
          selectedNodeId: animation.selection?.selectedNodeId ?? null,
          layerUi: animation.selection?.layerUi ?? {},
          operationBatches: animation.operationBatches,
          operationHistory: animation.operationBatches,
          updated: nowStamp(),
        };
        return {
          ...project,
          chats: existingChat
            ? project.chats.map((chat) => (chat.id === targetChatId ? nextChat : chat))
            : [nextChat, ...project.chats],
        };
      }),
    );
    setActiveChatId(targetChatId);
    setPreviewRefreshSignal((value) => value + 1);
    setActivity(`Opened animation ${animation.name}`);
  }

  function attachAnimationReference(animation: ProjectAnimationRecord | null = currentAnimation) {
    if (!animation) {
      setActivity("Select an animation before attaching it to the chat");
      return;
    }
    const attachment = animationToAttachment(animation);
    const exists = composerReferences.some((ref) => ref.kind === "animation" && ref.animationId === animation.id);
    if (exists) {
      removePendingReference(attachment.id);
      setActivity(`Removed animation ${animation.name} from the next prompt`);
      return;
    }
    setPendingReferences((current) =>
      current.some((ref) => ref.kind === "animation" && ref.animationId === animation.id)
        ? current
        : [...current, attachment],
    );
    updateCurrentChat((chat) => ({
      ...chat,
      updated: nowStamp(),
      references: uniqueAttachments([...chat.references, attachment]),
    }));
    setActivity(`Attached animation ${animation.name} to the next prompt`);
  }

  async function deleteProjectAnimation(animation: ProjectAnimationRecord | null = currentAnimation) {
    if (!activeProject || !animation) {
      setActivity("Select a saved animation before deleting");
      return;
    }
    const confirmed = window.confirm(`Delete "${animation.name}" from this project? This removes the project animation file and clears it from preview.`);
    if (!confirmed) return;
    if (desktopRuntime && animation.scene) {
      try {
        await projectService.deleteProjectAnimation(activeProject.path, animation.id);
      } catch (error) {
        setActivity(`Delete rejected: ${String(error)}`);
        return;
      }
    }
    setProjects((current) =>
      current.map((project) => (project.id === activeProject.id ? removeProjectAnimation(project, animation.id) : project)),
    );
    if (activeChatId === animation.chatId) {
      setSelectedNode(null);
    }
    setActivity(`Deleted animation ${animation.name}`);
  }

  function undoLastBatch() {
    const batchId = undoStack[0];
    const batch = operationBatches.find((item) => item.id === batchId);
    if (!batch || batch.status !== "applied") {
      setActivity("No applied operation batch is available to undo");
      return;
    }
    const nextDocument = applyOperationBatch(currentDocument, batch, "undo");
    if (!nextDocument) {
      setActivity("Undo could not run because no document is open");
      return;
    }
    const timestamp = nowStamp();
    const undone = { ...batch, status: "undone" as const, updatedAt: timestamp };
    updateCurrentChat((chat) => ({
      ...chat,
      document: nextDocument,
      operationBatches: [undone, ...(chat.operationBatches ?? []).filter((item) => item.id !== batch.id)],
      operationHistory: [undone, ...(chat.operationBatches ?? []).filter((item) => item.id !== batch.id)].slice(0, 12),
      undoStack: (chat.undoStack ?? []).filter((id) => id !== batch.id),
      redoStack: [batch.id, ...(chat.redoStack ?? [])],
      updated: nowStamp(),
    }));
    setActivity(`Undid batch ${batch.id}`);
  }

  function redoLastBatch() {
    const batchId = redoStack[0];
    const batch = operationBatches.find((item) => item.id === batchId);
    if (!batch) {
      setActivity("No operation batch is available to redo");
      return;
    }
    const reapplied = { ...batch, status: "applied" as const, validationResult: validateOperationBatch(currentDocument, batch) };
    if (!reapplied.validationResult.ok) {
      setActivity(reapplied.validationResult.message);
      return;
    }
    const nextDocument = applyOperationBatch(currentDocument, reapplied, "apply");
    if (!nextDocument) {
      setActivity("Redo could not run because no document is open");
      return;
    }
    const timestamp = nowStamp();
    const updated = { ...reapplied, documentRevisionId: documentRevisionId(nextDocument), updatedAt: timestamp, appliedAt: timestamp };
    updateCurrentChat((chat) => ({
      ...chat,
      document: nextDocument,
      operationBatches: [updated, ...(chat.operationBatches ?? []).filter((item) => item.id !== batch.id)],
      operationHistory: [updated, ...(chat.operationBatches ?? []).filter((item) => item.id !== batch.id)].slice(0, 12),
      undoStack: [batch.id, ...(chat.undoStack ?? [])],
      redoStack: (chat.redoStack ?? []).filter((id) => id !== batch.id),
      updated: nowStamp(),
    }));
    setActivity(`Redid batch ${batch.id}`);
  }

  // ── Project creation callback ────────────────────────────────────────
  function handleProjectCreated(info: { id: string; name: string; path: string }) {
    const chatTitle = desktopRuntime ? `Project created: ${info.path}` : "Browser preview opened an in-memory project.";
    const chat = createChat(info.id, "Project brief", [
      { id: Date.now(), role: "system", text: chatTitle },
    ]);
    const project: ProjectRecord = {
      id: info.id,
      name: info.name,
      path: info.path,
      chats: [chat],
    };
    setProjects((current) => [project, ...current]);
    setActiveProjectId(info.id);
    setActiveChatId(chat.id);
    setActivity(desktopRuntime ? `Project created at ${info.path}` : "Browser preview project. Disk was not written.");
  }

  // ── Smart message routing (chat vs. generate) ─────────────────────────
  async function sendMessage() {
    const trimmed = prompt.trim();
    if (!trimmed && composerReferences.length === 0) return;
    if (runState !== "idle") return;
    if (!activeProjectId || !activeChatId) {
      newChat();
      setActivity("Start a chat first");
      return;
    }
    const references = composerReferences;
    
    // Apply manual strategy override if selected
    let finalPrompt = trimmed;
    if (selectedStrategy !== "auto") {
      const strategyHint = selectedStrategy === "svg" 
        ? " [use svg vector style]"
        : selectedStrategy === "sprite"
        ? " [use sprite style]"
        : " [use dynamic style]";
      finalPrompt = `${trimmed}${strategyHint}`;
    }
    
    const combinedPrompt = `${finalPrompt}${layerReferencePrompt(references)}`;

    appendUserMessage(trimmed || "Use the attached reference image.", references);
    updateChat(activeProjectId, activeChatId, (chat) => ({
      ...chat,
      title: chat.title === "New motion chat" || chat.title === "New character chat" || chat.title === "Project brief"
        ? promptTitle(trimmed || references[0]?.name || "Reference motion")
        : chat.title,
      updated: nowStamp(),
    }));
    setPrompt("");
    setPendingReferences([]);
    setActivity("Thinking");
    setRunState("thinking");

    try {
      if (!desktopRuntime) {
        appendMessage("assistant", localChatFallback(trimmed));
        setActivity("Answered in chat mode");
        return;
      }
      
      const result = await generationService.assistantMessage(combinedPrompt, providerPayload(), references, generationContext());
      
      if (result.kind === "chat") {
        appendMessage("assistant", result.message || localChatFallback(trimmed));
        setActivity(`Answered through ${result.source}`);
      } else {
        // Document created or updated
        const generationBatch = createGenerationBatch(result as any, currentDocument, combinedPrompt, "ai");
        const initialState = result.activeState || firstAvailableState(result.document);
        const nextOperationBatches = [generationBatch, ...(activeChat?.operationBatches ?? [])];
        let animationRecord = createLocalProjectAnimationRecord(
          activeChatId,
          result.document,
          [generationBatch],
          initialState,
          activeChat?.layerUi ?? {},
        );
        let saveNote = "";
        if (desktopRuntime && activeProject) {
          try {
            animationRecord = await projectService.saveProjectAnimation(
              activeProject.path,
              activeProject.name,
              activeChatId,
              result.document.name,
              result.document,
              [generationBatch],
              { activeState: initialState, selectedNodeId, layerUi },
            );
          } catch (error) {
            saveNote = `\n\nProject file save warning: ${String(error)}`;
          }
        }

        setProjects((current) =>
          current.map((project) => {
            if (project.id !== activeProjectId) return project;
            const withAnimation = upsertProjectAnimation(project, animationRecord);
            return {
              ...withAnimation,
              chats: withAnimation.chats.map((chat) =>
                chat.id === activeChatId
                  ? {
                      ...chat,
                      updated: nowStamp(),
                      document: result.document,
                      activeState: initialState,
                      operationBatches: nextOperationBatches,
                      operationHistory: nextOperationBatches.slice(0, 12),
                      undoStack: [generationBatch.id, ...(chat.undoStack ?? [])],
                      redoStack: [],
                    }
                  : chat,
              ),
            };
          }),
        );
        
        const generatedPartSummary = result.planSummary?.partNames.length
          ? result.planSummary.partNames.slice(0, 6).join(", ")
          : "validated document layers";
        const generatedTimelineSummary = result.planSummary?.timelineNames.length
          ? result.planSummary.timelineNames.join(", ")
          : result.document.timelines.map((timeline) => timeline.name).join(", ");
          
        setActivity(`${result.source}: ${result.message}${saveNote ? " (project file save needs attention)" : ""}`);
        appendMessage(
          "assistant",
          `**${result.document.name} is ready.**\n\nProvider: ${activeProviderLabel}\n\nSubject: ${result.planSummary?.subjectLabel ?? "validated Strut document"}\n\nParts: ${generatedPartSummary}\n\nTimelines: ${generatedTimelineSummary}\n\n${result.message}${saveNote}`,
          generationBatch.id,
        );
      }
    } catch (error) {
      setActivity(String(error));
      appendMessage("assistant", `**Generation stopped**\n\nProvider: ${activeProviderLabel}\n\n${String(error)}`);
    } finally {
      setRunState("idle");
    }
  }

  // ── Render ───────────────────────────────────────────────────────────
  return (
    <main className="strut-shell">
      <Sidebar
        projects={projects}
        activeChatId={activeChatId}
        collapsedProjectIds={collapsedProjectIds}
        sidebarMenu={sidebarMenu}
        clockTick={clockTick}
        onNavigateHome={() => {
          setActiveProjectId(null);
          setActiveChatId(null);
          setMainPanel("chat");
        }}
        onNewChat={newChat}
        onOpenNewProject={newProjectModal.open}
        onOpenSearch={searchModal.open}
        onOpenProviders={() => setMainPanel("providers")}
        onOpenSettings={() => setMainPanel("settings")}
        onOpenProject={openProject}
        onOpenChat={openChat}
        onToggleProjectCollapsed={toggleProjectCollapsed}
        onToggleProjectPinned={toggleProjectPinned}
        onToggleChatPinned={toggleChatPinned}
        onRenameProject={renameProject}
        onRenameChat={renameChat}
        onRemoveProject={removeProject}
        onDeleteChat={deleteChat}
        onOpenProjectFolder={(project) => void openProjectFolder(project)}
        onSetSidebarMenu={setSidebarMenu}
      />

      <section className="workspace">
        <WorkspaceTopbar
          activeProject={activeProject}
          activeChat={activeChat}
          viewMode={viewMode}
          viewModes={viewModes}
          activity={activity}
          topbarMenu={topbarMenu}
          onSetViewMode={setViewMode}
          onSetTopbarMenu={setTopbarMenu}
          onToggleChatPinned={toggleChatPinned}
          onRenameChat={renameChat}
          onDeleteChat={deleteChat}
          onToggleProjectPinned={toggleProjectPinned}
          onRenameProject={renameProject}
          onRemoveProject={removeProject}
          onOpenProjectFolder={(project) => void openProjectFolder(project)}
          onSetMainPanel={setMainPanel}
        />

        {/* Search Command Palette */}
        <SearchCommandModal
          open={searchModal.isOpen}
          onClose={searchModal.close}
          projects={projects}
          clockTick={clockTick}
          onOpenProject={(projectId) => { openProject(projectId); searchModal.close(); }}
          onOpenChat={(projectId, chatId) => { openChat(projectId, chatId); searchModal.close(); }}
        />

        {/* New Project Dialog */}
        <NewProjectDialog
          open={newProjectModal.isOpen}
          onClose={newProjectModal.close}
          desktopRuntime={desktopRuntime}
          defaultLocation={defaultLocation}
          onProjectCreated={handleProjectCreated}
        />

        {/* Providers Page */}
        {mainPanel === "providers" ? (
          <ProvidersPage
            providerMode={providerMode}
            setProviderMode={setProviderMode}
            localAdapters={localAdapters}
            selectedLocalAdapterId={selectedLocalAdapterId}
            setSelectedLocalAdapterId={setSelectedLocalAdapterId}
            selectedByokProviderId={selectedByokProviderId}
            setSelectedByokProviderId={setSelectedByokProviderId}
            apiKey={apiKey}
            setApiKey={setApiKey}
            providerEndpoint={providerEndpoint}
            setProviderEndpoint={setProviderEndpoint}
            providerModel={providerModel}
            setProviderModel={setProviderModel}
            onSaveProvider={() => void saveProvider()}
            onTestProvider={() => void testProvider()}
            activity={activity}
            desktopRuntime={desktopRuntime}
          />
        ) : null}

        {/* Settings Page */}
        {mainPanel === "settings" ? (
          <SettingsPage themeMode={themeMode} setThemeMode={setThemeMode} />
        ) : null}

        {/* Home Panel */}
        {mainPanel === "chat" && !activeChat ? (
          <HomePanel
            projects={projects}
            onNewProject={newProjectModal.open}
            onOpenProviders={() => setMainPanel("providers")}
            onStartChat={() => newChat(projects[0]?.id ?? null)}
          />
        ) : null}

        {/* Chat Layout */}
        {mainPanel === "chat" && activeChat ? (
          <section className={viewMode === "preview" ? `chat-layout with-preview ${layersRailCollapsed ? "layers-collapsed" : ""}` : "chat-layout"}>
            <div className="chat-panel">
              <div className="message-stack">
                {activeChat.messages.length === 0 ? (
                  <div className="home-heading">
                    <h1>What motion should Strut build?</h1>
                    <p>Animate a logo, SVG, loader, product state, storyboard, mascot, or full scene.</p>
                  </div>
                ) : null}
                {activeChat.messages.map((message) => <ChatMessageView key={message.id} message={message} />)}
              </div>
              <div className="composer">
                {composerReferences.length ? (
                  <div className="reference-tray">
                    {composerReferences.map((reference) => (
                      <div className={`reference-chip ${reference.kind === "layer" ? "layer-reference-chip" : reference.kind === "animation" ? "animation-reference-chip" : ""}`} key={reference.id}>
                        {reference.kind === "layer" ? <Layers3 size={14} /> : reference.kind === "animation" ? <Film size={14} /> : <img src={reference.dataUrl} alt="" />}
                        <span>{reference.kind === "layer" ? `Layer: ${reference.name}` : reference.kind === "animation" ? `Animation: ${reference.name}` : reference.name}</span>
                        <button aria-label={`Remove reference ${reference.name}`} type="button" onClick={() => removePendingReference(reference.id)}>
                          <X size={13} />
                        </button>
                      </div>
                    ))}
                  </div>
                ) : null}
                <div className="prompt-examples" aria-label="Prompt examples">
                  {["Quiet loader", "Soft logo", "Button state", "Calm mascot", "State badge", "Tiny success"].map((example) => (
                    <button key={example} type="button" onClick={() => setPrompt((current) => current || `Make a ${example.toLowerCase()} as an editable Strut animation`)}>
                      {example}
                    </button>
                  ))}
                </div>
                <div className="composer-toolbar" aria-label="Composer tools">
                  <button aria-expanded={composerToolsOpen} type="button" onClick={() => setComposerToolsOpen((isOpen) => !isOpen)}>
                    <MoreHorizontal size={15} />
                    Tools
                  </button>
                  {composerToolsOpen ? (
                    <div className="composer-tool-actions">
                      <button aria-label="Refresh preview state" disabled={!currentDocument} title="Refresh preview state" type="button" onClick={refreshCurrentAnimation}>
                        <RefreshCw size={15} />
                      </button>
                      <button aria-label="Save project" disabled={!activeProject || !currentDocument} title="Save project" type="button" onClick={() => void saveActiveProject()}>
                        <Save size={15} />
                      </button>
                      <button aria-label="Load project from disk" disabled={!activeProject} title="Load project from disk" type="button" onClick={() => void loadActiveProject()}>
                        <FolderOpen size={15} />
                      </button>
                      <button aria-label="Undo" disabled={!undoStack.length} title="Undo" type="button" onClick={undoLastBatch}>
                        <RotateCcw size={15} />
                      </button>
                      <button aria-label="Redo" disabled={!redoStack.length} title="Redo" type="button" onClick={redoLastBatch}>
                        <RotateCw size={15} />
                      </button>
                      <button aria-label={`Provider ${activeProviderLabel}`} className="provider-composer-button" type="button" onClick={() => setMainPanel("providers")}>
                        <Cpu size={15} />
                        {activeProviderLabel}
                      </button>
                    </div>
                  ) : null}
                  {runState !== "idle" ? (
                    <div className="generation-loader" role="status" aria-live="polite">
                      <span aria-hidden="true" />
                      <strong>{activeProviderLabel}</strong>
                      <em>{runState === "thinking" ? "thinking" : "generating"}</em>
                    </div>
                  ) : null}
                </div>
                <div className="composer-input-wrapper">
                  <textarea 
                    aria-label="Motion prompt" 
                    value={prompt} 
                    onChange={(event) => {
                      const value = event.currentTarget.value;
                      setPrompt(value);
                      // Open slash menu when user types "/"
                      if (value === "/" || (value.length > 1 && value[value.length - 2] === " " && value[value.length - 1] === "/")) {
                        setSlashMenuOpen(true);
                      } else if (!value.endsWith("/")) {
                        setSlashMenuOpen(false);
                      }
                    }}
                    onKeyDown={(event) => {
                      if (event.key === "Enter" && !event.shiftKey) {
                        event.preventDefault();
                        void sendMessage();
                      } else if (event.key === "Escape" && slashMenuOpen) {
                        setSlashMenuOpen(false);
                      }
                    }}
                    placeholder="Ask Strut for calm, low-energy motion for a logo, SVG, UI state, icon, mascot, storyboard, or scene. Type / for options" 
                  />
                  {slashMenuOpen && (
                    <div className="slash-menu">
                      <button
                        type="button"
                        className={selectedStrategy === "auto" ? "selected" : ""}
                        onClick={() => {
                          setSelectedStrategy("auto");
                          setSlashMenuOpen(false);
                          setPrompt(prompt.replace(/\/$/, ""));
                        }}
                      >
                        <span className="slash-command">/auto</span>
                        <span className="slash-description">Let AI choose the best style</span>
                      </button>
                      <button
                        type="button"
                        className={selectedStrategy === "svg" ? "selected" : ""}
                        onClick={() => {
                          setSelectedStrategy("svg");
                          setSlashMenuOpen(false);
                          setPrompt(prompt.replace(/\/$/, ""));
                        }}
                      >
                        <span className="slash-command">/svg</span>
                        <span className="slash-description">Simple SVG vector style (logos, icons, loaders)</span>
                      </button>
                      <button
                        type="button"
                        className={selectedStrategy === "sprite" ? "selected" : ""}
                        onClick={() => {
                          setSelectedStrategy("sprite");
                          setSlashMenuOpen(false);
                          setPrompt(prompt.replace(/\/$/, ""));
                        }}
                      >
                        <span className="slash-command">/sprite</span>
                        <span className="slash-description">Complex sprite style (mascots, characters)</span>
                      </button>
                      <button
                        type="button"
                        className={selectedStrategy === "dynamic" ? "selected" : ""}
                        onClick={() => {
                          setSelectedStrategy("dynamic");
                          setSlashMenuOpen(false);
                          setPrompt(prompt.replace(/\/$/, ""));
                        }}
                      >
                        <span className="slash-command">/dynamic</span>
                        <span className="slash-description">Dynamic provider plan (flexible approach)</span>
                      </button>
                    </div>
                  )}
                </div>
                <div className="composer-strategy-indicator" title={`Animation strategy: ${selectedStrategy}`}>
                  <span>{selectedStrategy === "auto" ? "🎯 Auto" : selectedStrategy === "svg" ? "🎨 SVG" : selectedStrategy === "sprite" ? "🎮 Sprite" : "⚡ Dynamic"}</span>
                </div>
                <div className="composer-controls">
                  <div className="composer-left">
                    <input
                      ref={fileInputRef}
                      aria-label="Attach reference images"
                      className="reference-input"
                      type="file"
                      accept="image/png,image/jpeg,image/webp,image/gif,image/svg+xml"
                      multiple
                      onChange={(event) => void attachReferenceImages(event.currentTarget.files)}
                    />
                    <button aria-label="Attach reference images" type="button" onClick={() => fileInputRef.current?.click()}>
                      <ImagePlus size={16} />
                      Reference
                    </button>
                  </div>
                  <button aria-label="Generate" disabled={runState !== "idle"} type="button" onClick={() => void sendMessage()}>
                    <Send size={17} />
                  </button>
                </div>
              </div>
            </div>
            {viewMode === "preview" ? (
              <div className="preview-area">
                <PreviewPane
                  activeAnimationId={currentAnimation?.id ?? null}
                  activeMachine={activeMachine}
                  activeState={currentActiveState}
                  document={currentDocument}
                  onDeleteAnimation={() => void deleteProjectAnimation()}
                  onOpenAnimation={openProjectAnimation}
                  onReferenceAnimation={() => attachAnimationReference()}
                  projectAnimations={projectAnimations}
                  refreshSignal={previewRefreshSignal}
                  setActiveState={setCurrentActiveState}
                />
                <LayerRail
                  collapsed={layersRailCollapsed}
                  layers={layers}
                  onAttachLayer={attachLayerReference}
                  onToggleCollapsed={() => setLayersRailCollapsed((isCollapsed) => !isCollapsed)}
                  pendingReferences={composerReferences}
                  selectedNodeId={selectedNodeId}
                />
              </div>
            ) : null}
          </section>
        ) : null}
      </section>
    </main>
  );
}

/* ────────────────────────────────────────────────────────────────────────── */
/*  Inline rendering components                                              */
/* ────────────────────────────────────────────────────────────────────────── */

function renderInlineMarkdown(value: string): ReactNode[] {
  const nodes: ReactNode[] = [];
  const pattern = /(`[^`]+`|\*\*[^*]+\*\*|\*[^*]+\*)/g;
  let cursor = 0;
  let match: RegExpExecArray | null;
  while ((match = pattern.exec(value)) !== null) {
    if (match.index > cursor) nodes.push(value.slice(cursor, match.index));
    const token = match[0];
    if (token.startsWith("`")) nodes.push(<code key={`${match.index}-code`}>{token.slice(1, -1)}</code>);
    else if (token.startsWith("**")) nodes.push(<strong key={`${match.index}-strong`}>{token.slice(2, -2)}</strong>);
    else nodes.push(<em key={`${match.index}-em`}>{token.slice(1, -1)}</em>);
    cursor = pattern.lastIndex;
  }
  if (cursor < value.length) nodes.push(value.slice(cursor));
  return nodes;
}

function MarkdownResponse({ text }: { text: string }) {
  const blocks = text.split(/\n{2,}/).map((block) => block.trim()).filter(Boolean);
  if (blocks.length === 0) return null;
  return (
    <div className="markdown-response">
      {blocks.map((block, blockIndex) => {
        const lines = block.split(/\n/).map((line) => line.trim()).filter(Boolean);
        if (lines.length > 0 && lines.every((line) => line.startsWith("- "))) {
          return (
            <ul key={`list-${blockIndex}`}>
              {lines.map((line, lineIndex) => (
                <li key={`${blockIndex}-${lineIndex}`}>{renderInlineMarkdown(line.slice(2))}</li>
              ))}
            </ul>
          );
        }
        return (
          <p key={`paragraph-${blockIndex}`}>
            {lines.map((line, lineIndex) => (
              <Fragment key={`${blockIndex}-${lineIndex}`}>
                {lineIndex > 0 ? <br /> : null}
                {renderInlineMarkdown(line)}
              </Fragment>
            ))}
          </p>
        );
      })}
    </div>
  );
}

function ChatMessageView({ compact = false, message }: { compact?: boolean; message: ChatMessage }) {
  const roleLabel = message.role === "assistant" ? "Strut" : titleCase(message.role);
  return (
    <div className={`message ${compact ? "compact-message" : ""} ${message.role}`}>
      <span className="message-role">{roleLabel}</span>
      <div className="message-body">
        {message.role === "user" ? <span className="message-text">{message.text}</span> : <MarkdownResponse text={message.text} />}
        {message.operationBatchId ? <span className="message-batch-link">Batch {message.operationBatchId}</span> : null}
        {message.attachments?.length ? (
          <span className="message-attachments">
            {message.attachments.map((attachment) => (
              <span className={`message-attachment ${attachment.kind === "layer" ? "layer-attachment" : attachment.kind === "animation" ? "animation-attachment" : ""}`} key={attachment.id}>
                {attachment.kind === "layer" ? <Layers3 size={13} /> : attachment.kind === "animation" ? <Film size={13} /> : <img src={attachment.dataUrl} alt="" />}
                <em>{attachment.kind === "layer" ? `Layer: ${attachment.name}` : attachment.kind === "animation" ? `Animation: ${attachment.name}` : attachment.name}</em>
              </span>
            ))}
          </span>
        ) : null}
      </div>
    </div>
  );
}

/* ────────────────────────────────────────────────────────────────────────── */
/*  SVG rendering components                                                 */
/* ────────────────────────────────────────────────────────────────────────── */

function CharacterPreview({
  activeState,
  document,
  layerUi,
  onSelectNode,
  playSignal = 0,
  selectedNodeId,
}: {
  activeState: string;
  document: StrutDocument;
  layerUi?: Record<string, LayerUiState>;
  onSelectNode?: (nodeId: string) => void;
  playSignal?: number;
  selectedNodeId?: string | null;
}) {
  const artboard = document.artboards[0] ?? emptyArtboard;
  const width = artboard.width || 960;
  const height = artboard.height || 540;
  const [elapsedMs, setElapsedMs] = useState(0);
  useEffect(() => {
    const duration = stateAnimationDuration(document, activeState);
    const loops = stateAnimationLoops(document, activeState);
    if (duration <= 0) {
      setElapsedMs(0);
      return;
    }
    let frame = 0;
    let start = performance.now();
    setElapsedMs(0);
    const tick = (now: number) => {
      const elapsed = now - start;
      setElapsedMs(elapsed);
      if (loops || elapsed < duration) {
        frame = window.requestAnimationFrame(tick);
      }
    };
    frame = window.requestAnimationFrame(tick);
    return () => window.cancelAnimationFrame(frame);
  }, [activeState, document, playSignal]);
  const nodeOverrides = stateNodeOverrides(document, activeState, elapsedMs);
  return (
    <svg className="character-preview" data-character={artboard.name} data-state={activeState} data-testid="character-preview" viewBox={`0 0 ${width} ${height}`} role="img">
      <rect className="preview-bg" width={width} height={height} rx="18" />
      <g className={`document-scene state-${cssIdent(activeState)}`}>
        {artboard.nodes.map((node) => (
          <StrutNodePreview key={node.id} layerUi={layerUi ?? {}} node={node} nodeOverrides={nodeOverrides} onSelectNode={onSelectNode} selectedNodeId={selectedNodeId} />
        ))}
      </g>
      <text className="state-label" x={width / 2} y={height - 24} textAnchor="middle">{titleCase(activeState || "none")}</text>
    </svg>
  );
}

function StrutNodePreview({
  layerUi: layerUiMap,
  node,
  nodeOverrides,
  onSelectNode,
  selectedNodeId,
}: {
  layerUi: Record<string, LayerUiState>;
  node: StrutNode;
  nodeOverrides: Map<string, StateNodeOverride>;
  onSelectNode?: (nodeId: string) => void;
  selectedNodeId?: string | null;
}) {
  const ui = layerUiFor(layerUiMap, node.id);
  const selected = selectedNodeId === node.id;
  if (!ui.visible) return null;
  const override = nodeOverrides.get(node.id);
  const style = nodeStyle(node.style, override);
  const common = {
    "data-node-id": node.id,
    "data-node-name": node.name,
    "data-selected": selected ? "true" : undefined,
    "data-locked": ui.locked ? "true" : undefined,
    className: `strut-node selectable-node node-${cssIdent(node.name)} kind-${cssIdent(node.kind)} ${selected ? "selected" : ""} ${ui.locked ? "locked" : ""}`,
    transform: override?.transform ? undefined : transformAttribute(node.transform),
    style,
    onClick: (event: MouseEvent<SVGGElement>) => {
      event.stopPropagation();
      if (!ui.locked) onSelectNode?.(node.id);
    },
  };
  const children = node.children?.map((child) => (
    <StrutNodePreview key={child.id} layerUi={layerUiMap} node={child} nodeOverrides={nodeOverrides} onSelectNode={onSelectNode} selectedNodeId={selectedNodeId} />
  ));
  return (
    <g {...common}>
      <StrutShape node={node} />
      {selected ? <SelectionHalo node={node} /> : null}
      {children}
    </g>
  );
}

function StrutShape({ node }: { node: StrutNode }) {
  const shape = node.shape ?? { type: "none" };
  if (node.kind === "group" || shape.type === "none") return null;
  if (shape.type === "rect") return <rect x={shape.x} y={shape.y} width={shape.width} height={shape.height} rx={shape.rx} />;
  if (shape.type === "ellipse") return <ellipse cx={shape.cx} cy={shape.cy} rx={shape.rx} ry={shape.ry} />;
  if (shape.type === "path") return <path d={shape.d} />;
  if (shape.type === "text") return <text x={shape.x} y={shape.y} fontSize={shape.size}>{shape.value}</text>;
  return null;
}

function SelectionHalo({ node }: { node: StrutNode }) {
  const shape = node.shape ?? { type: "none" };
  if (shape.type === "rect") return <rect className="node-selection-halo" x={shape.x - 6} y={shape.y - 6} width={shape.width + 12} height={shape.height + 12} rx={Math.max(shape.rx, 8)} />;
  if (shape.type === "ellipse") return <ellipse className="node-selection-halo" cx={shape.cx} cy={shape.cy} rx={shape.rx + 7} ry={shape.ry + 7} />;
  if (shape.type === "path") return <path className="node-selection-halo" d={shape.d} />;
  if (shape.type === "text") return <rect className="node-selection-halo" x={shape.x - 8} y={shape.y - shape.size - 7} width={Math.max(shape.value.length * shape.size * 0.58, 24) + 16} height={shape.size + 14} rx={7} />;
  const bounds = nodeBounds(node);
  if (!bounds) return null;
  return <rect className="node-selection-halo group-halo" x={bounds.x - 10} y={bounds.y - 10} width={bounds.width + 20} height={bounds.height + 20} rx={12} />;
}

function nodeBounds(node: StrutNode): { x: number; y: number; width: number; height: number } | null {
  const shape = node.shape ?? { type: "none" };
  if (shape.type === "rect") return { x: shape.x, y: shape.y, width: shape.width, height: shape.height };
  if (shape.type === "ellipse") return { x: shape.cx - shape.rx, y: shape.cy - shape.ry, width: shape.rx * 2, height: shape.ry * 2 };
  if (shape.type === "text") return { x: shape.x, y: shape.y - shape.size, width: Math.max(shape.value.length * shape.size * 0.58, 24), height: shape.size };
  const childBounds = (node.children ?? []).map(nodeBounds).filter((bounds): bounds is NonNullable<ReturnType<typeof nodeBounds>> => Boolean(bounds));
  if (!childBounds.length) return null;
  const minX = Math.min(...childBounds.map((b) => b.x));
  const minY = Math.min(...childBounds.map((b) => b.y));
  const maxX = Math.max(...childBounds.map((b) => b.x + b.width));
  const maxY = Math.max(...childBounds.map((b) => b.y + b.height));
  return { x: minX, y: minY, width: maxX - minX, height: maxY - minY };
}

function nodeStyle(style: StrutNode["style"], override?: StateNodeOverride): CSSProperties {
  return {
    fill: style?.fill ?? undefined,
    stroke: style?.stroke ?? undefined,
    strokeWidth: style?.stroke_width,
    opacity: override?.opacity ?? style?.opacity,
    transform: override?.transform,
    transformBox: override?.transform ? "fill-box" : undefined,
    transformOrigin: override?.transform ? "center" : undefined,
    strokeLinecap: style?.linecap as CSSProperties["strokeLinecap"],
    strokeLinejoin: style?.linejoin as CSSProperties["strokeLinejoin"],
  };
}

function transformAttribute(transform: StrutNode["transform"]) {
  if (!transform) return undefined;
  const parts = [];
  if (transform.translate_x || transform.translate_y) parts.push(`translate(${transform.translate_x ?? 0} ${transform.translate_y ?? 0})`);
  if (transform.rotate) parts.push(`rotate(${transform.rotate})`);
  if (transform.scale_x !== undefined || transform.scale_y !== undefined) parts.push(`scale(${transform.scale_x ?? 1} ${transform.scale_y ?? 1})`);
  return parts.length ? parts.join(" ") : undefined;
}

/* ────────────────────────────────────────────────────────────────────────── */
/*  Sub-panel components                                                     */
/* ────────────────────────────────────────────────────────────────────────── */

function HomePanel({
  onNewProject,
  onOpenProviders,
  onStartChat,
  projects,
}: {
  onNewProject: () => void;
  onOpenProviders: () => void;
  onStartChat: () => void;
  projects: ProjectRecord[];
}) {
  return (
    <section className="empty-home">
      <div className="empty-hero">
        <div className="empty-mark">
          <img src="/strut-mark.svg" alt="" />
        </div>
        <h1>Start a motion project</h1>
        <p>Select a folder, open a project chat, or ask Strut to sketch a logo, SVG, UI state, mascot, storyboard, or full animation.</p>
        <div className="empty-actions">
          <button type="button" onClick={onNewProject}>
            <FolderPlus size={16} />
            Select folder
          </button>
          <button type="button" onClick={onStartChat}>
            <MessageSquarePlus size={16} />
            Start chat
          </button>
        </div>
      </div>
      <div className="home-card-grid">
        <button type="button" onClick={onNewProject}>
          <span>New project</span>
          <em>Create a folder with scene, assets, and export directories.</em>
        </button>
        <button type="button" onClick={onStartChat} disabled={projects.length === 0}>
          <span>Plan first</span>
          <em>Start from a prompt, reference, or rough direction before generating motion.</em>
        </button>
        <button type="button" onClick={onOpenProviders}>
          <span>Connect providers</span>
          <em>Choose a real local CLI, Ollama, or BYOK model.</em>
        </button>
      </div>
    </section>
  );
}

function LayerRail({
  collapsed,
  layers,
  onAttachLayer,
  onToggleCollapsed,
  pendingReferences,
  selectedNodeId,
}: {
  collapsed: boolean;
  layers: StrutNode[];
  onAttachLayer: (layer: StrutNode) => void;
  onToggleCollapsed: () => void;
  pendingReferences: ReferenceAttachment[];
  selectedNodeId: string | null;
}) {
  const attachedLayerIds = new Set(
    pendingReferences.filter((ref) => ref.kind === "layer").map((ref) => ref.nodeId).filter(Boolean),
  );
  return (
    <aside className={`layers-rail ${collapsed ? "collapsed" : ""}`} aria-label="Scene layers rail">
      <button aria-label={collapsed ? "Expand layers" : "Collapse layers"} className="layers-rail-toggle" type="button" onClick={onToggleCollapsed}>
        <Layers3 size={16} />
        {collapsed ? null : <span>Layers</span>}
      </button>
      {collapsed ? null : (
        <>
          <div className="layers-rail-heading">
            <strong>Scene layers</strong>
            <em>{layers.length ? `${layers.length} AI-named` : "No scene"}</em>
          </div>
          {layers.length ? (
            <div className="layer-attach-list">
              {layers.map((layer) => {
                const isAttached = attachedLayerIds.has(layer.id);
                return (
                  <button
                    aria-label={`${isAttached ? "Remove" : "Attach"} layer ${layer.name} ${layer.kind}`}
                    aria-pressed={isAttached}
                    className={`${selectedNodeId === layer.id ? "active" : ""} ${isAttached ? "attached" : ""}`}
                    key={layer.id}
                    type="button"
                    onClick={() => onAttachLayer(layer)}
                  >
                    <span>{layer.name}</span>
                    <em>{layer.role ?? layer.kind}</em>
                    {isAttached ? <strong>Attached</strong> : <strong>Add</strong>}
                  </button>
                );
              })}
            </div>
          ) : (
            <p className="panel-empty">No editable layers yet.</p>
          )}
        </>
      )}
    </aside>
  );
}

function PreviewPane({
  activeAnimationId,
  activeMachine,
  activeState,
  document,
  layerUi,
  onDeleteAnimation,
  onOpenAnimation,
  onReferenceAnimation,
  onSelectNode,
  projectAnimations = [],
  refreshSignal = 0,
  selectedNodeId,
  selectedTargetLabel,
  setActiveState,
  showSelectionAffordances = false,
}: {
  activeAnimationId?: string | null;
  activeMachine: StateMachine;
  activeState: string;
  document: StrutDocument | null;
  layerUi?: Record<string, LayerUiState>;
  onDeleteAnimation?: () => void;
  onOpenAnimation?: (animation: ProjectAnimationRecord) => void;
  onReferenceAnimation?: () => void;
  onSelectNode?: (nodeId: string | null) => void;
  projectAnimations?: ProjectAnimationRecord[];
  refreshSignal?: number;
  selectedNodeId?: string | null;
  selectedTargetLabel?: string;
  setActiveState: (state: string) => void;
  showSelectionAffordances?: boolean;
}) {
  const [playSignal, setPlaySignal] = useState(0);
  useEffect(() => {
    if (refreshSignal > 0) setPlaySignal((value) => value + 1);
  }, [refreshSignal]);
  return (
    <aside className={showSelectionAffordances ? "preview-pane selection-aware" : "preview-pane"}>
      <div className="preview-title">
        <div>
          <span>Preview</span>
          <em>{document ? `${document.name} / ${activeMachine.name}` : "No generated scene"}</em>
        </div>
        <div className="preview-title-actions" aria-label="Animation actions">
          <button aria-label="Edit animation in chat" disabled={!document} title="Edit animation in chat" type="button" onClick={onReferenceAnimation}>
            <Edit3 size={15} />
          </button>
          <button aria-label="Delete animation from project" disabled={!document} title="Delete animation from project" type="button" onClick={onDeleteAnimation}>
            <Trash2 size={15} />
          </button>
        </div>
      </div>
      <div className="preview-stage">
        {document ? (
          <CharacterPreview
            activeState={activeState}
            document={document}
            layerUi={layerUi}
            onSelectNode={onSelectNode ? (nodeId) => onSelectNode(nodeId) : undefined}
            playSignal={playSignal}
            selectedNodeId={selectedNodeId}
          />
        ) : (
          <div className="preview-empty">
            <ImagePlus size={26} />
            <strong>No scene yet</strong>
            <span>Attach a reference or describe a logo, SVG, UI state, mascot, storyboard, or scene.</span>
          </div>
        )}
      </div>
      {document ? (
        activeMachine.states.length > 0 ? (
          <div className="state-row">
            {activeMachine.states.map((state) => (
              <button className={state === activeState ? "active" : ""} key={state} type="button" onClick={() => setActiveState(state)}>
                <Route size={13} />
                {titleCase(state)}
              </button>
            ))}
          </div>
        ) : (
          <div className="state-row empty-state-row">
            <span>No generated animations yet.</span>
            <button type="button" onClick={() => window.document.getElementById("composer-input")?.focus()}>Ask for a motion</button>
          </div>
        )
      ) : null}
      {projectAnimations.length ? (
        <div className="project-animation-list" aria-label="Project animations">
          <div className="project-animation-list-heading">
            <strong>Project animations</strong>
            <em>{projectAnimations.length} saved</em>
          </div>
          {projectAnimations.map((animation) => (
            <button
              aria-pressed={animation.id === activeAnimationId}
              className={animation.id === activeAnimationId ? "active" : ""}
              key={animation.id}
              type="button"
              onClick={() => onOpenAnimation?.(animation)}
            >
              <Film size={13} />
              <span>{animation.name}</span>
            </button>
          ))}
        </div>
      ) : null}
      {showSelectionAffordances ? (
        <div className="preview-edit-hint">
          <strong>{selectedTargetLabel ?? "No selection"}</strong>
          <span>{selectedNodeId ? "Preview selection is bound to the semantic scene node." : "Select a visible part or layer to target AI edits."}</span>
        </div>
      ) : null}
    </aside>
  );
}

// We need to import StateMachine for the PreviewPane type signature
import type { StateMachine } from "./types";

export default App;
