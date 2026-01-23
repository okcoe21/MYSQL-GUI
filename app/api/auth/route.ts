import { NextRequest, NextResponse } from "next/server";
import mysql from "mysql2/promise";
import { login, logout } from "@/lib/session";

export const runtime = "nodejs";

export async function POST(req: NextRequest) {
    try {
        const { host, user, password, port } = await req.json();

        // 1. Attempt to connect to MySQL server to validate credentials
        const connection = await mysql.createConnection({
            host,
            user,
            password,
            port: Number(port) || 3306,
        });

        await connection.end();

        // 2. If successful, encrypted credentials and store in session cookie
        await login({ host, user, password, port });

        return NextResponse.json({ success: true });
    } catch (error: any) {
        console.error("Login Error:", error);
        return NextResponse.json(
            { success: false, error: error.message || "Failed to connect to MySQL server" },
            { status: 401 }
        );
    }
}

export async function DELETE() {
    await logout();
    return NextResponse.json({ success: true });
}
