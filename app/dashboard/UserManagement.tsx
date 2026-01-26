"use client";

import { useState, useEffect } from "react";
import styles from "./dashboard.module.css";
import { Users, Shield, Lock, Unlock, AlertCircle, Loader2 } from "lucide-react";

export default function UserManagement() {
    const [users, setUsers] = useState<any[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState("");
    const [note, setNote] = useState("");

    useEffect(() => {
        fetchUsers();
    }, []);

    const fetchUsers = async () => {
        setLoading(true);
        setError("");
        try {
            const res = await fetch("/api/users");
            const result = await res.json();
            if (result.success) {
                setUsers(result.users);
                if (result.note) setNote(result.note);
            } else {
                setError(result.error);
            }
        } catch {
            setError("Failed to fetch users");
        } finally {
            setLoading(false);
        }
    };

    if (loading) {
        return (
            <div className={styles.emptyState}>
                <Loader2 className={`animate-spin ${styles.emptyIcon}`} size={48} />
                <p>Loading user management...</p>
            </div>
        );
    }

    return (
        <div className={styles.wrapper}>
            <div className={styles.count} style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <div>
                    <Users size={18} style={{ marginRight: 8, verticalAlign: "middle" }} />
                    User Management - {users.length} users
                </div>
                <button className={styles.actionBtn} onClick={fetchUsers}>Refresh</button>
            </div>

            {note && (
                <div className={styles.warning} style={{ marginBottom: "1rem" }}>
                    <Shield size={16} style={{ marginRight: 8, verticalAlign: "middle" }} />
                    {note}
                </div>
            )}

            {error && (
                <div className={styles.warning} style={{ borderColor: "#ef4444", color: "#ef4444", marginBottom: "1rem" }}>
                    <AlertCircle size={16} style={{ marginRight: 8, verticalAlign: "middle" }} />
                    {error}
                </div>
            )}

            <div className={styles.tableContainer}>
                <table className={styles.table}>
                    <thead>
                        <tr>
                            <th>User</th>
                            <th>Host</th>
                            <th>Status</th>
                            <th>Privileges</th>
                        </tr>
                    </thead>
                    <tbody>
                        {users.map((u, i) => (
                            <tr key={`${u.user}-${u.host}-${i}`}>
                                <td>
                                    <div style={{ display: "flex", alignItems: "center", gap: "8px", fontWeight: 500 }}>
                                        {u.user}
                                    </div>
                                </td>
                                <td><code>{u.host}</code></td>
                                <td>
                                    {u.account_locked === "Y" ? (
                                        <span style={{ color: "#ef4444", display: "flex", alignItems: "center", gap: 4 }}>
                                            <Lock size={12} /> Locked
                                        </span>
                                    ) : (
                                        <span style={{ color: "#10b981", display: "flex", alignItems: "center", gap: 4 }}>
                                            <Unlock size={12} /> Active
                                        </span>
                                    )}
                                </td>
                                <td>
                                    <button className={styles.actionBtn} style={{ padding: "4px 8px", fontSize: "0.7rem" }}>
                                        View Grants
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
