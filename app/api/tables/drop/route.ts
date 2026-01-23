import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";
import { sanitizeIdentifier } from "@/lib/sanitize";

export const runtime = "nodejs";

export async function DELETE(req: NextRequest) {
    try {
        const { database, table, confirmed } = await req.json();

        if (!database || !table) {
            return NextResponse.json({ success: false, error: "Database and Table names are required" }, { status: 400 });
        }

        if (!confirmed) {
            return NextResponse.json({ success: false, error: "CONFIRMATION_REQUIRED", message: `Are you sure you want to DROP the table [${table}]? This action cannot be undone.` });
        }

        const sanitizedDb = sanitizeIdentifier(database);
        const sanitizedTable = sanitizeIdentifier(table);

        const query = `DROP TABLE ${sanitizedDb}.${sanitizedTable}`;

        await executeQuery(query);

        return NextResponse.json({ success: true });
    } catch (error: any) {
        console.error("Drop Table Error:", error);
        return NextResponse.json(
            { success: false, error: error.message || "Failed to drop table" },
            { status: 500 }
        );
    }
}
