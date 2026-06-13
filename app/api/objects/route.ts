import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";
import { sanitizeIdentifier } from "@/lib/sanitize";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

async function getProcedures(database: string) {
    try {
        return await executeQuery(`SELECT ROUTINE_NAME, ROUTINE_DEFINITION, DATA_TYPE FROM INFORMATION_SCHEMA.ROUTINES WHERE ROUTINE_SCHEMA = ? AND ROUTINE_TYPE = 'PROCEDURE'`, [database]);
    } catch (error: any) {
        if (error.message?.includes("mysql.proc") || error.message?.includes("Column count")) {
            const statusList: any = await executeQuery(`SHOW PROCEDURE STATUS WHERE Db = ?`, [database]);
            const results = [];
            for (const row of statusList) {
                const name = row.Name || row.name;
                let definition = "";
                try {
                    const createResult: any = await executeQuery(`SHOW CREATE PROCEDURE \`${sanitizeIdentifier(database)}\`.\`${sanitizeIdentifier(name)}\``);
                    if (createResult && createResult[0]) {
                        definition = createResult[0]["Create Procedure"] || createResult[0]["create procedure"] || "";
                    }
                } catch (err) {
                    console.error("Failed to get create procedure definition for", name, err);
                }
                results.push({
                    ROUTINE_NAME: name,
                    ROUTINE_DEFINITION: definition,
                    DATA_TYPE: ""
                });
            }
            return results;
        }
        throw error;
    }
}

async function getFunctions(database: string) {
    try {
        return await executeQuery(`SELECT ROUTINE_NAME, ROUTINE_DEFINITION, DATA_TYPE FROM INFORMATION_SCHEMA.ROUTINES WHERE ROUTINE_SCHEMA = ? AND ROUTINE_TYPE = 'FUNCTION'`, [database]);
    } catch (error: any) {
        if (error.message?.includes("mysql.proc") || error.message?.includes("Column count")) {
            const statusList: any = await executeQuery(`SHOW FUNCTION STATUS WHERE Db = ?`, [database]);
            const results = [];
            for (const row of statusList) {
                const name = row.Name || row.name;
                let definition = "";
                try {
                    const createResult: any = await executeQuery(`SHOW CREATE FUNCTION \`${sanitizeIdentifier(database)}\`.\`${sanitizeIdentifier(name)}\``);
                    if (createResult && createResult[0]) {
                        definition = createResult[0]["Create Function"] || createResult[0]["create function"] || "";
                    }
                } catch (err) {
                    console.error("Failed to get create function definition for", name, err);
                }
                results.push({
                    ROUTINE_NAME: name,
                    ROUTINE_DEFINITION: definition,
                    DATA_TYPE: ""
                });
            }
            return results;
        }
        throw error;
    }
}

export async function GET(req: NextRequest) {
    const { searchParams } = new URL(req.url);
    const database = searchParams.get("database");
    const type = searchParams.get("type"); // views, procedures, functions

    if (!database) {
        return NextResponse.json({ success: false, error: "Database is required" }, { status: 400 });
    }

    try {
        let results: any = [];

        if (type === "views") {
            results = await executeQuery(`SELECT TABLE_NAME, VIEW_DEFINITION, IS_UPDATABLE FROM INFORMATION_SCHEMA.VIEWS WHERE TABLE_SCHEMA = ?`, [database]);
        } else if (type === "procedures") {
            results = await getProcedures(database);
        } else if (type === "functions") {
            results = await getFunctions(database);
        } else {
            // Fetch names only for sidebar if no type specified
            let views: any = [];
            try {
                views = await executeQuery(`SELECT TABLE_NAME as name FROM INFORMATION_SCHEMA.VIEWS WHERE TABLE_SCHEMA = ?`, [database]);
            } catch (err) {
                console.error("Failed to fetch views summary:", err);
            }

            let procedures: any = [];
            try {
                procedures = await executeQuery(`SELECT ROUTINE_NAME as name FROM INFORMATION_SCHEMA.ROUTINES WHERE ROUTINE_SCHEMA = ? AND ROUTINE_TYPE = 'PROCEDURE'`, [database]);
            } catch (error: any) {
                if (error.message?.includes("mysql.proc") || error.message?.includes("Column count")) {
                    try {
                        const statusList: any = await executeQuery(`SHOW PROCEDURE STATUS WHERE Db = ?`, [database]);
                        procedures = statusList.map((row: any) => ({ name: row.Name || row.name }));
                    } catch (err) {
                        console.error("Failed to fetch procedures via SHOW fallback:", err);
                    }
                } else {
                    console.error("Failed to fetch procedures summary:", error);
                }
            }

            let functions: any = [];
            try {
                functions = await executeQuery(`SELECT ROUTINE_NAME as name FROM INFORMATION_SCHEMA.ROUTINES WHERE ROUTINE_SCHEMA = ? AND ROUTINE_TYPE = 'FUNCTION'`, [database]);
            } catch (error: any) {
                if (error.message?.includes("mysql.proc") || error.message?.includes("Column count")) {
                    try {
                        const statusList: any = await executeQuery(`SHOW FUNCTION STATUS WHERE Db = ?`, [database]);
                        functions = statusList.map((row: any) => ({ name: row.Name || row.name }));
                    } catch (err) {
                        console.error("Failed to fetch functions via SHOW fallback:", err);
                    }
                } else {
                    console.error("Failed to fetch functions summary:", error);
                }
            }

            return NextResponse.json({
                success: true,
                views: views.map((v: any) => v.name),
                procedures: procedures.map((p: any) => p.name),
                functions: functions.map((f: any) => f.name)
            });
        }

        return NextResponse.json({ success: true, data: results });
    } catch (error: any) {
        console.error("Fetch Objects Error:", error);
        return NextResponse.json({ success: false, error: error.message }, { status: 500 });
    }
}
