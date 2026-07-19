import React, { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { ArrowLeft, Save, Play, CheckCircle, AlertCircle, Loader2 } from "lucide-react";
import styles from "./settings.module.css";
import { saveLLMConfig, getLLMConfig } from "../../lib/api";
import { llmComplete } from "../../lib/llm";
import { useLLM } from "../../context/LLMContext";

export default function SettingsPage() {
    const navigate = useNavigate();
    const { refreshConfig } = useLLM();
    const [provider, setProvider] = useState("anthropic");
    const [key, setKey] = useState("");
    const [ollamaHost, setOllamaHost] = useState("http://localhost:11434");
    const [ollamaModel, setOllamaModel] = useState("llama3");

    const [loading, setLoading] = useState(false);
    const [testing, setTesting] = useState(false);
    const [message, setMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);

    useEffect(() => {
        loadConfig();
    }, []);

    const loadConfig = async () => {
        setLoading(true);
        try {
            const res = await getLLMConfig();
            if (res.success && res.config) {
                setProvider(res.config.provider);
                setKey(res.config.key || "");
                setOllamaHost(res.config.ollama_host || "http://localhost:11434");
                setOllamaModel(res.config.ollama_model || "llama3");
            }
        } catch (err) {
            setMessage({ type: "error", text: "Failed to load LLM configuration" });
        } finally {
            setLoading(false);
        }
    };

    const handleSave = async (e: React.FormEvent) => {
        e.preventDefault();
        setLoading(true);
        setMessage(null);

        try {
            const res = await saveLLMConfig(provider, key, ollamaHost, ollamaModel);
            if (res.success) {
                setMessage({ type: "success", text: "LLM configuration saved securely in OS Keychain!" });
                await refreshConfig();
                // Reload configuration to get masked keys representation
                await loadConfig();
            } else {
                setMessage({ type: "error", text: "Failed to save configuration" });
            }
        } catch (err: any) {
            setMessage({ type: "error", text: err.message || "An error occurred while saving" });
        } finally {
            setLoading(false);
        }
    };

    const handleTest = async () => {
        setTesting(true);
        setMessage(null);

        try {
            // If the key is masked, we should fetch the unmasked key from the backend to run the test.
            let testKey = key;
            if (key.includes("...") || key === "****") {
                const { getLLMConfigUnmasked } = await import("../../lib/api");
                const res = await getLLMConfigUnmasked();
                if (res.success && res.config) {
                    testKey = res.config.key || "";
                }
            }

            const testConfig = {
                provider,
                key: testKey,
                ollama_host: ollamaHost,
                ollama_model: ollamaModel
            };

            const schema = {
                tables: ["users"],
                columns: {
                    users: [{ name: "id", type: "INT" }, { name: "email", type: "VARCHAR" }]
                }
            };

            const result = await llmComplete("Select all users", schema, testConfig);
            if (result && result.text) {
                setMessage({
                    type: "success",
                    text: `Connection test successful! Response: "${result.text}"`
                });
            } else {
                setMessage({ type: "error", text: "Test failed: Received empty response from provider." });
            }
        } catch (err: any) {
            setMessage({ type: "error", text: `Test failed: ${err.message || err}` });
        } finally {
            setTesting(false);
        }
    };

    if (loading && !key) {
        return (
            <div className={styles.container}>
                <div style={{ display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", minHeight: "200px" }}>
                    <Loader2 className="animate-spin" size={48} color="#7c3aed" />
                    <p style={{ marginTop: "1rem", fontWeight: 700 }}>Loading keychain configuration...</p>
                </div>
            </div>
        );
    }

    return (
        <div className={styles.container}>
            <div className={styles.header}>
                <h1 className={styles.title}>LLM settings</h1>
                <button className={styles.backBtn} onClick={() => navigate("/dashboard")}>
                    <ArrowLeft size={16} />
                    Back
                </button>
            </div>

            <div className={styles.card}>
                <form onSubmit={handleSave}>
                    <div className={styles.formGroup}>
                        <label className={styles.label}>LLM Provider</label>
                        <select
                            className={styles.select}
                            value={provider}
                            onChange={(e) => setProvider(e.target.value)}
                        >
                            <option value="anthropic">Anthropic (Claude)</option>
                            <option value="openai">OpenAI (GPT)</option>
                            <option value="gemini">Google Gemini</option>
                            <option value="ollama">Ollama (Local LLM)</option>
                        </select>
                    </div>

                    {provider !== "ollama" && (
                        <div className={styles.formGroup}>
                            <label className={styles.label}>API Key</label>
                            <input
                                type="password"
                                className={styles.input}
                                placeholder="Enter API Key (will be stored in OS keychain)"
                                value={key}
                                onChange={(e) => setKey(e.target.value)}
                            />
                        </div>
                    )}

                    {provider === "ollama" && (
                        <>
                            <div className={styles.formGroup}>
                                <label className={styles.label}>Ollama Host</label>
                                <input
                                    type="text"
                                    className={styles.input}
                                    placeholder="http://localhost:11434"
                                    value={ollamaHost}
                                    onChange={(e) => setOllamaHost(e.target.value)}
                                />
                            </div>
                            <div className={styles.formGroup}>
                                <label className={styles.label}>Ollama Model</label>
                                <input
                                    type="text"
                                    className={styles.input}
                                    placeholder="llama3"
                                    value={ollamaModel}
                                    onChange={(e) => setOllamaModel(e.target.value)}
                                />
                            </div>
                        </>
                    )}

                    {message && (
                        <div className={`${styles.message} ${message.type === "success" ? styles.successMessage : styles.errorMessage}`}>
                            {message.type === "success" ? <CheckCircle size={18} /> : <AlertCircle size={18} />}
                            <div>{message.text}</div>
                        </div>
                    )}

                    <div className={styles.actions}>
                        <button type="submit" className={styles.btnPrimary} disabled={loading}>
                            {loading ? <Loader2 className="animate-spin" size={16} /> : <Save size={16} />}
                            Save Configuration
                        </button>
                        <button type="button" className={styles.btnSecondary} onClick={handleTest} disabled={testing}>
                            {testing ? <Loader2 className="animate-spin" size={16} /> : <Play size={16} />}
                            Test Connection
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
}
