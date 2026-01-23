"use client";

import { useState } from "react";
import styles from "./dashboard.module.css";
import { Play, AlertTriangle, CheckCircle } from "lucide-react";

interface SqlEditorProps {
    initialDatabase?: string;
}

interface QueryResult {
    success: boolean;
    data?: any[];
    columns?: string[];
    affectedRows?: number | null;
    message?: string;
    error?: string;
}

export default function SqlEditor({ initialDatabase }: SqlEditorProps) {
    const [query, setQuery] = useState("");
    const [loading, setLoading] = useState(false);
    const [results, setResults] = useState<QueryResult | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [showWarning, setShowWarning] = useState(false);

    const runQuery = async (confirmed = false) => {
        setLoading(true);
        setError(null);
        setShowWarning(false);

        try {
            const res = await fetch("/api/query", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    query,
                    database: initialDatabase,
                    confirmed
                }),
            });

            const data: QueryResult = await res.json();

            if (data.success) {
                setResults(data);
            } else if (data.error === "DESTRUCTIVE_QUERY") {
                setShowWarning(true);
                setError(data.message || "Destructive query detected");
            } else {
                setError(data.error || "Failed to execute query");
            }
        } catch {
            setError("Failed to execute query");
        } finally {
            setLoading(false);
        }
    };

    const handleClear = () => {
        setQuery("");
        setResults(null);
        setError(null);
    };

    const handleFormat = () => {
        // Simple mock formatter: uppercase keywords
        const keywords = ["SELECT", "FROM", "WHERE", "GROUP BY", "ORDER BY", "LIMIT", "INSERT INTO", "VALUES", "UPDATE", "SET", "DELETE", "JOIN", "LEFT JOIN", "RIGHT JOIN", "ON", "AND", "OR", "IN"];
        let formatted = query;
        keywords.forEach(kw => {
            const regex = new RegExp(`\\b${kw}\\b`, "gi");
            formatted = formatted.replace(regex, kw);
        });
        setQuery(formatted);
    };

    return (
        <div className={styles.container}>
            <div className={styles.editorGroup}>
                <textarea
                    className={styles.editor}
                    value={query}
                    onChange={(e) => setQuery(e.target.value)}
                    placeholder="Enter your SQL query here... (e.g. SELECT * FROM users LIMIT 10)"
                    spellCheck={false}
                />
                <div className={styles.controls}>
                    <div style={{ display: "flex", gap: "8px" }}>
                        <button className={styles.actionBtn} onClick={handleClear} title="Clear Query">
                            Clear
                        </button>
                        <button className={styles.actionBtn} onClick={handleFormat} title="Format Query">
                            Format
                        </button>
                    </div>
                    <button
                        className={styles.primaryBtn}
                        onClick={() => runQuery(false)}
                        disabled={loading || !query.trim()}
                        style={{ padding: "0.4rem 1.5rem" }}
                    >
                        <Play size={16} style={{ marginRight: 8, verticalAlign: "middle" }} />
                        Run Query
                    </button>
                </div>
            </div>

            {showWarning && (
                <div className={styles.warning}>
                    <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                        <AlertTriangle size={20} />
                        <strong>Warning: Destructive Query Detected</strong>
                    </div>
                    <p style={{ margin: "0.5rem 0" }}>{error}</p>
                    <div className={styles.warningControls}>
                        <button className={styles.confirmBtn} onClick={() => runQuery(true)}>Yes, Execute</button>
                        <button className={styles.cancelBtn} onClick={() => { setShowWarning(false); setError(null); }}>Cancel</button>
                    </div>
                </div>
            )}

            {!showWarning && error && (
                <div className={styles.warning} style={{ borderColor: "#ef4444", color: "#ef4444", backgroundColor: "rgba(239, 68, 68, 0.1)" }}>
                    {error}
                </div>
            )}

            {results && results.success && (
                <div className={styles.resultsArea}>
                    {results.affectedRows !== undefined && results.affectedRows !== null ? (
                        <div className={styles.successMessage}>
                            <CheckCircle size={16} style={{ marginRight: 8, verticalAlign: "middle" }} />
                            Query successful. Affected rows: {results.affectedRows}
                        </div>
                    ) : (
                        <div className={styles.tableContainer}>
                            <table className={styles.table}>
                                <thead>
                                    <tr>
                                        {results.columns?.map((col: string) => (
                                            <th key={col}>{col}</th>
                                        ))}
                                    </tr>
                                </thead>
                                <tbody>
                                    {!results.data || results.data.length === 0 ? (
                                        <tr>
                                            <td colSpan={results.columns?.length || 1} style={{ textAlign: "center", padding: "2rem" }}>
                                                Query returned no results
                                            </td>
                                        </tr>
                                    ) : (
                                        results.data.map((row: any, idx: number) => (
                                            <tr key={idx}>
                                                {results.columns?.map((col: string) => (
                                                    <td key={col}>{row[col] === null ? "NULL" : String(row[col])}</td>
                                                ))}
                                            </tr>
                                        ))
                                    )}
                                </tbody>
                            </table>
                        </div>
                    )}
                </div>
            )}
        </div>
    );
}
