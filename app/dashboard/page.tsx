"use client";

import { useState } from "react";
import Sidebar from "./Sidebar";
import styles from "./dashboard.module.css";
import { Database, Table, Home, Layout, FileText, Code, Search, Trash2, Upload, Download, Clock, Wand2 } from "lucide-react";
import TableData from "./TableData";
import SqlEditor from "./SqlEditor";
import DbOverview from "./DbOverview";
import StructureView from "./StructureView";
import InsertView from "./InsertView";
import OperationsView from "./OperationsView";
import ServerOverview from "./ServerOverview";
import CreateTable from "./CreateTable";
import ImportView from "./ImportView";
import ExportView from "./ExportView";
import HistoryView from "./HistoryView";
import VisualQueryBuilder from "./VisualQueryBuilder";
import ObjectView from "./ObjectView";
import UserManagement from "./UserManagement";

type ViewType = "browse" | "structure" | "sql" | "search" | "insert" | "operations" | "db_overview" | "server_overview" | "create_table" | "import" | "export" | "history" | "query_builder" | "object_view" | "users";

export default function DashboardPage() {
    const [selectedDb, setSelectedDb] = useState<string | undefined>();
    const [selectedTable, setSelectedTable] = useState<string | undefined>();
    const [activeView, setActiveView] = useState<ViewType>("server_overview");
    const [sidebarKey, setSidebarKey] = useState(0);
    const [externalQuery, setExternalQuery] = useState("");
    const [selectedObject, setSelectedObject] = useState<{ name: string; type: "view" | "procedure" | "function" } | undefined>();

    const refreshSidebar = () => setSidebarKey(prev => prev + 1);

    const handleSelectTable = (db: string, table: string) => {
        setSelectedDb(db);
        setSelectedTable(table);
        setActiveView("browse");
    };

    const handleSelectObject = (db: string, name: string, type: "view" | "procedure" | "function") => {
        setSelectedDb(db);
        setSelectedTable(undefined);
        setSelectedObject({ name, type });
        setActiveView("object_view");
    };

    const handleSelectDb = (db: string) => {
        setSelectedDb(db);
        setSelectedTable(undefined);
        setSelectedObject(undefined);
        setActiveView("db_overview");
    };

    const handleRunHistoryQuery = (query: string) => {
        setExternalQuery(query);
        setActiveView("sql");
    };

    const handleTableDropped = () => {
        refreshSidebar();
        if (selectedDb) {
            handleSelectDb(selectedDb);
        } else {
            setActiveView("server_overview");
        }
    };

    const renderContent = () => {
        if (activeView === "server_overview") {
            return (
                <ServerOverview
                    onSelectDb={handleSelectDb}
                    onRefreshSidebar={refreshSidebar}
                />
            );
        }

        if (activeView === "users") {
            return <UserManagement />;
        }

        if (!selectedDb) {
            return (
                <div className={styles.emptyState}>
                    <Home size={48} className={styles.emptyIcon} />
                    <h2>Server: localhost</h2>
                    <p>Select a database from the sidebar to begin</p>
                </div>
            );
        }

        // Database level views (no table selected)
        if (!selectedTable) {
            switch (activeView) {
                case "sql":
                    return <SqlEditor initialDatabase={selectedDb} externalQuery={externalQuery} />;
                case "history":
                    return <HistoryView onRunQuery={handleRunHistoryQuery} />;
                case "query_builder":
                    return <VisualQueryBuilder database={selectedDb} onRunQuery={handleRunHistoryQuery} />;
                case "import":
                    return <ImportView database={selectedDb} onSuccess={refreshSidebar} />;
                case "export":
                    return <ExportView database={selectedDb} />;
                case "create_table":
                    return (
                        <CreateTable
                            database={selectedDb}
                            onBack={() => setActiveView("db_overview")}
                            onSuccess={(table) => handleSelectTable(selectedDb, table)}
                        />
                    );
                case "db_overview":
                default:
                    if (activeView === "object_view" && selectedObject) {
                        return <ObjectView database={selectedDb} name={selectedObject.name} type={selectedObject.type} />;
                    }
                    return (
                        <DbOverview
                            database={selectedDb}
                            onSelectTable={(t) => handleSelectTable(selectedDb, t)}
                            onCreateTable={() => setActiveView("create_table")}
                            onRefreshSidebar={refreshSidebar}
                        />
                    );
            }
        }

        // Table level views
        switch (activeView) {
            case "browse":
                return <TableData database={selectedDb} table={selectedTable} />;
            case "structure":
                return <StructureView database={selectedDb} table={selectedTable} />;
            case "sql":
                return <SqlEditor initialDatabase={selectedDb} externalQuery={externalQuery} />;
            case "history":
                return <HistoryView onRunQuery={handleRunHistoryQuery} />;
            case "query_builder":
                return <VisualQueryBuilder database={selectedDb} initialTable={selectedTable} onRunQuery={handleRunHistoryQuery} />;
            case "insert":
                return <InsertView database={selectedDb} table={selectedTable} onSuccess={() => setActiveView("browse")} />;
            case "operations":
                return <OperationsView database={selectedDb} table={selectedTable} onTableDropped={handleTableDropped} />;
            default:
                return <TableData database={selectedDb} table={selectedTable} />;
        }
    };

    return (
        <div className={styles.container}>
            <Sidebar
                key={sidebarKey}
                onSelectTable={handleSelectTable}
                onSelectObject={handleSelectObject}
                onSelectDb={handleSelectDb}
                onSelectUsers={() => setActiveView("users")}
                selectedDb={selectedDb}
                selectedTable={selectedTable}
                selectedObject={selectedObject}
            />

            <main className={styles.mainContent}>
                <div className={styles.topBar}>
                    <div className={styles.breadcrumbs}>
                        <div className={styles.breadcrumbItem} onClick={() => setActiveView("server_overview")}>
                            <Home size={14} /> Server: localhost
                        </div>
                        {selectedDb && (
                            <>
                                <span className={styles.breadcrumbSeparator}>&raquo;</span>
                                <div className={styles.breadcrumbItem} onClick={() => handleSelectDb(selectedDb)}>
                                    <Database size={14} /> {selectedDb}
                                </div>
                            </>
                        )}
                        {selectedTable && (
                            <>
                                <span className={styles.breadcrumbSeparator}>&raquo;</span>
                                <div className={styles.breadcrumbItem} onClick={() => setActiveView("browse")}>
                                    <Table size={14} /> {selectedTable}
                                </div>
                            </>
                        )}
                        {selectedObject && (
                            <>
                                <span className={styles.breadcrumbSeparator}>&raquo;</span>
                                <div className={styles.breadcrumbItem}>
                                    {selectedObject.name} ({selectedObject.type})
                                </div>
                            </>
                        )}
                    </div>
                </div>

                {/* Tabs - Only show when a Database or Table is selected */}
                {(selectedDb || selectedTable) && (
                    <div className={styles.tabs}>
                        {selectedTable ? (
                            <>
                                <button className={`${styles.tab} ${activeView === "browse" ? styles.active : ""}`} onClick={() => setActiveView("browse")}>
                                    <Layout size={14} className={styles.tabIcon} /> Browse
                                </button>
                                <button className={`${styles.tab} ${activeView === "db_overview" ? styles.active : ""}`} onClick={() => setActiveView("db_overview")}>
                                    <Layout size={14} className={styles.tabIcon} /> Structure
                                </button>
                                <button className={`${styles.tab} ${activeView === "sql" ? styles.active : ""}`} onClick={() => { setActiveView("sql"); setExternalQuery(""); }}>
                                    <Code size={14} className={styles.tabIcon} /> SQL
                                </button>
                                <button className={`${styles.tab} ${activeView === "history" ? styles.active : ""}`} onClick={() => setActiveView("history")}>
                                    <Clock size={14} className={styles.tabIcon} /> History
                                </button>
                                <button className={`${styles.tab} ${activeView === "query_builder" ? styles.active : ""}`} onClick={() => setActiveView("query_builder")}>
                                    <Wand2 size={14} className={styles.tabIcon} /> Query Builder
                                </button>
                                <button className={`${styles.tab} ${activeView === "import" ? styles.active : ""}`} onClick={() => setActiveView("import")}>
                                    <Upload size={14} className={styles.tabIcon} /> Import
                                </button>
                                <button className={`${styles.tab} ${activeView === "export" ? styles.active : ""}`} onClick={() => setActiveView("export")}>
                                    <Download size={14} className={styles.tabIcon} /> Export
                                </button>
                                <button className={`${styles.tab} ${activeView === "operations" ? styles.active : ""}`} onClick={() => setActiveView("operations")}>
                                    <Trash2 size={14} className={styles.tabIcon} /> Operations
                                </button>
                            </>
                        ) : (
                            <>
                                <button className={`${styles.tab} ${activeView === "db_overview" ? styles.active : ""}`} onClick={() => setActiveView("db_overview")}>
                                    <Layout size={14} className={styles.tabIcon} /> Structure
                                </button>
                                <button className={`${styles.tab} ${activeView === "sql" ? styles.active : ""}`} onClick={() => setActiveView("sql")}>
                                    <Code size={14} className={styles.tabIcon} /> SQL
                                </button>
                                <button className={`${styles.tab} ${activeView === "import" ? styles.active : ""}`} onClick={() => setActiveView("import")}>
                                    <Upload size={14} className={styles.tabIcon} /> Import
                                </button>
                                <button className={`${styles.tab} ${activeView === "export" ? styles.active : ""}`} onClick={() => setActiveView("export")}>
                                    <Download size={14} className={styles.tabIcon} /> Export
                                </button>
                            </>
                        )}
                    </div>
                )}

                <div className={styles.contentArea}>
                    {renderContent()}
                </div>
            </main>
        </div>
    );
}
