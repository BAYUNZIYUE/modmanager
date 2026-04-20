import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

type ModInfo = {
  id: string;
  slug: string;
  name: string;
  description: string;
  provider: "CurseForge" | "Modrinth" | "Bmclapi";
  authors: { name: string; url?: string }[];
  icon_url?: string;
  website_url?: string;
  downloads: number;
  categories: string[];
};

type ModVersion = {
  id: string;
  mod_id: string;
  version_number: string;
  version_name: string;
  version_type: "Release" | "Beta" | "Alpha";
  game_versions: string[];
  loaders: string[];
  download_url: string;
  filename: string;
  file_size: number;
  date: string;
};

type LocalMod = {
  path: string;
  filename: string;
  enabled: boolean;
  file_size: number;
  metadata?: {
    name: string;
    installed_version_number: string;
    provider: string;
  };
};

type Page = "explore" | "local" | "settings";

function App() {
  const [page, setPage] = useState<Page>("explore");
  const [provider, setProvider] = useState<string>("modrinth");
  const [searchQuery, setSearchQuery] = useState("");
  const [gameVersion, setGameVersion] = useState("");
  const [modLoader, setModLoader] = useState("");
  const [searchResults, setSearchResults] = useState<ModInfo[]>([]);
  const [localMods, setLocalMods] = useState<LocalMod[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedMod, setSelectedMod] = useState<ModInfo | null>(null);
  const [modVersions, setModVersions] = useState<ModVersion[]>([]);
  const [installing, setInstalling] = useState(false);

  const searchMods = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<string>("search_mods", {
        query: searchQuery || undefined,
        gameVersion: gameVersion || undefined,
        loader: modLoader || undefined,
        provider,
        offset: 0,
      });
      const parsed = JSON.parse(result);
      setSearchResults(parsed.mods || []);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const loadLocalMods = async () => {
    setLoading(true);
    try {
      const result = await invoke<string>("list_local_mods");
      const parsed = JSON.parse(result);
      setLocalMods(parsed);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const selectMod = async (mod: ModInfo) => {
    setSelectedMod(mod);
    setLoading(true);
    try {
      const result = await invoke<string>("get_mod_versions", {
        modId: mod.id,
        provider: mod.provider === "CurseForge" ? "curseforge" : "modrinth",
      });
      const parsed = JSON.parse(result);
      setModVersions(parsed);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const installVersion = async (version: ModVersion) => {
    setInstalling(true);
    try {
      const modName = selectedMod?.name || version.filename;
      await invoke("install_mod", {
        version: JSON.stringify(version),
        modName,
        provider: selectedMod?.provider === "CurseForge" ? "curseforge" : "modrinth",
      });
      alert(`Installed ${version.filename}`);
    } catch (e) {
      setError(String(e));
    } finally {
      setInstalling(false);
    }
  };

  const toggleMod = async (mod: LocalMod) => {
    try {
      await invoke("toggle_mod", { modPath: mod.path });
      loadLocalMods();
    } catch (e) {
      setError(String(e));
    }
  };

  const deleteMod = async (mod: LocalMod) => {
    if (!confirm(`Delete ${mod.filename}?`)) return;
    try {
      await invoke("delete_mod", { modPath: mod.path });
      loadLocalMods();
    } catch (e) {
      setError(String(e));
    }
  };

  useEffect(() => {
    if (page === "explore") {
      searchMods();
    } else if (page === "local") {
      loadLocalMods();
    }
  }, [page]);

  return (
    <div className="app">
      <div className="layout">
        <div className="sidebar">
          <h3 style={{ color: "#fff", marginBottom: 16, fontSize: 16 }}>
            Mod Manager
          </h3>
          <div
            className={`sidebar-item ${page === "explore" ? "active" : ""}`}
            onClick={() => setPage("explore")}
          >
            Explore Mods
          </div>
          <div
            className={`sidebar-item ${page === "local" ? "active" : ""}`}
            onClick={() => setPage("local")}
          >
            Local Mods
          </div>
          <div
            className={`sidebar-item ${page === "settings" ? "active" : ""}`}
            onClick={() => setPage("settings")}
          >
            Settings
          </div>
        </div>

        <div className="main-content">
          {page === "explore" && (
            <>
              <div className="top-bar">
                <input
                  className="search-input"
                  placeholder="Search mods..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && searchMods()}
                />
                <select
                  className="filter-select"
                  value={provider}
                  onChange={(e) => setProvider(e.target.value)}
                >
                  <option value="modrinth">Modrinth</option>
                  <option value="curseforge">CurseForge</option>
                </select>
                <input
                  className="search-input"
                  style={{ width: 120 }}
                  placeholder="MC Version"
                  value={gameVersion}
                  onChange={(e) => setGameVersion(e.target.value)}
                />
                <select
                  className="filter-select"
                  value={modLoader}
                  onChange={(e) => setModLoader(e.target.value)}
                >
                  <option value="">All Loaders</option>
                  <option value="fabric">Fabric</option>
                  <option value="forge">Forge</option>
                  <option value="neoforge">NeoForge</option>
                  <option value="quilt">Quilt</option>
                </select>
                <button className="btn btn-primary" onClick={searchMods}>
                  Search
                </button>
              </div>

              {selectedMod ? (
                <div className="content-area">
                  <button
                    className="btn"
                    style={{ marginBottom: 16 }}
                    onClick={() => setSelectedMod(null)}
                  >
                    Back to results
                  </button>
                  <h2 style={{ color: "#fff", marginBottom: 8 }}>
                    {selectedMod.name}
                  </h2>
                  <p style={{ color: "#a0a0b0", marginBottom: 16 }}>
                    {selectedMod.description}
                  </p>
                  <h3 style={{ color: "#fff", marginBottom: 12 }}>
                    Versions
                  </h3>
                  {loading ? (
                    <div className="loading">Loading versions...</div>
                  ) : (
                    <div className="mod-list">
                      {modVersions.map((v) => (
                        <div key={v.id} className="mod-list-item">
                          <div style={{ flex: 1 }}>
                            <div className="mod-list-name">{v.version_name}</div>
                            <div className="mod-list-version">
                              {v.game_versions.join(", ")} |{" "}
                              {v.loaders.join(", ")} |{" "}
                              {v.version_type} |{" "}
                              {(v.file_size / 1024 / 1024).toFixed(2)} MB
                            </div>
                          </div>
                          <button
                            className="btn btn-success"
                            disabled={installing}
                            onClick={() => installVersion(v)}
                          >
                            {installing ? "Installing..." : "Install"}
                          </button>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              ) : (
                <div className="content-area">
                  {error && (
                    <div className="error-message">{error}</div>
                  )}
                  {loading ? (
                    <div className="loading">Searching...</div>
                  ) : (
                    <div className="mod-grid">
                      {searchResults.map((mod) => (
                        <div
                          key={mod.id}
                          className="mod-card"
                          onClick={() => selectMod(mod)}
                        >
                          <div className="mod-card-header">
                            {mod.icon_url ? (
                              <img
                                className="mod-icon"
                                src={mod.icon_url}
                                alt={mod.name}
                              />
                            ) : (
                              <div className="mod-icon" />
                            )}
                            <div>
                              <div className="mod-card-title">{mod.name}</div>
                              <div className="mod-card-author">
                                by {mod.authors.map((a) => a.name).join(", ")}
                              </div>
                            </div>
                          </div>
                          <div className="mod-card-desc">{mod.description}</div>
                          <div className="mod-card-footer">
                            <span className="mod-downloads">
                              {(mod.downloads / 1000).toFixed(1)}k downloads
                            </span>
                            <span
                              className={`mod-provider-badge ${mod.provider.toLowerCase()}`}
                            >
                              {mod.provider}
                            </span>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              )}
            </>
          )}

          {page === "local" && (
            <>
              <div className="top-bar">
                <h2 style={{ color: "#fff" }}>Local Mods</h2>
                <button
                  className="btn btn-primary"
                  onClick={loadLocalMods}
                  style={{ marginLeft: "auto" }}
                >
                  Refresh
                </button>
              </div>
              <div className="content-area">
                {error && (
                  <div className="error-message">{error}</div>
                )}
                <div className="mod-list">
                  {localMods.map((mod, i) => (
                    <div
                      key={i}
                      className={`mod-list-item ${mod.enabled ? "" : "disabled"}`}
                    >
                      <div className="mod-list-name">
                        {mod.metadata?.name || mod.filename}
                      </div>
                      <div className="mod-list-version">
                        {mod.metadata?.installed_version_number || "Unknown"}
                      </div>
                      <div className="mod-list-actions">
                        <button
                          className={`btn ${mod.enabled ? "btn-danger" : "btn-success"}`}
                          onClick={() => toggleMod(mod)}
                        >
                          {mod.enabled ? "Disable" : "Enable"}
                        </button>
                        <button
                          className="btn btn-danger"
                          onClick={() => deleteMod(mod)}
                        >
                          Delete
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </>
          )}

          {page === "settings" && (
            <div className="content-area">
              <h2 style={{ color: "#fff", marginBottom: 16 }}>Settings</h2>
              <p style={{ color: "#a0a0b0" }}>
                Settings page - configure mod directories, preferred versions,
                and language preferences.
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default App;
