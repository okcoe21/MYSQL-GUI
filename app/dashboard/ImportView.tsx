"use client";

import { useState } from "react";
import styles from "./dashboard.module.css";
import { Upload, Loader2, Database, AlertCircle, CheckCircle, XCircle } from "lucide-react";

interface ImportViewProps {
    database: string;
    onSuccess: () => void;
}

export default function ImportView({ database, onSuccess }: ImportViewProps) {
    const [file, setFile] = useState<File | null>(null);
    const [loading, setLoading] = useState(false);
    const [result, setResult] = useState<{ message: string; errors?: any[] } | null>(null);
    const [error, setError] = useState("");

    const handleImport = async () => {
        if (!file) return;

        setLoading(true);
        setError("");
        setResult(null);

        const formData = new FormData();
        formData.append("database", database);
        formData.append("file", file);

        try {
            const res = await fetch("/api/import", {
                method: "POST",
                body: formData,
            });
            const data = await res.json();

            if (data.success) {
                setResult(data);
                onSuccess();
            } else {
                setError(data.error);
            }
        } catch {
            setError("Failed to upload and import SQL file");
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className={styles.wrapper}>
            <div className={styles.count}>
                <div>
                    Database: <strong>{database}</strong> &raquo; Import
                </div>
            </div>

            <div className={styles.form} style={{ padding: "2rem" }}>
                <div style={{ textAlign: "center", marginBottom: "2rem" }}>
                    <Upload size={48} color="#38bdf8" style={{ marginBottom: "1rem" }} />
                    <h2 style={{ fontSize: "1.5rem", marginBottom: "0.5rem" }}>Import SQL Script</h2>
                    <p style={{ color: "#94a3b8" }}>
                        Upload a <code>.sql</code> file to execute commands against <strong>{database}</strong>.
                    </p>
                </div>

                {!result ? (
                    <>
                        <div style={{ marginBottom: "1.5rem" }}>
                            <input
                                type="file"
                                accept=".sql"
                                style={{ display: "none" }}
                                id="sql-upload"
                                onChange={(e) => setFile(e.target.files?.[0] || null)}
                            />
                            <label
                                htmlFor="sql-upload"
                                className={styles.tableContainer}
                                style={{
                                    display: "flex",
                                    flexDirection: "column",
                                    alignItems: "center",
                                    padding: "3rem",
                                    cursor: "pointer",
                                    borderStyle: "dashed",
                                    borderColor: file ? "#38bdf8" : "#334155"
                                }}
                            >
                                <Upload size={32} style={{ color: file ? "#38bdf8" : "#475569", marginBottom: "1rem" }} />
                                <span style={{ color: file ? "#f8fafc" : "#94a3b8" }}>
                                    {file ? file.name : "Click to select a .sql file"}
                                </span>
                            </label>
                        </div>

                        {error && (
                            <div className={styles.warning} style={{ marginBottom: "1.5rem", color: "#ef4444", borderColor: "#ef4444" }}>
                                <AlertCircle size={16} style={{ marginRight: 8, verticalAlign: "middle" }} />
                                {error}
                            </div>
                        )}

                        <div className={styles.warning} style={{ marginBottom: "2rem", backgroundColor: "rgba(245, 158, 11, 0.05)" }}>
                            <p style={{ margin: 0, fontSize: "0.8125rem" }}>
                                <strong>Warning:</strong> Importing SQL can overwrite existing data or modify table structures.
                                Please ensure you have a backup of your target database.
                            </p>
                        </div>

                        <div style={{ display: "flex", justifyContent: "center" }}>
                            <button
                                className={styles.primaryBtn}
                                onClick={handleImport}
                                disabled={loading || !file}
                                style={{ padding: "0.75rem 2.5rem", fontSize: "1rem" }}
                            >
                                {loading ? <Loader2 className="animate-spin" size={20} style={{ marginRight: 8 }} /> : null}
                                {loading ? "Importing Data..." : "Run Import"}
                            </button>
                        </div>
                    </>
                ) : (
                    <div className={styles.successMessage} style={{ backgroundColor: "rgba(16, 185, 129, 0.05)", border: "1px solid #10b981", padding: "1.5rem" }}>
                        <div style={{ display: "flex", alignItems: "center", gap: "12px", marginBottom: "1rem" }}>
                            <CheckCircle size={24} color="#10b981" />
                            <strong style={{ color: "#10b981", fontSize: "1.125rem" }}>Import Complete</strong>
                        </div>
                        <p style={{ color: "#f8fafc", marginBottom: "1rem" }}>{result.message}</p>

                        {result.errors && result.errors.length > 0 && (
                            <div style={{ marginTop: "1rem" }}>
                                <p style={{ color: "#ef4444", fontWeight: "bold", fontSize: "0.875rem", marginBottom: "0.5rem" }}>
                                    Errors encountered ({result.errors.length}):
                                </p>
                                <div className={styles.tableContainer} style={{ maxHeight: "200px", padding: "0.5rem" }}>
                                    {result.errors.map((err, i) => (
                                        <div key={i} style={{ marginBottom: "8px", fontSize: "0.75rem", borderBottom: "1px solid #334155", paddingBottom: "4px" }}>
                                            <code style={{ color: "#94a3b8" }}>{err.statement}</code>
                                            <div style={{ color: "#ef4444" }}>{err.error}</div>
                                        </div>
                                    ))}
                                </div>
                            </div>
                        )}

                        <div style={{ marginTop: "1.5rem", display: "flex", justifyContent: "center" }}>
                            <button className={styles.actionBtn} onClick={() => setResult(null)}>
                                Import Another File
                            </button>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}
