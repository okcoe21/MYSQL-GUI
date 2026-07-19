

import { useState, useEffect, useMemo } from "react";
import styles from "./dashboard.module.css";
import { AlertCircle, Loader2, Share2, ZoomIn, ZoomOut, Maximize } from "lucide-react";
import * as api from "../../lib/api";

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
            const tableData = await api.listTables(database);

            const relationData = await api.getRelations(database);

            if (tableData.success && relationData.success) {
                setTables(tableData.tables || []);
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
                <Loader2 className={`animate-spin ${styles.tabIcon}`} size={24} />
                Generating ER Diagram...
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

    return (
        <div className={`${styles.wrapper} ${styles.diagramWrapper}`}>
            <div className={`${styles.count} ${styles.flexBetween}`}>
                <div>
                    <Share2 size={18} className={styles.tabIcon} />
                    ER Diagram: <strong>{database}</strong>
                </div>
                <div className={`${styles.flexRow} ${styles.flexGapSm}`}>
                    <button className={styles.actionBtn} onClick={() => setZoom(z => Math.max(0.2, z - 0.1))}><ZoomOut size={14} /></button>
                    <button className={styles.actionBtn} onClick={() => setZoom(1)}>{Math.round(zoom * 100)}%</button>
                    <button className={styles.actionBtn} onClick={() => setZoom(z => Math.min(2, z + 0.1))}><ZoomIn size={14} /></button>
                </div>
            </div>

            <div className={styles.diagramCanvas}>
                <svg
                    width={2000}
                    height={2000}
                    style={{
                        transform: `scale(${zoom})`,
                        transformOrigin: "0 0",
                        transition: "var(--transition-fast)"
                    }}
                >
                    <defs>
                        <marker id="arrow" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth">
                            <path d="M0,0 L0,6 L9,3 z" fill="var(--accent)" />
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
                                stroke="var(--accent)"
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
                                    fill="var(--bg-panel)"
                                    stroke="var(--border)"
                                    strokeWidth="1"
                                />
                                <rect
                                    width="180"
                                    height="30"
                                    fill="var(--bg-header)"
                                />
                                <text
                                    x="90"
                                    y="20"
                                    textAnchor="middle"
                                    fill="var(--accent)"
                                    style={{ fontSize: "var(--font-xs)", fontWeight: "bold", fontFamily: "inherit" }}
                                >
                                    {table}
                                </text>
                                <text
                                    x="90"
                                    y="55"
                                    textAnchor="middle"
                                    fill="var(--text-muted)"
                                    style={{ fontSize: "var(--font-xs)", fontFamily: "inherit" }}
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
