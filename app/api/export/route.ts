import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";
import { sanitizeIdentifier } from "@/lib/sanitize";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export async function GET(req: NextRequest) {
    const { searchParams } = new URL(req.url);
    const database = searchParams.get("database");
    const format = searchParams.get("format") || "sql"; // sql, json, csv
    const includeStructure = searchParams.get("structure") !== "false";
    const includeData = searchParams.get("data") !== "false";

    if (!database) {
        return NextResponse.json({ success: false, error: "Database name is required" }, { status: 400 });
    }

    try {
        const sanitizedDb = sanitizeIdentifier(database);

        // Get all tables
        const tables: any = await executeQuery(`SHOW TABLES FROM ${sanitizedDb}`);
        const tableNames = tables.map((row: any) => Object.values(row)[0]);

        let result: any;
        let contentType = "text/plain";
        let extension = "txt";

        if (format === "sql") {
            contentType = "text/sql";
            extension = "sql";
            let sqlDump = `-- MySQL GUI Dump\n-- Database: \`${sanitizedDb}\`\n\n`;

            for (const tableName of tableNames) {
                const sanitizedTable = sanitizeIdentifier(tableName);

                if (includeStructure) {
                    const createRes: any = await executeQuery(`SHOW CREATE TABLE ${sanitizedDb}.${sanitizedTable}`);
                    sqlDump += `DROP TABLE IF EXISTS \`${sanitizedTable}\`;\n${createRes[0]["Create Table"]};\n\n`;
                }

                if (includeData) {
                    const rows: any = await executeQuery(`SELECT * FROM ${sanitizedDb}.${sanitizedTable}`);
                    if (rows.length > 0) {
                        const columns = Object.keys(rows[0]);
                        const colNames = columns.map(c => `\`${sanitizeIdentifier(c)}\``).join(", ");
                        const values = rows.map((row: any) =>
                            `(${columns.map(col => {
                                const val = row[col];
                                if (val === null) return "NULL";
                                if (typeof val === "number") return val;
                                return `'${String(val).replace(/'/g, "''")}'`;
                            }).join(", ")})`
                        ).join(",\n");
                        sqlDump += `INSERT INTO \`${sanitizedTable}\` (${colNames}) VALUES\n${values};\n\n`;
                    }
                }
            }
            result = sqlDump;
        } else if (format === "json") {
            contentType = "application/json";
            extension = "json";
            const fullData: Record<string, any> = {};
            for (const tableName of tableNames) {
                const sanitizedTable = sanitizeIdentifier(tableName);
                if (includeData) {
                    const rows: any = await executeQuery(`SELECT * FROM ${sanitizedDb}.${sanitizedTable}`);
                    fullData[tableName] = rows;
                } else {
                    fullData[tableName] = [];
                }
            }
            result = JSON.stringify(fullData, null, 2);
        } else if (format === "csv") {
            contentType = "text/csv";
            extension = "zip"; // CSV export of multiple tables might be better as a zip or combined?
            // For simplicity, let's just do a combined CSV with markers or just the first table?
            // Let's do a combined format with table names as headers if multiple.
            let csv = "";
            for (const tableName of tableNames) {
                const sanitizedTable = sanitizeIdentifier(tableName);
                if (includeData) {
                    const rows: any = await executeQuery(`SELECT * FROM ${sanitizedDb}.${sanitizedTable}`);
                    if (rows.length > 0) {
                        const columns = Object.keys(rows[0]);
                        csv += `Table: ${tableName}\n`;
                        csv += columns.join(",") + "\n";
                        rows.forEach((row: any) => {
                            csv += columns.map(col => `"${String(row[col] || "").replace(/"/g, '""')}"`).join(",") + "\n";
                        });
                        csv += "\n";
                    }
                }
            }
            result = csv;
            extension = "csv";
        }

        return new NextResponse(result, {
            headers: {
                "Content-Type": contentType,
                "Content-Disposition": `attachment; filename="${database}_export.${extension}"`,
            },
        });
    } catch (error: any) {
        console.error("Export Error:", error);
        return NextResponse.json({ success: false, error: error.message || "Failed to export database" }, { status: 500 });
    }
}
