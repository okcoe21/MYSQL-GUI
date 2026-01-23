"use client";

import { useState } from "react";
import styles from "./dashboard.module.css";
import { Download, Loader2, Database, AlertCircle, FileText } from "lucide-react";

interface ExportViewProps {
    database: string;
}

export default function ExportView({ database }: ExportViewProps) {
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState("");

    const handleExport = async () => {
        setLoading(true);
        setError("");
        try {
            // We just trigger a direct download via window.location
            window.location.href = `/api/export?database=${encodeURIComponent(database)}`;

            // Wait a bit to reset loading
            setTimeout(() => setLoading(false), 2000);
        } catch {
            setError("Failed to initiate export");
            setLoading(false);
        }
    };

    return (
        <div className={styles.wrapper}>
            <div className={styles.count}>
                <div>
                    Database: <strong>{database}</strong> &raquo; Export
                </div>
            </div>

            <div className={styles.form} style={{ padding: "2rem" }}>
                <div style={{ textAlign: "center", marginBottom: "2rem" }}>
                    <Download size={48} color="#38bdf8" style={{ marginBottom: "1rem" }} />
                    <h2 style={{ fontSize: "1.5rem", marginBottom: "0.5rem" }}>Export Database Dump</h2>
                    <p style={{ color: "#94a3b8" }}>
                        Download a complete SQL file containing both the <strong>schema (structure)</strong>
                        and <strong>data</strong> of all tables in <code>{database}</code>.
                    </p>
                </div>

                {error && (
                    <div className={styles.warning} style={{ marginBottom: "1.5rem", color: "#ef4444", borderColor: "#ef4444" }}>
                        <AlertCircle size={16} style={{ marginRight: 8, verticalAlign: "middle" }} />
                        {error}
                    </div>
                )}

                <div className={styles.warning} style={{ marginBottom: "2rem" }}>
                    <ul style={{ paddingLeft: "1.5rem", margin: 0 }}>
                        <li>Exports will include <code>DROP TABLE IF EXISTS</code> commands.</li>
                        <li>Format will be standard <code>.sql</code>.</li>
                        <li>Large databases may take a few moments to generate.</li>
                    </ul>
                </div>

                <div style={{ display: "flex", justifyContent: "center" }}>
                    <button
                        className={styles.primaryBtn}
                        onClick={handleExport}
                        disabled={loading}
                        style={{ padding: "0.75rem 2.5rem", fontSize: "1rem" }}
                    >
                        {loading ? <Loader2 className="animate-spin" size={20} style={{ marginRight: 8 }} /> : <FileText size={20} style={{ marginRight: 8 }} />}
                        {loading ? "Generating Dump..." : "Export as SQL File"}
                    </button>
                </div>
            </div>
        </div>
    );
}
