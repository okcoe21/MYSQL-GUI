import React, { createContext, useContext, useState, useEffect } from "react";
import { getLLMConfig } from "../lib/api";

export interface LLMConfigData {
    provider: string;
    key?: string;
    ollama_host?: string;
    ollama_model?: string;
}

interface LLMContextType {
    config: LLMConfigData | null;
    loading: boolean;
    refreshConfig: () => Promise<void>;
}

const LLMContext = createContext<LLMContextType | undefined>(undefined);

export function LLMProvider({ children }: { children: React.ReactNode }) {
    const [config, setConfig] = useState<LLMConfigData | null>(null);
    const [loading, setLoading] = useState(true);

    const refreshConfig = async () => {
        setLoading(true);
        try {
            const res = await getLLMConfig();
            if (res.success && res.config) {
                setConfig(res.config);
            }
        } catch (err) {
            console.error("Failed to load LLM config:", err);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        refreshConfig();
    }, []);

    return (
        <LLMContext.Provider value={{ config, loading, refreshConfig }}>
            {children}
        </LLMContext.Provider>
    );
}

export function useLLM() {
    const context = useContext(LLMContext);
    if (!context) {
        throw new Error("useLLM must be used within an LLMProvider");
    }
    return context;
}
