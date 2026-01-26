"use client";

import { useState, useEffect } from "react";
import styles from "./dashboard.module.css";
import { Clock, Loader2, AlertCircle, RefreshCw, Terminal, Info } from "lucide-react";

export default function SlowQueryLog() {
    const [logs, setLogs] = useState<any[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState("");
    const [setupRequired, setSetupRequired] = useState(false);

    const fetchLogs = async () => {
        setLoading(true);
        setError("");
        setSetupRequired(false);
        try {
            const res = await fetch("/api/server/slow-log");
            const data = await res.json();
            if (data.success) {
                setLogs(data.logs);
            } else {
                setError(data.error);
                if (data.setupRequired) setSetupRequired(true);
            }
        } catch {
            setError("Failed to fetch slow query logs");
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchLogs();
    }, []);

    if (loading) {
        return (
            <div className={styles.loading}>
                <Loader2 className="animate-spin" size={24} style={{ marginRight: 8 }} />
                Analyzing slow queries...
            </div>
        );
    }

    if (setupRequired) {
        return (
            <div className={styles.wrapper}>
                <div className={styles.count}>
                    <Clock size={18} style={{ marginRight: 8, verticalAlign: "middle" }} />
                    Slow Query Analysis
                </div>
                <div className={styles.form} style={{ padding: "2rem", textAlign: "center" }}>
                    <Info size={48} color="#f59e0b" style={{ marginBottom: "1rem" }} />
                    <h3>Logging to Table Disabled</h3>
                    <p style={{ color: "var(--text-muted)", maxWidth: "500px", margin: "1rem auto" }}>
                        To view slow queries, MySQL must be configured to log outputs to a table and the slow_query_log must be enabled.
                    </p>
                    <div className={styles.tableContainer} style={{ background: "#000", padding: "1rem", borderRadius: "8px", textAlign: "left", display: "inline-block" }}>
                        <code style={{ color: "#10b981", fontSize: "0.85rem" }}>
                            SET GLOBAL log_output = 'TABLE';<br />
                            SET GLOBAL slow_query_log = 'ON';
                        </code>
                    </div>
                    <div style={{ marginTop: "2rem" }}>
                        <button className={styles.primaryBtn} onClick={fetchLogs}>
                            <RefreshCw size={16} style={{ marginRight: 8 }} /> Check Configuration
                        </button>
                    </div>
                </div>
            </div>
        );
    }

    return (
        <div className={styles.wrapper}>
            <div className={styles.count} style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <div>
                    <Clock size={18} style={{ marginRight: 8, verticalAlign: "middle" }} />
                    Slow Query Analysis (Last 100)
                </div>
                <button className={styles.actionBtn} onClick={fetchLogs}>
                    <RefreshCw size={14} style={{ marginRight: 8 }} /> Refresh
                </button>
            </div>

            {error && (
                <div className={styles.warning} style={{ color: "#ef4444", borderColor: "#ef4444", marginBottom: "1rem" }}>
                    <AlertCircle size={16} style={{ marginRight: 8, verticalAlign: "middle" }} />
                    {error}
                </div>
            )}

            <div className={styles.tableContainer}>
                <table className={styles.table}>
                    <thead>
                        <tr>
                            <th>Execution Time</th>
                            <th>Query Time</th>
                            <th>Lock Time</th>
                            <th>Rows Searched</th>
                            <th>SQL Query</th>
                        </tr>
                    </thead>
                    <tbody>
                        {logs.length === 0 ? (
                            <tr>
                                <td colSpan={5} style={{ textAlign: "center", padding: "2rem", color: "var(--text-muted)" }}>
                                    No slow queries recorded yet.
                                </td>
                            </tr>
                        ) : (
                            logs.map((log, i) => (
                                <tr key={i}>
                                    <td style={{ fontSize: "0.8rem", whiteSpace: "nowrap" }}>{new Date(log.start_time).toLocaleString()}</td>
                                    <td><span style={{ color: "#f59e0b", fontWeight: "bold" }}>{log.query_time}s</span></td>
                                    <td>{log.lock_time}s</td>
                                    <td>{log.rows_examined}</td>
                                    <td style={{ maxWidth: "400px" }}>
                                        <div style={{
                                            background: "rgba(0,0,0,0.2)",
                                            padding: "6px",
                                            borderRadius: "4px",
                                            fontFamily: "monospace",
                                            fontSize: "0.75rem",
                                            overflowX: "auto"
                                        }}>
                                            {log.sql_text}
                                        </div>
                                    </td>
                                </tr>
                            ))
                        )}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
