"use client";

import { useState, useEffect } from "react";
import styles from "./dashboard.module.css";
import { AlertCircle, Loader2, Trash2 } from "lucide-react";

interface TableDataProps {
    database: string;
    table: string;
}

export default function TableData({ database, table }: TableDataProps) {
    const [data, setData] = useState<any[]>([]);
    const [columns, setColumns] = useState<string[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState("");
    const [deletingIdx, setDeletingIdx] = useState<number | null>(null);

    const fetchData = async () => {
        setLoading(true);
        setError("");
        try {
            const res = await fetch(`/api/data?database=${database}&table=${table}`);
            const result = await res.json();
            if (result.success) {
                setData(result.data);
                setColumns(result.columns);
            } else {
                setError(result.error);
            }
        } catch {
            setError("Failed to fetch data");
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchData();
    }, [database, table]);

    const handleDeleteRow = async (row: any, idx: number) => {
        if (!confirm("Are you sure you want to delete this row?")) return;

        setDeletingIdx(idx);
        try {
            const res = await fetch("/api/data/delete", {
                method: "DELETE",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ database, table, where: row }),
            });

            const result = await res.json();
            if (result.success) {
                // Refresh data
                fetchData();
            } else {
                alert("Error: " + result.error);
            }
        } catch {
            alert("Failed to delete row");
        } finally {
            setDeletingIdx(null);
        }
    };

    if (loading) {
        return (
            <div className={styles.loading}>
                <Loader2 className="animate-spin" size={24} style={{ marginRight: 8 }} />
                Loading table data...
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

    return (
        <div className={styles.wrapper}>
            <div className={styles.count}>
                Showing first {data.length} rows
            </div>
            <div className={styles.tableContainer}>
                <table className={styles.table}>
                    <thead>
                        <tr>
                            <th style={{ width: "40px" }}></th>
                            {columns.map((col) => (
                                <th key={col}>{col}</th>
                            ))}
                        </tr>
                    </thead>
                    <tbody>
                        {data.length === 0 ? (
                            <tr>
                                <td colSpan={columns.length + 1} style={{ textAlign: "center", padding: "2rem" }}>
                                    Table is empty
                                </td>
                            </tr>
                        ) : (
                            data.map((row, idx) => (
                                <tr key={idx}>
                                    <td>
                                        <button
                                            className={`${styles.actionBtn} danger`}
                                            onClick={() => handleDeleteRow(row, idx)}
                                            disabled={deletingIdx === idx}
                                            title="Delete Row"
                                        >
                                            {deletingIdx === idx ? <Loader2 className="animate-spin" size={14} /> : <Trash2 size={14} />}
                                        </button>
                                    </td>
                                    {columns.map((col) => (
                                        <td key={col} title={String(row[col])}>
                                            {row[col] === null ? <em style={{ color: "#64748b" }}>NULL</em> : String(row[col])}
                                        </td>
                                    ))}
                                </tr>
                            ))
                        )}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
