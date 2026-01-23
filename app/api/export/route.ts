import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";
import { sanitizeIdentifier } from "@/lib/sanitize";

export const runtime = "nodejs";

export async function GET(req: NextRequest) {
    const { searchParams } = new URL(req.url);
    const database = searchParams.get("database");

    if (!database) {
        return NextResponse.json({ success: false, error: "Database name is required" }, { status: 400 });
    }

    try {
        const sanitizedDb = sanitizeIdentifier(database);
        let sqlDump = `-- MySQL GUI Dump\n`;
        sqlDump += `-- Host: localhost\n`;
        sqlDump += `-- Generation Time: ${new Date().toISOString()}\n`;
        sqlDump += `-- Database: \`${sanitizedDb}\`\n\n`;
        sqlDump += `SET SQL_MODE = "NO_AUTO_VALUE_ON_ZERO";\n`;
        sqlDump += `SET time_zone = "+00:00";\n\n`;

        // Get all tables
        const tables: any = await executeQuery(`SHOW TABLES FROM ${sanitizedDb}`);
        const tableNames = tables.map((row: any) => Object.values(row)[0]);

        for (const tableName of tableNames) {
            const sanitizedTable = sanitizeIdentifier(tableName);

            sqlDump += `-- --------------------------------------------------------\n\n`;
            sqlDump += `-- Table structure for table \`${sanitizedTable}\`\n\n`;
            sqlDump += `DROP TABLE IF EXISTS \`${sanitizedTable}\`;\n`;

            // Get Create Table statement
            const createRes: any = await executeQuery(`SHOW CREATE TABLE ${sanitizedDb}.${sanitizedTable}`);
            const createSql = createRes[0]["Create Table"];
            sqlDump += `${createSql};\n\n`;

            // Get data
            const rows: any = await executeQuery(`SELECT * FROM ${sanitizedDb}.${sanitizedTable}`);
            if (rows.length > 0) {
                sqlDump += `-- Dumping data for table \`${sanitizedTable}\`\n\n`;

                const columns = Object.keys(rows[0]);
                const colNames = columns.map(c => `\`${sanitizeIdentifier(c)}\``).join(", ");

                // Group inserts in batches of 100 for efficiency
                const batchSize = 100;
                for (let i = 0; i < rows.length; i += batchSize) {
                    const batch = rows.slice(i, i + batchSize);
                    sqlDump += `INSERT INTO \`${sanitizedTable}\` (${colNames}) VALUES\n`;

                    const values = batch.map((row: any) => {
                        const rowValues = columns.map(col => {
                            const val = row[col];
                            if (val === null) return "NULL";
                            if (typeof val === "number") return val;
                            // Escape single quotes
                            return `'${String(val).replace(/'/g, "''")}'`;
                        }).join(", ");
                        return `(${rowValues})`;
                    }).join(",\n");

                    sqlDump += `${values};\n`;
                }
                sqlDump += `\n`;
            }
        }

        // Return as a file download
        return new NextResponse(sqlDump, {
            headers: {
                "Content-Type": "text/sql",
                "Content-Disposition": `attachment; filename="${database}_dump.sql"`,
            },
        });
    } catch (error: any) {
        console.error("Export Error:", error);
        return NextResponse.json({ success: false, error: error.message || "Failed to export database" }, { status: 500 });
    }
}
