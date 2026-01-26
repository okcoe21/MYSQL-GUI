"use client";

import { useState, useEffect } from "react";
import { History, Play, Trash2, Star, Clock } from "lucide-react";
import styles from "./dashboard.module.css";

interface HistoryItem {
    query: string;
    timestamp: number;
    database?: string;
    isFavorite?: boolean;
}

interface HistoryViewProps {
    onRunQuery: (query: string) => void;
}

export default function HistoryView({ onRunQuery }: HistoryViewProps) {
    const [history, setHistory] = useState<HistoryItem[]>([]);

    useEffect(() => {
        const saved = localStorage.getItem("sql_history");
        if (saved) {
            setHistory(JSON.parse(saved));
        }
    }, []);

    const saveHistory = (newHistory: HistoryItem[]) => {
        setHistory(newHistory);
        localStorage.setItem("sql_history", JSON.stringify(newHistory));
    };

    const toggleFavorite = (index: number) => {
        const newHistory = [...history];
        newHistory[index].isFavorite = !newHistory[index].isFavorite;
        saveHistory(newHistory);
    };

    const deleteItem = (index: number) => {
        const newHistory = history.filter((_, i) => i !== index);
        saveHistory(newHistory);
    };

    const clearHistory = () => {
        if (confirm("Clear all history?")) {
            saveHistory([]);
        }
    };

    return (
        <div className={styles.wrapper}>
            <div className={styles.count}>
                <span>Previous Queries ({history.length})</span>
                {history.length > 0 && (
                    <button className={styles.actionBtn} onClick={clearHistory}>
                        Clear All
                    </button>
                )}
            </div>

            {history.length === 0 ? (
                <div className={styles.emptyState}>
                    <History size={48} className={styles.emptyIcon} />
                    <p>No query history yet.</p>
                </div>
            ) : (
                <div className={styles.tableContainer} style={{ background: "transparent", border: "none" }}>
                    <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
                        {history.sort((a, b) => (b.isFavorite ? 1 : 0) - (a.isFavorite ? 1 : 0) || b.timestamp - a.timestamp).map((item, idx) => (
                            <div key={item.timestamp} className={styles.form} style={{ padding: "1rem", maxWidth: "100%" }}>
                                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: "8px" }}>
                                    <div style={{ display: "flex", alignItems: "center", gap: "8px", color: "var(--text-muted)", fontSize: "0.75rem" }}>
                                        <Clock size={12} />
                                        {new Date(item.timestamp).toLocaleString()}
                                        {item.database && <span> • DB: {item.database}</span>}
                                    </div>
                                    <div style={{ display: "flex", gap: "8px" }}>
                                        <button
                                            className={styles.themeBtn}
                                            onClick={() => toggleFavorite(idx)}
                                            style={{ color: item.isFavorite ? "var(--primary)" : "var(--text-muted)" }}
                                        >
                                            <Star size={16} fill={item.isFavorite ? "currentColor" : "none"} />
                                        </button>
                                        <button className={styles.themeBtn} onClick={() => deleteItem(idx)}>
                                            <Trash2 size={16} />
                                        </button>
                                        <button className={styles.primaryBtn} onClick={() => onRunQuery(item.query)} style={{ padding: "4px 12px", fontSize: "0.75rem" }}>
                                            <Play size={12} style={{ marginRight: 4 }} />
                                            Re-run
                                        </button>
                                    </div>
                                </div>
                                <code style={{
                                    display: "block",
                                    padding: "8px",
                                    background: "var(--input-bg)",
                                    borderRadius: "4px",
                                    fontSize: "0.85rem",
                                    whiteSpace: "pre-wrap",
                                    border: "1px solid var(--border)"
                                }}>
                                    {item.query}
                                </code>
                            </div>
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
}
