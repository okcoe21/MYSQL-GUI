"use client";

import { useState, useEffect } from "react";
import styles from "./dashboard.module.css";
import { Database, Loader2, AlertCircle, Play, Sparkles, CheckCircle } from "lucide-react";

interface MockDataViewProps {
    database: string;
    table: string;
    onSuccess: () => void;
}

export default function MockDataView({ database, table, onSuccess }: MockDataViewProps) {
    const [columns, setColumns] = useState<any[]>([]);
    const [blueprint, setBlueprint] = useState<Record<string, string>>({});
    const [count, setCount] = useState(50);
    const [loading, setLoading] = useState(false);
    const [generating, setGenerating] = useState(false);
    const [error, setError] = useState("");
    const [success, setSuccess] = useState("");

    useEffect(() => {
        fetchStructure();
    }, [database, table]);

    const fetchStructure = async () => {
        setLoading(true);
        setError("");
        try {
            const res = await fetch(`/api/structure?database=${database}&table=${table}`);
            const result = await res.json();
            if (result.success) {
                setColumns(result.structure);
                // Initialize blueprint
                const initial: Record<string, string> = {};
                result.structure.forEach((col: any) => {
                    if (col.Extra !== "auto_increment") {
                        if (col.Type.includes("int")) {
                            initial[col.Field] = "integer";
                        } else if (col.Type.includes("date")) {
                            initial[col.Field] = "date";
                        } else if (col.Type.includes("datetime") || col.Type.includes("timestamp")) {
                            initial[col.Field] = "datetime";
                        } else {
                            initial[col.Field] = "text";
                        }
                    }
                });
                setBlueprint(initial);
            } else {
                setError(result.error);
            }
        } catch {
            setError("Failed to fetch table structure");
        } finally {
            setLoading(false);
        }
    };

    const handleGenerate = async () => {
        setGenerating(true);
        setError("");
        setSuccess("");
        try {
            const res = await fetch("/api/data/generate", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ database, table, count, blueprint }),
            });
            const result = await res.json();
            if (result.success) {
                setSuccess(result.message);
                onSuccess();
            } else {
                setError(result.error);
            }
        } catch {
            setError("Failed to generate mock data");
        } finally {
            setGenerating(false);
        }
    };

    if (loading) {
        return (
            <div className={styles.loading}>
                <Loader2 className="animate-spin" size={24} style={{ marginRight: 8 }} />
                Loading table schema...
            </div>
        );
    }

    return (
        <div className={styles.wrapper}>
            <div className={styles.count}>
                Table: <strong>{table}</strong> &raquo; Mock Data Generator
            </div>

            <div className={styles.form} style={{ padding: "1.5rem" }}>
                <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "1.5rem", color: "var(--primary)" }}>
                    <Sparkles size={20} />
                    <h3 style={{ margin: 0 }}>Configure Generators</h3>
                </div>

                <div className={styles.tableContainer} style={{ marginBottom: "1.5rem" }}>
                    <table className={styles.table}>
                        <thead>
                            <tr>
                                <th>Column</th>
                                <th>Type</th>
                                <th>Generator</th>
                            </tr>
                        </thead>
                        <tbody>
                            {columns.filter(c => c.Extra !== "auto_increment").map((col) => (
                                <tr key={col.Field}>
                                    <td><strong>{col.Field}</strong></td>
                                    <td><code style={{ fontSize: "0.75rem" }}>{col.Type}</code></td>
                                    <td>
                                        <select
                                            className={styles.formInput}
                                            value={blueprint[col.Field]}
                                            onChange={(e) => setBlueprint(prev => ({ ...prev, [col.Field]: e.target.value }))}
                                            style={{ padding: "4px 8px" }}
                                        >
                                            <option value="text">Random Text</option>
                                            <option value="name">Full Name</option>
                                            <option value="email">Email Address</option>
                                            <option value="phone">Phone Number</option>
                                            <option value="date">Date</option>
                                            <option value="datetime">DateTime</option>
                                            <option value="integer">Integer</option>
                                        </select>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>

                <div style={{ display: "flex", alignItems: "center", gap: "1rem", marginBottom: "1.5rem" }}>
                    <label style={{ fontSize: "0.9rem" }}>Number of rows to generate:</label>
                    <input
                        type="number"
                        className={styles.formInput}
                        style={{ width: "100px" }}
                        value={count}
                        onChange={(e) => setCount(parseInt(e.target.value) || 0)}
                        min="1"
                        max="1000"
                    />
                </div>

                {error && (
                    <div className={styles.warning} style={{ color: "#ef4444", borderColor: "#ef4444", marginBottom: "1rem" }}>
                        <AlertCircle size={16} style={{ marginRight: 8, verticalAlign: "middle" }} />
                        {error}
                    </div>
                )}

                {success && (
                    <div className={styles.warning} style={{ color: "#10b981", borderColor: "#10b981", marginBottom: "1rem", backgroundColor: "rgba(16, 185, 129, 0.05)" }}>
                        <CheckCircle size={16} style={{ marginRight: 8, verticalAlign: "middle" }} />
                        {success}
                    </div>
                )}

                <div style={{ display: "flex", justifyContent: "flex-end" }}>
                    <button
                        className={styles.primaryBtn}
                        onClick={handleGenerate}
                        disabled={generating || Object.keys(blueprint).length === 0}
                    >
                        {generating ? <Loader2 className="animate-spin" size={16} style={{ marginRight: 8 }} /> : <Play size={16} style={{ marginRight: 8 }} />}
                        Generate Data
                    </button>
                </div>
            </div>
        </div>
    );
}
