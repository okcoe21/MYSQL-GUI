import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";
import { sanitizeIdentifier } from "@/lib/sanitize";

export const runtime = "nodejs";

export async function DELETE(req: NextRequest) {
    try {
        const { database, confirmed } = await req.json();

        if (!database) {
            return NextResponse.json({ success: false, error: "Database name is required" }, { status: 400 });
        }

        if (!confirmed) {
            return NextResponse.json({
                success: false,
                error: "CONFIRMATION_REQUIRED",
                message: `Are you sure you want to DROP the entire database [${database}]? All tables and data will be permanently deleted.`
            });
        }

        const sanitizedDb = sanitizeIdentifier(database);
        const query = `DROP DATABASE ${sanitizedDb}`;

        await executeQuery(query);

        return NextResponse.json({ success: true });
    } catch (error: any) {
        console.error("Drop Database Error:", error);
        return NextResponse.json(
            { success: false, error: error.message || "Failed to drop database" },
            { status: 500 }
        );
    }
}
