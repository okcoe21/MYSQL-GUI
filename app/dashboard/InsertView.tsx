"use client";

import { useState, useEffect } from "react";
import styles from "./dashboard.module.css";
import { AlertCircle, Loader2, Save, X } from "lucide-react";

interface InsertViewProps {
    database: string;
    table: string;
    onSuccess: () => void;
}

export default function InsertView({ database, table, onSuccess }: InsertViewProps) {
    const [structure, setStructure] = useState<any[]>([]);
    const [formData, setFormData] = useState<{ [key: string]: any }>({});
    const [loading, setLoading] = useState(false);
    const [submitting, setSubmitting] = useState(false);
    const [error, setError] = useState("");
    const [successMsg, setSuccessMsg] = useState("");

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
                setStructure(result.structure);
                // Initialize form data with nulls or empty strings
                const initialData: any = {};
                result.structure.forEach((col: any) => {
                    if (col.Extra !== "auto_increment") {
                        initialData[col.Field] = col.Default !== null ? col.Default : "";
                    }
                });
                setFormData(initialData);
            } else {
                setError(result.error);
            }
        } catch {
            setError("Failed to fetch table structure for insertion");
        } finally {
            setLoading(false);
        }
    };

    const handleInputChange = (field: string, value: any) => {
        setFormData(prev => ({ ...prev, [field]: value }));
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setSubmitting(true);
        setError("");
        setSuccessMsg("");

        try {
            const res = await fetch("/api/data/insert", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ database, table, data: formData }),
            });

            const result = await res.json();
            if (result.success) {
                setSuccessMsg(`Successfully inserted row! ID: ${result.insertId || "N/A"}`);
                // Reset form or keep? Conventional is to clear or leave for next insert.
                // We'll call onSuccess after a short delay if we want to switch back to browse.
                setTimeout(() => {
                    onSuccess();
                }, 1500);
            } else {
                setError(result.error);
            }
        } catch {
            setError("Failed to insert row");
        } finally {
            setSubmitting(false);
        }
    };

    if (loading) {
        return (
            <div className={styles.loading}>
                <Loader2 className="animate-spin" size={24} style={{ marginRight: 8 }} />
                Loading structure...
            </div>
        );
    }

    return (
        <div className={styles.wrapper}>
            <div className={styles.count}>
                Insert into: <strong>{database}.{table}</strong>
            </div>

            {error && (
                <div className={styles.warning} style={{ borderColor: "#ef4444", color: "#ef4444", marginBottom: "1rem" }}>
                    <AlertCircle size={16} style={{ marginRight: 8, verticalAlign: "middle" }} />
                    {error}
                </div>
            )}

            {successMsg && (
                <div className={styles.successMessage} style={{ marginBottom: "1rem" }}>
                    {successMsg}
                </div>
            )}

            <form className={styles.form} onSubmit={handleSubmit}>
                {structure.map((col) => {
                    if (col.Extra === "auto_increment") {
                        return (
                            <div key={col.Field} className={styles.fieldRow}>
                                <div className={styles.fieldLabel}>
                                    {col.Field} <span style={{ fontSize: "0.7rem", color: "#64748b" }}>(Auto)</span>
                                </div>
                                <div className={styles.fieldInputWrapper}>
                                    <input className={styles.formInput} value="AUTO_INCREMENT" disabled style={{ opacity: 0.5 }} />
                                </div>
                            </div>
                        );
                    }

                    const isNullable = col.Null === "YES";
                    let inputType = "text";
                    if (col.Type.includes("int")) {
                        inputType = "number";
                    } else if (col.Type === "date" || col.Type.startsWith("date(")) {
                        inputType = "date";
                    } else if (col.Type.includes("datetime") || col.Type.includes("timestamp")) {
                        inputType = "datetime-local";
                    } else if (col.Type.includes("time")) {
                        inputType = "time";
                    }

                    return (
                        <div key={col.Field} className={styles.fieldRow}>
                            <div className={styles.fieldLabel}>
                                {col.Field}
                                {col.Key === "PRI" && <span title="Primary Key" style={{ marginLeft: 4, color: "#f59e0b" }}>*</span>}
                                {!isNullable && <span title="Required" style={{ marginLeft: 4, color: "#ef4444" }}>!</span>}
                            </div>
                            <div className={styles.fieldInputWrapper}>
                                <input
                                    type={inputType}
                                    className={styles.formInput}
                                    value={formData[col.Field] || ""}
                                    onChange={(e) => handleInputChange(col.Field, e.target.value)}
                                    placeholder={isNullable ? "NULL" : ""}
                                    required={!isNullable}
                                />
                            </div>
                        </div>
                    );
                })}

                <div className={styles.formFooter}>
                    <button type="button" className={styles.cancelBtn} onClick={onSuccess}>
                        <X size={16} style={{ marginRight: 8, verticalAlign: "middle" }} />
                        Cancel
                    </button>
                    <button type="submit" className={styles.primaryBtn} disabled={submitting}>
                        {submitting ? (
                            <Loader2 className="animate-spin" size={16} />
                        ) : (
                            <>
                                <Save size={16} style={{ marginRight: 8, verticalAlign: "middle" }} />
                                Insert Row
                            </>
                        )}
                    </button>
                </div>
            </form>
        </div>
    );
}
