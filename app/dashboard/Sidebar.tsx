import { useState, useEffect } from "react";
import { Database, Table, ChevronRight, ChevronDown, LogOut, Plus, Check, X, Sun, Moon, Eye, Terminal, Variable, Users, Activity, Share2, Clock } from "lucide-react";
import { useRouter } from "next/navigation";
import styles from "./dashboard.module.css";
import { useTheme } from "@/lib/ThemeProvider";

interface SidebarProps {
    onSelectTable: (db: string, table: string) => void;
    onSelectObject: (db: string, name: string, type: "view" | "procedure" | "function") => void;
    onSelectDb: (db: string) => void;
    onSelectUsers: () => void;
    onSelectPerformance: () => void;
    onSelectSlowLog: () => void;
    onSelectDiagram: (db: string) => void;
    selectedDb?: string;
    selectedTable?: string;
    selectedObject?: { name: string; type: string };
}

export default function Sidebar({ onSelectTable, onSelectObject, onSelectDb, onSelectUsers, onSelectPerformance, onSelectSlowLog, onSelectDiagram, selectedDb, selectedTable, selectedObject }: SidebarProps) {
    const router = useRouter();
    const { theme, toggleTheme } = useTheme();
    const [databases, setDatabases] = useState<string[]>([]);
    const [expandedDb, setExpandedDb] = useState<string | null>(null);
    const [dbObjects, setDbObjects] = useState<{
        [db: string]: {
            tables: string[];
            views: string[];
            procedures: string[];
            functions: string[];
        }
    }>({});
    const [loadingDb, setLoadingDb] = useState(false);
    const [searchTerm, setSearchTerm] = useState("");
    const [showCreateDb, setShowCreateDb] = useState(false);
    const [newDbName, setNewDbName] = useState("");

    useEffect(() => {
        fetchDatabases();
    }, []);

    useEffect(() => {
        if (selectedDb) {
            setExpandedDb(selectedDb);
            fetchDbObjects(selectedDb);
        }
    }, [selectedDb]);

    const fetchDatabases = async () => {
        setLoadingDb(true);
        try {
            const res = await fetch("/api/databases");
            const data = await res.json();
            if (data.success) {
                setDatabases(data.databases);
            }
        } catch (err) {
            console.error("Failed to fetch databases", err);
        } finally {
            setLoadingDb(false);
        }
    };

    const handleCreateDb = async () => {
        if (!newDbName.trim()) return;
        try {
            const res = await fetch("/api/databases/create", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ name: newDbName }),
            });
            const result = await res.json();
            if (result.success) {
                setNewDbName("");
                setShowCreateDb(false);
                fetchDatabases();
            } else {
                alert(result.error);
            }
        } catch {
            alert("Failed to create database");
        }
    };

    const fetchDbObjects = async (db: string) => {
        if (dbObjects[db]) return;
        try {
            const tableRes = await fetch(`/api/tables?database=${db}`);
            const tableData = await tableRes.json();

            const objectRes = await fetch(`/api/objects?database=${db}`);
            const objectData = await objectRes.json();

            if (tableData.success && objectData.success) {
                setDbObjects((prev) => ({
                    ...prev,
                    [db]: {
                        tables: tableData.tables,
                        views: objectData.views,
                        procedures: objectData.procedures,
                        functions: objectData.functions
                    }
                }));
            }
        } catch (err) {
            console.error("Failed to fetch database objects", err);
        }
    };

    const toggleDb = (db: string) => {
        if (expandedDb === db) {
            setExpandedDb(null);
        } else {
            setExpandedDb(db);
            fetchDbObjects(db);
            onSelectDb(db);
        }
    };

    const handleLogout = async () => {
        await fetch("/api/auth", { method: "DELETE" });
        router.push("/login");
    };

    const filteredDatabases = databases.filter(db =>
        db.toLowerCase().includes(searchTerm.toLowerCase())
    );

    return (
        <aside className={styles.sidebar}>
            <div className={styles.sidebarHeader}>
                <span className={styles.brand}>MySQL GUI</span>
                <div className={styles.headerActions}>
                    <button onClick={toggleTheme} className={styles.themeBtn} title="Toggle Theme">
                        {theme === "dark" ? <Sun size={18} /> : <Moon size={18} />}
                    </button>
                    <button onClick={handleLogout} className={styles.logoutBtn} title="Logout">
                        <LogOut size={18} />
                    </button>
                </div>
            </div>

            <div className={styles.sidebarSearch}>
                <div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
                    <input
                        type="text"
                        placeholder="Search databases..."
                        className={styles.searchInput}
                        value={searchTerm}
                        onChange={(e) => setSearchTerm(e.target.value)}
                        style={{ flex: 1 }}
                    />
                    <button
                        className={styles.logoutBtn}
                        onClick={() => setShowCreateDb(!showCreateDb)}
                        title="New Database"
                    >
                        <Plus size={18} />
                    </button>
                </div>
                {showCreateDb && (
                    <div style={{ marginTop: "8px", display: "flex", gap: "4px" }}>
                        <input
                            type="text"
                            placeholder="Database name"
                            className={styles.searchInput}
                            value={newDbName}
                            onChange={(e) => setNewDbName(e.target.value)}
                            onKeyDown={(e) => e.key === "Enter" && handleCreateDb()}
                            autoFocus
                        />
                        <button className={styles.logoutBtn} onClick={handleCreateDb} style={{ color: "#10b981" }}>
                            <Check size={16} />
                        </button>
                        <button className={styles.logoutBtn} onClick={() => setShowCreateDb(false)} style={{ color: "#ef4444" }}>
                            <X size={16} />
                        </button>
                    </div>
                )}

                <button
                    className={`${styles.tableBtn} ${!selectedDb && !selectedTable ? styles.active : ""}`}
                    onClick={onSelectUsers}
                    style={{ marginTop: "12px", width: "100%", textAlign: "left", paddingLeft: "12px" }}
                >
                    <Users size={16} style={{ marginRight: 8, verticalAlign: "middle" }} />
                    User Management
                </button>

                <button
                    className={styles.tableBtn}
                    onClick={onSelectPerformance}
                    style={{ width: "100%", textAlign: "left", paddingLeft: "12px" }}
                >
                    <Activity size={16} style={{ marginRight: 8, verticalAlign: "middle" }} />
                    Performance
                </button>

                <button
                    className={styles.tableBtn}
                    onClick={onSelectSlowLog}
                    style={{ width: "100%", textAlign: "left", paddingLeft: "12px" }}
                >
                    <Clock size={16} style={{ marginRight: 8, verticalAlign: "middle" }} />
                    Slow Query Log
                </button>
            </div>

            <div className={styles.sidebarContent}>
                {loadingDb ? (
                    <div style={{ padding: "1rem", color: "#94a3b8", fontSize: "0.875rem" }}>Loading databases...</div>
                ) : (
                    filteredDatabases.map((db) => (
                        <div key={db} className={styles.dbItem}>
                            <button
                                className={`${styles.dbHeader} ${selectedDb === db ? styles.active : ""}`}
                                onClick={() => toggleDb(db)}
                            >
                                {expandedDb === db ? <ChevronDown size={14} style={{ marginRight: 8 }} /> : <ChevronRight size={14} style={{ marginRight: 8 }} />}
                                <Database size={16} style={{ marginRight: 8 }} />
                                {db}
                            </button>

                            {expandedDb === db && (
                                <div className={styles.tableList}>
                                    {/* DB Summary Link */}
                                    <button
                                        className={`${styles.tableBtn} ${selectedDb === db && !selectedTable && !selectedObject ? styles.active : ""}`}
                                        onClick={() => onSelectDb(db)}
                                        style={{ fontWeight: "bold" }}
                                    >
                                        <Activity size={12} style={{ marginRight: 8 }} /> Overview
                                    </button>

                                    {/* Diagram Link */}
                                    <button
                                        className={styles.tableBtn}
                                        onClick={(e) => { e.stopPropagation(); onSelectDiagram(db); }}
                                        style={{ color: "var(--primary)" }}
                                    >
                                        <Share2 size={12} style={{ marginRight: 8 }} /> ER Diagram
                                    </button>

                                    {dbObjects[db] ? (
                                        <>
                                            {/* Tables Section */}
                                            {dbObjects[db].tables.map((table) => (
                                                <button
                                                    key={`table-${table}`}
                                                    className={`${styles.tableBtn} ${selectedTable === table && selectedDb === db ? styles.active : ""}`}
                                                    onClick={() => onSelectTable(db, table)}
                                                >
                                                    <Table size={14} style={{ marginRight: 8, verticalAlign: "middle" }} />
                                                    {table}
                                                </button>
                                            ))}

                                            {/* Views Section */}
                                            {dbObjects[db].views.length > 0 && (
                                                <div className={styles.objectSectionLabel}>Views</div>
                                            )}
                                            {dbObjects[db].views.map((view) => (
                                                <button
                                                    key={`view-${view}`}
                                                    className={`${styles.tableBtn} ${selectedObject?.name === view && selectedObject?.type === "view" ? styles.active : ""}`}
                                                    onClick={() => onSelectObject(db, view, "view")}
                                                >
                                                    <Eye size={14} style={{ marginRight: 8, verticalAlign: "middle" }} />
                                                    {view}
                                                </button>
                                            ))}

                                            {/* Procedures Section */}
                                            {dbObjects[db].procedures.length > 0 && (
                                                <div className={styles.objectSectionLabel}>Procedures</div>
                                            )}
                                            {dbObjects[db].procedures.map((proc) => (
                                                <button
                                                    key={`proc-${proc}`}
                                                    className={`${styles.tableBtn} ${selectedObject?.name === proc && selectedObject?.type === "procedure" ? styles.active : ""}`}
                                                    onClick={() => onSelectObject(db, proc, "procedure")}
                                                >
                                                    <Terminal size={14} style={{ marginRight: 8, verticalAlign: "middle" }} />
                                                    {proc}
                                                </button>
                                            ))}

                                            {/* Functions Section */}
                                            {dbObjects[db].functions.length > 0 && (
                                                <div className={styles.objectSectionLabel}>Functions</div>
                                            )}
                                            {dbObjects[db].functions.map((func) => (
                                                <button
                                                    key={`func-${func}`}
                                                    className={`${styles.tableBtn} ${selectedObject?.name === func && selectedObject?.type === "function" ? styles.active : ""}`}
                                                    onClick={() => onSelectObject(db, func, "function")}
                                                >
                                                    <Variable size={14} style={{ marginRight: 8, verticalAlign: "middle" }} />
                                                    {func}
                                                </button>
                                            ))}
                                        </>
                                    ) : (
                                        <div style={{ padding: "0.25rem 0.5rem", color: "#64748b", fontSize: "0.75rem" }}>Loading...</div>
                                    )}
                                </div>
                            )}
                        </div>
                    ))
                )}
            </div>
        </aside>
    );
}
