"use client";

import { useState, useEffect } from "react";
import styles from "./dashboard.module.css";
import { AlertCircle, Loader2, Code, Info } from "lucide-react";

interface ObjectViewProps {
    database: string;
    name: string;
    type: "view" | "procedure" | "function";
}

export default function ObjectView({ database, name, type }: ObjectViewProps) {
    const [data, setData] = useState<any>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState("");

    useEffect(() => {
        fetchObjectData();
    }, [database, name, type]);

    const fetchObjectData = async () => {
        setLoading(true);
        setError("");
        try {
            const res = await fetch(`/api/objects?database=${database}&type=${type}s`);
            const result = await res.json();
            if (result.success) {
                const found = result.data.find((item: any) =>
                    (item.TABLE_NAME || item.ROUTINE_NAME) === name
                );
                setData(found);
            } else {
                setError(result.error);
            }
        } catch {
            setError(`Failed to fetch ${type} details`);
        } finally {
            setLoading(false);
        }
    };

    if (loading) {
        return (
            <div className={styles.loading}>
                <Loader2 className="animate-spin" size={24} style={{ marginRight: 8 }} />
                Loading {type} details...
            </div>
        );
    }

    if (error) {
        return (
            <div className={styles.error}>
                <AlertCircle size={24} style={{ marginRight: 8 }} />
                {error}
            </div>
        );
    }

    if (!data) return null;

    const definition = data.VIEW_DEFINITION || data.ROUTINE_DEFINITION;

    return (
        <div className={styles.wrapper}>
            <div className={styles.count} style={{ display: "flex", justifyContent: "space-between" }}>
                <div>
                    {type.charAt(0).toUpperCase() + type.slice(1)}: <strong>{name}</strong>
                </div>
                {data.DATA_TYPE && <div className={styles.linkText}>Return Type: {data.DATA_TYPE}</div>}
            </div>

            <div className={styles.form} style={{ padding: "1.5rem" }}>
                <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "1rem", color: "var(--primary)" }}>
                    <Code size={18} />
                    <h3 style={{ margin: 0 }}>Definition</h3>
                </div>
                <code style={{
                    display: "block",
                    padding: "1rem",
                    background: "var(--input-bg)",
                    borderRadius: "8px",
                    border: "1px solid var(--border)",
                    whiteSpace: "pre-wrap",
                    fontSize: "0.9rem",
                    fontFamily: "monospace",
                    maxHeight: "60vh",
                    overflowY: "auto"
                }}>
                    {definition}
                </code>
            </div>

            {type === "view" && (
                <div className={styles.warning} style={{ marginTop: "1rem", border: "1px solid var(--border)" }}>
                    <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                        <Info size={16} />
                        <strong>Properties</strong>
                    </div>
                    <p style={{ margin: "0.5rem 0", fontSize: "0.85rem" }}>
                        Updatable: <span style={{ color: data.IS_UPDATABLE === "YES" ? "#10b981" : "#ef4444" }}>{data.IS_UPDATABLE}</span>
                    </p>
                </div>
            )}
        </div>
    );
}
