import { invoke } from "@tauri-apps/api/core";

export async function login(host: string, port: string, user: string, password?: string) {
    try {
        const success = await invoke<boolean>("cmd_auth_login", { host, port, user, password });
        return { success };
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function logout() {
    try {
        const success = await invoke<boolean>("cmd_auth_logout");
        return { success };
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function listDatabases() {
    try {
        const databases = await invoke<string[]>("cmd_list_databases");
        return { success: true, databases };
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function createDatabase(name: string) {
    try {
        await invoke("cmd_create_database", { name });
        return { success: true };
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function dropDatabase(database: string, confirmed: boolean) {
    try {
        const result = await invoke<any>("cmd_drop_database", { name: database, confirmed });
        return result;
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function listTables(database: string) {
    try {
        const tables = await invoke<string[]>("cmd_list_tables", { db: database });
        return { success: true, tables };
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function createTable(database: string, name: string, columns: any[]) {
    try {
        await invoke("cmd_create_table", { db: database, name, columns });
        return { success: true };
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function dropTable(database: string, table: string, confirmed: boolean) {
    try {
        const result = await invoke<any>("cmd_drop_table", { db: database, name: table, confirmed });
        return result;
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function getData(database: string, table: string, limit: number, offset: number, sortCol?: string, sortOrder?: string) {
    try {
        const result = await invoke<any>("cmd_get_data", {
            db: database,
            table,
            limit,
            offset,
            sortCol: sortCol || null,
            sortOrder: sortOrder || null
        });
        return result;
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function insertRow(database: string, table: string, data: any) {
    try {
        const result = await invoke<any>("cmd_insert_row", { db: database, table, data });
        return result;
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function updateRow(database: string, table: string, column: string, value: any, where: any) {
    try {
        const success = await invoke<boolean>("cmd_update_row", { db: database, table, column, value, whereClause: where });
        return { success };
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function deleteRow(database: string, table: string, where: any) {
    try {
        const result = await invoke<any>("cmd_delete_row", { db: database, table, whereClause: where });
        return result;
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function generateMockData(database: string, table: string, count: number, blueprint: any) {
    try {
        const result = await invoke<any>("cmd_generate_mock_data", { db: database, table, count, blueprint });
        return result;
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function getStructure(database: string, table: string) {
    try {
        const result = await invoke<any>("cmd_get_structure", { db: database, table });
        return result;
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function getRelations(database: string) {
    try {
        const result = await invoke<any>("cmd_get_relations", { db: database });
        return result;
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function getSchemaSuggestions(database: string) {
    try {
        const result = await invoke<any>("cmd_get_schema_suggestions", { db: database });
        return result;
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function getObjects(database: string) {
    try {
        const result = await invoke<any>("cmd_get_objects", { db: database });
        return result;
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function getServerStatus() {
    try {
        const result = await invoke<any>("cmd_server_status");
        return result;
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function getServerMetrics() {
    try {
        const result = await invoke<any>("cmd_server_metrics");
        return result;
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function getServerSlowLog() {
    try {
        const result = await invoke<any>("cmd_server_slow_log");
        return result;
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function executeSqlQuery(query: string, database?: string, confirmed?: boolean) {
    try {
        const result = await invoke<any>("cmd_execute_query", { sql: query, db: database || null, confirmed: confirmed || false });
        return result;
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function getDatabaseStats(database: string) {
    try {
        const result = await invoke<any>("cmd_get_database_stats", { db: database });
        return result;
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function exportDatabase(database: string, format: string, includeStructure: boolean, includeData: boolean) {
    try {
        const result = await invoke<string>("cmd_export", {
            db: database,
            format,
            includeStructure,
            includeData
        });
        return result;
    } catch (error: any) {
        throw new Error(error.toString());
    }
}

export async function importSql(database: string, sql: string) {
    try {
        const result = await invoke<any>("cmd_import", { db: database, sql });
        return result;
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function listUsers() {
    try {
        const result = await invoke<any>("cmd_list_users");
        return result;
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function saveLLMConfig(provider: string, key?: string, ollamaHost?: string, ollamaModel?: string) {
    try {
        const success = await invoke<boolean>("cmd_save_llm_config", {
            provider,
            key: key || null,
            ollamaHost: ollamaHost || null,
            ollamaModel: ollamaModel || null
        });
        return { success };
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function getLLMConfig() {
    try {
        const config = await invoke<any>("cmd_get_llm_config");
        return { success: true, config };
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

export async function getLLMConfigUnmasked() {
    try {
        const config = await invoke<any>("cmd_get_llm_config_unmasked");
        return { success: true, config };
    } catch (error: any) {
        return { success: false, error: error.toString() };
    }
}

