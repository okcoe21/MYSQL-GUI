import { useState } from "react";
import { useNavigate } from "react-router-dom";
import styles from "../login.module.css";
import * as api from "../lib/api";

export default function LoginPage() {
    const navigate = useNavigate();
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState("");
    const [formData, setFormData] = useState({
        host: "localhost",
        user: "root",
        password: "",
        port: "3306",
    });

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setLoading(true);
        setError("");

        try {
            const data = await api.login(formData.host, formData.port, formData.user, formData.password || undefined);

            if (data.success) {
                navigate("/dashboard");
            } else {
                setError(data.error || "Failed to connect to MySQL");
            }
        } catch (err) {
            setError("An unexpected error occurred");
        } finally {
            setLoading(false);
        }
    };

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const { name, value } = e.target;
        setFormData((prev) => ({ ...prev, [name]: value }));
    };

    return (
        <div className={styles.container}>
            <div className={styles.card}>
                <h1 className={styles.title}>MySQL GUI</h1>
                <p className={styles.subtitle}>Connect to your MySQL server</p>

                {error && <div className={styles.error}>{error}</div>}

                <form onSubmit={handleSubmit}>
                    <div className={styles.formGroup}>
                        <label className={styles.label}>Host</label>
                        <input
                            type="text"
                            name="host"
                            value={formData.host}
                            onChange={handleChange}
                            className={styles.input}
                            required
                        />
                    </div>

                    <div className={styles.formGroup}>
                        <label className={styles.label}>Port</label>
                        <input
                            type="text"
                            name="port"
                            value={formData.port}
                            onChange={handleChange}
                            className={styles.input}
                            required
                        />
                    </div>

                    <div className={styles.formGroup}>
                        <label className={styles.label}>Username</label>
                        <input
                            type="text"
                            name="user"
                            value={formData.user}
                            onChange={handleChange}
                            className={styles.input}
                            required
                        />
                    </div>

                    <div className={styles.formGroup}>
                        <label className={styles.label}>Password</label>
                        <input
                            type="password"
                            name="password"
                            value={formData.password}
                            onChange={handleChange}
                            className={styles.input}
                        />
                    </div>

                    <button type="submit" className={styles.button} disabled={loading}>
                        {loading ? "Connecting..." : "Connect"}
                    </button>
                </form>
            </div>
        </div>
    );
}
