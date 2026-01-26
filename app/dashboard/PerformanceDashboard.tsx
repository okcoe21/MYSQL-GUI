"use client";

import { useState, useEffect, useRef } from "react";
import styles from "./dashboard.module.css";
import { Activity, Loader2, RefreshCw, Zap, Users, Globe, Database } from "lucide-react";

interface MetricSnapshot {
    timestamp: number;
    questions: number;
    threads_connected: number;
    threads_running: number;
    bytes_received: number;
    bytes_sent: number;
}

export default function PerformanceDashboard() {
    const [history, setHistory] = useState<MetricSnapshot[]>([]);
    const [loading, setLoading] = useState(true);
    const [isPaused, setIsPaused] = useState(false);
    const intervalRef = useRef<NodeJS.Timeout | null>(null);

    const fetchMetrics = async () => {
        try {
            const res = await fetch("/api/server/metrics");
            const data = await res.json();
            if (data.success) {
                setHistory(prev => {
                    const newHistory = [...prev, { ...data.metrics, timestamp: data.timestamp }];
                    return newHistory.slice(-30); // Keep last 30 points
                });
                setLoading(false);
            }
        } catch (err) {
            console.error("Failed to fetch metrics", err);
        }
    };

    useEffect(() => {
        fetchMetrics();
        intervalRef.current = setInterval(() => {
            if (!isPaused) fetchMetrics();
        }, 3000);

        return () => {
            if (intervalRef.current) clearInterval(intervalRef.current);
        };
    }, [isPaused]);

    const renderLineChart = (data: number[], color: string, label: string) => {
        if (data.length < 2) return <div className={styles.emptyState}>Gathering data...</div>;

        const width = 300;
        const height = 100;
        const padding = 10;
        const max = Math.max(...data) || 1;
        const points = data.map((val, i) => {
            const x = (i / (data.length - 1)) * (width - 2 * padding) + padding;
            const y = height - ((val / max) * (height - 2 * padding)) - padding;
            return `${x},${y}`;
        }).join(" ");

        return (
            <div className={styles.form} style={{ padding: "1rem" }}>
                <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "0.5rem" }}>
                    <span style={{ fontSize: "0.8rem", color: "var(--text-muted)" }}>{label}</span>
                    <span style={{ fontSize: "0.8rem", fontWeight: "bold" }}>{data[data.length - 1].toLocaleString()}</span>
                </div>
                <svg width="100%" height={height} viewBox={`0 0 ${width} ${height}`}>
                    <polyline
                        fill="none"
                        stroke={color}
                        strokeWidth="2"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        points={points}
                        style={{ transition: "all 0.3s ease" }}
                    />
                    <path
                        d={`M ${points} L ${width - padding},${height - padding} L ${padding},${height - padding} Z`}
                        fill={color}
                        fillOpacity="0.1"
                    />
                </svg>
            </div>
        );
    };

    if (loading) {
        return (
            <div className={styles.emptyState}>
                <Loader2 className={`animate-spin ${styles.emptyIcon}`} size={48} />
                <p>Initializing dashboard...</p>
            </div>
        );
    }

    const currentMetrics = history[history.length - 1];

    return (
        <div className={styles.wrapper}>
            <div className={styles.count} style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <div>
                    <Activity size={18} style={{ marginRight: 8, verticalAlign: "middle" }} />
                    Performance Insights
                </div>
                <button className={styles.actionBtn} onClick={() => setIsPaused(!isPaused)}>
                    {isPaused ? "Resume Monitoring" : "Pause Monitoring"}
                </button>
            </div>

            <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(320px, 1fr))", gap: "1.5rem", marginTop: "1rem" }}>
                {renderLineChart(history.map(h => h.questions), "#38bdf8", "Queries History (Questions)")}
                {renderLineChart(history.map(h => h.threads_connected), "#10b981", "Active Connections")}
                {renderLineChart(history.map(h => h.threads_running), "#f59e0b", "Running Threads")}
                {renderLineChart(history.map(h => h.bytes_sent / 1024), "#8b5cf6", "Traffic Out (KB)")}
            </div>

            <div style={{ marginTop: "2rem" }}>
                <div className={styles.count}>Current Snapshot</div>
                <div className={styles.tableContainer}>
                    <table className={styles.table}>
                        <thead>
                            <tr>
                                <th>Metric</th>
                                <th>Value</th>
                                <th>Description</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td><Zap size={14} style={{ marginRight: 8 }} /> Questions</td>
                                <td><strong>{currentMetrics.questions.toLocaleString()}</strong></td>
                                <td>Total number of queries sent to the server</td>
                            </tr>
                            <tr>
                                <td><Users size={14} style={{ marginRight: 8 }} /> Connections</td>
                                <td><strong>{currentMetrics.threads_connected}</strong></td>
                                <td>Number of currently open connections</td>
                            </tr>
                            <tr>
                                <td><Globe size={14} style={{ marginRight: 8 }} /> Total Traffic</td>
                                <td><strong>{((currentMetrics.bytes_received + currentMetrics.bytes_sent) / 1024 / 1024).toFixed(2)} MB</strong></td>
                                <td>Total data transferred (In + Out)</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    );
}
