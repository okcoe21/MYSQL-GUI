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
    const [format, setFormat] = useState("sql");
    const [includeStructure, setIncludeStructure] = useState(true);
    const [includeData, setIncludeData] = useState(true);

    const handleExport = async () => {
        setLoading(true);
        setError("");
        try {
            const query = new URLSearchParams({
                database,
                format,
                structure: includeStructure.toString(),
                data: includeData.toString()
            }).toString();

            window.location.href = `/api/export?${query}`;

            setTimeout(() => setLoading(false), 2000);
        } catch {
            setError("Failed to initiate export");
            setLoading(false);
        }
    };

    return (
        <div className={styles.wrapper}>
            <div className={styles.count}>
                Database: <strong>{database}</strong> &raquo; Export Options
            </div>

            <div className={styles.form} style={{ padding: "2rem", maxWidth: "800px", margin: "0 auto" }}>
                <div style={{ textAlign: "center", marginBottom: "2rem" }}>
                    <Download size={48} color="#38bdf8" style={{ marginBottom: "1rem" }} />
                    <h2 style={{ fontSize: "1.5rem", marginBottom: "0.5rem" }}>Export Settings</h2>
                </div>

                {error && (
                    <div className={styles.warning} style={{ marginBottom: "1.5rem", color: "#ef4444", borderColor: "#ef4444" }}>
                        <AlertCircle size={16} style={{ marginRight: 8, verticalAlign: "middle" }} />
                        {error}
                    </div>
                )}

                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "2rem", marginBottom: "2rem" }}>
                    <div className={styles.tableContainer} style={{ padding: "1.5rem" }}>
                        <h4 style={{ marginBottom: "1rem", color: "var(--primary)" }}>File Format</h4>
                        <div style={{ display: "flex", flexDirection: "column", gap: "0.75rem" }}>
                            {["sql", "json", "csv"].map(f => (
                                <label key={f} style={{ display: "flex", alignItems: "center", gap: "10px", cursor: "pointer" }}>
                                    <input
                                        type="radio"
                                        name="format"
                                        value={f}
                                        checked={format === f}
                                        onChange={(e) => setFormat(e.target.value)}
                                    />
                                    <span style={{ textTransform: "uppercase", fontWeight: format === f ? "bold" : "normal" }}>{f}</span>
                                </label>
                            ))}
                        </div>
                    </div>

                    <div className={styles.tableContainer} style={{ padding: "1.5rem" }}>
                        <h4 style={{ marginBottom: "1rem", color: "var(--primary)" }}>Content</h4>
                        <div style={{ display: "flex", flexDirection: "column", gap: "0.75rem" }}>
                            <label style={{ display: "flex", alignItems: "center", gap: "10px", cursor: "pointer" }}>
                                <input
                                    type="checkbox"
                                    checked={includeStructure}
                                    onChange={(e) => setIncludeStructure(e.target.checked)}
                                    disabled={format !== "sql"}
                                />
                                <span>Include Structure (CREATE TABLE)</span>
                            </label>
                            <label style={{ display: "flex", alignItems: "center", gap: "10px", cursor: "pointer" }}>
                                <input
                                    type="checkbox"
                                    checked={includeData}
                                    onChange={(e) => setIncludeData(e.target.checked)}
                                />
                                <span>Include Data (INSERT statements / rows)</span>
                            </label>
                        </div>
                    </div>
                </div>

                <div style={{ display: "flex", justifyContent: "center" }}>
                    <button
                        className={styles.primaryBtn}
                        onClick={handleExport}
                        disabled={loading || (!includeStructure && !includeData)}
                        style={{ padding: "0.75rem 3rem", fontSize: "1rem" }}
                    >
                        {loading ? <Loader2 className="animate-spin" size={20} style={{ marginRight: 8 }} /> : <FileText size={20} style={{ marginRight: 8 }} />}
                        {loading ? "Preparing Export..." : `Download ${format.toUpperCase()} Export`}
                    </button>
                </div>
            </div>
        </div>
    );
}
