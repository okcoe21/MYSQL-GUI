"use client";

import { useState, useEffect } from "react";
import { Database, Table, ChevronRight, ChevronDown, LogOut, Plus, Check, X } from "lucide-react";
import { useRouter } from "next/navigation";
import styles from "./dashboard.module.css";

interface SidebarProps {
    onSelectTable: (db: string, table: string) => void;
    onSelectDb: (db: string) => void;
    selectedDb?: string;
    selectedTable?: string;
}

export default function Sidebar({ onSelectTable, onSelectDb, selectedDb, selectedTable }: SidebarProps) {
    const router = useRouter();
    const [databases, setDatabases] = useState<string[]>([]);
    const [expandedDb, setExpandedDb] = useState<string | null>(null);
    const [tables, setTables] = useState<{ [db: string]: string[] }>({});
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
            fetchTables(selectedDb);
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

    const fetchTables = async (db: string) => {
        if (tables[db]) return; // Already fetched
        try {
            const res = await fetch(`/api/tables?database=${db}`);
            const data = await res.json();
            if (data.success) {
                setTables((prev) => ({ ...prev, [db]: data.tables }));
            }
        } catch (err) {
            console.error("Failed to fetch tables", err);
        }
    };

    const toggleDb = (db: string) => {
        if (expandedDb === db) {
            setExpandedDb(null);
        } else {
            setExpandedDb(db);
            fetchTables(db);
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
                <button onClick={handleLogout} className={styles.logoutBtn} title="Logout">
                    <LogOut size={18} />
                </button>
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
                                    {tables[db] ? (
                                        tables[db].map((table) => (
                                            <button
                                                key={table}
                                                className={`${styles.tableBtn} ${selectedTable === table && selectedDb === db ? styles.active : ""}`}
                                                onClick={() => onSelectTable(db, table)}
                                            >
                                                <Table size={14} style={{ marginRight: 8, verticalAlign: "middle" }} />
                                                {table}
                                            </button>
                                        ))
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
