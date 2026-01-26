"use client";

import { useState, useEffect } from "react";
import { Plus, Trash2, Play, Code, Search } from "lucide-react";
import styles from "./dashboard.module.css";

interface Condition {
    column: string;
    operator: string;
    value: string;
}

interface VisualQueryBuilderProps {
    database: string;
    initialTable?: string;
    onRunQuery: (query: string) => void;
}

export default function VisualQueryBuilder({ database, initialTable, onRunQuery }: VisualQueryBuilderProps) {
    const [tables, setTables] = useState<string[]>([]);
    const [selectedTable, setSelectedTable] = useState(initialTable || "");
    const [columns, setColumns] = useState<string[]>([]);
    const [selectedColumns, setSelectedColumns] = useState<string[]>(["*"]);
    const [conditions, setConditions] = useState<Condition[]>([]);
    const [limit, setLimit] = useState(100);
    const [generatedSql, setGeneratedSql] = useState("");

    useEffect(() => {
        const fetchTables = async () => {
            const res = await fetch(`/api/tables?database=${database}`);
            const data = await res.json();
            if (data.success) setTables(data.tables);
        };
        fetchTables();
    }, [database]);

    useEffect(() => {
        if (selectedTable) {
            const fetchColumns = async () => {
                const res = await fetch(`/api/structure?database=${database}&table=${selectedTable}`);
                const data = await res.json();
                if (data.success) {
                    setColumns(data.structure.map((c: any) => c.Field));
                }
            };
            fetchColumns();
        } else {
            setColumns([]);
        }
    }, [database, selectedTable]);

    useEffect(() => {
        if (!selectedTable) {
            setGeneratedSql("");
            return;
        }

        let sql = `SELECT ${selectedColumns.join(", ")} FROM ${selectedTable}`;

        if (conditions.length > 0) {
            const whereClause = conditions
                .filter(c => c.column && c.value)
                .map(c => `${c.column} ${c.operator} '${c.value}'`)
                .join(" AND ");
            if (whereClause) sql += ` WHERE ${whereClause}`;
        }

        sql += ` LIMIT ${limit}`;
        setGeneratedSql(sql);
    }, [selectedTable, selectedColumns, conditions, limit]);

    const addCondition = () => {
        setConditions([...conditions, { column: columns[0] || "", operator: "=", value: "" }]);
    };

    const removeCondition = (index: number) => {
        setConditions(conditions.filter((_, i) => i !== index));
    };

    const updateCondition = (index: number, field: keyof Condition, value: string) => {
        const newConditions = [...conditions];
        newConditions[index][field] = value;
        setConditions(newConditions);
    };

    const toggleColumn = (col: string) => {
        if (col === "*") {
            setSelectedColumns(["*"]);
            return;
        }

        const newCols = selectedColumns.filter(c => c !== "*");
        if (newCols.includes(col)) {
            const updated = newCols.filter(c => c !== col);
            setSelectedColumns(updated.length === 0 ? ["*"] : updated);
        } else {
            setSelectedColumns([...newCols, col]);
        }
    };

    return (
        <div className={styles.wrapper}>
            <div className={styles.form}>
                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "2rem" }}>
                    <div>
                        <label className={styles.label}>Select Table</label>
                        <select
                            className={styles.input}
                            value={selectedTable}
                            onChange={(e) => setSelectedTable(e.target.value)}
                        >
                            <option value="">-- Choose a table --</option>
                            {tables.map(t => <option key={t} value={t}>{t}</option>)}
                        </select>

                        {selectedTable && (
                            <div style={{ marginTop: "1.5rem" }}>
                                <label className={styles.label}>Columns</label>
                                <div style={{ display: "flex", flexWrap: "wrap", gap: "8px", marginTop: "8px" }}>
                                    <button
                                        className={selectedColumns.includes("*") ? styles.primaryBtn : styles.actionBtn}
                                        onClick={() => toggleColumn("*")}
                                        style={{ fontSize: "0.8rem", padding: "4px 12px" }}
                                    >
                                        * (All)
                                    </button>
                                    {columns.map(col => (
                                        <button
                                            key={col}
                                            className={selectedColumns.includes(col) ? styles.primaryBtn : styles.actionBtn}
                                            onClick={() => toggleColumn(col)}
                                            style={{ fontSize: "0.8rem", padding: "4px 12px" }}
                                        >
                                            {col}
                                        </button>
                                    ))}
                                </div>
                            </div>
                        )}
                    </div>

                    <div>
                        <label className={styles.label}>Filters (WHERE)</label>
                        <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
                            {conditions.map((c, i) => (
                                <div key={i} style={{ display: "flex", gap: "8px", alignItems: "center" }}>
                                    <select
                                        className={styles.input}
                                        style={{ flex: 1 }}
                                        value={c.column}
                                        onChange={(e) => updateCondition(i, "column", e.target.value)}
                                    >
                                        {columns.map(col => <option key={col} value={col}>{col}</option>)}
                                    </select>
                                    <select
                                        className={styles.input}
                                        style={{ width: "80px" }}
                                        value={c.operator}
                                        onChange={(e) => updateCondition(i, "operator", e.target.value)}
                                    >
                                        <option value="=">=</option>
                                        <option value="!=">!=</option>
                                        <option value=">">&gt;</option>
                                        <option value="<">&lt;</option>
                                        <option value="LIKE">LIKE</option>
                                    </select>
                                    <input
                                        className={styles.input}
                                        style={{ flex: 1 }}
                                        placeholder="Value"
                                        value={c.value}
                                        onChange={(e) => updateCondition(i, "value", e.target.value)}
                                    />
                                    <button className={styles.themeBtn} onClick={() => removeCondition(i)}>
                                        <Trash2 size={16} />
                                    </button>
                                </div>
                            ))}
                            <button className={styles.actionBtn} onClick={addCondition} style={{ alignSelf: "flex-start", marginTop: "4px" }}>
                                <Plus size={14} style={{ marginRight: 4 }} /> Add Filter
                            </button>
                        </div>

                        <div style={{ marginTop: "1.5rem" }}>
                            <label className={styles.label}>Limit</label>
                            <input
                                type="number"
                                className={styles.input}
                                value={limit}
                                onChange={(e) => setLimit(parseInt(e.target.value) || 10)}
                                style={{ width: "100px" }}
                            />
                        </div>
                    </div>
                </div>

                {generatedSql && (
                    <div style={{ marginTop: "2rem" }}>
                        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-end", marginBottom: "8px" }}>
                            <label className={styles.label} style={{ margin: 0 }}>Generated SQL</label>
                            <button className={styles.primaryBtn} onClick={() => onRunQuery(generatedSql)}>
                                <Play size={14} style={{ marginRight: 6 }} /> Run Query
                            </button>
                        </div>
                        <code style={{
                            display: "block",
                            padding: "1rem",
                            background: "var(--input-bg)",
                            borderRadius: "8px",
                            border: "1px solid var(--border)",
                            color: "var(--primary)",
                            fontSize: "1rem",
                            fontFamily: "monospace"
                        }}>
                            {generatedSql}
                        </code>
                    </div>
                )}

                {!selectedTable && (
                    <div className={styles.emptyState} style={{ marginTop: "2rem" }}>
                        <Search size={48} className={styles.emptyIcon} />
                        <p>Select a table to start building your query</p>
                    </div>
                )}
            </div>
        </div>
    );
}
