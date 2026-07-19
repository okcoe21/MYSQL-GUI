

import { useState, useEffect } from "react";
import styles from "./dashboard.module.css";
import { AlertCircle, Loader2, Database, Trash2, Plus, Server, Activity, Clock, Zap, Users } from "lucide-react";
import * as api from "../../lib/api";

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
            const dbData = await api.listDatabases();
            if (dbData.success) setDatabases(dbData.databases || []);

            const statusData = await api.getServerStatus();
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
            const result = await api.createDatabase(newDbName);
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
            const result = await api.dropDatabase(db, true);
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
                <div className={styles.metricGrid}>
                    <div className={styles.metricCard}>
                        <div className={styles.metricIconWrapper}>
                            <Clock size={20} className={styles.tabIcon} />
                        </div>
                        <div>
                            <div className={styles.metricLabel}>Uptime</div>
                            <div className={styles.metricValue}>{formatUptime(status.uptime)}</div>
                        </div>
                    </div>
                    <div className={styles.metricCard}>
                        <div className={styles.metricIconWrapper}>
                            <Users size={20} className={styles.tabIcon} />
                        </div>
                        <div>
                            <div className={styles.metricLabel}>Connections</div>
                            <div className={styles.metricValue}>{status.threads_connected}</div>
                        </div>
                    </div>
                    <div className={styles.metricCard}>
                        <div className={styles.metricIconWrapper}>
                            <Activity size={20} className={styles.tabIcon} />
                        </div>
                        <div>
                            <div className={styles.metricLabel}>Running</div>
                            <div className={styles.metricValue}>{status.threads_running}</div>
                        </div>
                    </div>
                    <div className={styles.metricCard}>
                        <div className={styles.metricIconWrapper}>
                            <Zap size={20} className={styles.tabIcon} />
                        </div>
                        <div>
                            <div className={styles.metricLabel}>Total Queries</div>
                            <div className={styles.metricValue}>{status.queries.toLocaleString()}</div>
                        </div>
                    </div>
                </div>
            )}

            <div className={`${styles.count} ${styles.flexBetween}`}>
                <div className={`${styles.flexRow} ${styles.flexGapSm}`}>
                    <Database size={18} className={styles.tabIcon} />
                    Databases ({databases.length})
                </div>
                <button className={styles.actionBtn} onClick={() => setShowCreateDb(!showCreateDb)}>
                    <Plus size={14} className={styles.tabIcon} /> Create Database
                </button>
            </div>

            {showCreateDb && (
                <div className={`${styles.form} ${styles.marginBottomMd}`} style={{ padding: "var(--space-4)" }}>
                    <div className={`${styles.flexRow} ${styles.flexGapSm}`}>
                        <input
                            type="text"
                            placeholder="New database name"
                            className={styles.formInput}
                            value={newDbName}
                            onChange={(e) => setNewDbName(e.target.value)}
                            onKeyDown={(e) => e.key === "Enter" && handleCreateDb()}
                            autoFocus
                        />
                        <button className={styles.primaryBtn} onClick={handleCreateDb}>
                            Create
                        </button>
                        <button className={styles.actionBtn} onClick={() => setShowCreateDb(false)}>Cancel</button>
                    </div>
                </div>
            )}

            {error && (
                <div className={styles.warning} style={{ borderColor: "#ef4444", color: "#ef4444", marginBottom: "var(--space-4)" }}>
                    <AlertCircle size={16} className={styles.tabIcon} />
                    {error}
                </div>
            )}

            {showConfirm && (
                <div className={styles.warning} style={{ marginBottom: "var(--space-4)", border: "1px solid #ef4444", backgroundColor: "var(--bg-base)" }}>
                    <div className={`${styles.flexRow} ${styles.flexGapSm}`}>
                        <AlertCircle size={20} color="#ef4444" />
                        <strong style={{ color: "#ef4444" }}>Confirm Database Drop</strong>
                    </div>
                    <p style={{ margin: "var(--space-2) 0", color: "var(--foreground)" }}>
                        Are you sure you want to <strong>DROP</strong> the entire database <code>{showConfirm}</code>?
                        This Action is irreversible.
                    </p>
                    <div className={styles.formFooter} style={{ marginTop: "var(--space-4)", justifyContent: "flex-start" }}>
                        <button className={styles.dangerBtn} onClick={() => handleDropDatabase(showConfirm, true)}>
                            Drop Database
                        </button>
                        <button className={styles.actionBtn} onClick={() => setShowConfirm(null)}>Cancel</button>
                    </div>
                </div>
            )}

            <div className={styles.tableContainer} style={{ marginBottom: "var(--space-6)" }}>
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
                                    style={{ cursor: "pointer" }}
                                    className={styles.linkText}
                                >
                                    <div className={`${styles.flexRow} ${styles.flexGapSm}`}>
                                        <Database size={16} />
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
                <Activity size={18} className={styles.tabIcon} />
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
