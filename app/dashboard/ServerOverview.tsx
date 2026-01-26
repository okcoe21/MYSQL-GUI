"use client";

import { useState, useEffect } from "react";
import styles from "./dashboard.module.css";
import { AlertCircle, Loader2, Database, Trash2, Plus, Server, Activity, Clock, Zap, Users } from "lucide-react";

interface ServerOverviewProps {
    onSelectDb: (db: string) => void;
    onRefreshSidebar: () => void;
}

interface ServerStatus {
    uptime: number;
    threads_connected: number;
    threads_running: number;
    queries: number;
    slow_queries: number;
}

export default function ServerOverview({ onSelectDb, onRefreshSidebar }: ServerOverviewProps) {
    const [databases, setDatabases] = useState<string[]>([]);
    const [status, setStatus] = useState<ServerStatus | null>(null);
    const [processes, setProcesses] = useState<any[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState("");
    const [droppingDb, setDroppingDb] = useState<string | null>(null);
    const [showConfirm, setShowConfirm] = useState<string | null>(null);
    const [newDbName, setNewDbName] = useState("");
    const [showCreateDb, setShowCreateDb] = useState(false);

    useEffect(() => {
        fetchServerData();
    }, []);

    const fetchServerData = async () => {
        setLoading(true);
        setError("");
        try {
            const dbRes = await fetch("/api/databases");
            const dbData = await dbRes.json();
            if (dbData.success) setDatabases(dbData.databases);

            const statusRes = await fetch("/api/server/status");
            const statusData = await statusRes.json();
            if (statusData.success) {
                setStatus(statusData.status);
                setProcesses(statusData.processes);
            }
        } catch {
            setError("Failed to fetch server information");
        } finally {
            setLoading(false);
        }
    };

    const formatUptime = (seconds: number) => {
        const days = Math.floor(seconds / 86400);
        const hours = Math.floor((seconds % 86400) / 3600);
        const mins = Math.floor((seconds % 3600) / 60);
        return `${days}d ${hours}h ${mins}m`;
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
                fetchServerData();
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
                fetchServerData();
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
            {/* Status Metrics Cards */}
            {status && (
                <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))", gap: "1rem", marginBottom: "2rem" }}>
                    <div className={styles.form} style={{ padding: "1rem", display: "flex", gap: "1rem", alignItems: "center" }}>
                        <div style={{ background: "rgba(56, 189, 248, 0.1)", padding: "10px", borderRadius: "8px" }}>
                            <Clock size={20} color="#38bdf8" />
                        </div>
                        <div>
                            <div style={{ fontSize: "0.75rem", color: "var(--text-muted)" }}>Uptime</div>
                            <div style={{ fontWeight: "bold" }}>{formatUptime(status.uptime)}</div>
                        </div>
                    </div>
                    <div className={styles.form} style={{ padding: "1rem", display: "flex", gap: "1rem", alignItems: "center" }}>
                        <div style={{ background: "rgba(16, 185, 129, 0.1)", padding: "10px", borderRadius: "8px" }}>
                            <Users size={20} color="#10b981" />
                        </div>
                        <div>
                            <div style={{ fontSize: "0.75rem", color: "var(--text-muted)" }}>Connections</div>
                            <div style={{ fontWeight: "bold" }}>{status.threads_connected}</div>
                        </div>
                    </div>
                    <div className={styles.form} style={{ padding: "1rem", display: "flex", gap: "1rem", alignItems: "center" }}>
                        <div style={{ background: "rgba(245, 158, 11, 0.1)", padding: "10px", borderRadius: "8px" }}>
                            <Activity size={20} color="#f59e0b" />
                        </div>
                        <div>
                            <div style={{ fontSize: "0.75rem", color: "var(--text-muted)" }}>Running</div>
                            <div style={{ fontWeight: "bold" }}>{status.threads_running}</div>
                        </div>
                    </div>
                    <div className={styles.form} style={{ padding: "1rem", display: "flex", gap: "1rem", alignItems: "center" }}>
                        <div style={{ background: "rgba(139, 92, 246, 0.1)", padding: "10px", borderRadius: "8px" }}>
                            <Zap size={20} color="#8b5cf6" />
                        </div>
                        <div>
                            <div style={{ fontSize: "0.75rem", color: "var(--text-muted)" }}>Total Queries</div>
                            <div style={{ fontWeight: "bold" }}>{status.queries.toLocaleString()}</div>
                        </div>
                    </div>
                </div>
            )}

            <div className={styles.count} style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <div>
                    <Database size={18} style={{ marginRight: 8, verticalAlign: "middle" }} />
                    Databases ({databases.length})
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

            <div className={styles.tableContainer} style={{ marginBottom: "3rem" }}>
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

            {/* Process List Section */}
            <div className={styles.count}>
                <Activity size={18} style={{ marginRight: 8, verticalAlign: "middle" }} />
                Active Processes ({processes.length})
            </div>
            <div className={styles.tableContainer}>
                <table className={styles.table}>
                    <thead>
                        <tr>
                            <th>ID</th>
                            <th>User</th>
                            <th>Host</th>
                            <th>DB</th>
                            <th>Command</th>
                            <th>Time</th>
                            <th>Status</th>
                            <th>Info</th>
                        </tr>
                    </thead>
                    <tbody>
                        {processes.map((proc) => (
                            <tr key={proc.Id}>
                                <td>{proc.Id}</td>
                                <td>{proc.User}</td>
                                <td title={proc.Host}>{proc.Host?.split(':')[0]}</td>
                                <td>{proc.db || <em style={{ color: "var(--text-muted)" }}>None</em>}</td>
                                <td>{proc.Command}</td>
                                <td>{proc.Time}s</td>
                                <td>{proc.State || "Init"}</td>
                                <td title={proc.Info} style={{ maxWidth: "200px", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                                    {proc.Info || <em style={{ color: "var(--text-muted)" }}>None</em>}
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
