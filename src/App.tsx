import { invoke } from "@tauri-apps/api/core";
import { ArrowLeft, ArrowRight, BookOpen, Check, ChevronDown, CircleAlert, Copy, FileText, FolderOpen, Layers3, RefreshCw, Search, ShieldCheck, Sparkles, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { categoryCounts, filterAgents } from "./catalog";
import type { Agent, Catalog, FileStatus } from "./types";

const statusText: Record<FileStatus, string> = {
  present: "Found", missing: "Not found", ambiguous: "Ambiguous", not_available: "Not available"
};

function Status({ status }: { status: FileStatus }) {
  return <span className={`status status-${status}`}><span className="status-dot" />{statusText[status]}</span>;
}

function App() {
  const [catalog, setCatalog] = useState<Catalog | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState("all");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  async function refresh() {
    setLoading(true);
    setError(null);
    try {
      const next = await invoke<Catalog>("load_catalog");
      setCatalog(next);
    } catch (reason) {
      setError(String(reason));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => { void refresh(); }, []);
  useEffect(() => {
    const closeOnEscape = (event: KeyboardEvent) => { if (event.key === "Escape") setSelectedId(null); };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, []);

  const agents = catalog?.agents ?? [];
  const categories = useMemo(() => categoryCounts(agents), [agents]);
  const results = useMemo(() => filterAgents(agents, query, category), [agents, query, category]);
  const selected = agents.find((agent) => agent.id === selectedId) ?? null;

  function selectAgent(agent: Agent) {
    setSelectedId(agent.id);
    setCopied(false);
  }

  async function copyPath(path: string) {
    try {
      await navigator.clipboard.writeText(path);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1800);
    } catch { setCopied(false); }
  }

  return <div className="app-shell">
    <aside className="sidebar">
      <div className="brand"><div className="brand-mark"><Layers3 size={21} strokeWidth={2.25} /></div><div><strong>Agency Atlas</strong><span>DESKTOP CATALOG</span></div></div>
      <div className="sidebar-section-label">LIBRARY</div>
      <button className={`nav-item ${category === "all" ? "active" : ""}`} onClick={() => { setCategory("all"); setSelectedId(null); }}><BookOpen size={17} /><span>All agents</span><em>{agents.length}</em></button>
      <div className="sidebar-section-label categories-label">CATEGORIES <span>{categories.length}</span></div>
      <nav className="category-nav" aria-label="Categories">
        {categories.map((item) => <button key={item.id} className={`nav-item ${category === item.id ? "active" : ""}`} onClick={() => { setCategory(item.id); setSelectedId(null); }}><span className="nav-bullet" /><span>{item.label}</span><em>{item.count}</em></button>)}
      </nav>
      <div className="sidebar-bottom"><ShieldCheck size={17} /><div><strong>Read-only catalog</strong><span>Source and Codex files are never changed.</span></div></div>
    </aside>

    <main className="main-area">
      <header className="topbar"><div className="breadcrumb">Workspace <span>/</span> Agent catalog</div><div className="topbar-right"><span className="topbar-state"><span /> LOCAL REPOSITORY</span><span className="topbar-divider" /><button className="icon-button" onClick={() => void refresh()} title="Refresh catalog" aria-label="Refresh catalog"><RefreshCw size={17} className={loading ? "spin" : ""} /></button></div></header>

      <div className="content">
        <div className="eyebrow"><span className="eyebrow-line" />EXPLORE THE AGENCY</div>
        <div className="heading-row"><div><h1>Agent catalog<span className="heading-period">.</span></h1><p className="intro">A clear view of every specialist in this repository. Explore roles, inspect their source, and see how Codex files map back.</p></div><div className="total-card"><span>IN THE LIBRARY</span><strong>{agents.length.toString().padStart(3, "0")}</strong><small>source agents</small></div></div>

        {error && <div className="notice error"><CircleAlert size={18} /><div><strong>Catalog unavailable</strong><p>{error}</p></div><button onClick={() => void refresh()}>Retry</button></div>}
        {catalog && catalog.warnings.length > 0 && <details className="notice warning"><summary><CircleAlert size={18} /> {catalog.warnings.length} source file{catalog.warnings.length === 1 ? "" : "s"} could not be read <ChevronDown size={16} /></summary><ul>{catalog.warnings.map((warning, index) => <li key={index}>{warning}</li>)}</ul></details>}

        <div className="controls"><label className="search-box"><Search size={19} /><input aria-label="Search agents" autoFocus placeholder="Search agents, skills, categories, paths..." value={query} onChange={(event) => { setQuery(event.target.value); setSelectedId(null); }} />{query && <button aria-label="Clear search" onClick={() => setQuery("")}><X size={16} /></button>}<kbd>⌕</kbd></label><div className="mobile-filter"><select aria-label="Filter category" value={category} onChange={(event) => { setCategory(event.target.value); setSelectedId(null); }}><option value="all">All categories</option>{categories.map((item) => <option value={item.id} key={item.id}>{item.label}</option>)}</select><ChevronDown size={16} /></div></div>

        <div className="results-head"><div><strong>{category === "all" ? "All specialists" : categories.find((item) => item.id === category)?.label}</strong><span>{results.length} result{results.length === 1 ? "" : "s"}{query && <> for “{query}”</>}</span></div><span className="sort-label">A—Z BY NAME</span></div>

        {loading && !catalog ? <div className="empty-state"><RefreshCw className="spin" size={30} /><h2>Reading the repository</h2><p>Collecting agent definitions and Codex file status.</p></div> : results.length === 0 ? <div className="empty-state"><div className="empty-icon"><Search size={29} /></div><h2>No agents found</h2><p>Try a different term or choose another category.</p><button onClick={() => { setQuery(""); setCategory("all"); }}>Clear filters <ArrowRight size={15} /></button></div> : <div className="agent-grid">{results.map((agent) => <button key={agent.id} className={`agent-card ${selectedId === agent.id ? "selected" : ""}`} onClick={() => selectAgent(agent)}><div className="card-top"><div className="avatar">{agent.emoji || <Sparkles size={19} />}</div><span className="card-arrow"><ArrowRight size={17} /></span></div><div className="card-category">{agent.categoryLabel}</div><h2>{agent.name}</h2><p>{agent.description || "No description in source metadata."}</p><div className="card-bottom"><span><FileText size={14} />{agent.sourcePath.split("/").at(-1)}</span><span className={`tiny-dot ${agent.mapping.generated}`} title={`Generated TOML: ${statusText[agent.mapping.generated]}`} /></div></button>)}</div>}
      </div>
    </main>

    {selected && <div className="detail-backdrop" onMouseDown={(event) => { if (event.target === event.currentTarget) setSelectedId(null); }}><section className="detail-panel" role="dialog" aria-modal="true" aria-label={`${selected.name} details`}><div className="detail-top"><button className="back-button" onClick={() => setSelectedId(null)}><ArrowLeft size={17} /> Back to catalog</button><button className="icon-button close-detail" aria-label="Close details" onClick={() => setSelectedId(null)}><X size={20} /></button></div><div className="detail-scroll"><div className="detail-avatar">{selected.emoji || <Sparkles size={30} />}</div><div className="detail-category">{selected.categoryLabel}</div><h2>{selected.name}</h2><p className="detail-description">{selected.description || "No description in source metadata."}</p>{selected.vibe && <blockquote>“{selected.vibe.replace(/^['"]|['"]$/g, "")}”</blockquote>}{selected.metadataWarning && <div className="metadata-warning"><CircleAlert size={17} />Metadata issue: {selected.metadataWarning}</div>}<div className="detail-rule" /><div className="detail-section-title"><FileText size={17} /> Source definition</div><div className="path-card"><span>REPOSITORY-RELATIVE PATH / STABLE ID</span><div><code>{selected.sourcePath}</code><button title="Copy source path" aria-label="Copy source path" onClick={() => void copyPath(selected.sourcePath)}>{copied ? <Check size={17} /> : <Copy size={17} />}</button></div></div><div className="detail-section-title mapping-title"><FolderOpen size={17} /> Codex TOML mapping</div><p className="section-help">Filename derived from the source name by the repository converter.</p><div className="mapping-list"><div><span>Generated in repository</span><Status status={selected.mapping.generated} /></div><div><span>Installed in user profile</span><Status status={selected.mapping.installed} /></div></div>{selected.mapping.slug && <div className="mapping-filename"><span>EXPECTED FILENAME</span><code>{selected.mapping.slug}.toml</code></div>}{selected.mapping.conflictingSources.length > 0 && <div className="ambiguity"><CircleAlert size={18} /><div><strong>Shared filename</strong><p>This source has the same conversion slug as {selected.mapping.conflictingSources.join(", ")}. The TOML cannot be attributed to either source.</p></div></div>}<div className="detail-footer"><ShieldCheck size={16} /> Read-only view · No agent execution</div></div></section></div>}
  </div>;
}

export default App;
