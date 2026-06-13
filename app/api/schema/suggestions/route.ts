import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export async function GET(req: NextRequest) {
    const { searchParams } = new URL(req.url);
    const database = searchParams.get("database");

    if (!database) {
        return NextResponse.json({ error: "Database name is required" }, { status: 400 });
    }

    try {
        const query = `
            SELECT TABLE_NAME, COLUMN_NAME, DATA_TYPE 
            FROM INFORMATION_SCHEMA.COLUMNS 
            WHERE TABLE_SCHEMA = ?
            ORDER BY TABLE_NAME, ORDINAL_POSITION
        `;
        const results: any = await executeQuery(query, [database]);

        const tablesSet = new Set<string>();
        const columns: { [tableName: string]: { name: string, type: string }[] } = {};

        for (const row of results) {
            const tableName = row.TABLE_NAME;
            const columnName = row.COLUMN_NAME;
            const dataType = row.DATA_TYPE;

            tablesSet.add(tableName);

            if (!columns[tableName]) {
                columns[tableName] = [];
            }
            columns[tableName].push({
                name: columnName,
                type: dataType
            });
        }

        const tables = Array.from(tablesSet);

        return NextResponse.json({
            tables,
            columns
        });
    } catch (error: any) {
        console.error("Schema Suggestions Error:", error);
        if (error.message?.includes("Unauthorized")) {
            return NextResponse.json({ error: error.message }, { status: 401 });
        }
        return NextResponse.json(
            { error: error.message || "Failed to fetch schema suggestions" },
            { status: 500 }
        );
    }
}
