"use client";

import { useState } from "react";
import styles from "./dashboard.module.css";
import { Plus, Trash2, ArrowLeft, Save, Loader2, AlertCircle } from "lucide-react";

interface Column {
    name: string;
    type: string;
    length: string;
    isNull: boolean;
    isPrimary: boolean;
    isAutoIncrement: boolean;
}

interface CreateTableProps {
    database: string;
    onBack: () => void;
    onSuccess: (tableName: string) => void;
}

const DATA_TYPES = [
    "INT", "VARCHAR", "TEXT", "DATE", "DATETIME", "TIMESTAMP", "DECIMAL",
    "TINYINT", "BIGINT", "FLOAT", "DOUBLE", "BOOLEAN", "BLOB", "JSON"
];

export default function CreateTable({ database, onBack, onSuccess }: CreateTableProps) {
    const [tableName, setTableName] = useState("");
    const [columns, setColumns] = useState<Column[]>([
        { name: "id", type: "INT", length: "", isNull: false, isPrimary: true, isAutoIncrement: true }
    ]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState("");

    const addColumn = () => {
        setColumns([
            ...columns,
            { name: "", type: "VARCHAR", length: "255", isNull: true, isPrimary: false, isAutoIncrement: false }
        ]);
    };

    const removeColumn = (index: number) => {
        if (columns.length === 1) return;
        setColumns(columns.filter((_, i) => i !== index));
    };

    const updateColumn = (index: number, field: keyof Column, value: any) => {
        const newColumns = [...columns];
        newColumns[index] = { ...newColumns[index], [field]: value };

        // Ensure only one auto_increment and it must be primary
        if (field === "isAutoIncrement" && value) {
            newColumns[index].isPrimary = true;
            // Unset other auto_increments
            newColumns.forEach((col, i) => {
                if (i !== index) col.isAutoIncrement = false;
            });
        }

        setColumns(newColumns);
    };

    const handleCreate = async () => {
        if (!tableName.trim()) {
            setError("Table name is required");
            return;
        }

        if (columns.some(col => !col.name.trim())) {
            setError("All columns must have a name");
            return;
        }

        setLoading(true);
        setError("");
        try {
            const res = await fetch("/api/tables/create", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ database, table: tableName, columns }),
            });
            const result = await res.json();
            if (result.success) {
                onSuccess(tableName);
            } else {
                setError(result.error);
            }
        } catch {
            setError("Failed to create table");
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className={styles.wrapper}>
            <div className={styles.topBar} style={{ padding: "0 0 1rem 0", background: "none", border: "none" }}>
                <div style={{ display: "flex", alignItems: "center", gap: "1rem" }}>
                    <button className={styles.actionBtn} onClick={onBack}>
                        <ArrowLeft size={16} />
                    </button>
                    <h2 style={{ fontSize: "1.25rem", margin: 0 }}>Create table in: <strong>{database}</strong></h2>
                </div>
                <div style={{ display: "flex", gap: "0.75rem" }}>
                    <button className={styles.actionBtn} onClick={addColumn}>
                        <Plus size={16} style={{ marginRight: 6 }} /> Add Column
                    </button>
                    <button className={styles.primaryBtn} onClick={handleCreate} disabled={loading}>
                        {loading ? <Loader2 className="animate-spin" size={16} /> : <Save size={16} style={{ marginRight: 6 }} />}
                        Save Table
                    </button>
                </div>
            </div>

            {error && (
                <div className={styles.warning} style={{ marginBottom: "1rem", color: "#ef4444", borderColor: "#ef4444" }}>
                    <AlertCircle size={16} style={{ marginRight: 8, verticalAlign: "middle" }} />
                    {error}
                </div>
            )}

            <div className={styles.form} style={{ maxWidth: "100%", marginBottom: "1.5rem" }}>
                <div className={styles.fieldRow}>
                    <div className={styles.fieldLabel}>Table Name</div>
                    <div className={styles.fieldInputWrapper}>
                        <input
                            type="text"
                            className={styles.formInput}
                            placeholder="my_table"
                            value={tableName}
                            onChange={(e) => setTableName(e.target.value)}
                        />
                    </div>
                </div>
            </div>

            <div className={styles.tableContainer}>
                <table className={styles.table}>
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th>Type</th>
                            <th>Length/Values</th>
                            <th>Null</th>
                            <th>AI</th>
                            <th>Primary</th>
                            <th style={{ width: "50px" }}></th>
                        </tr>
                    </thead>
                    <tbody>
                        {columns.map((col, idx) => (
                            <tr key={idx}>
                                <td>
                                    <input
                                        type="text"
                                        className={styles.formInput}
                                        style={{ fontSize: "0.8125rem" }}
                                        value={col.name}
                                        onChange={(e) => updateColumn(idx, "name", e.target.value)}
                                    />
                                </td>
                                <td>
                                    <select
                                        className={styles.formInput}
                                        style={{ fontSize: "0.8125rem" }}
                                        value={col.type}
                                        onChange={(e) => updateColumn(idx, "type", e.target.value)}
                                    >
                                        {DATA_TYPES.map(type => <option key={type} value={type}>{type}</option>)}
                                    </select>
                                </td>
                                <td>
                                    <input
                                        type="text"
                                        className={styles.formInput}
                                        style={{ fontSize: "0.8125rem" }}
                                        placeholder="(optional)"
                                        value={col.length}
                                        onChange={(e) => updateColumn(idx, "length", e.target.value)}
                                    />
                                </td>
                                <td style={{ textAlign: "center" }}>
                                    <input
                                        type="checkbox"
                                        checked={col.isNull}
                                        onChange={(e) => updateColumn(idx, "isNull", e.target.checked)}
                                    />
                                </td>
                                <td style={{ textAlign: "center" }}>
                                    <input
                                        type="checkbox"
                                        checked={col.isAutoIncrement}
                                        onChange={(e) => updateColumn(idx, "isAutoIncrement", e.target.checked)}
                                    />
                                </td>
                                <td style={{ textAlign: "center" }}>
                                    <input
                                        type="checkbox"
                                        checked={col.isPrimary}
                                        onChange={(e) => updateColumn(idx, "isPrimary", e.target.checked)}
                                    />
                                </td>
                                <td>
                                    <button className={`${styles.logoutBtn} danger`} onClick={() => removeColumn(idx)}>
                                        <Trash2 size={16} />
                                    </button>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
