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
            // Fetch tables first
            const tableRes = await fetch(`/api/tables?database=${db}`);
            const tableData = await tableRes.json();

            // Fetch other database objects (views, procedures, functions)
            let views: string[] = [];
            let procedures: string[] = [];
            let functions: string[] = [];

            try {
                const objectRes = await fetch(`/api/objects?database=${db}`);
                const objectData = await objectRes.json();
                if (objectData.success) {
                    views = objectData.views || [];
                    procedures = objectData.procedures || [];
                    functions = objectData.functions || [];
                } else {
                    console.warn(`Failed to fetch objects for DB ${db}:`, objectData.error);
                }
            } catch (err) {
                console.error(`Error fetching objects for DB ${db}:`, err);
            }

            if (tableData.success) {
                setDbObjects((prev) => ({
                    ...prev,
                    [db]: {
                        tables: tableData.tables || [],
                        views,
                        procedures,
                        functions
                    }
                }));
            } else {
                console.error(`Failed to fetch tables for DB ${db}:`, tableData.error);
                // Set empty arrays to prevent permanent "Loading..." state
                setDbObjects((prev) => ({
                    ...prev,
                    [db]: {
                        tables: [],
                        views: [],
                        procedures: [],
                        functions: []
                    }
                }));
            }
        } catch (err) {
            console.error("Failed to fetch database objects", err);
            // Ensure we clear loading state by setting empty lists on error
            setDbObjects((prev) => ({
                ...prev,
                [db]: {
                    tables: [],
                    views: [],
                    procedures: [],
                    functions: []
                }
            }));
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
                <div className={`${styles.flexRow} ${styles.flexGapSm}`}>
                    <input
                        type="text"
                        placeholder="Search databases..."
                        className={`${styles.searchInput} ${styles.flex1}`}
                        value={searchTerm}
                        onChange={(e) => setSearchTerm(e.target.value)}
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
                    <div className={styles.sidebarCreateDbForm}>
                        <input
                            type="text"
                            placeholder="Database name"
                            className={styles.searchInput}
                            value={newDbName}
                            onChange={(e) => setNewDbName(e.target.value)}
                            onKeyDown={(e) => e.key === "Enter" && handleCreateDb()}
                            autoFocus
                        />
                        <button className={styles.logoutBtn} onClick={handleCreateDb}>
                            <Check size={16} />
                        </button>
                        <button className={styles.logoutBtn} onClick={() => setShowCreateDb(false)}>
                            <X size={16} />
                        </button>
                    </div>
                )}

                <button
                    className={`${styles.tableBtn} ${styles.marginTopSm} ${!selectedDb && !selectedTable ? styles.active : ""}`}
                    onClick={onSelectUsers}
                >
                    <Users size={16} className={styles.tabIcon} />
                    User Management
                </button>

                <button
                    className={styles.tableBtn}
                    onClick={onSelectPerformance}
                >
                    <Activity size={16} className={styles.tabIcon} />
                    Performance
                </button>

                <button
                    className={styles.tableBtn}
                    onClick={onSelectSlowLog}
                >
                    <Clock size={16} className={styles.tabIcon} />
                    Slow Query Log
                </button>
            </div>

            <div className={styles.sidebarContent}>
                {loadingDb ? (
                    <div className={styles.pageInfo}>Loading databases...</div>
                ) : (
                    filteredDatabases.map((db) => (
                        <div key={db} className={styles.dbItem}>
                            <button
                                className={`${styles.dbHeader} ${selectedDb === db ? styles.active : ""}`}
                                onClick={() => toggleDb(db)}
                            >
                                {expandedDb === db ? <ChevronDown size={14} className={styles.tabIcon} /> : <ChevronRight size={14} className={styles.tabIcon} />}
                                <Database size={16} className={styles.tabIcon} />
                                {db}
                            </button>

                            {expandedDb === db && (
                                <div className={styles.tableList}>
                                    {/* DB Summary Link */}
                                    <button
                                        className={`${styles.tableBtn} ${styles.indentOverview} ${selectedDb === db && !selectedTable && !selectedObject ? styles.active : ""}`}
                                        onClick={() => onSelectDb(db)}
                                    >
                                        <Activity size={12} className={styles.tabIcon} /> Overview
                                    </button>

                                    {/* Diagram Link */}
                                    <button
                                        className={`${styles.tableBtn} ${styles.indentDiagram}`}
                                        onClick={(e) => { e.stopPropagation(); onSelectDiagram(db); }}
                                    >
                                        <Share2 size={12} className={styles.tabIcon} /> ER Diagram
                                    </button>

                                    {dbObjects[db] ? (
                                        <>
                                            {/* Tables Section */}
                                            {dbObjects[db].tables.map((table) => (
                                                <button
                                                    key={`table-${table}`}
                                                    className={`${styles.tableBtn} ${styles.indentTable} ${selectedTable === table && selectedDb === db ? styles.active : ""}`}
                                                    onClick={() => onSelectTable(db, table)}
                                                >
                                                    <Table size={14} className={styles.tabIcon} />
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
                                                    className={`${styles.tableBtn} ${styles.indentView} ${selectedObject?.name === view && selectedObject?.type === "view" ? styles.active : ""}`}
                                                    onClick={() => onSelectObject(db, view, "view")}
                                                >
                                                    <Eye size={14} className={styles.tabIcon} />
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
                                                    className={`${styles.tableBtn} ${styles.indentProcedure} ${selectedObject?.name === proc && selectedObject?.type === "procedure" ? styles.active : ""}`}
                                                    onClick={() => onSelectObject(db, proc, "procedure")}
                                                >
                                                    <Terminal size={14} className={styles.tabIcon} />
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
                                                    className={`${styles.tableBtn} ${styles.indentFunction} ${selectedObject?.name === func && selectedObject?.type === "function" ? styles.active : ""}`}
                                                    onClick={() => onSelectObject(db, func, "function")}
                                                >
                                                    <Variable size={14} className={styles.tabIcon} />
                                                    {func}
                                                </button>
                                            ))}
                                        </>
                                    ) : (
                                        <div className={styles.pageInfo}>Loading...</div>
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
