"use client";

import { useState } from "react";
import styles from "./dashboard.module.css";
import { AlertCircle, Loader2, Trash2, Database, Table as TableIcon } from "lucide-react";

interface OperationsViewProps {
    database: string;
    table: string;
    onTableDropped: () => void;
}

export default function OperationsView({ database, table, onTableDropped }: OperationsViewProps) {
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState("");
    const [showConfirm, setShowConfirm] = useState(false);

    const handleDropTable = async (confirmed = false) => {
        if (!confirmed) {
            setShowConfirm(true);
            return;
        }

        setLoading(true);
        setError("");
        try {
            const res = await fetch("/api/tables/drop", {
                method: "DELETE",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ database, table, confirmed: true }),
            });

            const result = await res.json();
            if (result.success) {
                onTableDropped();
            } else {
                setError(result.error);
            }
        } catch {
            setError("Failed to drop table");
        } finally {
            setLoading(false);
            setShowConfirm(false);
        }
    };

    return (
        <div className={styles.wrapper}>
            <div className={styles.count}>
                Operations for: <strong>{database}.{table}</strong>
            </div>

            {error && (
                <div className={styles.warning} style={{ borderColor: "#ef4444", color: "#ef4444", marginBottom: "1rem" }}>
                    <AlertCircle size={16} style={{ marginRight: 8, verticalAlign: "middle" }} />
                    {error}
                </div>
            )}

            <div className={styles.form} style={{ border: "1px solid #ef4444", backgroundColor: "rgba(239, 68, 68, 0.05)" }}>
                <h3 style={{ color: "#ef4444", marginBottom: "1rem", display: "flex", alignItems: "center", gap: 8 }}>
                    <Trash2 size={20} /> Danger Zone
                </h3>

                <p style={{ fontSize: "0.875rem", color: "#94a3b8", marginBottom: "1.5rem" }}>
                    The following actions are destructive and cannot be undone. Please proceed with caution.
                </p>

                <div className={styles.fieldRow}>
                    <div className={styles.fieldLabel}>
                        Drop the table (DROP)
                    </div>
                    <div className={styles.fieldInputWrapper}>
                        <button
                            className={styles.dangerBtn}
                            onClick={() => handleDropTable(false)}
                            disabled={loading}
                        >
                            Drop the table
                        </button>
                    </div>
                </div>
            </div>

            {showConfirm && (
                <div className={styles.warning} style={{ marginTop: "1rem", backgroundColor: "rgba(239, 68, 68, 0.1)", borderColor: "#ef4444" }}>
                    <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                        <AlertCircle size={20} color="#ef4444" />
                        <strong style={{ color: "#ef4444" }}>Confirmation Required</strong>
                    </div>
                    <p style={{ margin: "0.5rem 0", color: "#f8fafc" }}>
                        Are you sure you want to <strong>DROP</strong> the table <code>{table}</code>?
                        This will delete the table and all its data permanently.
                    </p>
                    <div className={styles.warningControls}>
                        <button className={styles.dangerBtn} style={{ padding: "0.4rem 1rem" }} onClick={() => handleDropTable(true)}>
                            Yes, DROP TABLE
                        </button>
                        <button className={styles.cancelBtn} onClick={() => setShowConfirm(false)}>Cancel</button>
                    </div>
                </div>
            )}
        </div>
    );
}
