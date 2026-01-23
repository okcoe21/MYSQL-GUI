import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";
import { sanitizeIdentifier } from "@/lib/sanitize";

export const runtime = "nodejs";

export async function POST(req: NextRequest) {
    try {
        const formData = await req.formData();
        const database = formData.get("database") as string;
        const file = formData.get("file") as File;

        if (!database || !file) {
            return NextResponse.json({ success: false, error: "Database and file are required" }, { status: 400 });
        }

        const sanitizedDb = sanitizeIdentifier(database);
        const sqlContent = await file.text();

        // Split SQL into statements
        // Note: This is a basic splitter. Complex SQL with strings containing ; will need a better parser.
        // For now, we use a regex that tries to avoid splitting inside quotes (very simplified)
        const statements = sqlContent
            .split(/;(?=(?:[^']*'[^']*')*[^']*$)/)
            .map(s => s.trim())
            .filter(s => {
                if (!s) return false;
                // Temporarily strip comments to see if there's actual code
                const noSingleLine = s.replace(/(--|#)[^\n]*/g, "");
                const noBlock = noSingleLine.replace(/\/\*[\s\S]*?\*\//g, "");
                return noBlock.trim().length > 0;
            });

        let successCount = 0;
        let errors = [];

        for (const statement of statements) {
            try {
                await executeQuery(statement, [], database);
                successCount++;
            } catch (err: any) {
                console.error("Statement execution error:", statement, err);
                errors.push({ statement: statement.substring(0, 100) + "...", error: err.message });
                // We continue on error by default, like phpMyAdmin
            }
        }

        return NextResponse.json({
            success: true,
            message: `Executed ${successCount} statements.`,
            errors: errors.length > 0 ? errors : undefined
        });
    } catch (error: any) {
        console.error("Import Error:", error);
        return NextResponse.json({ success: false, error: error.message || "Failed to import SQL" }, { status: 500 });
    }
}
