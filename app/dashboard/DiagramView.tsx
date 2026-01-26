"use client";

import { useState, useEffect, useMemo } from "react";
import styles from "./dashboard.module.css";
import { AlertCircle, Loader2, Share2, ZoomIn, ZoomOut, Maximize } from "lucide-react";

interface Relation {
    TABLE_NAME: string;
    COLUMN_NAME: string;
    REFERENCED_TABLE_NAME: string;
    REFERENCED_COLUMN_NAME: string;
}

interface DiagramViewProps {
    database: string;
}

export default function DiagramView({ database }: DiagramViewProps) {
    const [tables, setTables] = useState<string[]>([]);
    const [relations, setRelations] = useState<Relation[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState("");
    const [zoom, setZoom] = useState(1);

    useEffect(() => {
        fetchData();
    }, [database]);

    const fetchData = async () => {
        setLoading(true);
        setError("");
        try {
            const tableRes = await fetch(`/api/tables?database=${database}`);
            const tableData = await tableRes.json();

            const relationRes = await fetch(`/api/schema/relations?database=${database}`);
            const relationData = await relationRes.json();

            if (tableData.success && relationData.success) {
                setTables(tableData.tables);
                setRelations(relationData.relations);
            } else {
                setError(tableData.error || relationData.error);
            }
        } catch {
            setError("Failed to fetch schema information");
        } finally {
            setLoading(false);
        }
    };

    // Simple layout calculation: Arrange tables in a grid
    const tablePositions = useMemo(() => {
        const positions: Record<string, { x: number, y: number }> = {};
        const cols = Math.ceil(Math.sqrt(tables.length));
        const spacingX = 250;
        const spacingY = 150;

        tables.forEach((table, i) => {
            const row = Math.floor(i / cols);
            const col = i % cols;
            positions[table] = {
                x: 50 + col * spacingX,
                y: 50 + row * spacingY
            };
        });
        return positions;
    }, [tables]);

    if (loading) {
        return (
            <div className={styles.loading}>
                <Loader2 className="animate-spin" size={24} style={{ marginRight: 8 }} />
                Generating ER Diagram...
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
        <div className={styles.wrapper} style={{ height: "calc(100vh - 160px)", display: "flex", flexDirection: "column" }}>
            <div className={styles.count} style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <div>
                    <Share2 size={18} style={{ marginRight: 8, verticalAlign: "middle" }} />
                    ER Diagram: <strong>{database}</strong>
                </div>
                <div style={{ display: "flex", gap: "8px" }}>
                    <button className={styles.actionBtn} onClick={() => setZoom(z => Math.max(0.2, z - 0.1))}><ZoomOut size={14} /></button>
                    <button className={styles.actionBtn} onClick={() => setZoom(1)}>{Math.round(zoom * 100)}%</button>
                    <button className={styles.actionBtn} onClick={() => setZoom(z => Math.min(2, z + 0.1))}><ZoomIn size={14} /></button>
                </div>
            </div>

            <div style={{
                flex: 1,
                background: "var(--input-bg)",
                borderRadius: "12px",
                border: "1px solid var(--border)",
                overflow: "auto",
                position: "relative",
                padding: "20px"
            }}>
                <svg
                    width={2000}
                    height={2000}
                    style={{
                        transform: `scale(${zoom})`,
                        transformOrigin: "0 0",
                        transition: "transform 0.2s ease"
                    }}
                >
                    <defs>
                        <marker id="arrow" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth">
                            <path d="M0,0 L0,6 L9,3 z" fill="var(--text-muted)" />
                        </marker>
                    </defs>

                    {/* Draw Relations */}
                    {relations.map((rel, i) => {
                        const start = tablePositions[rel.TABLE_NAME];
                        const end = tablePositions[rel.REFERENCED_TABLE_NAME];
                        if (!start || !end) return null;

                        // Calculate points (center-ish of boxes)
                        const x1 = start.x + 180;
                        const y1 = start.y + 40;
                        const x2 = end.x;
                        const y2 = end.y + 40;

                        return (
                            <path
                                key={`rel-${i}`}
                                d={`M ${x1} ${y1} C ${x1 + 50} ${y1}, ${x2 - 50} ${y2}, ${x2} ${y2}`}
                                stroke="var(--primary)"
                                strokeWidth="2"
                                strokeOpacity="0.4"
                                fill="none"
                                markerEnd="url(#arrow)"
                            />
                        );
                    })}

                    {/* Draw Tables */}
                    {tables.map((table) => {
                        const pos = tablePositions[table];
                        return (
                            <g key={table} transform={`translate(${pos.x},${pos.y})`}>
                                <rect
                                    width="180"
                                    height="80"
                                    rx="8"
                                    fill="var(--bg-secondary)"
                                    stroke="var(--border)"
                                    strokeWidth="1"
                                />
                                <rect
                                    width="180"
                                    height="30"
                                    rx="8"
                                    fill="var(--primary-transparent)"
                                />
                                <text
                                    x="90"
                                    y="20"
                                    textAnchor="middle"
                                    fill="var(--primary)"
                                    style={{ fontSize: "12px", fontWeight: "bold" }}
                                >
                                    {table}
                                </text>
                                <text
                                    x="90"
                                    y="55"
                                    textAnchor="middle"
                                    fill="var(--text-muted)"
                                    style={{ fontSize: "10px" }}
                                >
                                    {relations.filter(r => r.TABLE_NAME === table).length} FKs
                                </text>
                            </g>
                        );
                    })}
                </svg>
            </div>
        </div>
    );
}
