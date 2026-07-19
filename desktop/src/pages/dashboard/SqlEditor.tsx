import { useState, useEffect, useRef } from "react";
import styles from "./dashboard.module.css";
import { Play, AlertTriangle, CheckCircle } from "lucide-react";
import { getSuggestions, SuggestionItem, SchemaData } from "../../lib/sqlAutocomplete";
import { explainQuery } from "../../lib/sqlExplainer";
import * as api from "../../lib/api";
import { useLLM } from "../../context/LLMContext";
import { llmComplete } from "../../lib/llm";


interface SqlEditorProps {
    initialDatabase?: string;
    externalQuery?: string;
}

interface QueryResult {
    success: boolean;
    data?: any[];
    columns?: string[];
    affectedRows?: number | null;
    message?: string;
    error?: string;
}

export default function SqlEditor({ initialDatabase, externalQuery }: SqlEditorProps) {
    const { config: llmConfig, loading: llmLoading } = useLLM();
    const [query, setQuery] = useState(externalQuery || "");
    const [loading, setLoading] = useState(false);
    const [results, setResults] = useState<QueryResult | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [showWarning, setShowWarning] = useState(false);
    const [explainText, setExplainText] = useState("");
    const [explainVisible, setExplainVisible] = useState(false);

    // Natural Language to SQL states
    const [nlAvailable, setNlAvailable] = useState(false);
    const [nlProvider, setNlProvider] = useState<string | null>(null);
    const [nlModel, setNlModel] = useState<string | null>(null);
    const [nlPrompt, setNlPrompt] = useState("");
    const [nlLoading, setNlLoading] = useState(false);
    const [nlError, setNlError] = useState("");
    const [flashActive, setFlashActive] = useState(false);

    useEffect(() => {
        if (!llmLoading && llmConfig) {
            const hasKey = llmConfig.provider === "ollama" || (llmConfig.key && llmConfig.key !== "");
            setNlAvailable(!!hasKey);
            setNlProvider(llmConfig.provider);
            setNlModel(llmConfig.provider === "anthropic" ? "claude-haiku-3-5" :
                        llmConfig.provider === "openai" ? "gpt-4o-mini" :
                        llmConfig.provider === "gemini" ? "gemini-1.5-flash" :
                        llmConfig.ollama_model || "llama3");
        }
    }, [llmConfig, llmLoading]);


    // Autocomplete states
    const [suggestions, setSuggestions] = useState<SuggestionItem[]>([]);
    const [selectedIndex, setSelectedIndex] = useState(0);
    const [dropdownVisible, setDropdownVisible] = useState(false);

    const schemaDataRef = useRef<SchemaData | null>(null);
    const textareaRef = useRef<HTMLTextAreaElement | null>(null);

    useEffect(() => {
        if (!initialDatabase) {
            schemaDataRef.current = null;
            return;
        }

        let isMounted = true;
        api.getSchemaSuggestions(initialDatabase)
            .then((data) => {
                if (isMounted) {
                    schemaDataRef.current = data as any;
                }
            })
            .catch((err) => {
                console.error("Failed to fetch schema suggestions:", err);
            });

        return () => {
            isMounted = false;
        };
    }, [initialDatabase]);

    const triggerAutocomplete = (text: string, cursorPos: number) => {
        if (!schemaDataRef.current) {
            setSuggestions([]);
            setDropdownVisible(false);
            return;
        }
        const list = getSuggestions(text, cursorPos, schemaDataRef.current);
        setSuggestions(list);
        setSelectedIndex(0);
        setDropdownVisible(list.length > 0);
    };

    const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
        const val = e.target.value;
        const cursorPos = e.target.selectionStart;
        setQuery(val);
        triggerAutocomplete(val, cursorPos);
    };

    const insertSuggestion = (suggestion: SuggestionItem) => {
        const textarea = textareaRef.current;
        if (!textarea) return;

        const cursorPos = textarea.selectionStart;
        const textBefore = query.slice(0, cursorPos);
        const textAfter = query.slice(cursorPos);

        const currentTokenMatch = textBefore.match(/([a-zA-Z0-9_]+)$/);
        const currentToken = currentTokenMatch ? currentTokenMatch[1] : "";

        const beforeToken = textBefore.slice(0, -currentToken.length);
        const replacement = suggestion.label;

        const newQuery = beforeToken + replacement + textAfter;
        setQuery(newQuery);
        setSuggestions([]);
        setDropdownVisible(false);

        const newCursorPos = beforeToken.length + replacement.length;
        setTimeout(() => {
            textarea.focus();
            textarea.setSelectionRange(newCursorPos, newCursorPos);
        }, 0);
    };

    const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
        if (dropdownVisible && suggestions.length > 0) {
            if (e.key === "ArrowDown") {
                e.preventDefault();
                setSelectedIndex((prev) => (prev + 1) % suggestions.length);
            } else if (e.key === "ArrowUp") {
                e.preventDefault();
                setSelectedIndex((prev) => (prev - 1 + suggestions.length) % suggestions.length);
            } else if (e.key === "Enter" || e.key === "Tab") {
                e.preventDefault();
                insertSuggestion(suggestions[selectedIndex]);
            } else if (e.key === "Escape") {
                e.preventDefault();
                setDropdownVisible(false);
            }
        }
    };

    const handleBlur = () => {
        setTimeout(() => {
            setDropdownVisible(false);
        }, 150);
    };

    useEffect(() => {
        if (externalQuery) {
            setQuery(externalQuery);
        }
    }, [externalQuery]);

    const saveToHistory = (executedQuery: string) => {
        const saved = localStorage.getItem("sql_history");
        const history = saved ? JSON.parse(saved) : [];
        const newItem = {
            query: executedQuery,
            timestamp: Date.now(),
            database: initialDatabase,
        };
        // Keep unique queries, or just add all? Let's add all but limit to 50
        const updated = [newItem, ...history].slice(0, 50);
        localStorage.setItem("sql_history", JSON.stringify(updated));
    };

    const runQuery = async (confirmed = false) => {
        setLoading(true);
        setError(null);
        setShowWarning(false);

        try {
            const data: QueryResult = await api.executeSqlQuery(query, initialDatabase, confirmed);

            if (data.success) {
                setResults(data);
                if (!confirmed) saveToHistory(query);
            } else if (data.error === "DESTRUCTIVE_QUERY") {
                setShowWarning(true);
                setError(data.message || "Destructive query detected");
            } else {
                setError(data.error || "Failed to execute query");
            }
        } catch {
            setError("Failed to execute query");
        } finally {
            setLoading(false);
        }
    };

    const handleClear = () => {
        setQuery("");
        setResults(null);
        setError(null);
    };

    const handleFormat = () => {
        const keywords = ["SELECT", "FROM", "WHERE", "GROUP BY", "ORDER BY", "LIMIT", "INSERT INTO", "VALUES", "UPDATE", "SET", "DELETE", "JOIN", "LEFT JOIN", "RIGHT JOIN", "ON", "AND", "OR", "IN"];
        let formatted = query;
        keywords.forEach(kw => {
            const regex = new RegExp(`\\b${kw}\\b`, "gi");
            formatted = formatted.replace(regex, kw);
        });
        setQuery(formatted);
    };

    const handleExplain = () => {
        const text = explainQuery(query);
        setExplainText(text);
        setExplainVisible(true);
    };

    const handleGenerate = async () => {
        if (!nlPrompt.trim() || nlLoading) return;
        setNlLoading(true);
        setNlError("");

        try {
            const { getLLMConfigUnmasked } = await import("../../lib/api");
            const configRes = await getLLMConfigUnmasked();
            if (!configRes.success || !configRes.config) {
                throw new Error(configRes.error || "Failed to load LLM configuration from OS Keychain");
            }

            const result = await llmComplete(nlPrompt, schemaDataRef.current, configRes.config);
            let sql = result.text.trim();
            if (sql.startsWith("```")) {
                sql = sql.replace(/^```(sql)?\s*/i, "").replace(/\s*```$/, "").trim();
            }

            setQuery(sql);
            setNlPrompt("");
            setFlashActive(true);

            if (textareaRef.current) {
                const textarea = textareaRef.current;
                textarea.value = sql;
                const event = new Event("input", { bubbles: true });
                textarea.dispatchEvent(event);
                textarea.focus();
            }
        } catch (err: any) {
            setNlError(err.message || "Failed to generate query");
        } finally {
            setNlLoading(false);
        }
    };

    const renderExplanation = (text: string) => {
        if (!text.includes('⚠')) {
            return <span>{text}</span>;
        }
        const warningMatch = text.match(/^(⚠[^.]+?\.)\s*(.*)$/);
        if (warningMatch) {
            const [_, warningPart, restPart] = warningMatch;
            return (
                <>
                    <span className={styles.warningText}>{warningPart}</span>
                    {restPart && <span> {restPart}</span>}
                </>
            );
        }
        return <span className={styles.warningText}>{text}</span>;
    };

    return (
        <div className={styles.wrapper}>
            {nlAvailable && (
                <div className={styles.nlSection}>
                    <div className={styles.nlLabelRow}>
                        <span className={styles.nlLabel}>✦ NATURAL LANGUAGE → SQL</span>
                        <span className={styles.nlProviderBadge}>via {nlModel}</span>
                    </div>
                    <div className={styles.nlInputRow}>
                        <input
                            type="text"
                            className={styles.nlInput}
                            placeholder="e.g. show all users who signed up last month"
                            value={nlPrompt}
                            onChange={(e) => {
                                setNlPrompt(e.target.value);
                                setNlError("");
                            }}
                            onKeyDown={(e) => {
                                if (e.key === "Enter") {
                                    handleGenerate();
                                }
                            }}
                            disabled={nlLoading}
                        />
                        <button
                            className={`${styles.btnGenerate} ${nlLoading ? styles.loading : ""}`}
                            onClick={handleGenerate}
                            disabled={nlLoading || !nlPrompt.trim()}
                        >
                            Generate →
                        </button>
                    </div>
                    {nlError && <div className={styles.nlError}>{nlError}</div>}
                </div>
            )}
            <div className={styles.editorGroup}>
                <div className={styles.editorWrapper}>
                    <textarea
                        ref={textareaRef}
                        className={`${styles.editor} ${flashActive ? styles.sqlTextareaFlash : ""}`}
                        value={query}
                        onChange={handleChange}
                        onKeyDown={handleKeyDown}
                        onBlur={handleBlur}
                        onTransitionEnd={() => setFlashActive(false)}
                        placeholder="Enter your SQL query here... (e.g. SELECT * FROM users LIMIT 10)"
                        spellCheck={false}
                    />
                    {dropdownVisible && suggestions.length > 0 && (
                        <div className={styles.autocompleteDropdown}>
                            {suggestions.map((item, idx) => (
                                <div
                                    key={idx}
                                    className={`${styles.autocompleteRow} ${
                                        idx === selectedIndex ? styles.autocompleteRowActive : ""
                                    }`}
                                    onClick={() => insertSuggestion(item)}
                                >
                                    <span className={styles.autocompleteLabel}>{item.label}</span>
                                    <span className={styles.autocompleteKind}>{item.kind}</span>
                                </div>
                            ))}
                        </div>
                    )}
                </div>
                <div className={styles.controls}>
                    <div className={`${styles.flexRow} ${styles.flexGapSm}`}>
                        <button className={styles.actionBtn} onClick={handleClear} title="Clear Query">
                            Clear
                        </button>
                        <button className={styles.actionBtn} onClick={handleFormat} title="Format Query">
                            Format
                        </button>
                    </div>
                    <div className={`${styles.flexRow} ${styles.flexGapSm}`}>
                        <button
                            className={styles.actionBtn}
                            onClick={handleExplain}
                            disabled={!query.trim()}
                            title="Explain Query"
                        >
                            Explain ✦
                        </button>
                        <button
                            className={styles.primaryBtn}
                            onClick={() => runQuery(false)}
                            disabled={loading || !query.trim()}
                        >
                            <Play size={16} className={styles.tabIcon} />
                            Run Query
                        </button>
                    </div>
                </div>
            </div>

            {explainVisible && (
                <div className={styles.explainPanel}>
                    <div className={styles.explainHeader}>
                        <span className={styles.explainTitle}>✦ QUERY EXPLANATION</span>
                        <button
                            className={styles.explainCloseBtn}
                            onClick={() => {
                                setExplainVisible(false);
                                setExplainText("");
                            }}
                            title="Close Explanation"
                        >
                            ✕
                        </button>
                    </div>
                    <div className={styles.explainBody}>
                        {renderExplanation(explainText)}
                    </div>
                </div>
            )}

            {showWarning && (
                <div className={`${styles.warning} ${styles.marginTopSm}`}>
                    <div className={`${styles.flexRow} ${styles.flexGapSm}`}>
                        <AlertTriangle size={20} />
                        <strong>Warning: Destructive Query Detected</strong>
                    </div>
                    <p style={{ margin: "var(--space-2) 0" }}>{error}</p>
                    <div className={styles.warningControls}>
                        <button className={styles.confirmBtn} onClick={() => runQuery(true)}>Yes, Execute</button>
                        <button className={styles.cancelBtn} onClick={() => { setShowWarning(false); setError(null); }}>Cancel</button>
                    </div>
                </div>
            )}

            {!showWarning && error && (
                <div className={`${styles.warning} ${styles.marginTopSm}`} style={{ borderColor: "#ef4444", color: "#ef4444", backgroundColor: "var(--bg-base)" }}>
                    {error}
                </div>
            )}

            {results && results.success && (
                <div className={styles.resultsArea}>
                    {results.affectedRows !== undefined && results.affectedRows !== null ? (
                        <div className={styles.successMessage}>
                            <CheckCircle size={16} className={styles.tabIcon} />
                            Query successful. Affected rows: {results.affectedRows}
                        </div>
                    ) : (
                        <div className={styles.tableContainer}>
                            <table className={styles.table}>
                                <thead>
                                    <tr>
                                        {results.columns?.map((col: string) => (
                                            <th key={col}>{col}</th>
                                        ))}
                                    </tr>
                                </thead>
                                <tbody>
                                    {!results.data || results.data.length === 0 ? (
                                        <tr>
                                            <td colSpan={results.columns?.length || 1} style={{ textAlign: "center", padding: "var(--space-6)" }}>
                                                Query returned no results
                                            </td>
                                        </tr>
                                    ) : (
                                        results.data.map((row: any, idx: number) => (
                                            <tr key={idx}>
                                                {results.columns?.map((col: string) => (
                                                    <td key={col}>{row[col] === null ? "NULL" : String(row[col])}</td>
                                                ))}
                                            </tr>
                                        ))
                                    )}
                                </tbody>
                            </table>
                        </div>
                    )}
                </div>
            )}
        </div>
    );
}
