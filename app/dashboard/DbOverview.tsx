"use client";

import { useState, useEffect } from "react";
import styles from "./dashboard.module.css";
import { AlertCircle, Loader2, Table as TableIcon, Trash2, Plus, Check, X } from "lucide-react";

interface DbOverviewProps {
    database: string;
    onSelectTable: (table: string) => void;
    onCreateTable: () => void;
    onRefreshSidebar: () => void;
}

export default function DbOverview({ database, onSelectTable, onCreateTable, onRefreshSidebar }: DbOverviewProps) {
    const [stats, setStats] = useState<any[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState("");
    const [droppingTable, setDroppingTable] = useState<string | null>(null);
    const [showConfirmDrop, setShowConfirmDrop] = useState<string | null>(null);

    useEffect(() => {
        fetchStats();
    }, [database]);

    const fetchStats = async () => {
        setLoading(true);
        setError("");
        try {
            const res = await fetch(`/api/database-stats?database=${database}`);
            const result = await res.json();
            if (result.success) {
                setStats(result.stats);
            } else {
                setError(result.error);
            }
        } catch {
            setError("Failed to fetch database statistics");
        } finally {
            setLoading(false);
        }
    };


    const handleDropTable = async (table: string, confirmed = false) => {
        if (!confirmed) {
            setShowConfirmDrop(table);
            return;
        }

        setDroppingTable(table);
        try {
            const res = await fetch("/api/tables/drop", {
                method: "DELETE",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ database, table, confirmed: true }),
            });
            const result = await res.json();
            if (result.success) {
                fetchStats();
                onRefreshSidebar();
            } else {
                alert(result.error);
            }
        } catch {
            alert("Failed to drop table");
        } finally {
            setDroppingTable(null);
            setShowConfirmDrop(null);
        }
    };

    const formatSize = (bytes: number) => {
        if (bytes === 0) return "0 B";
        const k = 1024;
        const sizes = ["B", "KB", "MB", "GB", "TB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
    };

    if (loading && stats.length === 0) {
        return (
            <div className={styles.loading}>
                <Loader2 className={`animate-spin ${styles.tabIcon}`} size={24} />
                Loading database overview...
            </div>
        );
    }

    if (error) {
        return (
            <div className={styles.error}>
                <AlertCircle className={styles.tabIcon} size={24} />
                {error}
            </div>
        );
    }

    const totalRows = stats.reduce((acc, curr) => acc + (curr.rowCount || 0), 0);
    const totalSize = stats.reduce((acc, curr) => acc + (curr.dataSize || 0), 0);

    return (
        <div className={styles.wrapper}>
            <div className={`${styles.count} ${styles.flexBetween}`}>
                <div>
                    Database: <strong>{database}</strong> - {stats.length} tables
                </div>
                <button className={styles.actionBtn} onClick={onCreateTable}>
                    <Plus size={14} className={styles.tabIcon} /> Create Table
                </button>
            </div>

            {showConfirmDrop && (
                <div className={styles.warning} style={{ marginBottom: "var(--space-4)", border: "1px solid #ef4444", backgroundColor: "var(--bg-base)" }}>
                    <p style={{ margin: "0 0 var(--space-4) 0", color: "var(--foreground)" }}>
                        Are you sure you want to <strong>DROP</strong> table <code>{showConfirmDrop}</code>?
                    </p>
                    <div className={`${styles.flexRow} ${styles.flexGapSm}`}>
                        <button className={styles.dangerBtn} onClick={() => handleDropTable(showConfirmDrop, true)}>
                            Drop Table
                        </button>
                        <button className={styles.actionBtn} onClick={() => setShowConfirmDrop(null)}>Cancel</button>
                    </div>
                </div>
            )}

            <div className={styles.tableContainer}>
                <table className={styles.table}>
                    <thead>
                        <tr>
                            <th>Table</th>
                            <th>Rows</th>
                            <th>Engine</th>
                            <th>Collation</th>
                            <th>Size</th>
                            <th style={{ width: "80px" }}>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {stats.length === 0 ? (
                            <tr>
                                <td colSpan={6} style={{ textAlign: "center", padding: "var(--space-6)" }}>
                                    No tables found in this database
                                </td>
                            </tr>
                        ) : (
                            stats.map((table) => (
                                <tr key={table.tableName}>
                                    <td onClick={() => onSelectTable(table.tableName)} style={{ cursor: "pointer" }}>
                                        <div className={`${styles.flexRow} ${styles.flexGapSm}`}>
                                            <TableIcon size={14} />
                                            {table.tableName}
                                        </div>
                                    </td>
                                    <td>{table.rowCount?.toLocaleString() || 0}</td>
                                    <td>{table.engine}</td>
                                    <td>{table.collation}</td>
                                    <td suppressHydrationWarning>{formatSize(table.dataSize)}</td>
                                    <td>
                                        <button
                                            className={`${styles.actionBtn} danger`}
                                            onClick={() => handleDropTable(table.tableName)}
                                            disabled={droppingTable === table.tableName}
                                            title="Drop Table"
                                        >
                                            {droppingTable === table.tableName ? <Loader2 className="animate-spin" size={14} /> : <Trash2 size={14} />}
                                        </button>
                                    </td>
                                </tr>
                            ))
                        )}
                    </tbody>
                    <tfoot>
                        <tr style={{ fontWeight: "bold", backgroundColor: "var(--bg-header)" }}>
                            <td>Total: {stats.length} tables</td>
                            <td>{totalRows.toLocaleString()}</td>
                            <td>-</td>
                            <td>-</td>
                            <td suppressHydrationWarning>{formatSize(totalSize)}</td>
                            <td></td>
                        </tr>
                    </tfoot>
                </table>
            </div>
        </div>
    );
}
