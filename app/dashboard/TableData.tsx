import { useState, useEffect, useCallback } from "react";
import styles from "./dashboard.module.css";
import { AlertCircle, Loader2, Trash2, ArrowUp, ArrowDown, ChevronLeft, ChevronRight } from "lucide-react";

interface TableDataProps {
    database: string;
    table: string;
}

export default function TableData({ database, table }: TableDataProps) {
    const [data, setData] = useState<any[]>([]);
    const [columns, setColumns] = useState<string[]>([]);
    const [structure, setStructure] = useState<any[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState("");
    const [deletingIdx, setDeletingIdx] = useState<number | null>(null);

    // Pagination & Sorting State
    const [limit, setLimit] = useState(50);
    const [offset, setOffset] = useState(0);
    const [total, setTotal] = useState(0);
    const [sortCol, setSortCol] = useState<string | null>(null);
    const [sortOrder, setSortOrder] = useState<"ASC" | "DESC">("ASC");

    // Inline Editing State
    const [editingCell, setEditingCell] = useState<{ idx: number; col: string } | null>(null);
    const [editValue, setEditValue] = useState("");

    const fetchStructure = useCallback(async () => {
        try {
            const res = await fetch(`/api/structure?database=${database}&table=${table}`);
            const result = await res.json();
            if (result.success) {
                setStructure(result.structure);
            }
        } catch {
            // Ignore structure fetch errors
        }
    }, [database, table]);

    const fetchData = useCallback(async () => {
        setLoading(true);
        setError("");
        try {
            let url = `/api/data?database=${database}&table=${table}&limit=${limit}&offset=${offset}`;
            if (sortCol) {
                url += `&sortCol=${sortCol}&sortOrder=${sortOrder}`;
            }
            const res = await fetch(url);
            const result = await res.json();
            if (result.success) {
                setData(result.data);
                setColumns(result.columns);
                setTotal(result.pagination?.total || 0);
            } else {
                setError(result.error);
            }
        } catch {
            setError("Failed to fetch data");
        } finally {
            setLoading(false);
        }
    }, [database, table, limit, offset, sortCol, sortOrder]);

    useEffect(() => {
        setOffset(0); // Reset offset when table or sort changes
        fetchData();
        fetchStructure();
    }, [database, table, sortCol, sortOrder, fetchData, fetchStructure]);

    useEffect(() => {
        fetchData();
    }, [offset, fetchData]);

    const handleSort = (col: string) => {
        if (sortCol === col) {
            setSortOrder(sortOrder === "ASC" ? "DESC" : "ASC");
        } else {
            setSortCol(col);
            setSortOrder("ASC");
        }
    };

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

    const handleUpdateCell = async (idx: number, col: string) => {
        if (editValue === String(data[idx][col])) {
            setEditingCell(null);
            return;
        }

        try {
            const row = data[idx];
            const res = await fetch("/api/data/update", {
                method: "PATCH",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    database,
                    table,
                    column: col,
                    value: editValue,
                    where: row,
                }),
            });

            const result = await res.json();
            if (result.success) {
                // Optimistically update local state or just refresh
                const newData = [...data];
                newData[idx][col] = editValue;
                setData(newData);
            } else {
                alert("Error: " + result.error);
            }
        } catch {
            alert("Failed to update cell");
        } finally {
            setEditingCell(null);
        }
    };

    if (loading && data.length === 0) {
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

    const totalPages = Math.ceil(total / limit);
    const currentPage = Math.floor(offset / limit) + 1;

    return (
        <div className={styles.wrapper}>
            <div className={styles.count}>
                <span>
                    Showing {offset + 1} to {Math.min(offset + limit, total)} of {total} rows
                </span>
                {loading && <Loader2 className="animate-spin" size={16} />}
            </div>
            <div className={styles.tableContainer}>
                <table className={styles.table}>
                    <thead>
                        <tr>
                            <th style={{ width: "40px" }}></th>
                            {columns.map((col) => (
                                <th
                                    key={col}
                                    className={styles.sortableHeader}
                                    onClick={() => handleSort(col)}
                                >
                                    {col}
                                    {sortCol === col && (
                                        <span className={`${styles.sortIcon} ${styles.active}`}>
                                            {sortOrder === "ASC" ? <ArrowUp size={14} /> : <ArrowDown size={14} />}
                                        </span>
                                    )}
                                </th>
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
                                        <td
                                            key={col}
                                            className={styles.editableCell}
                                            onDoubleClick={() => {
                                                const colInfo = structure.find(s => s.Field === col);
                                                let initialValue = row[col] === null ? "" : String(row[col]);
                                                if (colInfo && (colInfo.Type.includes("datetime") || colInfo.Type.includes("timestamp")) && initialValue) {
                                                    // Convert MySQL datetime to datetime-local format
                                                    initialValue = initialValue.replace(' ', 'T').slice(0, 16);
                                                }
                                                setEditingCell({ idx, col });
                                                setEditValue(initialValue);
                                            }}
                                        >
                                            {editingCell?.idx === idx && editingCell?.col === col ? (() => {
                                                const colInfo = structure.find(s => s.Field === col);
                                                let inputType = "text";
                                                if (colInfo) {
                                                    if (colInfo.Type.includes("int")) {
                                                        inputType = "number";
                                                    } else if (colInfo.Type === "date" || colInfo.Type.startsWith("date(")) {
                                                        inputType = "date";
                                                    } else if (colInfo.Type.includes("datetime") || colInfo.Type.includes("timestamp")) {
                                                        inputType = "datetime-local";
                                                    } else if (colInfo.Type.includes("time")) {
                                                        inputType = "time";
                                                    }
                                                }
                                                return (
                                                    <input
                                                        type={inputType}
                                                        className={styles.inlineInput}
                                                        autoFocus
                                                        value={editValue}
                                                        onChange={(e) => setEditValue(e.target.value)}
                                                        onBlur={() => handleUpdateCell(idx, col)}
                                                        onKeyDown={(e) => {
                                                            if (e.key === "Enter") handleUpdateCell(idx, col);
                                                            if (e.key === "Escape") setEditingCell(null);
                                                        }}
                                                    />
                                                );
                                            })() : (
                                                <span title={String(row[col])}>
                                                    {row[col] === null ? <em style={{ color: "#64748b" }}>NULL</em> : (() => {
                                                        const colInfo = structure.find(s => s.Field === col);
                                                        if (colInfo && (colInfo.Type.includes('datetime') || colInfo.Type.includes('timestamp')) && row[col]) {
                                                            try {
                                                                const date = new Date(row[col]);
                                                                return date.toLocaleString();
                                                            } catch {
                                                                return String(row[col]);
                                                            }
                                                        }
                                                        return String(row[col]);
                                                    })()}
                                                </span>
                                            )}
                                        </td>
                                    ))}
                                </tr>
                            ))
                        )}
                    </tbody>
                </table>
            </div>

            {total > 0 && (
                <div className={styles.pagination}>
                    <div className={styles.pageInfo}>
                        Page {currentPage} of {totalPages}
                    </div>
                    <div className={styles.paginationControls}>
                        <button
                            className={styles.pageBtn}
                            onClick={() => setOffset(Math.max(0, offset - limit))}
                            disabled={currentPage === 1}
                        >
                            <ChevronLeft size={16} />
                        </button>
                        {[...Array(Math.min(5, totalPages))].map((_, i) => {
                            let pageNum = currentPage;
                            if (currentPage <= 3) pageNum = i + 1;
                            else if (currentPage >= totalPages - 2) pageNum = totalPages - 4 + i;
                            else pageNum = currentPage - 2 + i;

                            if (pageNum > 0 && pageNum <= totalPages) {
                                return (
                                    <button
                                        key={pageNum}
                                        className={`${styles.pageBtn} ${currentPage === pageNum ? styles.active : ""}`}
                                        onClick={() => setOffset((pageNum - 1) * limit)}
                                    >
                                        {pageNum}
                                    </button>
                                );
                            }
                            return null;
                        })}
                        <button
                            className={styles.pageBtn}
                            onClick={() => setOffset(offset + limit)}
                            disabled={currentPage === totalPages}
                        >
                            <ChevronRight size={16} />
                        </button>
                    </div>
                </div>
            )}
        </div>
    );
}
