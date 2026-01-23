import mysql from "mysql2/promise";
import { getSession } from "./session";

export async function getDbConnection(database?: string) {
    const session = await getSession();
    if (!session) {
        throw new Error("Unauthorized: No session found");
    }

    const { host, user, password, port } = session;

    return await mysql.createConnection({
        host,
        user,
        password,
        port: Number(port) || 3306,
        database, // Optional: connect to a specific DB
        rowsAsArray: false,
    });
}

/**
 * Executes a query and returns the results.
 * Closes the connection automatically.
 */
export async function executeQuery(query: string, params: unknown[] = [], database?: string) {
    const connection = await getDbConnection(database);
    try {
        const [results] = await connection.query(query, params);
        return results;
    } finally {
        await connection.end();
    }
}
