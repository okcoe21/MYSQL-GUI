"use client";

import { useState, useEffect } from "react";
import styles from "./dashboard.module.css";
import { AlertCircle, Loader2, Database, Trash2, Plus, Check, X, Server } from "lucide-react";

interface ServerOverviewProps {
    onSelectDb: (db: string) => void;
    onRefreshSidebar: () => void;
}

export default function ServerOverview({ onSelectDb, onRefreshSidebar }: ServerOverviewProps) {
    const [databases, setDatabases] = useState<string[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState("");
    const [droppingDb, setDroppingDb] = useState<string | null>(null);
    const [showConfirm, setShowConfirm] = useState<string | null>(null);
    const [newDbName, setNewDbName] = useState("");
    const [showCreateDb, setShowCreateDb] = useState(false);

    useEffect(() => {
        fetchDatabases();
    }, []);

    const fetchDatabases = async () => {
        setLoading(true);
        setError("");
        try {
            const res = await fetch("/api/databases");
            const result = await res.json();
            if (result.success) {
                setDatabases(result.databases);
            } else {
                setError(result.error);
            }
        } catch {
            setError("Failed to fetch databases");
        } finally {
            setLoading(false);
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
                onRefreshSidebar();
            } else {
                setError(result.error);
            }
        } catch {
            setError("Failed to create database");
        }
    };

    const handleDropDatabase = async (db: string, confirmed = false) => {
        if (!confirmed) {
            setShowConfirm(db);
            return;
        }

        setDroppingDb(db);
        setError("");
        try {
            const res = await fetch("/api/databases/drop", {
                method: "DELETE",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ database: db, confirmed: true }),
            });

            const result = await res.json();
            if (result.success) {
                fetchDatabases();
                onRefreshSidebar();
            } else {
                setError(result.error);
            }
        } catch {
            setError(`Failed to drop database ${db}`);
        } finally {
            setDroppingDb(null);
            setShowConfirm(null);
        }
    };

    if (loading && databases.length === 0) {
        return (
            <div className={styles.emptyState}>
                <Loader2 className={`animate-spin ${styles.emptyIcon}`} size={48} />
                <p>Loading server information...</p>
            </div>
        );
    }

    return (
        <div className={styles.wrapper}>
            <div className={styles.count} style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <div>
                    <Server size={18} style={{ marginRight: 8, verticalAlign: "middle" }} />
                    Server: <strong>localhost</strong> - {databases.length} databases
                </div>
                <button className={styles.actionBtn} onClick={() => setShowCreateDb(!showCreateDb)}>
                    <Plus size={14} style={{ marginRight: 4 }} /> Create Database
                </button>
            </div>

            {showCreateDb && (
                <div className={styles.form} style={{ marginBottom: "1.5rem", padding: "1rem" }}>
                    <div style={{ display: "flex", gap: "8px" }}>
                        <input
                            type="text"
                            placeholder="New database name"
                            className={styles.formInput}
                            value={newDbName}
                            onChange={(e) => setNewDbName(e.target.value)}
                            onKeyDown={(e) => e.key === "Enter" && handleCreateDb()}
                            autoFocus
                        />
                        <button className={styles.primaryBtn} onClick={handleCreateDb} style={{ padding: "0.4rem 1rem" }}>
                            Create
                        </button>
                        <button className={styles.actionBtn} onClick={() => setShowCreateDb(false)}>Cancel</button>
                    </div>
                </div>
            )}

            {error && (
                <div className={styles.warning} style={{ borderColor: "#ef4444", color: "#ef4444", marginBottom: "1rem" }}>
                    <AlertCircle size={16} style={{ marginRight: 8, verticalAlign: "middle" }} />
                    {error}
                </div>
            )}

            {showConfirm && (
                <div className={styles.warning} style={{ marginBottom: "1.5rem", border: "1px solid #ef4444", backgroundColor: "rgba(239, 68, 68, 0.05)" }}>
                    <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                        <AlertCircle size={20} color="#ef4444" />
                        <strong style={{ color: "#ef4444" }}>Confirm Database Drop</strong>
                    </div>
                    <p style={{ margin: "0.5rem 0", color: "#f8fafc" }}>
                        Are you sure you want to <strong>DROP</strong> the entire database <code>{showConfirm}</code>?
                        This Action is irreversible.
                    </p>
                    <div className={styles.formFooter} style={{ marginTop: "1rem", justifyContent: "flex-start" }}>
                        <button className={styles.dangerBtn} style={{ padding: "0.4rem 1.25rem" }} onClick={() => handleDropDatabase(showConfirm, true)}>
                            Drop Database
                        </button>
                        <button className={styles.actionBtn} onClick={() => setShowConfirm(null)}>Cancel</button>
                    </div>
                </div>
            )}

            <div className={styles.tableContainer}>
                <table className={styles.table}>
                    <thead>
                        <tr>
                            <th>Database</th>
                            <th style={{ width: "100px" }}>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {databases.map((db) => (
                            <tr key={db}>
                                <td
                                    onClick={() => onSelectDb(db)}
                                    style={{ cursor: "pointer", fontWeight: 500 }}
                                    className={styles.linkText}
                                >
                                    <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                                        <Database size={16} color="#38bdf8" />
                                        {db}
                                    </div>
                                </td>
                                <td>
                                    <button
                                        className={`${styles.actionBtn} danger`}
                                        onClick={() => handleDropDatabase(db)}
                                        disabled={droppingDb === db}
                                        title="Drop Database"
                                    >
                                        {droppingDb === db ? <Loader2 className="animate-spin" size={14} /> : <Trash2 size={14} />}
                                    </button>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
