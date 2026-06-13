"use client";

import { useState, useEffect, useRef } from "react";
import styles from "./dashboard.module.css";
import { Play, AlertTriangle, CheckCircle } from "lucide-react";
import { getSuggestions, SuggestionItem, SchemaData } from "@/lib/sqlAutocomplete";

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
    const [query, setQuery] = useState(externalQuery || "");
    const [loading, setLoading] = useState(false);
    const [results, setResults] = useState<QueryResult | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [showWarning, setShowWarning] = useState(false);

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
        fetch(`/api/schema/suggestions?database=${encodeURIComponent(initialDatabase)}`)
            .then((res) => res.json())
            .then((data) => {
                if (isMounted) {
                    schemaDataRef.current = data;
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
            const res = await fetch("/api/query", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    query,
                    database: initialDatabase,
                    confirmed
                }),
            });

            const data: QueryResult = await res.json();

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

    return (
        <div className={styles.wrapper}>
            <div className={styles.editorGroup}>
                <div className={styles.editorWrapper}>
                    <textarea
                        ref={textareaRef}
                        className={styles.editor}
                        value={query}
                        onChange={handleChange}
                        onKeyDown={handleKeyDown}
                        onBlur={handleBlur}
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
