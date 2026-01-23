"use client";

import { useState, useEffect } from "react";
import styles from "./dashboard.module.css";
import { AlertCircle, Loader2, Key } from "lucide-react";

interface StructureViewProps {
    database: string;
    table: string;
}

export default function StructureView({ database, table }: StructureViewProps) {
    const [structure, setStructure] = useState<any[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState("");

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
            } else {
                setError(result.error);
            }
        } catch {
            setError("Failed to fetch table structure");
        } finally {
            setLoading(false);
        }
    };

    if (loading) {
        return (
            <div className={styles.loading}>
                <Loader2 className="animate-spin" size={24} style={{ marginRight: 8 }} />
                Loading table structure...
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
                Table: <strong>{table}</strong> - {structure.length} columns
            </div>
            <div className={styles.tableContainer}>
                <table className={styles.table}>
                    <thead>
                        <tr>
                            <th>Column</th>
                            <th>Type</th>
                            <th>Null</th>
                            <th>Key</th>
                            <th>Default</th>
                            <th>Extra</th>
                        </tr>
                    </thead>
                    <tbody>
                        {structure.map((col) => (
                            <tr key={col.Field}>
                                <td>
                                    <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                                        {col.Key === "PRI" && <Key size={14} color="#f59e0b" />}
                                        {col.Field}
                                    </div>
                                </td>
                                <td>{col.Type}</td>
                                <td>{col.Null}</td>
                                <td>{col.Key}</td>
                                <td>{col.Default === null ? <em style={{ color: "#64748b" }}>NULL</em> : col.Default}</td>
                                <td>{col.Extra}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
