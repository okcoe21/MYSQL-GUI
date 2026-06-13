import mysql from "mysql2/promise";
import { getSession } from "./session";

// Global cache to persist pools across hot-reloads in development
const globalForDb = global as unknown as {
    pools: Map<string, mysql.Pool>;
};

const pools = globalForDb.pools || new Map<string, mysql.Pool>();

if (process.env.NODE_ENV !== "production") {
    globalForDb.pools = pools;
}

export async function getDbConnection(database?: string) {
    const session = await getSession();
    if (!session) {
        throw new Error("Unauthorized: No session found");
    }

    const { host, user, password, port } = session;
    const key = `${host}:${port}:${user}:${password || ""}:${database || ""}`;

    let pool = pools.get(key);
    if (!pool) {
        pool = mysql.createPool({
            host,
            user,
            password,
            port: Number(port) || 3306,
            database, // Optional: connect to a specific DB
            rowsAsArray: false,
            waitForConnections: true,
            connectionLimit: 10,
            queueLimit: 0,
            idleTimeout: 60000, // Close idle connections after 60s
            enableKeepAlive: true,
            keepAliveInitialDelay: 10000,
        });
        pools.set(key, pool);
    }

    return await pool.getConnection();
}

/**
 * Executes a query and returns the results.
 * Releases the connection back to the pool automatically.
 */
export async function executeQuery(query: string, params: unknown[] = [], database?: string) {
    const connection = await getDbConnection(database);
    try {
        const [results] = await connection.query(query, params);
        return results;
    } finally {
        connection.release();
    }
}

