import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export async function GET(req: NextRequest) {
    const { searchParams } = new URL(req.url);
    const database = searchParams.get("database");

    if (!database) {
        return NextResponse.json({ success: false, error: "Database name is required" }, { status: 400 });
    }

    try {
        // Fetch stats from information_schema
        // We use ? as a parameter to safely pass the database name
        const results: any = await executeQuery(
            `SELECT 
                TABLE_NAME as tableName, 
                TABLE_ROWS as rowCount, 
                DATA_LENGTH as dataSize, 
                ENGINE as engine,
                TABLE_COLLATION as collation
             FROM information_schema.tables 
             WHERE TABLE_SCHEMA = ?`,
            [database]
        );

        return NextResponse.json({ success: true, stats: results });
    } catch (error: any) {
        console.error("Fetch DB Stats Error:", error);
        return NextResponse.json(
            { success: false, error: error.message || "Failed to fetch database statistics" },
            { status: 500 }
        );
    }
}
