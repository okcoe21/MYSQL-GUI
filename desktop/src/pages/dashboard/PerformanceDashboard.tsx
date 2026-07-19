

import { useState, useEffect, useRef } from "react";
import styles from "./dashboard.module.css";
import { Activity, Loader2, RefreshCw, Zap, Users, Globe, Database } from "lucide-react";
import * as api from "../../lib/api";

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
            const data = await api.getServerMetrics();
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
            <div className={styles.form} style={{ padding: "var(--space-4)" }}>
                <div className={`${styles.flexBetween} ${styles.marginBottomSm}`}>
                    <span className={styles.metricLabel}>{label}</span>
                    <span style={{ fontSize: "var(--font-xs)", fontWeight: "bold" }}>{data[data.length - 1].toLocaleString()}</span>
                </div>
                <svg width="100%" height={height} viewBox={`0 0 ${width} ${height}`}>
                    <polyline
                        fill="none"
                        stroke={color}
                        strokeWidth="2"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        points={points}
                        style={{ transition: "var(--transition-fast)" }}
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
            <div className={`${styles.count} ${styles.flexBetween}`}>
                <div>
                    <Activity size={18} className={styles.tabIcon} />
                    Performance Insights
                </div>
                <button className={styles.actionBtn} onClick={() => setIsPaused(!isPaused)}>
                    {isPaused ? "Resume Monitoring" : "Pause Monitoring"}
                </button>
            </div>

            <div className={styles.metricGrid} style={{ gridTemplateColumns: "repeat(auto-fit, minmax(320px, 1fr))" }}>
                {renderLineChart(history.map(h => h.questions), "var(--accent)", "Queries History (Questions)")}
                {renderLineChart(history.map(h => h.threads_connected), "var(--text-muted)", "Active Connections")}
                {renderLineChart(history.map(h => h.threads_running), "var(--accent)", "Running Threads")}
                {renderLineChart(history.map(h => h.bytes_sent / 1024), "var(--text-muted)", "Traffic Out (KB)")}
            </div>

            <div className={styles.marginTopMd}>
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
                                <td><Zap size={14} className={styles.tabIcon} /> Questions</td>
                                <td><strong>{currentMetrics.questions.toLocaleString()}</strong></td>
                                <td>Total number of queries sent to the server</td>
                            </tr>
                            <tr>
                                <td><Users size={14} className={styles.tabIcon} /> Connections</td>
                                <td><strong>{currentMetrics.threads_connected}</strong></td>
                                <td>Number of currently open connections</td>
                            </tr>
                            <tr>
                                <td><Globe size={14} className={styles.tabIcon} /> Total Traffic</td>
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
