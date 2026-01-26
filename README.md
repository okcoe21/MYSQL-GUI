# MySQL GUI Administration Tool
Developed by okcoe21

A high-performance, modern, and secure MySQL administration interface designed for local development and professional database management. This is an open-source project.

---

## Overview

This tool provides a streamlined web-based interface for interacting with MySQL databases without the overhead of heavy administration suites. Built with Next.js and TypeScript, it focuses on visual excellence, security, and a robust user experience for developers.

---

## Key Features

### Database & Server Management
- **Server Monitoring**: Real-time health metrics (Uptime, Connections, Queries) and a live process list.
- **Advanced Schema Support**: Manage Tables, Views, Stored Procedures, and Functions from a unified sidebar.
- **User Management**: Browser-based interface for auditing MySQL accounts and status.
- **Slow Query Analysis**: Detect and analyze long-running queries to optimize database performance.

### Developer Productivity
- **Visual ER Diagrams**: Auto-generated relationship maps using SVG to visualize schema architecture.
- **Interactive Query Builder**: Drag-and-drop style interface for generating SELECT queries without writing SQL.
- **Query History & Favorites**: Local-first storage for your most important snippets and recent execution history.
- **Mock Data Generator**: Seed tables with realistic sample data (Names, Emails, Dates) for rapid prototyping.

### Data Manipulation & Portability
- **Inline Result Editing**: Edit table data directly in the grid with auto-save functionality.
- **Power Search & Filter**: Server-side filtering, sorting, and pagination for high-volume datasets.
- **Multi-Format Export**: Download backups in SQL, JSON, or CSV formats with granular content control.
- **Script Importer**: Reliable execution of large SQL scripts with real-time error reporting.

---

## Security Architecture

The application is engineered with a security-first mindset:

- **Local Data Privacy**: Designed for local execution to ensure that sensitive database credentials never leave your environment.
- **Encrypted Sessions**: Utilizes JWT-based session management with server-side encryption to store MySQL credentials.
- **Sanitization Layer**: Employs strict identifier sanitization and backtick-wrapping for all database, table, and column names to mitigate SQL injection risks.
- **Destructive Action Safeguards**: Real-time analysis of SQL queries to detect and require confirmation for DROP, TRUNCATE, and DELETE operations.

---

## Technical Stack

- **Frontend**: Next.js 15, React 19, Lucide React (Icons)
- **Backend**: Next.js API Routes (Node.js runtime)
- **Database Driver**: mysql2 (with Connection Pooling)
- **Styling**: Unified CSS Modules for a premium, custom interface
- **Security**: jose (JWT), Encrypted Credentials

---

## Installation and Setup

### Prerequisites

- Node.js v18.0.0 or higher
- A running MySQL Server instance

### Steps to Run Locally

1. **Clone the project**:
   ```bash
   git clone https://github.com/okcoe21/MYSQL-GUI.git
   cd mysql-gui
   ```

2. **Install dependencies**:
   ```bash
   npm install
   ```

3. **Configure Environment**:
   Create a `.env.local` file in the root directory. You can generate a secure secret using:
   ```bash
   # Linux/macOS
   echo "SESSION_SECRET=$(openssl rand -hex 32)" > .env.local
   ```

4. **Launch Application**:
   ```bash
   npm run dev
   ```
   Access the interface at http://localhost:3000.

---

## Contributing

As an open-source project, contributions are welcome. Please feel free to submit issues or pull requests to help improve the tool for the developer community.

---
*Made by okcoe21*
