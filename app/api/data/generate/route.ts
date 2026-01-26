import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";
import { sanitizeIdentifier } from "@/lib/sanitize";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export async function POST(req: NextRequest) {
    try {
        const { database, table, count, blueprint } = await req.json();

        if (!database || !table || !count || !blueprint) {
            return NextResponse.json({ success: false, error: "Missing required parameters" }, { status: 400 });
        }

        const sanitizedDb = sanitizeIdentifier(database);
        const sanitizedTable = sanitizeIdentifier(table);

        const columns = Object.keys(blueprint);
        const colNames = columns.map(c => `\`${sanitizeIdentifier(c)}\``).join(", ");

        const generateValue = (type: string) => {
            switch (type) {
                case "name":
                    const firstNames = ["John", "Jane", "Michael", "Sarah", "Chris", "Emma", "David", "Olivia"];
                    const lastNames = ["Smith", "Johnson", "Brown", "Taylor", "Miller", "Wilson", "Moore"];
                    return `${firstNames[Math.floor(Math.random() * firstNames.length)]} ${lastNames[Math.floor(Math.random() * lastNames.length)]}`;
                case "email":
                    const domains = ["example.com", "test.org", "gmail.com", "outlook.com"];
                    return `${Math.random().toString(36).substring(7)}@${domains[Math.floor(Math.random() * domains.length)]}`;
                case "phone":
                    return `+1-${Math.floor(100 + Math.random() * 900)}-${Math.floor(100 + Math.random() * 900)}-${Math.floor(1000 + Math.random() * 9000)}`;
                case "date":
                    const start = new Date(2020, 0, 1).getTime();
                    const end = new Date().getTime();
                    return new Date(start + Math.random() * (end - start)).toISOString().split('T')[0];
                case "integer":
                    return Math.floor(Math.random() * 10000);
                case "text":
                default:
                    const words = ["lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing", "elit"];
                    return Array.from({ length: 5 }, () => words[Math.floor(Math.random() * words.length)]).join(" ");
            }
        };

        const batchSize = 100;
        let inserted = 0;

        for (let i = 0; i < count; i += batchSize) {
            const currentBatch = Math.min(batchSize, count - i);
            const values: any[] = [];
            const placeholders: string[] = [];

            for (let j = 0; j < currentBatch; j++) {
                const rowValues = columns.map(col => generateValue(blueprint[col]));
                placeholders.push(`(${columns.map(() => "?").join(", ")})`);
                values.push(...rowValues);
            }

            const sql = `INSERT INTO ${sanitizedDb}.${sanitizedTable} (${colNames}) VALUES ${placeholders.join(", ")}`;
            await executeQuery(sql, values);
            inserted += currentBatch;
        }

        return NextResponse.json({ success: true, message: `Successfully inserted ${inserted} rows into ${table}` });
    } catch (error: any) {
        console.error("Mock Data Error:", error);
        return NextResponse.json({ success: false, error: error.message }, { status: 500 });
    }
}
