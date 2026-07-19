import React, { createContext, useContext, useState } from "react";

interface ConnectionContextType {
    host: string;
    port: string;
    user: string;
    currentDb: string | undefined;
    setConnection: (conn: { host: string; port: string; user: string }) => void;
    setCurrentDb: (db: string | undefined) => void;
    clearConnection: () => void;
}

const ConnectionContext = createContext<ConnectionContextType | undefined>(undefined);

export function ConnectionProvider({ children }: { children: React.ReactNode }) {
    const [host, setHost] = useState("");
    const [port, setPort] = useState("");
    const [user, setUser] = useState("");
    const [currentDb, setCurrentDb] = useState<string | undefined>();

    const setConnection = (conn: { host: string; port: string; user: string }) => {
        setHost(conn.host);
        setPort(conn.port);
        setUser(conn.user);
    };

    const clearConnection = () => {
        setHost("");
        setPort("");
        setUser("");
        setCurrentDb(undefined);
    };

    return (
        <ConnectionContext.Provider
            value={{
                host,
                port,
                user,
                currentDb,
                setConnection,
                setCurrentDb,
                clearConnection,
            }}
        >
            {children}
        </ConnectionContext.Provider>
    );
}

export function useConnection() {
    const context = useContext(ConnectionContext);
    if (!context) {
        throw new Error("useConnection must be used within a ConnectionProvider");
    }
    return context;
}
