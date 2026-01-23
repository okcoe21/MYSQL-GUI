import { NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";

export const runtime = "nodejs";

export async function GET() {
    try {
        const results: any = await executeQuery("SHOW DATABASES");
        console.log("SHOW DATABASES results:", JSON.stringify(results));

        const databases = results.map((row: any) => {
            const dbName = row.Database || row.database || Object.values(row)[0];
            return dbName;
        });

        console.log("Extracted databases:", databases);

        return NextResponse.json({ success: true, databases });
    } catch (error: any) {
        console.error("Fetch Databases Error:", error);
        return NextResponse.json(
            { success: false, error: error.message || "Failed to fetch databases" },
            { status: 500 }
        );
    }
}
